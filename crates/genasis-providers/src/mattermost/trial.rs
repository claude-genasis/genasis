//! Trial flavor — forwards every Mattermost trait call to the trial-app's
//! `/api/mattermost/*` HTTP endpoints.

use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use serde_json::{json, Value};

use genasis_core::config::slugify;
use genasis_core::error::{Error, Result};

use super::{ChannelRef, HumanUserRef, HumanUserSpec, MattermostProvider, PostRef};

/// Default actor used when the trait does not surface one (the upstream
/// trait inherits the actor from the bot's auth token in real
/// Mattermost, but the trial-app needs an explicit `actor` field).
/// Override via `GENASIS_TRIAL_ACTOR` env var.
const DEFAULT_ACTOR: &str = "agent";

#[derive(Debug, Clone)]
pub struct TrialMattermost {
    base_url: String,
    secret: String,
    team_token: String,
    client: Client,
}

impl TrialMattermost {
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
        // tenant's sim namespace. Empty token = trial-app falls
        // through to DEFAULT_TEAM_TOKEN, matching pre-ADR-016
        // behaviour.
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

    fn actor() -> String {
        std::env::var("GENASIS_TRIAL_ACTOR").unwrap_or_else(|_| DEFAULT_ACTOR.to_string())
    }

    /// v0.5.5 D-001: idempotent get-or-create of a sim channel via the
    /// auth-free `/api/trial/bootstrap` endpoint. Looks up the team's
    /// project name from `/api/trial/team-app/status`, then POSTs a
    /// bootstrap request with the (project, channel) pair. Bootstrap is
    /// idempotent — re-running with the same payload returns the
    /// existing rows untouched — so this safely replaces the legacy
    /// `/api/mattermost/channels` POST against deployments that still
    /// require TRIAL_SHARED_SECRET. Returns `Err` when status probe
    /// fails or team isn't yet bootstrapped (caller falls back to the
    /// legacy path with its own better error message).
    async fn bootstrap_lookup_channel(&self, name: &str, display_name: &str) -> Result<ChannelRef> {
        let status_url = format!(
            "{}/api/trial/team-app/status?team={}",
            self.base_url, self.team_token
        );
        let s = self
            .client
            .get(&status_url)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("trial mm status probe: {e}")))?;
        if !s.status().is_success() {
            return Err(Error::Provider(format!(
                "trial mm status probe: {}",
                s.status()
            )));
        }
        let sv: Value = s
            .json()
            .await
            .map_err(|e| Error::Provider(format!("trial mm status json: {e}")))?;
        let team_exists = sv
            .get("team_exists")
            .and_then(|x| x.as_bool())
            .unwrap_or(false);
        if !team_exists {
            return Err(Error::Provider(
                "trial mm bootstrap-lookup: team_exists=false".into(),
            ));
        }
        let project_name = sv
            .get("project_name")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        if project_name.is_empty() {
            return Err(Error::Provider(
                "trial mm bootstrap-lookup: empty project_name in status".into(),
            ));
        }
        let project_slug = slugify(&project_name);

        // Bootstrap is idempotent for (team_token, project, channels).
        // We pass the *target* channel as the lookup key — if the row
        // already exists (which it will when `try_bootstrap_trial_app`
        // already ran with the same channel name), the server returns
        // it; otherwise it's created on the fly.
        let body = json!({
            "team_token": self.team_token,
            "project": {"slug": project_slug, "name": project_name},
            "channels": [{
                "key": name,
                "name": name,
                "display_name": display_name,
            }],
        });
        let r = self
            .client
            .post(format!("{}/api/trial/bootstrap", self.base_url))
            .header(
                reqwest::header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            )
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("trial mm bootstrap-lookup: {e}")))?;
        if !r.status().is_success() {
            let status = r.status();
            let text = r.text().await.unwrap_or_default();
            return Err(Error::Provider(format!(
                "trial mm bootstrap-lookup {status}: {text}"
            )));
        }
        let rv: Value = r
            .json()
            .await
            .map_err(|e| Error::Provider(format!("trial mm bootstrap-lookup json: {e}")))?;
        let channels = rv
            .get("channels")
            .and_then(|x| x.as_array())
            .ok_or_else(|| {
                Error::Provider("trial mm bootstrap-lookup: response missing channels".into())
            })?;
        let ch = channels
            .iter()
            .find(|c| c.get("name").and_then(|n| n.as_str()) == Some(name))
            .ok_or_else(|| {
                Error::Provider(format!(
                    "trial mm bootstrap-lookup: channel `{name}` not in response"
                ))
            })?;
        Ok(ChannelRef {
            id: ch
                .get("id")
                .and_then(|i| i.as_i64())
                .map(|i| i.to_string())
                .unwrap_or_default(),
            name: ch
                .get("name")
                .and_then(|x| x.as_str())
                .unwrap_or(name)
                .into(),
        })
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
        // v0.5.5 D-001: Route through `/api/trial/bootstrap` (auth-free,
        // idempotent — ADR-016 §4) instead of the legacy
        // `/api/mattermost/channels` POST. Old deployed trial-app
        // instances still gate `/api/mattermost/channels` behind
        // TRIAL_SHARED_SECRET, but `/api/trial/bootstrap` is the
        // current token-as-capability route that all deployed versions
        // accept. We use bootstrap's `channels[]` slot as a
        // get-or-create lookup keyed on `(team_token, channel name)`.
        // Falls back to the legacy POST when the bootstrap-as-lookup
        // path is unusable (e.g. team_token not set, status probe
        // fails, or bootstrap rejects).
        if !self.team_token.is_empty() {
            if let Ok(ch) = self.bootstrap_lookup_channel(name, display_name).await {
                return Ok(ch);
            }
        }

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
            if status.as_u16() == 401 {
                return Err(Error::Provider(format!(
                    "trial mm ensure_channel: 401 from {}. \
                     The deployed trial-app is older than this binary's \
                     auth contract (see README §'Known limitations'). \
                     Ask the operator to redeploy, or self-host the \
                     trial-app from agents-pool.",
                    self.base_url
                )));
            }
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
        let cid: i64 = channel_id.parse().map_err(|e| {
            Error::Provider(format!(
                "trial mm post_root: bad channel_id `{channel_id}`: {e}"
            ))
        })?;
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

    async fn post_thread(&self, channel_id: &str, root_id: &str, message: &str) -> Result<PostRef> {
        let cid: i64 = channel_id.parse().map_err(|e| {
            Error::Provider(format!(
                "trial mm post_thread: bad channel_id `{channel_id}`: {e}"
            ))
        })?;
        let rid: i64 = root_id.parse().map_err(|e| {
            Error::Provider(format!(
                "trial mm post_thread: bad root_id `{root_id}`: {e}"
            ))
        })?;
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

    async fn ensure_human_user(
        &self,
        spec: &HumanUserSpec,
        _team_id: Option<&str>,
    ) -> Result<HumanUserRef> {
        // Trial sim has no real auth; return a stable deterministic stub
        // so the wizard / cmd_humans flow can be exercised end-to-end
        // without a real Mattermost. No temp password since there is no
        // login.
        Ok(HumanUserRef {
            user_id: format!("trial-human-{}", spec.username),
            username: spec.username.clone(),
            email: spec.email.clone(),
            temp_password: None,
            must_change_password: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headers_include_team_token_when_set() {
        let p = TrialMattermost::new("http://t", "sec", "abc123");
        let h = p.headers();
        assert_eq!(h.get("x-genasis-team-token").unwrap(), "abc123");
        assert_eq!(h.get("x-genasis-trial-secret").unwrap(), "sec");
    }

    #[test]
    fn headers_omit_team_token_when_empty() {
        let p = TrialMattermost::new("http://t", "sec", "");
        let h = p.headers();
        assert!(h.get("x-genasis-team-token").is_none());
    }

    /// End-to-end smoke test against a running trial-app. Run with:
    /// `TRIAL_BASE=http://localhost:3000 TRIAL_SECRET=trialsecret \
    ///   TRIAL_TEAM_TOKEN=<hex> \
    ///   cargo test -p genasis-providers --lib mm_trial_e2e -- --ignored --nocapture`
    #[tokio::test]
    #[ignore]
    async fn mm_trial_e2e_channel_then_post() {
        let base = std::env::var("TRIAL_BASE").unwrap_or_else(|_| "http://localhost:3000".into());
        let secret = std::env::var("TRIAL_SECRET").unwrap_or_else(|_| "trialsecret".into());
        let team_token = std::env::var("TRIAL_TEAM_TOKEN").unwrap_or_default();
        let p = TrialMattermost::new(base, secret, team_token);
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
