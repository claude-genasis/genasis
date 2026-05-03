//! Upstream Mattermost (mattermost.com / standard self-hosted) flavor.

use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;
use serde_json::json;

use genasis_core::error::{Error, Result};

use super::{ChannelRef, MattermostProvider, PostRef};

#[derive(Debug, Clone)]
pub struct UpstreamMattermost {
    base_url: String,
    admin_token: String,
    client: Client,
}

impl UpstreamMattermost {
    pub fn new(base_url: impl Into<String>, admin_token: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            admin_token: admin_token.into(),
            client: Client::new(),
        }
    }

    fn url(&self, suffix: &str) -> String {
        format!("{}/api/v4{suffix}", self.base_url)
    }

    fn headers(&self) -> HeaderMap {
        let mut h = HeaderMap::new();
        if let Ok(v) = HeaderValue::from_str(&format!("Bearer {}", self.admin_token)) {
            h.insert(AUTHORIZATION, v);
        }
        h.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        h
    }
}

#[async_trait]
impl MattermostProvider for UpstreamMattermost {
    async fn ping(&self) -> Result<serde_json::Value> {
        let url = format!("{}/api/v4/system/ping", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("mm ping: {e}")))?;
        let text = resp.text().await.unwrap_or_default();
        Ok(serde_json::from_str(&text).unwrap_or_else(|_| json!({"raw": text})))
    }

    async fn ensure_channel(
        &self,
        team_id: &str,
        name: &str,
        display_name: &str,
    ) -> Result<ChannelRef> {
        let body = json!({
            "team_id": team_id,
            "name": name,
            "display_name": display_name,
            "type": "O",
        });
        let resp = self
            .client
            .post(self.url("/channels"))
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("mm create_channel: {e}")))?;
        let v: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| Error::Provider(format!("mm channel json: {e}")))?;
        Ok(ChannelRef {
            id: v
                .get("id")
                .and_then(|x| x.as_str())
                .unwrap_or_default()
                .into(),
            name: name.into(),
        })
    }

    async fn post_root(&self, channel_id: &str, message: &str) -> Result<PostRef> {
        let body = json!({"channel_id": channel_id, "message": message});
        post(&self.client, &self.url("/posts"), self.headers(), &body).await
    }

    async fn post_thread(
        &self,
        channel_id: &str,
        root_id: &str,
        message: &str,
    ) -> Result<PostRef> {
        let body = json!({
            "channel_id": channel_id,
            "root_id": root_id,
            "message": message,
        });
        post(&self.client, &self.url("/posts"), self.headers(), &body).await
    }

    async fn ensure_bot(&self, username: &str, display_name: &str) -> Result<String> {
        let body = json!({"username": username, "display_name": display_name});
        let resp = self
            .client
            .post(self.url("/bots"))
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("mm bot: {e}")))?;
        let v: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| Error::Provider(format!("mm bot json: {e}")))?;
        Ok(v.get("user_id")
            .or_else(|| v.get("id"))
            .and_then(|x| x.as_str())
            .unwrap_or_default()
            .into())
    }
}

async fn post(
    client: &Client,
    url: &str,
    headers: HeaderMap,
    body: &serde_json::Value,
) -> Result<PostRef> {
    let resp = client
        .post(url)
        .headers(headers)
        .json(body)
        .send()
        .await
        .map_err(|e| Error::Provider(format!("mm post: {e}")))?;
    let v: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| Error::Provider(format!("mm post json: {e}")))?;
    Ok(PostRef {
        id: v
            .get("id")
            .and_then(|x| x.as_str())
            .unwrap_or_default()
            .into(),
    })
}
