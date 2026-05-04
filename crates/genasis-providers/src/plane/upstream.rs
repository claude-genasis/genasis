//! Upstream Plane (makeplane / plane.so SaaS) flavor.

use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use serde_json::json;

use genasis_core::error::{Error, Result};

use super::{CycleRef, IssueRef, LabelRef, PlaneProvider};

#[derive(Debug, Clone)]
pub struct UpstreamPlane {
    base_url: String,
    workspace_slug: String,
    api_key: String,
    client: Client,
}

impl UpstreamPlane {
    pub fn new(
        base_url: impl Into<String>,
        workspace_slug: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            workspace_slug: workspace_slug.into(),
            api_key: api_key.into(),
            client: Client::new(),
        }
    }

    fn url(&self, suffix: &str) -> String {
        format!(
            "{}/api/v1/workspaces/{}{}",
            self.base_url, self.workspace_slug, suffix
        )
    }

    fn headers(&self) -> HeaderMap {
        let mut h = HeaderMap::new();
        if let Ok(v) = HeaderValue::from_str(&self.api_key) {
            h.insert("x-api-key", v);
        }
        h.insert(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        h
    }
}

#[async_trait]
impl PlaneProvider for UpstreamPlane {
    async fn health(&self) -> Result<serde_json::Value> {
        let url = format!("{}/api/v1/health/", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("plane health: {e}")))?;
        let text = resp.text().await.unwrap_or_default();
        Ok(serde_json::from_str(&text).unwrap_or_else(|_| json!({"raw": text})))
    }

    async fn ensure_project(&self, name: &str, identifier: &str) -> Result<String> {
        let body = json!({"name": name, "identifier": identifier});
        let resp = self
            .client
            .post(self.url("/projects/"))
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("plane create_project: {e}")))?;
        let v: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| Error::Provider(format!("plane create_project json: {e}")))?;
        v.get("id")
            .and_then(|x| x.as_str())
            .map(String::from)
            .ok_or_else(|| Error::Provider(format!("plane create_project: no id in {v}")))
    }

    async fn ensure_label(&self, project_id: &str, name: &str, color: &str) -> Result<LabelRef> {
        let body = json!({"name": name, "color": color});
        let resp = self
            .client
            .post(self.url(&format!("/projects/{project_id}/labels/")))
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("plane create_label: {e}")))?;
        let v: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| Error::Provider(format!("plane label json: {e}")))?;
        Ok(LabelRef {
            id: v
                .get("id")
                .and_then(|x| x.as_str())
                .unwrap_or_default()
                .into(),
            name: name.into(),
        })
    }

    async fn create_cycle(
        &self,
        project_id: &str,
        name: &str,
        start: chrono::NaiveDate,
        end: chrono::NaiveDate,
    ) -> Result<CycleRef> {
        let body = json!({
            "name": name,
            "start_date": start.to_string(),
            "end_date": end.to_string(),
        });
        let resp = self
            .client
            .post(self.url(&format!("/projects/{project_id}/cycles/")))
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("plane create_cycle: {e}")))?;
        let v: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| Error::Provider(format!("plane cycle json: {e}")))?;
        Ok(CycleRef {
            id: v
                .get("id")
                .and_then(|x| x.as_str())
                .unwrap_or_default()
                .into(),
            name: name.into(),
        })
    }

    async fn create_issue(
        &self,
        project_id: &str,
        title: &str,
        description: &str,
    ) -> Result<IssueRef> {
        let body = json!({"name": title, "description_html": description});
        let resp = self
            .client
            .post(self.url(&format!("/projects/{project_id}/issues/")))
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("plane create_issue: {e}")))?;
        let v: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| Error::Provider(format!("plane issue json: {e}")))?;
        Ok(IssueRef {
            id: v
                .get("id")
                .and_then(|x| x.as_str())
                .unwrap_or_default()
                .into(),
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
        project_id: &str,
        issue_id: &str,
        state_id: &str,
        assignees: &[String],
    ) -> Result<()> {
        let body = json!({"state": state_id, "assignees": assignees});
        self.client
            .patch(self.url(&format!("/projects/{project_id}/issues/{issue_id}/")))
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("plane transition: {e}")))?;
        Ok(())
    }
}
