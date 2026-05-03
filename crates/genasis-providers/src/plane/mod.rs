//! Plane provider — flavor-aware (`upstream` / `agent-aware` / `auto`).
//!
//! The trait surface is intentionally narrow — only the calls Genasis itself
//! performs (`init` provisioning, lifecycle transitions, label / cycle
//! management). Power users can drive Plane with `curl` directly.
//!
//! Flavor differences are absorbed inside each impl; callers do not branch.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use genasis_core::error::Result;

pub mod detect;
pub mod factory;
pub mod agent_aware;
pub mod upstream;
pub mod user_provisioner;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueRef {
    pub id: String,
    pub sequence_id: u64,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelRef {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleRef {
    pub id: String,
    pub name: String,
}

#[async_trait]
pub trait PlaneProvider: Send + Sync {
    /// `GET /api/v1/health` (or equivalent) — used by the auto detector.
    async fn health(&self) -> Result<serde_json::Value>;

    /// Create or upsert a project; returns the project UUID.
    async fn ensure_project(&self, name: &str, identifier: &str) -> Result<String>;

    /// Create or upsert a label and return its UUID.
    async fn ensure_label(&self, project_id: &str, name: &str, color: &str) -> Result<LabelRef>;

    /// Create a cycle (Sprint) and return its UUID.
    async fn create_cycle(
        &self,
        project_id: &str,
        name: &str,
        start: chrono::NaiveDate,
        end: chrono::NaiveDate,
    ) -> Result<CycleRef>;

    /// Create an issue with title + description; returns issue ref.
    async fn create_issue(
        &self,
        project_id: &str,
        title: &str,
        description: &str,
    ) -> Result<IssueRef>;

    /// Transition state and (optionally) set assignees in one call.
    async fn transition(
        &self,
        project_id: &str,
        issue_id: &str,
        state_id: &str,
        assignees: &[String],
    ) -> Result<()>;
}

pub use factory::{build, FlavorChoice};
