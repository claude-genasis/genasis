//! ADR-019 §0.5: REST Mattermost provisioner against the operator's
//! system-admin token.
//!
//! Endpoints validated against `mm.realstory.blog` (system_admin
//! account `admin@realstory.blog`). Body shapes encoded here match
//! Mattermost v6+ /api/v4.
//!
//! Multi-tenancy model (operator-hosted): one team per provisioned
//! Genasis team (`team-<team-slug>`), one scrum channel per app
//! (`scrum-<app-slug>`), agent users shared across all teams as
//! global system_users, humans isolated to their own team.
//!
//! Self-host model: identical surface — caller just points the
//! `MmAdmin` at the local docker-compose URL with a self-issued
//! system-admin PAT. No code path branches on flavor.

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use genasis_core::error::{Error, Result};

/// Mattermost team channel types. We always use Open (`O`) for
/// scrum/announcement-style channels so newly added members can
/// see history without explicit invitation.
pub const CHANNEL_OPEN: &str = "O";
pub const CHANNEL_PRIVATE: &str = "P";
/// Default team type — Open (anyone with an account can be added by
/// the team admin). Closed (`I`) teams require explicit invite to
/// even see; we don't use that for genasis-provisioned teams.
pub const TEAM_OPEN: &str = "O";

/// Wraps a system-admin Mattermost client. One instance per
/// `(url, admin_token)`. Cheap to clone.
#[derive(Clone)]
pub struct MmClient {
    pub url: String,
    http: reqwest::Client,
}

impl MmClient {
    pub fn new(url: &str, admin_token: &str) -> Result<Self> {
        let mut headers = HeaderMap::new();
        let mut auth = HeaderValue::from_str(&format!("Bearer {admin_token}"))
            .map_err(|e| Error::Provider(format!("invalid token header: {e}")))?;
        auth.set_sensitive(true);
        headers.insert(AUTHORIZATION, auth);
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let http = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| Error::Provider(format!("reqwest client build: {e}")))?;
        Ok(Self {
            url: url.trim_end_matches('/').to_string(),
            http,
        })
    }

    fn v4(&self, suffix: &str) -> String {
        format!("{}/api/v4/{}", self.url, suffix.trim_start_matches('/'))
    }

    /// `GET /users/me` — confirms the admin token actually parses
    /// and points at a `system_admin`. We refuse to provision with a
    /// token that doesn't have the `system_admin` role to avoid
    /// partial state if create-user calls fail later.
    pub async fn whoami(&self) -> Result<MmUser> {
        let resp = self
            .http
            .get(self.v4("users/me"))
            .send()
            .await
            .map_err(|e| Error::Provider(format!("mm whoami: {e}")))?;
        if !resp.status().is_success() {
            let s = resp.status();
            let b = resp.text().await.unwrap_or_default();
            return Err(Error::Provider(format!("mm whoami {s}: {b}")));
        }
        let user: MmUser = resp
            .json()
            .await
            .map_err(|e| Error::Provider(format!("mm whoami json: {e}")))?;
        if !user.roles.split_whitespace().any(|r| r == "system_admin") {
            return Err(Error::Provider(format!(
                "mm token belongs to {} but roles={:?} — system_admin required",
                user.email, user.roles
            )));
        }
        Ok(user)
    }

    /// GET-or-create the team. Idempotent on `name`.
    ///
    /// `expected_id`: when set, asserts ownership of the existing
    /// team. Mismatch returns an explicit error so two tenants
    /// can't silently inherit each other's MM team — see the Plane
    /// twin for rationale.
    pub async fn ensure_team(
        &self,
        name: &str,
        display_name: &str,
        expected_id: Option<&str>,
    ) -> Result<(MmTeam, Outcome)> {
        if let Some(existing) = self.team_by_name(name).await? {
            return match expected_id {
                Some(id) if id == existing.id => Ok((existing, Outcome::Reused)),
                Some(other) => Err(Error::Provider(format!(
                    "Mattermost team {name:?} exists with id={} but local \
                     snapshot expected id={}. Another tenant owns this team \
                     name — pick a different `--team-slug`.",
                    existing.id, other
                ))),
                None => Err(Error::Provider(format!(
                    "Mattermost team {name:?} already exists (id={}). Pick a \
                     different `--team-slug`, or re-run from the snapshot \
                     directory if this is the same team.",
                    existing.id
                ))),
            };
        }
        let body = serde_json::json!({
            "name": name,
            "display_name": display_name,
            "type": TEAM_OPEN,
        });
        let resp = self
            .http
            .post(self.v4("teams"))
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("create team: {e}")))?;
        let status = resp.status();
        if status.is_success() {
            let team: MmTeam = resp
                .json()
                .await
                .map_err(|e| Error::Provider(format!("team json: {e}")))?;
            return Ok((team, Outcome::Created));
        }
        if status == StatusCode::BAD_REQUEST || status == StatusCode::CONFLICT {
            // Race / name reuse — re-fetch.
            if let Some(existing) = self.team_by_name(name).await? {
                return Ok((existing, Outcome::Reused));
            }
        }
        let body = resp.text().await.unwrap_or_default();
        Err(Error::Provider(format!("create team {status}: {body}")))
    }

    async fn team_by_name(&self, name: &str) -> Result<Option<MmTeam>> {
        let resp = self
            .http
            .get(self.v4(&format!("teams/name/{name}")))
            .send()
            .await
            .map_err(|e| Error::Provider(format!("get team by name: {e}")))?;
        match resp.status() {
            StatusCode::OK => Ok(Some(
                resp.json()
                    .await
                    .map_err(|e| Error::Provider(format!("team json: {e}")))?,
            )),
            StatusCode::NOT_FOUND => Ok(None),
            other => {
                let body = resp.text().await.unwrap_or_default();
                Err(Error::Provider(format!("team-by-name {other}: {body}")))
            }
        }
    }

    /// GET-or-create a channel in the given team.
    ///
    /// `expected_id`: ownership assertion — see `ensure_team`. A
    /// channel is scoped to a team, so the collision risk is lower
    /// (one team's channel name doesn't conflict with another
    /// team's), but a re-run with a mismatched snapshot id is still
    /// worth catching loudly rather than silently rebinding to the
    /// wrong channel.
    pub async fn ensure_channel(
        &self,
        team_id: &str,
        name: &str,
        display_name: &str,
        channel_type: &str,
        expected_id: Option<&str>,
    ) -> Result<(MmChannel, Outcome)> {
        if let Some(existing) = self.channel_by_name(team_id, name).await? {
            return match expected_id {
                Some(id) if id == existing.id => Ok((existing, Outcome::Reused)),
                Some(other) => Err(Error::Provider(format!(
                    "Mattermost channel {name:?} in team {team_id} exists \
                     with id={} but local snapshot expected id={}.",
                    existing.id, other
                ))),
                None => Err(Error::Provider(format!(
                    "Mattermost channel {name:?} already exists in team \
                     {team_id} (id={}). Pick a different `--app-slug`.",
                    existing.id
                ))),
            };
        }
        let body = serde_json::json!({
            "team_id": team_id,
            "name": name,
            "display_name": display_name,
            "type": channel_type,
        });
        let resp = self
            .http
            .post(self.v4("channels"))
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("create channel: {e}")))?;
        let status = resp.status();
        if status.is_success() {
            let c: MmChannel = resp
                .json()
                .await
                .map_err(|e| Error::Provider(format!("channel json: {e}")))?;
            return Ok((c, Outcome::Created));
        }
        if status == StatusCode::BAD_REQUEST || status == StatusCode::CONFLICT {
            if let Some(existing) = self.channel_by_name(team_id, name).await? {
                return Ok((existing, Outcome::Reused));
            }
        }
        let body = resp.text().await.unwrap_or_default();
        Err(Error::Provider(format!("create channel {status}: {body}")))
    }

    async fn channel_by_name(&self, team_id: &str, name: &str) -> Result<Option<MmChannel>> {
        let resp = self
            .http
            .get(self.v4(&format!("teams/{team_id}/channels/name/{name}")))
            .send()
            .await
            .map_err(|e| Error::Provider(format!("channel-by-name: {e}")))?;
        match resp.status() {
            StatusCode::OK => {
                Ok(Some(resp.json().await.map_err(|e| {
                    Error::Provider(format!("channel json: {e}"))
                })?))
            }
            StatusCode::NOT_FOUND => Ok(None),
            other => {
                let body = resp.text().await.unwrap_or_default();
                Err(Error::Provider(format!("channel-by-name {other}: {body}")))
            }
        }
    }

    /// Look up a user by email. Returns None on 404 — caller decides
    /// whether to create or invite.
    pub async fn user_by_email(&self, email: &str) -> Result<Option<MmUser>> {
        let resp = self
            .http
            .get(self.v4(&format!("users/email/{email}")))
            .send()
            .await
            .map_err(|e| Error::Provider(format!("user-by-email: {e}")))?;
        match resp.status() {
            StatusCode::OK => Ok(Some(
                resp.json()
                    .await
                    .map_err(|e| Error::Provider(format!("user json: {e}")))?,
            )),
            StatusCode::NOT_FOUND => Ok(None),
            other => {
                let body = resp.text().await.unwrap_or_default();
                Err(Error::Provider(format!("user-by-email {other}: {body}")))
            }
        }
    }

    /// GET-or-create an agent user (admin-controlled, password is
    /// random and discarded — we only use the PAT). Idempotent on
    /// email.
    pub async fn ensure_agent_user(
        &self,
        email: &str,
        username: &str,
        password: &str,
    ) -> Result<(MmUser, Outcome)> {
        if let Some(existing) = self.user_by_email(email).await? {
            return Ok((existing, Outcome::Reused));
        }
        let body = serde_json::json!({
            "email": email,
            "username": username,
            "password": password,
        });
        let resp = self
            .http
            .post(self.v4("users"))
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("create user: {e}")))?;
        let status = resp.status();
        if status.is_success() {
            let u: MmUser = resp
                .json()
                .await
                .map_err(|e| Error::Provider(format!("user json: {e}")))?;
            return Ok((u, Outcome::Created));
        }
        if status == StatusCode::BAD_REQUEST {
            // Maybe a race created it after our probe — re-fetch.
            if let Some(existing) = self.user_by_email(email).await? {
                return Ok((existing, Outcome::Reused));
            }
        }
        let body = resp.text().await.unwrap_or_default();
        Err(Error::Provider(format!("create user {status}: {body}")))
    }

    /// Add user to team. GET-before-POST because Mattermost's
    /// `POST /teams/.../members` returns 201 even when the user is
    /// already a team member (the response body is the existing
    /// membership record, not a "no-op" sentinel). Without the GET
    /// probe we couldn't tell Created from Reused.
    pub async fn ensure_team_member(&self, team_id: &str, user_id: &str) -> Result<Outcome> {
        if self.is_team_member(team_id, user_id).await? {
            return Ok(Outcome::Reused);
        }
        let body = serde_json::json!({
            "team_id": team_id,
            "user_id": user_id,
        });
        let resp = self
            .http
            .post(self.v4(&format!("teams/{team_id}/members")))
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("add team member: {e}")))?;
        let status = resp.status();
        if status.is_success() {
            return Ok(Outcome::Created);
        }
        let body = resp.text().await.unwrap_or_default();
        let lc = body.to_ascii_lowercase();
        if lc.contains("already") || lc.contains("exists") {
            return Ok(Outcome::Reused);
        }
        Err(Error::Provider(format!("add team member {status}: {body}")))
    }

    async fn is_team_member(&self, team_id: &str, user_id: &str) -> Result<bool> {
        let resp = self
            .http
            .get(self.v4(&format!("teams/{team_id}/members/{user_id}")))
            .send()
            .await
            .map_err(|e| Error::Provider(format!("get team member: {e}")))?;
        Ok(resp.status() == StatusCode::OK)
    }

    /// Add user to channel. Same GET-before-POST pattern as
    /// `ensure_team_member` for the same reason.
    pub async fn ensure_channel_member(&self, channel_id: &str, user_id: &str) -> Result<Outcome> {
        if self.is_channel_member(channel_id, user_id).await? {
            return Ok(Outcome::Reused);
        }
        let body = serde_json::json!({ "user_id": user_id });
        let resp = self
            .http
            .post(self.v4(&format!("channels/{channel_id}/members")))
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("add channel member: {e}")))?;
        let status = resp.status();
        if status.is_success() {
            return Ok(Outcome::Created);
        }
        let body = resp.text().await.unwrap_or_default();
        let lc = body.to_ascii_lowercase();
        if lc.contains("already") || lc.contains("exists") {
            return Ok(Outcome::Reused);
        }
        Err(Error::Provider(format!(
            "add channel member {status}: {body}"
        )))
    }

    async fn is_channel_member(&self, channel_id: &str, user_id: &str) -> Result<bool> {
        let resp = self
            .http
            .get(self.v4(&format!("channels/{channel_id}/members/{user_id}")))
            .send()
            .await
            .map_err(|e| Error::Provider(format!("get channel member: {e}")))?;
        Ok(resp.status() == StatusCode::OK)
    }

    /// Issue a PAT for the given user. Mattermost requires
    /// `EnableUserAccessTokens=true` in config and the caller to be
    /// system_admin (or the user themselves with the appropriate
    /// scope). Returns the bare token string — store it carefully,
    /// upstream does not expose it again.
    pub async fn issue_pat(&self, user_id: &str, description: &str) -> Result<MmAccessToken> {
        let body = serde_json::json!({ "description": description });
        let resp = self
            .http
            .post(self.v4(&format!("users/{user_id}/tokens")))
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("issue PAT: {e}")))?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(Error::Provider(format!("issue PAT {status}: {body}")));
        }
        resp.json()
            .await
            .map_err(|e| Error::Provider(format!("PAT json: {e}")))
    }

    /// Send an email invitation for a human user to join the team.
    /// Used when the human doesn't already have an MM account and
    /// you want them to sign up themselves via the link in the
    /// email.
    pub async fn invite_human_by_email(&self, team_id: &str, emails: &[String]) -> Result<()> {
        let resp = self
            .http
            .post(self.v4(&format!("teams/{team_id}/invite/email")))
            .json(emails)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("invite by email: {e}")))?;
        if !resp.status().is_success() {
            let s = resp.status();
            let b = resp.text().await.unwrap_or_default();
            return Err(Error::Provider(format!("invite by email {s}: {b}")));
        }
        Ok(())
    }
}

/// Per-call outcome flag for idempotent `ensure_*` operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Created,
    Reused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MmUser {
    pub id: String,
    pub email: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub roles: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MmTeam {
    pub id: String,
    pub name: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MmChannel {
    pub id: String,
    pub name: String,
    pub display_name: String,
    #[serde(default, rename = "type")]
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MmAccessToken {
    pub id: String,
    pub token: String,
    pub user_id: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub is_active: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v4_url_builds() {
        let c = MmClient::new("https://mm.example.com/", "tok").unwrap();
        assert_eq!(c.v4("users/me"), "https://mm.example.com/api/v4/users/me");
        assert_eq!(c.v4("/teams"), "https://mm.example.com/api/v4/teams");
    }

    #[test]
    fn constants_match_mattermost_api() {
        assert_eq!(CHANNEL_OPEN, "O");
        assert_eq!(CHANNEL_PRIVATE, "P");
        assert_eq!(TEAM_OPEN, "O");
    }
}
