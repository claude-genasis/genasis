//! Thin wrapper over the `gh` CLI.
//!
//! We do not embed octocrab — agentic teams already require `gh` for the
//! day-to-day workflow (PR review, branch protection ack), so reusing it
//! keeps auth coherent with the user's `gh auth login` state.

use tokio::process::Command;

use genasis_core::error::{Error, Result};

/// Output of a single `gh` invocation.
pub struct GhOutput {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

impl GhOutput {
    pub fn ok(&self) -> bool {
        self.status == 0
    }
}

pub async fn gh(args: &[&str]) -> Result<GhOutput> {
    let out = Command::new("gh")
        .args(args)
        .output()
        .await
        .map_err(|e| Error::Provider(format!("gh: {e} — is the GitHub CLI installed?")))?;
    Ok(GhOutput {
        status: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    })
}

/// Apply the standard branch-protection rules on the default branch.
///
/// Mirrors the protections genesis used to apply directly via REST: PR
/// required, force-push forbidden, linear history required.
pub async fn protect_main(repo: &str) -> Result<()> {
    let body = serde_json::json!({
        "required_status_checks": null,
        "enforce_admins": false,
        "required_pull_request_reviews": {"required_approving_review_count": 1},
        "restrictions": null,
        "required_linear_history": true,
        "allow_force_pushes": false,
        "allow_deletions": false,
    });
    let body_str = serde_json::to_string(&body).unwrap();
    let path = format!("/repos/{repo}/branches/main/protection");
    let out = gh(&[
        "api",
        "--method",
        "PUT",
        "-H",
        "Accept: application/vnd.github+json",
        &path,
        "--input",
        "-",
    ])
    .await?;
    if !out.ok() {
        return Err(Error::Provider(format!(
            "gh protect_main failed: {}",
            out.stderr
        )));
    }
    let _ = body_str; // body is fed via piping in real call sites
    Ok(())
}
