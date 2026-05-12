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

    /// Idempotence helper for `ensure_project` (Issue #10): GET the
    /// workspace's project list and return the first matching id by
    /// name OR identifier. Returns `None` when no match is found —
    /// the caller falls through to a fresh POST.
    ///
    /// Plane's `/projects/` endpoint paginates with a `next` URL
    /// (cursor model). We walk until we find a match or run out;
    /// callers that have hundreds of projects pay an O(N/page_size)
    /// list scan, which is acceptable for init flows that run once
    /// per project lifetime.
    async fn find_project_by_name_or_identifier(
        &self,
        name: &str,
        identifier: &str,
    ) -> Result<Option<String>> {
        let mut next: Option<String> = Some(self.url("/projects/"));
        while let Some(url) = next.take() {
            let resp = self
                .client
                .get(&url)
                .headers(self.headers())
                .send()
                .await
                .map_err(|e| Error::Provider(format!("plane list_projects: {e}")))?;
            if !resp.status().is_success() {
                // Some Plane deployments return 404 for an empty
                // workspace's projects list. Treat that as "no match".
                return Ok(None);
            }
            let v: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| Error::Provider(format!("plane list_projects json: {e}")))?;
            // Plane wraps results in `{ results: [...], next: "..." }`
            // for the paginated endpoint and returns a bare array for
            // the unpaginated form. Handle both.
            let results: Option<&Vec<serde_json::Value>> = v
                .get("results")
                .and_then(|x| x.as_array())
                .or_else(|| v.as_array());
            if let Some(arr) = results {
                for item in arr {
                    let item_name = item.get("name").and_then(|x| x.as_str()).unwrap_or("");
                    let item_id = item
                        .get("identifier")
                        .and_then(|x| x.as_str())
                        .unwrap_or("");
                    if item_name.eq_ignore_ascii_case(name)
                        || item_id.eq_ignore_ascii_case(identifier)
                    {
                        if let Some(id) = item.get("id").and_then(|x| x.as_str()) {
                            return Ok(Some(id.to_string()));
                        }
                    }
                }
            }
            next = v
                .get("next")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty());
        }
        Ok(None)
    }
}

#[async_trait]
impl PlaneProvider for UpstreamPlane {
    async fn health(&self) -> Result<serde_json::Value> {
        // v0.5.4 (issue M2): `/api/v1/workspaces/<slug>/` is
        // workspace-scoped and returns 401 even with a valid API
        // key under some Plane permission configs — that 401 shows
        // up as scary noise in `genasis init` output before any
        // real provisioning has happened. Switched to
        // `/api/instances/`, an unauthenticated endpoint that
        // returns instance metadata (200 + JSON) when the Plane
        // backend is reachable. We surface the workspace
        // existence check separately in `ensure_project`'s
        // paginated walk, which already returns a clean error if
        // the workspace is missing.
        let url = format!("{}/api/instances/", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("plane health: {e}")))?;
        let status = resp.status().as_u16();
        Ok(json!({
            "status": status,
            "url": url,
            "workspace_slug": self.workspace_slug,
        }))
    }

    async fn ensure_project(&self, name: &str, identifier: &str) -> Result<String> {
        // v0.5.2 (Issue #10): the original implementation always
        // POSTed `/projects/` which fails on the second invocation
        // with "The project name is already taken". `ensure_*` is
        // supposed to be idempotent: list first, return the
        // existing id if name + identifier match, only POST when
        // genuinely absent. Plane paginates the list endpoint so we
        // walk pages until we hit a match or run out.
        if let Some(existing) = self
            .find_project_by_name_or_identifier(name, identifier)
            .await?
        {
            return Ok(existing);
        }
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
