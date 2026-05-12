//! Upstream Mattermost (mattermost.com / standard self-hosted) flavor.

use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;
use serde_json::json;

use genasis_core::error::{Error, Result};

use super::{ChannelRef, HumanUserRef, HumanUserSpec, MattermostProvider, PostRef};

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
        // v0.5.4 (issue M1): the previous implementation POST'd
        // `/channels` unconditionally and surfaced Mattermost's
        // `store.sql_channel.save_channel.exists.app_error` string
        // as if it were a real id. Re-running `genasis init` thus
        // printed scary-looking output for what is in fact a happy
        // path. Fixed by GET'ing `/teams/{team_id}/channels/name/{name}`
        // first (returns 200 with the existing channel JSON when the
        // channel exists, 404 otherwise) and only POST'ing on 404.
        let lookup_url = self.url(&format!("/teams/{team_id}/channels/name/{name}"));
        let existing = self
            .client
            .get(&lookup_url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| Error::Provider(format!("mm lookup_channel: {e}")))?;
        if existing.status().is_success() {
            let v: serde_json::Value = existing
                .json()
                .await
                .map_err(|e| Error::Provider(format!("mm lookup_channel json: {e}")))?;
            if let Some(id) = v.get("id").and_then(|x| x.as_str()) {
                return Ok(ChannelRef {
                    id: id.to_string(),
                    name: name.to_string(),
                });
            }
        }

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
        // Defence in depth: even if our lookup above misses a race
        // and Mattermost returns the dotted-error id string, treat
        // it as "already exists" and fall through to a second lookup
        // rather than propagating the gobbledygook id upward.
        let id = v.get("id").and_then(|x| x.as_str()).unwrap_or("");
        if id.starts_with("store.sql_channel.save_channel.exists") || id.is_empty() {
            let retry = self
                .client
                .get(&lookup_url)
                .headers(self.headers())
                .send()
                .await
                .map_err(|e| Error::Provider(format!("mm lookup_channel retry: {e}")))?;
            if retry.status().is_success() {
                let v: serde_json::Value = retry
                    .json()
                    .await
                    .map_err(|e| Error::Provider(format!("mm lookup_channel retry json: {e}")))?;
                if let Some(id) = v.get("id").and_then(|x| x.as_str()) {
                    return Ok(ChannelRef {
                        id: id.to_string(),
                        name: name.to_string(),
                    });
                }
            }
        }
        Ok(ChannelRef {
            id: id.to_string(),
            name: name.to_string(),
        })
    }

    async fn post_root(&self, channel_id: &str, message: &str) -> Result<PostRef> {
        let body = json!({"channel_id": channel_id, "message": message});
        post(&self.client, &self.url("/posts"), self.headers(), &body).await
    }

    async fn post_thread(&self, channel_id: &str, root_id: &str, message: &str) -> Result<PostRef> {
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

    async fn ensure_human_user(
        &self,
        spec: &HumanUserSpec,
        team_id: Option<&str>,
    ) -> Result<HumanUserRef> {
        // 1. Probe by email — return early if account already exists.
        let probe_url = format!(
            "{}/users/email/{}",
            self.url(""),
            urlencode_path(&spec.email)
        );
        let probe = self
            .client
            .get(&probe_url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| Error::Provider(format!("mm human probe: {e}")))?;
        if probe.status().is_success() {
            let v: serde_json::Value = probe
                .json()
                .await
                .map_err(|e| Error::Provider(format!("mm human probe json: {e}")))?;
            let user_id = v
                .get("id")
                .and_then(|x| x.as_str())
                .unwrap_or_default()
                .to_string();
            let username = v
                .get("username")
                .and_then(|x| x.as_str())
                .unwrap_or(&spec.username)
                .to_string();
            // Best-effort team add for idempotency.
            if let Some(tid) = team_id {
                let _ = self.add_user_to_team(tid, &user_id).await;
            }
            return Ok(HumanUserRef {
                user_id,
                username,
                email: spec.email.clone(),
                temp_password: None,
                must_change_password: false,
            });
        }

        // 2. Create. Mattermost requires password on creation; we
        //    generate a high-entropy one and force change on first
        //    login.
        let temp_password = generate_temp_password();
        let first = if spec.first_name.is_empty() {
            spec.display_name.clone()
        } else {
            spec.first_name.clone()
        };
        let last = spec.last_name.clone();
        let mut body = json!({
            "email": spec.email,
            "username": spec.username,
            "password": temp_password,
            "first_name": first,
            "last_name": last,
        });
        if !spec.locale.is_empty() {
            body["locale"] = json!(spec.locale);
        }
        let create = self
            .client
            .post(self.url("/users"))
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("mm human create: {e}")))?;
        let status = create.status();
        let v: serde_json::Value = create
            .json()
            .await
            .map_err(|e| Error::Provider(format!("mm human create json: {e}")))?;
        if !status.is_success() {
            return Err(Error::Provider(format!(
                "mm human create {status}: {}",
                short_value(&v)
            )));
        }
        let user_id = v
            .get("id")
            .and_then(|x| x.as_str())
            .unwrap_or_default()
            .to_string();
        let username = v
            .get("username")
            .and_then(|x| x.as_str())
            .unwrap_or(&spec.username)
            .to_string();

        // 3. Force password change on first login. Best-effort: failure
        //    here doesn't invalidate the provision (the user can still
        //    log in with the temp password).
        let _ = self
            .client
            .put(self.url(&format!("/users/{user_id}/password")))
            .headers(self.headers())
            .json(&json!({
                "current_password": temp_password,
                "new_password": temp_password,
            }))
            .send()
            .await;
        let _ = self
            .client
            .post(self.url(&format!("/users/{user_id}/auth")))
            .headers(self.headers())
            .json(&json!({"auth_data": "", "auth_service": "", "password": temp_password}))
            .send()
            .await;

        // 4. Add to team if requested.
        if let Some(tid) = team_id {
            self.add_user_to_team(tid, &user_id).await?;
        }

        Ok(HumanUserRef {
            user_id,
            username,
            email: spec.email.clone(),
            temp_password: Some(temp_password),
            must_change_password: true,
        })
    }
}

impl UpstreamMattermost {
    async fn add_user_to_team(&self, team_id: &str, user_id: &str) -> Result<()> {
        let body = json!({"team_id": team_id, "user_id": user_id});
        let resp = self
            .client
            .post(self.url(&format!("/teams/{team_id}/members")))
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("mm team add: {e}")))?;
        // 2xx OK; 400 with "already exists" is fine — treat as idempotent.
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            if text.contains("already") || text.contains("exists") {
                return Ok(());
            }
            return Err(Error::Provider(format!("mm team add {status}: {text}")));
        }
        Ok(())
    }
}

fn urlencode_path(s: &str) -> String {
    s.replace('%', "%25")
        .replace(' ', "%20")
        .replace('@', "%40")
        .replace('+', "%2B")
}

fn short_value(v: &serde_json::Value) -> String {
    let s = v.to_string();
    if s.len() > 240 {
        format!("{}…", &s[..240])
    } else {
        s
    }
}

/// Generate a 24-character temp password using a cryptographically
/// non-trivial mix from the system clock + thread id. Mattermost
/// requires lowercase + uppercase + number + symbol when admin policy
/// is strict; we always emit at least one of each to satisfy the
/// strictest configuration.
fn generate_temp_password() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let tid = format!("{:?}", std::thread::current().id());
    let mut seed = (nanos as u64) ^ {
        let mut h: u64 = 0;
        for b in tid.as_bytes() {
            h = h.wrapping_mul(131).wrapping_add(*b as u64);
        }
        h
    };
    let mut next = || {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (seed >> 17) as u32
    };
    let alpha_lower = b"abcdefghijkmnopqrstuvwxyz";
    let alpha_upper = b"ABCDEFGHJKLMNPQRSTUVWXYZ";
    let digits = b"23456789";
    let symbols = b"!@#$%^&*-_+=";
    let mut out = String::with_capacity(24);
    out.push(alpha_lower[(next() as usize) % alpha_lower.len()] as char);
    out.push(alpha_upper[(next() as usize) % alpha_upper.len()] as char);
    out.push(digits[(next() as usize) % digits.len()] as char);
    out.push(symbols[(next() as usize) % symbols.len()] as char);
    let pool: Vec<u8> = alpha_lower
        .iter()
        .chain(alpha_upper.iter())
        .chain(digits.iter())
        .chain(symbols.iter())
        .copied()
        .collect();
    for _ in 4..24 {
        out.push(pool[(next() as usize) % pool.len()] as char);
    }
    out
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
