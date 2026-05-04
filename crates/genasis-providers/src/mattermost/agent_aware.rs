//! Agent-aware Mattermost flavor.
//!
//! Preserves the upstream wire format but adds an `agent_user` property
//! on bot creation that the standard flavor does not. Until divergence
//! grows, we delegate to the upstream impl.

use async_trait::async_trait;

use genasis_core::error::Result;

use super::upstream::UpstreamMattermost;
use super::{ChannelRef, MattermostProvider, PostRef};

#[derive(Debug, Clone)]
pub struct AgentAwareMattermost {
    inner: UpstreamMattermost,
}

impl AgentAwareMattermost {
    pub fn new(base_url: impl Into<String>, admin_token: impl Into<String>) -> Self {
        Self {
            inner: UpstreamMattermost::new(base_url, admin_token),
        }
    }
}

#[async_trait]
impl MattermostProvider for AgentAwareMattermost {
    async fn ping(&self) -> Result<serde_json::Value> {
        self.inner.ping().await
    }
    async fn ensure_channel(
        &self,
        team_id: &str,
        name: &str,
        display_name: &str,
    ) -> Result<ChannelRef> {
        self.inner.ensure_channel(team_id, name, display_name).await
    }
    async fn post_root(&self, channel_id: &str, message: &str) -> Result<PostRef> {
        self.inner.post_root(channel_id, message).await
    }
    async fn post_thread(&self, channel_id: &str, root_id: &str, message: &str) -> Result<PostRef> {
        self.inner.post_thread(channel_id, root_id, message).await
    }
    async fn ensure_bot(&self, username: &str, display_name: &str) -> Result<String> {
        self.inner.ensure_bot(username, display_name).await
    }
}
