use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use genasis_core::config::Config;

#[derive(Parser, Debug)]
pub struct Args {
    /// Project root. Defaults to the current working directory.
    #[arg(long, value_name = "DIR", global = true)]
    pub project: Option<PathBuf>,

    #[command(subcommand)]
    pub op: DesignOp,
}

#[derive(Subcommand, Debug)]
pub enum DesignOp {
    /// Swap the design system using a reference URL. The actual extraction
    /// is performed by the designer agent's `ui-style-extractor` skill;
    /// this command persists the result, diffs it, and emits the issue plan.
    Swap {
        /// Reference URL (palette source).
        url: String,
        /// Path to the new design-system.md body produced by the extractor.
        #[arg(long)]
        body: PathBuf,
    },
    /// Print metadata about the current design-system.md.
    Status,
}

pub async fn run(args: Args) -> Result<()> {
    let project_root = resolve_project_root(args.project.as_deref())?;
    match args.op {
        DesignOp::Swap { url, body } => {
            let new_body = std::fs::read_to_string(&body)
                .with_context(|| format!("read new body: {}", body.display()))?;
            let outcome = genasis_design::run_swap(&project_root, &url, &new_body)?;
            println!("design swap from {url}:");
            println!("  previous present: {}", outcome.previous_present);
            println!("  impacted areas: {}", outcome.areas.len());
            for a in &outcome.areas {
                println!("    - {:?}", a);
            }
            println!("  planned issues:");
            for issue in &outcome.planned_issues {
                println!("    - [{}] {}", issue.label, issue.title);
            }
            println!("\nNext: run `genasis init` to push these issues to Plane,");
            println!("      then post the announcement on Mattermost.");
        }
        DesignOp::Status => {
            let target = project_root.join("docs").join("design-system.md");
            if let Ok(meta) = std::fs::metadata(&target) {
                println!("design-system.md: {} bytes", meta.len());
            } else {
                println!("design-system.md: missing — run `genasis attach` first.");
            }
        }
    }
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
