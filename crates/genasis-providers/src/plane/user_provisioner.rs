//! Drive the Node Playwright sub-process that creates per-agent Plane
//! users and issues their PATs.
//!
//! See `crates/genasis-cli/scripts/provision-plane-users.mjs` for the
//! stdio protocol.

use std::path::Path;

use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use genasis_core::error::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRequest {
    pub role: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisionInput {
    pub plane_url: String,
    pub workspace_slug: String,
    pub admin_email: String,
    pub admin_password: String,
    pub agents: Vec<AgentRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisionedAgent {
    pub role: String,
    pub email: String,
    pub user_id: String,
    pub pat: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisionOutput {
    pub status: String,
    #[serde(default)]
    pub agents: Vec<ProvisionedAgent>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

/// Spawn the Node Playwright provisioner with the given input.
pub async fn provision(script_path: &Path, input: &ProvisionInput) -> Result<ProvisionOutput> {
    if !script_path.is_file() {
        return Err(Error::Provider(format!(
            "provisioner script missing: {}",
            script_path.display()
        )));
    }
    let mut child = Command::new("node")
        .arg(script_path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| Error::Provider(format!("spawn node: {e} — install Node 18+")))?;

    let body = serde_json::to_vec(input)
        .map_err(|e| Error::Provider(format!("provisioner input json: {e}")))?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(&body)
            .await
            .map_err(|e| Error::Provider(format!("write provisioner stdin: {e}")))?;
        stdin.flush().await.ok();
    }
    drop(child.stdin.take());

    let out = child
        .wait_with_output()
        .await
        .map_err(|e| Error::Provider(format!("provisioner wait: {e}")))?;
    if !out.status.success() && out.status.code() != Some(2) {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(Error::Provider(format!(
            "provisioner exited {} — stderr: {stderr}",
            out.status
        )));
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    let line = stdout.lines().last().unwrap_or_default();
    serde_json::from_str(line).map_err(|e| {
        Error::Provider(format!(
            "provisioner output not JSON: {e}\nstdout: {stdout}"
        ))
    })
}
