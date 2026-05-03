use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use genasis_core::config::Config;
use genasis_i18n::t;
use genasis_overlay::{plan_detach, scan, summary, unified_diff};

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
}

pub async fn run(args: Args) -> Result<()> {
    let project_root = resolve_project_root(args.project.as_deref())?;
    tracing::info!(project_root = %project_root.display(), "detach: scanning agents");

    let report = scan(&project_root)?;
    if !report.skipped.is_empty() {
        for (path, why) in &report.skipped {
            tracing::warn!(path = %path.display(), reason = %why, "skipped agent");
        }
    }

    let plan = plan_detach(&report.agents)?;

    print!("{}", summary(&plan));
    if args.diff {
        println!();
        print!("{}", unified_diff(&plan));
    }

    if args.dry_run {
        return Ok(());
    }

    let applied = genasis_overlay::apply(&plan)?;
    println!(
        "\n{}",
        t!(
            "detach.wrote_summary",
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
