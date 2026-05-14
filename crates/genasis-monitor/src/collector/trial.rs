//! Trial-flavor collector — polls the trial-app's `/api/plane/issues`
//! and `/api/mattermost/posts` endpoints to keep the Sprint / Agents /
//! Log-tail widgets populated when the project's `[plane].flavor` /
//! `[mattermost].flavor` are `"trial"`.
//!
//! Mirrors `plane::poll_sprint` but talks to the trial-app instead of
//! a real Plane API. Returns the same `SprintSnapshot` + `AgentIssue`
//! shapes so widgets need no changes, plus a small `Vec<String>` of
//! recent chat lines for the log-tail widget.

use std::time::Duration;

use super::plane::{AgentIssue, IssueState, SprintSnapshot};

/// Snapshot returned by `poll_trial`. Combines all three widget
/// payloads in one struct so the run-loop can swap them in atomically.
pub struct TrialSnapshot {
    pub sprint: SprintSnapshot,
    pub agent_issues: Vec<AgentIssue>,
    pub log_tail: Vec<String>,
    /// Last `app_kind` + `app_features` reported by the trial-app —
    /// surfaced in the sprint widget header so the operator can tell
    /// which demo (Quiz / Todo / …) is currently published.
    pub app_kind: String,
    pub app_features: Vec<String>,
    /// D-082: total posts seen on the scrum channel — feeds the
    /// Network widget's MM call counter.
    pub posts_total: usize,
}

/// Fetch sprint + agents + log tail from a trial-app instance.
///
/// `base_url` should be the trial-app root (e.g.
/// `https://mmplane-trial.realstory.blog`). `team_token` is the
/// per-team isolation key from `[trial].team_token`. `project_slug`
/// and `channel_name` come from the resolved Mattermost scrum channel
/// + project ID. Errors are stringified so the run-loop can log them
/// without unwinding.
pub async fn poll_trial(
    base_url: &str,
    team_token: &str,
    project_slug: &str,
    channel_name: &str,
) -> Result<TrialSnapshot, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))?;

    let base = base_url.trim_end_matches('/');
    // D-082: posts_total 은 sim_posts 응답의 array length — Network 위젯
    // MM 카운터 채움에 사용. 200 으로 캡하지 않고 raw 길이 그대로.
    let mut posts_total: usize = 0;

    // (1) sim_issues — sprint counts + per-assignee AgentIssue rows
    let issues_url = format!("{base}/api/plane/issues?project_slug={project_slug}");
    let resp = client
        .get(&issues_url)
        .header("X-Genasis-Team-Token", team_token)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("trial issues GET {issues_url}: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!(
            "trial issues status: {} body: {}",
            resp.status(),
            resp.text().await.unwrap_or_default()
        ));
    }
    let issues_body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("trial issues parse: {e}"))?;
    let issues = issues_body
        .get("issues")
        .and_then(|r| r.as_array())
        .cloned()
        .unwrap_or_default();

    let mut sprint = SprintSnapshot {
        name: project_slug.to_string(),
        ..Default::default()
    };
    let mut agent_issues = Vec::new();
    for issue in &issues {
        let state_str = issue
            .get("state")
            .and_then(|s| s.as_str())
            .unwrap_or("todo");
        let state = match state_str {
            "todo" => IssueState::Todo,
            "inprogress" => IssueState::InProgress,
            "inreview" => IssueState::InReview,
            "done" => IssueState::Done,
            _ => IssueState::Todo,
        };
        match state {
            IssueState::Todo => sprint.todo += 1,
            IssueState::InProgress => sprint.in_progress += 1,
            IssueState::InReview => sprint.in_review += 1,
            IssueState::Done => sprint.done += 1,
        }
        sprint.total += 1;

        let assignee = issue.get("assignee").and_then(|a| a.as_str()).unwrap_or("");
        if assignee.is_empty() {
            continue;
        }
        let title = issue
            .get("title")
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .to_string();
        let seq = issue
            .get("sequence_id")
            .and_then(|s| s.as_u64())
            .map(|s| format!("#{s}"))
            .unwrap_or_default();
        agent_issues.push(AgentIssue {
            role: assignee.to_string(),
            issue_id: seq,
            issue_title: title,
            state,
            updated_at: 0,
        });
    }

    // (2) sim_posts — log-tail (newest N entries)
    let posts_url = format!("{base}/api/mattermost/posts?channel_name={channel_name}");
    let mut log_tail = Vec::new();
    match client
        .get(&posts_url)
        .header("X-Genasis-Team-Token", team_token)
        .header("Accept", "application/json")
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => {
            if let Ok(body) = r.json::<serde_json::Value>().await {
                if let Some(arr) = body.get("posts").and_then(|p| p.as_array()) {
                    posts_total = arr.len();
                    for post in arr.iter().rev().take(40).collect::<Vec<_>>().iter().rev() {
                        let actor = post.get("actor").and_then(|a| a.as_str()).unwrap_or("?");
                        let message = post
                            .get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("")
                            .lines()
                            .next()
                            .unwrap_or("");
                        if message.is_empty() {
                            continue;
                        }
                        log_tail.push(format!("[{actor}] {message}"));
                    }
                }
            }
        }
        Ok(r) => {
            log_tail.push(format!("(posts {}: {})", posts_url, r.status()));
        }
        Err(e) => {
            log_tail.push(format!("(posts {posts_url}: {e})"));
        }
    }

    // (3) sim_teams.app_kind + app_features — shown in sprint header
    let status_url = format!("{base}/api/trial/team-app/status?team={team_token}");
    let mut app_kind = String::new();
    let mut app_features = Vec::new();
    if let Ok(r) = client
        .get(&status_url)
        .header("X-Genasis-Team-Token", team_token)
        .send()
        .await
    {
        if r.status().is_success() {
            if let Ok(body) = r.json::<serde_json::Value>().await {
                app_kind = body
                    .get("app_kind")
                    .and_then(|k| k.as_str())
                    .unwrap_or("")
                    .to_string();
                if let Some(arr) = body.get("app_features").and_then(|f| f.as_array()) {
                    for f in arr {
                        if let Some(s) = f.as_str() {
                            app_features.push(s.to_string());
                        }
                    }
                }
            }
        }
    }

    Ok(TrialSnapshot {
        sprint,
        agent_issues,
        log_tail,
        app_kind,
        app_features,
        posts_total,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Live smoke test against the operator-hosted trial-app — proves
    /// D-025 wiring end-to-end. Run with `cargo test -p genasis-monitor
    /// d025_live --release -- --ignored --nocapture` to exercise the
    /// real network path; default `cargo test` skips it.
    #[tokio::test]
    #[ignore]
    async fn d025_live_smoke_against_hosted_trial() {
        let snap = poll_trial(
            "https://mmplane-trial.realstory.blog",
            "37236daa2e2e407bd9cb2e3e1158d095",
            "v516-final",
            "scrum-v516-final",
        )
        .await
        .expect("poll_trial should succeed against operator-hosted trial-app");
        assert!(
            snap.sprint.total > 0,
            "sprint should have ≥1 issue, got {} (sprint={:?})",
            snap.sprint.total,
            snap.sprint
        );
        assert!(!snap.log_tail.is_empty(), "log_tail should not be empty");
        eprintln!(
            "[d025] sprint: total={} todo={} done={} | agents={} | logs={} | app={} features={:?}",
            snap.sprint.total,
            snap.sprint.todo,
            snap.sprint.done,
            snap.agent_issues.len(),
            snap.log_tail.len(),
            snap.app_kind,
            snap.app_features
        );
    }
}
