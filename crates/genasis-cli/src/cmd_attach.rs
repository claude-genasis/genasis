use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use genasis_core::config::{Config, CONFIG_FILE_NAME};
use genasis_i18n::t;
use genasis_overlay::{plan_attach, scan, summary, unified_diff, AttachOptions};

const DEFAULT_FENCE_VERSION: &str = "1.0";

#[derive(Parser, Debug)]
pub struct Args {
    /// Project root. Defaults to the current working directory.
    #[arg(long, value_name = "DIR")]
    pub project: Option<PathBuf>,

    /// Print the planned changes and exit without writing anything.
    #[arg(long)]
    pub dry_run: bool,

    /// Show full per-file unified diffs in addition to the summary.
    #[arg(long)]
    pub diff: bool,

    /// Override Tampered / RoleMismatch refusals.
    #[arg(long)]
    pub force: bool,

    /// Fence version to write (default: 1.0).
    #[arg(long, default_value = DEFAULT_FENCE_VERSION)]
    pub fence_version: String,
}

pub async fn run(args: Args) -> Result<()> {
    let project_root = resolve_project_root(args.project.as_deref())?;
    tracing::info!(project_root = %project_root.display(), "attach: scanning agents");

    let report = scan(&project_root)?;
    if !report.skipped.is_empty() {
        for (path, why) in &report.skipped {
            tracing::warn!(path = %path.display(), reason = %why, "skipped agent");
        }
    }

    let context = build_context(&project_root)?;
    let opts = AttachOptions {
        fence_version: args.fence_version.clone(),
        context,
        force: args.force,
    };
    let plan = plan_attach(&report.agents, &opts)?;

    print!("{}", summary(&plan));
    if args.diff {
        println!();
        print!("{}", unified_diff(&plan));
    }

    if args.dry_run {
        return Ok(());
    }

    let refused = plan.refused().count();
    if refused > 0 && !args.force {
        anyhow::bail!("{}", t!("attach.refused", count = refused));
    }

    let applied = genasis_overlay::apply(&plan)?;
    println!(
        "\n{}",
        t!(
            "attach.wrote_summary",
            count = applied.written.len(),
            backups = applied.backups.len()
        )
    );
    Ok(())
}

fn resolve_project_root(arg: Option<&std::path::Path>) -> Result<PathBuf> {
    if let Some(p) = arg {
        return p
            .canonicalize()
            .with_context(|| format!("--project path does not exist: {}", p.display()));
    }
    let cwd = std::env::current_dir()?;
    if let Some(cfg) = Config::discover(&cwd) {
        if let Some(parent) = cfg.parent() {
            return Ok(parent.to_path_buf());
        }
    }
    Ok(cwd)
}

fn build_context(project_root: &std::path::Path) -> Result<serde_json::Value> {
    let cfg_path = project_root.join(CONFIG_FILE_NAME);
    let cfg = if cfg_path.is_file() {
        Config::load(&cfg_path)?
    } else {
        Config::default()
    };
    Ok(serde_json::json!({
        "project_name": cfg.project.name,
        "project_domain": cfg.project.domain,
        "plane_url": cfg.plane.as_ref().map(|p| p.url.clone()).unwrap_or_default(),
        "mm_url": cfg.mattermost.as_ref().map(|m| m.url.clone()).unwrap_or_default(),
        "plane_flavor": cfg.plane.as_ref().map(|p| p.flavor.clone()).unwrap_or_else(|| "auto".into()),
        "mm_flavor": cfg.mattermost.as_ref().map(|m| m.flavor.clone()).unwrap_or_else(|| "auto".into()),
    }))
}
