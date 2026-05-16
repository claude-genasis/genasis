//! ADR-019 §0.5: REST-only Plane provisioning against the operator's
//! shared `agentic` workspace.
//!
//! All endpoints below have been smoke-tested against
//! `plane.realstory.blog` with a workspace API key, so the request
//! shapes encoded here match Plane Community Edition v0.x.
//!
//! Idempotency: every public call does a GET-first probe and only
//! POSTs on missing. Caller can re-run the whole flow without
//! creating duplicates.
//!
//! Concurrency / scope: this provisioner is intentionally
//! workspace-scoped — it never tries to create the workspace itself
//! (workspace API keys cannot do that) and never tries to create
//! Plane user records (Plane CE has no public REST for that). The
//! agent users are expected to already exist as workspace members;
//! we just attach them to the new project. Humans are invited via
//! the workspace invitation endpoint.

use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use genasis_core::error::{Error, Result};

/// Plane member role IDs as defined by the upstream RBAC table.
pub const ROLE_GUEST: u8 = 5;
pub const ROLE_VIEWER: u8 = 10;
pub const ROLE_MEMBER: u8 = 15;
pub const ROLE_ADMIN: u8 = 20;

/// Wraps a workspace-scoped Plane API client. One instance per
/// `(url, api_key, workspace_slug)`; cheap to clone because all
/// fields are owned strings or an Arc'd reqwest client.
#[derive(Clone)]
pub struct PlaneClient {
    pub url: String,
    pub workspace_slug: String,
    api_key: String,
    http: reqwest::Client,
}

impl PlaneClient {
    pub fn new(url: &str, api_key: &str, workspace_slug: &str) -> Result<Self> {
        let mut headers = HeaderMap::new();
        // X-API-Key is the auth header for workspace-scoped tokens
        // on Plane CE. Validated against plane.realstory.blog.
        let mut k = HeaderValue::from_str(api_key)
            .map_err(|e| Error::Provider(format!("invalid API key header: {e}")))?;
        k.set_sensitive(true);
        headers.insert("X-API-Key", k);
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let http = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| Error::Provider(format!("reqwest client build: {e}")))?;
        Ok(Self {
            url: url.trim_end_matches('/').to_string(),
            workspace_slug: workspace_slug.to_string(),
            api_key: api_key.to_string(),
            http,
        })
    }

    fn ws_url(&self, suffix: &str) -> String {
        format!(
            "{}/api/v1/workspaces/{}/{}",
            self.url, self.workspace_slug, suffix
        )
    }

    /// Pull `/users/me` — used as a connectivity + auth probe. Returns
    /// `Ok(())` only if the API key resolves to a real user.
    pub async fn whoami(&self) -> Result<WorkspaceMember> {
        let url = format!("{}/api/v1/users/me/", self.url);
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("plane whoami: {e}")))?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(Error::Provider(format!("plane whoami {status}: {body}")));
        }
        resp.json::<WorkspaceMember>()
            .await
            .map_err(|e| Error::Provider(format!("plane whoami json: {e}")))
    }

    /// List members of the workspace. Used to discover which agent
    /// users (`pm`, `frontend`, ...) are already registered so we
    /// can attach them to a new project.
    pub async fn list_workspace_members(&self) -> Result<Vec<WorkspaceMember>> {
        let resp = self
            .http
            .get(self.ws_url("members/"))
            .send()
            .await
            .map_err(|e| Error::Provider(format!("list workspace members: {e}")))?;
        if !resp.status().is_success() {
            let s = resp.status();
            let b = resp.text().await.unwrap_or_default();
            return Err(Error::Provider(format!("list members {s}: {b}")));
        }
        resp.json::<Vec<WorkspaceMember>>()
            .await
            .map_err(|e| Error::Provider(format!("members json: {e}")))
    }

    /// Find a member by display_name (PM agents have display_name=`pm`,
    /// frontend agents `frontend`, etc.). Returns `None` if no such
    /// member is in this workspace yet — caller must surface an
    /// actionable error (operator has to register the agent
    /// out-of-band before re-running provision).
    pub async fn find_member_by_display_name(
        &self,
        display_name: &str,
    ) -> Result<Option<WorkspaceMember>> {
        let members = self.list_workspace_members().await?;
        Ok(members
            .into_iter()
            .find(|m| m.display_name.eq_ignore_ascii_case(display_name)))
    }

    /// GET-or-create the project. Idempotent on the `identifier`
    /// field. Plane returns 409 if the identifier is already taken
    /// in the workspace.
    ///
    /// `expected_id`: when present (provided by a prior
    /// `genasis.toml.snapshot`), the caller asserts they own this
    /// project. If the workspace already has a project with the same
    /// identifier *but a different id*, we return
    /// `Error::Provider(... slug collision ...)` instead of silently
    /// reusing — that path is how multi-tenant `agentic` workspace
    /// users would otherwise inherit each other's projects.
    /// When `None` and a project with this identifier exists, we
    /// likewise refuse: a first-time provision can't possibly
    /// "reuse" something we never created.
    pub async fn ensure_project(
        &self,
        name: &str,
        identifier: &str,
        expected_id: Option<&str>,
    ) -> Result<(ProjectRef, ProjectCreateOutcome)> {
        if let Some(existing) = self.find_project_by_identifier(identifier).await? {
            return match expected_id {
                Some(id) if id == existing.id => Ok((existing, ProjectCreateOutcome::Reused)),
                Some(other) => Err(Error::Provider(format!(
                    "Plane project identifier {identifier:?} exists with id \
                     {existing_id} but the local snapshot expected id {other}. \
                     Another team is already using this identifier — pick a \
                     different `--app-slug`.",
                    existing_id = existing.id
                ))),
                None => Err(Error::Provider(format!(
                    "Plane project identifier {identifier:?} already exists \
                     (id={existing_id}). Pick a different `--app-slug`, or if \
                     this is the same team re-run from the directory that \
                     contains its `genasis.toml.snapshot`.",
                    existing_id = existing.id
                ))),
            };
        }
        let body = serde_json::json!({
            "name": name,
            "identifier": identifier,
        });
        let resp = self
            .http
            .post(self.ws_url("projects/"))
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("create project: {e}")))?;
        match resp.status() {
            StatusCode::CREATED | StatusCode::OK => {
                let proj: ProjectRef = resp
                    .json()
                    .await
                    .map_err(|e| Error::Provider(format!("project json: {e}")))?;
                Ok((proj, ProjectCreateOutcome::Created))
            }
            StatusCode::CONFLICT => {
                // Another caller raced us — re-fetch and treat as reused.
                let existing = self
                    .find_project_by_identifier(identifier)
                    .await?
                    .ok_or_else(|| {
                        Error::Provider(format!(
                            "plane returned 409 for identifier {identifier} but \
                             project not found on re-fetch"
                        ))
                    })?;
                Ok((existing, ProjectCreateOutcome::Reused))
            }
            other => {
                let body = resp.text().await.unwrap_or_default();
                Err(Error::Provider(format!("create project {other}: {body}")))
            }
        }
    }

    async fn find_project_by_identifier(&self, identifier: &str) -> Result<Option<ProjectRef>> {
        let resp = self
            .http
            .get(self.ws_url("projects/"))
            .send()
            .await
            .map_err(|e| Error::Provider(format!("list projects: {e}")))?;
        if !resp.status().is_success() {
            let s = resp.status();
            let b = resp.text().await.unwrap_or_default();
            return Err(Error::Provider(format!("list projects {s}: {b}")));
        }
        // Plane wraps the project list in a paginated envelope with
        // `results: [...]`. Pull just the array; the cursor stuff
        // doesn't matter at provisioning scale (a workspace with
        // 1000+ projects is unrealistic for our use case).
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| Error::Provider(format!("projects json: {e}")))?;
        let arr = body
            .get("results")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        for p in arr {
            let proj: ProjectRef = serde_json::from_value(p)
                .map_err(|e| Error::Provider(format!("project entry decode: {e}")))?;
            if proj.identifier.eq_ignore_ascii_case(identifier) {
                return Ok(Some(proj));
            }
        }
        Ok(None)
    }

    /// Add a workspace member to a project. Idempotent — if the
    /// member is already attached the upstream returns 400 with a
    /// specific message; we swallow that and return `Reused`.
    pub async fn ensure_project_member(
        &self,
        project_id: &str,
        member_id: &str,
        role: u8,
    ) -> Result<ProjectCreateOutcome> {
        if self.is_project_member(project_id, member_id).await? {
            return Ok(ProjectCreateOutcome::Reused);
        }
        let body = serde_json::json!({
            "member": member_id,
            "role": role,
        });
        let resp = self
            .http
            .post(self.ws_url(&format!("projects/{project_id}/members/")))
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("add project member: {e}")))?;
        let status = resp.status();
        if status.is_success() {
            return Ok(ProjectCreateOutcome::Created);
        }
        let body = resp.text().await.unwrap_or_default();
        // Plane returns 400 with "already a member" — treat as reuse.
        if body.contains("already") || body.contains("exists") {
            return Ok(ProjectCreateOutcome::Reused);
        }
        Err(Error::Provider(format!(
            "add project member {status}: {body}"
        )))
    }

    async fn is_project_member(&self, project_id: &str, member_id: &str) -> Result<bool> {
        let resp = self
            .http
            .get(self.ws_url(&format!("projects/{project_id}/members/")))
            .send()
            .await
            .map_err(|e| Error::Provider(format!("list project members: {e}")))?;
        if !resp.status().is_success() {
            return Ok(false); // upstream noise → treat as not-member to retry create
        }
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| Error::Provider(format!("project members json: {e}")))?;
        let arr = body.as_array().cloned().unwrap_or_default();
        for m in arr {
            if m.get("id").and_then(|v| v.as_str()) == Some(member_id) {
                return Ok(true);
            }
            // Plane sometimes returns the embedded user under
            // `member` instead of `id`. Accept both shapes.
            if m.get("member").and_then(|v| v.as_str()) == Some(member_id) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Idempotent workspace invitation. If `email` already has a
    /// pending or accepted invitation, returns `Reused`.
    pub async fn ensure_workspace_invitation(
        &self,
        email: &str,
        role: u8,
    ) -> Result<(InvitationRef, ProjectCreateOutcome)> {
        if let Some(existing) = self.find_invitation_by_email(email).await? {
            return Ok((existing, ProjectCreateOutcome::Reused));
        }
        let body = serde_json::json!({
            "email": email,
            "role": role,
        });
        let resp = self
            .http
            .post(self.ws_url("invitations/"))
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("invite: {e}")))?;
        let status = resp.status();
        if status.is_success() {
            let inv: InvitationRef = resp
                .json()
                .await
                .map_err(|e| Error::Provider(format!("invitation json: {e}")))?;
            return Ok((inv, ProjectCreateOutcome::Created));
        }
        let body = resp.text().await.unwrap_or_default();
        Err(Error::Provider(format!("invite {status}: {body}")))
    }

    async fn find_invitation_by_email(&self, email: &str) -> Result<Option<InvitationRef>> {
        let resp = self
            .http
            .get(self.ws_url("invitations/"))
            .send()
            .await
            .map_err(|e| Error::Provider(format!("list invitations: {e}")))?;
        if !resp.status().is_success() {
            return Ok(None);
        }
        let arr: Vec<InvitationRef> = resp
            .json()
            .await
            .map_err(|e| Error::Provider(format!("invitations json: {e}")))?;
        Ok(arr
            .into_iter()
            .find(|i| i.email.eq_ignore_ascii_case(email)))
    }

    /// Plane CE has no public "invite a specific email directly into
    /// a project" endpoint — invited users land in the workspace
    /// first, and on accepting they show up as a workspace member.
    /// `genasis provision` therefore invites at the workspace level
    /// and surfaces in the post-completion summary that the human
    /// has to accept the email before they appear in the project
    /// member list. The next time the user runs `genasis team list`
    /// after acceptance, we'll be able to add them to the project
    /// (covered by `cmd_team` follow-up PR).
    pub fn note_workspace_invite_only(&self, email: &str) -> String {
        format!(
            "Plane invitation sent to {email} for workspace `{ws}`. They must \
             open the email link and accept before they show up as a workspace \
             member — at that point re-run `genasis team add human ...` (or \
             `genasis provision` for the initial flow) to attach them to the \
             project.",
            ws = self.workspace_slug
        )
    }
}

/// Whether an ensure_* call created a new resource or reused an
/// existing one. Surfaced in the user-facing log so the provisioning
/// summary can show "✓ created" vs "↺ reused".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectCreateOutcome {
    Created,
    Reused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMember {
    pub id: String,
    #[serde(default)]
    pub first_name: String,
    #[serde(default)]
    pub last_name: String,
    pub email: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub role: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRef {
    pub id: String,
    pub name: String,
    pub identifier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvitationRef {
    pub id: String,
    pub email: String,
    pub role: i32,
    #[serde(default)]
    pub accepted: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ws_url_concatenates() {
        let c = PlaneClient::new("https://plane.example.com/", "key", "agentic").unwrap();
        assert_eq!(
            c.ws_url("projects/"),
            "https://plane.example.com/api/v1/workspaces/agentic/projects/"
        );
    }

    #[test]
    fn role_constants_match_plane_rbac() {
        // Sanity — Plane CE assigns these integer codes:
        assert_eq!(ROLE_GUEST, 5);
        assert_eq!(ROLE_MEMBER, 15);
        assert_eq!(ROLE_ADMIN, 20);
    }
}
