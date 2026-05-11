//! Trial flavor — forwards every Plane trait call to the trial-app's
//! `/api/plane/*` HTTP endpoints. Used when an operator runs `genasis dev`
//! without a real Plane server: the trial-app stands in as a lightweight
//! Plane simulator so the agentic workflow can be exercised end-to-end.

use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use serde_json::{json, Value};

use genasis_core::error::{Error, Result};

use super::{CycleRef, IssueRef, LabelRef, PlaneProvider};

#[derive(Debug, Clone)]
pub struct TrialPlane {
    base_url: String,
    secret: String,
    team_token: String,
    client: Client,
}

impl TrialPlane {
    pub fn new(
        base_url: impl Into<String>,
        secret: impl Into<String>,
        team_token: impl Into<String>,
    ) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            secret: secret.into(),
            team_token: team_token.into(),
            client: Client::new(),
        }
    }

    fn url(&self, suffix: &str) -> String {
        format!("{}{}", self.base_url, suffix)
    }

    fn headers(&self) -> HeaderMap {
        let mut h = HeaderMap::new();
        if !self.secret.is_empty() {
            if let Ok(v) = HeaderValue::from_str(&self.secret) {
                h.insert("x-genasis-trial-secret", v);
            }
        }
        // ADR-016 §3: scope every server-to-server call into the
        // tenant's sim namespace. Empty token is allowed — the
        // trial-app falls through to DEFAULT_TEAM_TOKEN for it,
        // matching pre-ADR-016 behaviour.
        if !self.team_token.is_empty() {
            if let Ok(v) = HeaderValue::from_str(&self.team_token) {
                h.insert("x-genasis-team-token", v);
            }
        }
        h.insert(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        h
    }
}

fn slug_from_identifier(identifier: &str, fallback_name: &str) -> String {
    let from_id: String = identifier
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect::<String>()
        .to_ascii_lowercase();
    if !from_id.is_empty() {
        return from_id;
    }
    fallback_name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_ascii_lowercase()
}

#[async_trait]
impl PlaneProvider for TrialPlane {
    async fn health(&self) -> Result<Value> {
        let resp = self
            .client
            .get(self.url("/"))
            .send()
            .await
            .map_err(|e| Error::Provider(format!("trial plane health: {e}")))?;
        Ok(json!({
            "trial": true,
            "url": self.base_url,
            "status": resp.status().as_u16(),
        }))
    }

    async fn ensure_project(&self, name: &str, identifier: &str) -> Result<String> {
        let slug = slug_from_identifier(identifier, name);
        let body = json!({"slug": slug, "name": name});
        let resp = self
            .client
            .post(self.url("/api/plane/projects"))
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("trial plane ensure_project: {e}")))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(Error::Provider(format!(
                "trial plane ensure_project {status}: {text}"
            )));
        }
        let v: Value = resp
            .json()
            .await
            .map_err(|e| Error::Provider(format!("trial plane ensure_project json: {e}")))?;
        // Return the slug as the trait's "project_id" so subsequent calls
        // (create_issue, etc.) carry it through the same parameter.
        v.get("slug")
            .and_then(|x| x.as_str())
            .map(String::from)
            .ok_or_else(|| Error::Provider(format!("trial plane ensure_project: no slug in {v}")))
    }

    async fn ensure_label(&self, _project_id: &str, name: &str, _color: &str) -> Result<LabelRef> {
        // The trial sim does not model labels; return a stable stub so
        // callers that fan out over labels keep working.
        Ok(LabelRef {
            id: format!("trial-label-{name}"),
            name: name.into(),
        })
    }

    async fn create_cycle(
        &self,
        _project_id: &str,
        name: &str,
        _start: chrono::NaiveDate,
        _end: chrono::NaiveDate,
    ) -> Result<CycleRef> {
        Ok(CycleRef {
            id: format!("trial-cycle-{name}"),
            name: name.into(),
        })
    }

    async fn create_issue(
        &self,
        project_id: &str,
        title: &str,
        _description: &str,
    ) -> Result<IssueRef> {
        let body = json!({"project_slug": project_id, "title": title});
        let resp = self
            .client
            .post(self.url("/api/plane/issues"))
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("trial plane create_issue: {e}")))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(Error::Provider(format!(
                "trial plane create_issue {status}: {text}"
            )));
        }
        let v: Value = resp
            .json()
            .await
            .map_err(|e| Error::Provider(format!("trial plane create_issue json: {e}")))?;
        Ok(IssueRef {
            id: v
                .get("id")
                .and_then(|x| x.as_i64())
                .map(|i| i.to_string())
                .unwrap_or_default(),
            sequence_id: v.get("sequence_id").and_then(|x| x.as_u64()).unwrap_or(0),
            state: v
                .get("state")
                .and_then(|x| x.as_str())
                .unwrap_or_default()
                .into(),
        })
    }

    async fn transition(
        &self,
        _project_id: &str,
        issue_id: &str,
        state_id: &str,
        assignees: &[String],
    ) -> Result<()> {
        let mut body = json!({"state": state_id});
        if let Some(assignee) = assignees.first() {
            body["assignee"] = json!(assignee);
        }
        let resp = self
            .client
            .patch(self.url(&format!("/api/plane/issues/{issue_id}")))
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("trial plane transition: {e}")))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(Error::Provider(format!(
                "trial plane transition {status}: {text}"
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_passthrough_when_already_lowercase_alnum() {
        assert_eq!(slug_from_identifier("MYAPP", "irrelevant"), "myapp");
        assert_eq!(slug_from_identifier("my-app", "irrelevant"), "my-app");
    }

    #[test]
    fn slug_falls_back_to_name_when_identifier_empty() {
        assert_eq!(slug_from_identifier("", "Cool App"), "cool-app");
    }

    #[test]
    fn headers_include_team_token_when_set() {
        let p = TrialPlane::new("http://t", "sec", "abc123");
        let h = p.headers();
        assert_eq!(h.get("x-genasis-team-token").unwrap(), "abc123");
        assert_eq!(h.get("x-genasis-trial-secret").unwrap(), "sec");
    }

    #[test]
    fn headers_omit_team_token_when_empty() {
        let p = TrialPlane::new("http://t", "sec", "");
        let h = p.headers();
        assert!(h.get("x-genasis-team-token").is_none());
        assert_eq!(h.get("x-genasis-trial-secret").unwrap(), "sec");
    }

    #[test]
    fn headers_omit_secret_when_empty() {
        let p = TrialPlane::new("http://t", "", "abc123");
        let h = p.headers();
        assert_eq!(h.get("x-genasis-team-token").unwrap(), "abc123");
        assert!(h.get("x-genasis-trial-secret").is_none());
    }

    /// End-to-end smoke test against a running trial-app. Run with:
    /// `TRIAL_BASE=http://localhost:3000 TRIAL_SECRET=trialsecret \
    ///   TRIAL_TEAM_TOKEN=<hex> \
    ///   cargo test -p genasis-providers --lib trial_e2e -- --ignored --nocapture`
    #[tokio::test]
    #[ignore]
    async fn trial_e2e_create_then_transition() {
        let base = std::env::var("TRIAL_BASE").unwrap_or_else(|_| "http://localhost:3000".into());
        let secret = std::env::var("TRIAL_SECRET").unwrap_or_else(|_| "trialsecret".into());
        let team_token = std::env::var("TRIAL_TEAM_TOKEN").unwrap_or_default();
        let p = TrialPlane::new(base, secret, team_token);
        let project = p
            .ensure_project("Rust E2E", "RUSTE2E")
            .await
            .expect("ensure_project");
        let issue = p
            .create_issue(&project, "Hello from rust", "")
            .await
            .expect("create_issue");
        assert_eq!(issue.state, "todo");
        p.transition(&project, &issue.id, "done", &[])
            .await
            .expect("transition");
    }
}
