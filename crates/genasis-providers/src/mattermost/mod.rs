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
}

pub use factory::{build, FlavorChoice};
