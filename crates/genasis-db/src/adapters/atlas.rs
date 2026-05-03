//! Atlas (ariga.io) — declarative HCL/SQL schema migrations.
//!
//! Conventions:
//! - The user keeps schema files under `<project>/db/schema/` and an
//!   `atlas.hcl` config at `<project>/atlas.hcl`.
//! - `apply` runs `atlas migrate apply` against the env (defaults to `dev`).
//! - `diff` runs `atlas migrate diff` and returns the planned SQL.

use std::path::{Path, PathBuf};

use genasis_core::error::{Error, Result};
use tokio::process::Command;

pub async fn apply(project_root: &Path, env: Option<&str>) -> Result<String> {
    let cfg = atlas_config(project_root)?;
    let env = env.unwrap_or("dev");
    let out = Command::new("atlas")
        .current_dir(project_root)
        .args([
            "migrate",
            "apply",
            "--config",
            &format!("file://{}", cfg.display()),
            "--env",
            env,
        ])
        .output()
        .await
        .map_err(|e| Error::Db(format!("atlas: {e} — install: curl -sSf https://atlasgo.sh | sh")))?;
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    if !out.status.success() {
        return Err(Error::Db(format!(
            "atlas migrate apply failed: {}",
            String::from_utf8_lossy(&out.stderr)
        )));
    }
    Ok(stdout)
}

pub async fn diff(project_root: &Path) -> Result<String> {
    let cfg = atlas_config(project_root)?;
    let out = Command::new("atlas")
        .current_dir(project_root)
        .args([
            "migrate",
            "diff",
            "--config",
            &format!("file://{}", cfg.display()),
        ])
        .output()
        .await
        .map_err(|e| Error::Db(format!("atlas: {e}")))?;
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn atlas_config(project_root: &Path) -> Result<PathBuf> {
    let cfg = project_root.join("atlas.hcl");
    if !cfg.is_file() {
        return Err(Error::Db(format!(
            "atlas config not found at {} — see docs/PROVIDERS.md",
            cfg.display()
        )));
    }
    Ok(cfg)
}
