//! `genasis trial <subcommand>` — operations that target the
//! operator-hosted trial-app (ADR-013, ADR-016, ADR-017).
//!
//! Currently exposes a single subcommand:
//!
//! - `genasis trial publish` — signals to the trial-app that the
//!   example PRD's deliverable is "complete" so the user's
//!   ShowcasePanel unlocks. Reads `[trial].team_token` and
//!   `[plane].project_name` / `workspace_slug` from `genasis.toml`,
//!   POSTs them to `/api/trial/team-app/status`, and prints the
//!   landing URL.
//!
//! Future subcommands (deferred): `trial revoke`, `trial status`.

use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use clap::{Args as ClapArgs, Subcommand};

use genasis_core::config::{slugify, Config, CONFIG_FILE_NAME, DEFAULT_TEAM_TOKEN};

#[derive(ClapArgs, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub kind: Kind,

    /// Project root. Defaults to the current working directory.
    #[arg(long, value_name = "DIR", global = true)]
    pub project: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Kind {
    /// Tell the trial-app that the example PRD's deliverable is
    /// finished. The team's `app_status` flips to `'complete'` and
    /// the ShowcasePanel toggle activates on
    /// `<trial_url>/?tab=live&team=<token>` (ADR-017 §4).
    Publish(PublishArgs),
}

#[derive(ClapArgs, Debug)]
pub struct PublishArgs {
    /// Print the resolved request body and exit without sending.
    #[arg(long)]
    pub dry_run: bool,

    /// Project root. Defaults to the current working directory.
    /// Mirrors the top-level `--project` on the legacy `genasis trial`
    /// subcommand so `genasis publish --project /path/to/team` works
    /// the same way without the `trial` namespace.
    #[arg(long, value_name = "DIR")]
    pub project: Option<PathBuf>,
}

pub async fn run(args: Args) -> Result<()> {
    let root = if let Some(p) = args.project.as_deref() {
        p.canonicalize()
            .with_context(|| format!("--project path does not exist: {}", p.display()))?
    } else {
        std::env::current_dir()?
    };

    match args.kind {
        Kind::Publish(p) => run_publish(&root, p).await,
    }
}

/// Top-level `genasis publish` entry point (v0.5.3 simplification B).
/// Resolves project root from the `--project` flag on PublishArgs or
/// the current working directory, then runs the same `run_publish`
/// that the legacy `genasis trial publish` invokes.
pub async fn run_publish_with_project(args: PublishArgs) -> Result<()> {
    let root = if let Some(p) = args.project.as_deref() {
        p.canonicalize()
            .with_context(|| format!("--project path does not exist: {}", p.display()))?
    } else {
        std::env::current_dir()?
    };
    run_publish(&root, args).await
}

async fn run_publish(project_root: &std::path::Path, args: PublishArgs) -> Result<()> {
    let cfg_path = project_root.join(CONFIG_FILE_NAME);
    if !cfg_path.is_file() {
        return Err(anyhow!(
            "no {} at {} — run `genasis init --trial` first",
            CONFIG_FILE_NAME,
            cfg_path.display()
        ));
    }
    let cfg = Config::load(&cfg_path)?;

    let trial = cfg.trial.as_ref().ok_or_else(|| {
        anyhow!(
            "[trial] section missing from {} — this command only applies to trial-mode projects",
            cfg_path.display()
        )
    })?;
    if !trial.enabled {
        return Err(anyhow!(
            "[trial] enabled = false; nothing to publish to the trial-app"
        ));
    }

    let team_token = trial
        .team_token
        .clone()
        .filter(|t| !t.is_empty() && t != DEFAULT_TEAM_TOKEN)
        .ok_or_else(|| {
            anyhow!(
                "[trial] team_token is missing or set to the default sentinel — \
                 re-run `genasis init --trial` so a fresh per-team token is minted"
            )
        })?;

    let project_name = cfg
        .plane
        .as_ref()
        .and_then(|p| p.project_name.clone())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| cfg.project.name.clone());
    if project_name.is_empty() {
        return Err(anyhow!(
            "[project].name and [plane].project_name are both empty; cannot publish without a name"
        ));
    }
    let project_slug = slugify(&project_name);
    let trial_url = trial.url.trim_end_matches('/');

    let body = serde_json::json!({
        "team_token": team_token,
        "status": "complete",
        "project": { "slug": project_slug, "name": project_name },
    });

    if args.dry_run {
        println!(
            "→ would POST {trial_url}/api/trial/team-app/status\n{}",
            serde_json::to_string_pretty(&body)?
        );
        return Ok(());
    }

    let endpoint = format!("{trial_url}/api/trial/team-app/status");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    let mut req = client.post(&endpoint).json(&body);
    // The team_token is also placed in the X-Genasis-Team-Token
    // header for symmetry with the Plane/MM bridge routes. The
    // status route does not require it (token is in the body) but
    // attaching it here keeps tracing / access logs consistent.
    req = req.header("X-Genasis-Team-Token", &team_token);
    if !trial.shared_secret.is_empty() {
        req = req.header("X-Genasis-Trial-Secret", &trial.shared_secret);
    }
    let resp = req
        .send()
        .await
        .with_context(|| format!("POST {endpoint}"))?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!("trial publish returned {status}: {text}"));
    }

    let token_short: String = team_token.chars().take(8).collect();
    let landing = format!("{trial_url}/?tab=live&team={team_token}");
    println!("✓ trial-app updated — team {token_short}… is now 'complete'");
    println!("  open: {landing}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_trial_config(dir: &std::path::Path, body: &str) {
        std::fs::write(dir.join(CONFIG_FILE_NAME), body).unwrap();
    }

    #[tokio::test]
    async fn publish_dry_run_prints_body_with_resolved_slug() {
        let tmp = TempDir::new().unwrap();
        write_trial_config(
            tmp.path(),
            r#"
[project]
name = "Marketing Squad"

[plane]
url = "https://mmplane-trial.realstory.blog"
workspace_slug = "marketing-squad"
flavor = "trial"
project_name = "Marketing Squad"

[mattermost]
url = "https://mmplane-trial.realstory.blog"
team_name = "marketing-squad"
flavor = "trial"
channels = [
  { key = "scrum", name = "scrum-marketing-squad", display_name = "Marketing Squad — Scrum" },
]

[trial]
enabled = true
url = "https://mmplane-trial.realstory.blog"
shared_secret = ""
team_token = "abc123def456abc123def456abc123de"
"#,
        );
        let args = Args {
            kind: Kind::Publish(PublishArgs {
                dry_run: true,
                project: None,
            }),
            project: Some(tmp.path().to_path_buf()),
        };
        run(args).await.expect("dry-run succeeds");
    }

    #[tokio::test]
    async fn publish_errors_when_team_token_missing() {
        let tmp = TempDir::new().unwrap();
        write_trial_config(
            tmp.path(),
            r#"
[project]
name = "no-token-team"

[plane]
url = "https://mmplane-trial.realstory.blog"
workspace_slug = "no-token-team"
flavor = "trial"

[mattermost]
url = "https://mmplane-trial.realstory.blog"
team_name = "no-token-team"
flavor = "trial"

[trial]
enabled = true
url = "https://mmplane-trial.realstory.blog"
shared_secret = ""
"#,
        );
        let args = Args {
            kind: Kind::Publish(PublishArgs {
                dry_run: true,
                project: None,
            }),
            project: Some(tmp.path().to_path_buf()),
        };
        let err = run(args).await.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("team_token"),
            "expected missing-token error, got: {msg}"
        );
    }

    #[tokio::test]
    async fn publish_errors_when_no_config() {
        let tmp = TempDir::new().unwrap();
        let args = Args {
            kind: Kind::Publish(PublishArgs {
                dry_run: true,
                project: None,
            }),
            project: Some(tmp.path().to_path_buf()),
        };
        let err = run(args).await.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("no genasis.toml"), "got: {msg}");
    }
}
