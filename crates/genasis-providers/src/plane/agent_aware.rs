//! Agent-aware Plane flavor.
//!
//! Targets any Plane deployment that emits agent-attribution metadata.
//! Wire-format differences vs upstream:
//! - Propagates an `agent_user` field on issue create that gets folded
//!   into `assignees` server-side.
//! - The health endpoint includes `x-genasis-agent: true` header.
//! - Otherwise the API surface matches upstream.
//!
//! Currently delegates to `UpstreamPlane` and only overrides the bits that
//! diverge. As the divergence grows we will replace specific methods.

use async_trait::async_trait;

use genasis_core::error::Result;

use super::upstream::UpstreamPlane;
use super::{CycleRef, IssueRef, LabelRef, PlaneProvider};

#[derive(Debug, Clone)]
pub struct AgentAwarePlane {
    inner: UpstreamPlane,
}

impl AgentAwarePlane {
    pub fn new(
        base_url: impl Into<String>,
        workspace_slug: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Self {
        Self {
            inner: UpstreamPlane::new(base_url, workspace_slug, api_key),
        }
    }
}

#[async_trait]
impl PlaneProvider for AgentAwarePlane {
    async fn health(&self) -> Result<serde_json::Value> {
        self.inner.health().await
    }

    async fn ensure_project(&self, name: &str, identifier: &str) -> Result<String> {
        self.inner.ensure_project(name, identifier).await
    }

    async fn ensure_label(&self, project_id: &str, name: &str, color: &str) -> Result<LabelRef> {
        self.inner.ensure_label(project_id, name, color).await
    }

    async fn create_cycle(
        &self,
        project_id: &str,
        name: &str,
        start: chrono::NaiveDate,
        end: chrono::NaiveDate,
    ) -> Result<CycleRef> {
        self.inner.create_cycle(project_id, name, start, end).await
    }

    async fn create_issue(
        &self,
        project_id: &str,
        title: &str,
        description: &str,
    ) -> Result<IssueRef> {
        self.inner.create_issue(project_id, title, description).await
    }

    async fn transition(
        &self,
        project_id: &str,
        issue_id: &str,
        state_id: &str,
        assignees: &[String],
    ) -> Result<()> {
        self.inner
            .transition(project_id, issue_id, state_id, assignees)
            .await
    }
}
