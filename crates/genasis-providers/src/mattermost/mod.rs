//! Mattermost provider — flavor-aware (`upstream` / `agent-aware` / `auto`).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use genasis_core::error::Result;

pub mod agent_aware;
pub mod detect;
pub mod factory;
pub mod trial;
pub mod upstream;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelRef {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostRef {
    pub id: String,
}

/// Result of provisioning (or finding) a human user account on Mattermost.
/// `temp_password` is `Some` only when a brand-new account was created and
/// the caller must surface the password to the user (or store it in the
/// humans-lock file). For idempotent re-runs against an existing user the
/// field is `None`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanUserRef {
    pub user_id: String,
    pub username: String,
    pub email: String,
    /// `Some` on first creation, `None` if the user already existed.
    pub temp_password: Option<String>,
    /// True if Mattermost flagged the account so the user must change
    /// their password on first login. Populated by upstream on create.
    pub must_change_password: bool,
}

/// Spec passed to `ensure_human_user`. The provider is responsible for
/// translating these into the right wire format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanUserSpec {
    pub email: String,
    pub username: String,
    pub display_name: String,
    /// Optional split into first/last; if empty, providers fall back to
    /// `display_name`.
    #[serde(default)]
    pub first_name: String,
    #[serde(default)]
    pub last_name: String,
    /// Locale ("en" / "ko") — applied as Mattermost user.locale where
    /// supported.
    #[serde(default)]
    pub locale: String,
}

#[async_trait]
pub trait MattermostProvider: Send + Sync {
    async fn ping(&self) -> Result<serde_json::Value>;

    async fn ensure_channel(
        &self,
        team_id: &str,
        name: &str,
        display_name: &str,
    ) -> Result<ChannelRef>;

    async fn post_root(&self, channel_id: &str, message: &str) -> Result<PostRef>;
    async fn post_thread(&self, channel_id: &str, root_id: &str, message: &str) -> Result<PostRef>;

    async fn ensure_bot(&self, username: &str, display_name: &str) -> Result<String>;

    /// Idempotently provision a human user account. If a user with the
    /// given email already exists, return its ID with `temp_password =
    /// None`. Otherwise create a new account, optionally add it to the
    /// project team (if `team_id` is `Some`), and return the temporary
    /// password the caller should hand to the human (one-time).
    async fn ensure_human_user(
        &self,
        spec: &HumanUserSpec,
        team_id: Option<&str>,
    ) -> Result<HumanUserRef>;
}

pub use factory::{build, FlavorChoice};
