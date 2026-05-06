//! Plane API poller — sprint data + agent issue assignments.
//!
//! Polls Plane REST API at 30s intervals to populate:
//! - Sprint widget (name, d-day, todo/in-progress/in-review/done counts)
//! - Agent widget (which agent owns which issue, current state)

use std::collections::HashMap;

/// Sprint snapshot from Plane API.
#[derive(Debug, Clone, Default)]
pub struct SprintSnapshot {
    pub name: String,
    pub d_day: Option<i64>,
    pub todo: u32,
    pub in_progress: u32,
    pub in_review: u32,
    pub done: u32,
    pub total: u32,
}

impl SprintSnapshot {
    pub fn progress_pct(&self) -> f32 {
        if self.total == 0 {
            return 0.0;
        }
        self.done as f32 / self.total as f32 * 100.0
    }
}

/// Per-agent issue assignment.
#[derive(Debug, Clone)]
pub struct AgentIssue {
    pub role: String,
    pub issue_id: String,
    pub issue_title: String,
    pub state: IssueState,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueState {
    Todo,
    InProgress,
    InReview,
    Done,
}

impl std::fmt::Display for IssueState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueState::Todo => write!(f, "Todo"),
            IssueState::InProgress => write!(f, "In Progress"),
            IssueState::InReview => write!(f, "In Review"),
            IssueState::Done => write!(f, "Done"),
        }
    }
}

/// Fetch sprint data from Plane API.
///
/// Returns (sprint, agent_issues). Errors are logged but don't crash —
/// the monitor shows stale data with a warning indicator.
pub async fn poll_sprint(
    plane_url: &str,
    workspace_slug: &str,
    project_id: &str,
    api_key: &str,
) -> Result<(SprintSnapshot, Vec<AgentIssue>), String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))?;

    // Fetch active cycle (sprint)
    let cycles_url = format!(
        "{}/api/v1/workspaces/{}/projects/{}/cycles/",
        plane_url.trim_end_matches('/'),
        workspace_slug,
        project_id
    );
    let resp = client
        .get(&cycles_url)
        .header("X-API-Key", api_key)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("Plane API error: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Plane API status: {}", resp.status()));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Plane JSON parse: {e}"))?;

    let cycles = body
        .get("results")
        .and_then(|r| r.as_array())
        .cloned()
        .unwrap_or_default();

    // Find the active (current) cycle
    let active = cycles.iter().find(|c| {
        c.get("is_active")
            .and_then(|a| a.as_bool())
            .unwrap_or(false)
    });

    let mut sprint = SprintSnapshot::default();
    if let Some(cycle) = active {
        sprint.name = cycle
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("(unnamed)")
            .to_string();

        // Compute d-day from end_date
        if let Some(end_date) = cycle.get("end_date").and_then(|d| d.as_str()) {
            if let Ok(end) = chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d") {
                let today = chrono::Local::now().date_naive();
                sprint.d_day = Some((end - today).num_days());
            }
        }
    }

    // Fetch issues and count by state
    let issues_url = format!(
        "{}/api/v1/workspaces/{}/projects/{}/issues/",
        plane_url.trim_end_matches('/'),
        workspace_slug,
        project_id
    );
    let resp = client
        .get(&issues_url)
        .header("X-API-Key", api_key)
        .header("Accept", "application/json")
        .query(&[("per_page", "200")])
        .send()
        .await
        .map_err(|e| format!("Plane issues error: {e}"))?;

    let issues_body: serde_json::Value =
        resp.json().await.map_err(|e| format!("Issues JSON: {e}"))?;

    let issues = issues_body
        .get("results")
        .and_then(|r| r.as_array())
        .cloned()
        .unwrap_or_default();

    let mut agent_issues = Vec::new();

    for issue in &issues {
        let state_name = issue
            .get("state_detail")
            .and_then(|s| s.get("group"))
            .and_then(|g| g.as_str())
            .unwrap_or("");

        let state = match state_name {
            "backlog" | "unstarted" => IssueState::Todo,
            "started" => IssueState::InProgress,
            "completed" => IssueState::Done,
            _ => IssueState::Todo,
        };

        match state {
            IssueState::Todo => sprint.todo += 1,
            IssueState::InProgress => sprint.in_progress += 1,
            IssueState::InReview => sprint.in_review += 1,
            IssueState::Done => sprint.done += 1,
        }
        sprint.total += 1;

        // Extract assignee → agent role mapping
        if let Some(assignees) = issue.get("assignees").and_then(|a| a.as_array()) {
            let title = issue
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("")
                .to_string();
            let id = issue
                .get("sequence_id")
                .and_then(|s| s.as_u64())
                .map(|s| format!("#{s}"))
                .unwrap_or_default();

            for assignee in assignees {
                if let Some(uuid) = assignee.as_str() {
                    agent_issues.push(AgentIssue {
                        role: uuid.to_string(), // Will be mapped to role name by caller
                        issue_id: id.clone(),
                        issue_title: title.clone(),
                        state,
                        updated_at: 0,
                    });
                }
            }
        }
    }

    Ok((sprint, agent_issues))
}
