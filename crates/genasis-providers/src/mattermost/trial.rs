//! Trial flavor — forwards every Mattermost trait call to the trial-app's
//! `/api/mattermost/*` HTTP endpoints.

use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use serde_json::{json, Value};

use genasis_core::error::{Error, Result};

use super::{ChannelRef, MattermostProvider, PostRef};

/// Default actor used when the trait does not surface one (the upstream
/// trait inherits the actor from the bot's auth token in real
/// Mattermost, but the trial-app needs an explicit `actor` field).
/// Override via `GENASIS_TRIAL_ACTOR` env var.
const DEFAULT_ACTOR: &str = "agent";

#[derive(Debug, Clone)]
pub struct TrialMattermost {
    base_url: String,
    secret: String,
    client: Client,
}

impl TrialMattermost {
    pub fn new(base_url: impl Into<String>, secret: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            secret: secret.into(),
            client: Client::new(),
        }
    }

    fn url(&self, suffix: &str) -> String {
        format!("{}{}", self.base_url, suffix)
    }

    fn headers(&self) -> HeaderMap {
        let mut h = HeaderMap::new();
        if let Ok(v) = HeaderValue::from_str(&self.secret) {
            h.insert("x-genasis-trial-secret", v);
        }
        h.insert(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        h
    }

    fn actor() -> String {
        std::env::var("GENASIS_TRIAL_ACTOR")
            .unwrap_or_else(|_| DEFAULT_ACTOR.to_string())
    }
}

#[async_trait]
impl MattermostProvider for TrialMattermost {
    async fn ping(&self) -> Result<Value> {
        let resp = self
            .client
            .get(self.url("/"))
            .send()
            .await
            .map_err(|e| Error::Provider(format!("trial mm ping: {e}")))?;
        Ok(json!({
            "trial": true,
            "url": self.base_url,
            "status": resp.status().as_u16(),
        }))
    }

    async fn ensure_channel(
        &self,
        _team_id: &str,
        name: &str,
        display_name: &str,
    ) -> Result<ChannelRef> {
        let body = json!({"name": name, "display_name": display_name});
        let resp = self
            .client
            .post(self.url("/api/mattermost/channels"))
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("trial mm ensure_channel: {e}")))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(Error::Provider(format!(
                "trial mm ensure_channel {status}: {text}"
            )));
        }
        let v: Value = resp
            .json()
            .await
            .map_err(|e| Error::Provider(format!("trial mm ensure_channel json: {e}")))?;
        Ok(ChannelRef {
            id: v
                .get("id")
                .and_then(|x| x.as_i64())
                .map(|i| i.to_string())
                .unwrap_or_default(),
            name: v
                .get("name")
                .and_then(|x| x.as_str())
                .unwrap_or(name)
                .into(),
        })
    }

    async fn post_root(&self, channel_id: &str, message: &str) -> Result<PostRef> {
        let cid: i64 = channel_id
            .parse()
            .map_err(|e| Error::Provider(format!("trial mm post_root: bad channel_id `{channel_id}`: {e}")))?;
        let body = json!({
            "channel_id": cid,
            "actor": Self::actor(),
            "message": message,
        });
        let resp = self
            .client
            .post(self.url("/api/mattermost/posts"))
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("trial mm post_root: {e}")))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(Error::Provider(format!(
                "trial mm post_root {status}: {text}"
            )));
        }
        let v: Value = resp
            .json()
            .await
            .map_err(|e| Error::Provider(format!("trial mm post_root json: {e}")))?;
        Ok(PostRef {
            id: v
                .get("id")
                .and_then(|x| x.as_i64())
                .map(|i| i.to_string())
                .unwrap_or_default(),
        })
    }

    async fn post_thread(
        &self,
        channel_id: &str,
        root_id: &str,
        message: &str,
    ) -> Result<PostRef> {
        let cid: i64 = channel_id
            .parse()
            .map_err(|e| Error::Provider(format!("trial mm post_thread: bad channel_id `{channel_id}`: {e}")))?;
        let rid: i64 = root_id
            .parse()
            .map_err(|e| Error::Provider(format!("trial mm post_thread: bad root_id `{root_id}`: {e}")))?;
        let body = json!({
            "channel_id": cid,
            "root_id": rid,
            "actor": Self::actor(),
            "message": message,
        });
        let resp = self
            .client
            .post(self.url("/api/mattermost/posts"))
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("trial mm post_thread: {e}")))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(Error::Provider(format!(
                "trial mm post_thread {status}: {text}"
            )));
        }
        let v: Value = resp
            .json()
            .await
            .map_err(|e| Error::Provider(format!("trial mm post_thread json: {e}")))?;
        Ok(PostRef {
            id: v
                .get("id")
                .and_then(|x| x.as_i64())
                .map(|i| i.to_string())
                .unwrap_or_default(),
        })
    }

    async fn ensure_bot(&self, username: &str, _display_name: &str) -> Result<String> {
        // Trial sim has no bot identities; return a stable stub so callers
        // that record the bot id can still proceed.
        Ok(format!("trial-bot-{username}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// End-to-end smoke test against a running trial-app. Run with:
    /// `TRIAL_BASE=http://localhost:3000 TRIAL_SECRET=trialsecret \
    ///   cargo test -p genasis-providers --lib mm_trial_e2e -- --ignored --nocapture`
    #[tokio::test]
    #[ignore]
    async fn mm_trial_e2e_channel_then_post() {
        let base = std::env::var("TRIAL_BASE").unwrap_or_else(|_| "http://localhost:3000".into());
        let secret = std::env::var("TRIAL_SECRET").unwrap_or_else(|_| "trialsecret".into());
        let p = TrialMattermost::new(base, secret);
        let ch = p
            .ensure_channel("ignored-team-id", "scrum-rust-e2e", "Rust E2E Scrum")
            .await
            .expect("ensure_channel");
        assert_eq!(ch.name, "scrum-rust-e2e");
        let post = p
            .post_root(&ch.id, "Hello from the Rust trial provider")
            .await
            .expect("post_root");
        assert!(!post.id.is_empty());
        let thread = p
            .post_thread(&ch.id, &post.id, "thread reply")
            .await
            .expect("post_thread");
        assert!(!thread.id.is_empty());
    }
}
