use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use genasis_core::config::{Config, CONFIG_FILE_NAME};
use genasis_i18n::t;

#[derive(Parser, Debug)]
pub struct Args {
    /// Project root. Defaults to the current working directory.
    #[arg(long, value_name = "DIR")]
    pub project: Option<PathBuf>,
}

pub async fn run(args: Args) -> Result<()> {
    let project_root = resolve_project_root(args.project.as_deref())?;
    println!("{} — project: {}", t!("doctor.header"), project_root.display());

    section("Required tools");
    for tool in ["git", "curl", "tar", "bash"] {
        report_tool(tool, true);
    }

    section("Optional tools");
    for tool in [
        "node", "gh", "atlas", "psql", "mysql", "sqlite3", "duckdb", "rtk", "claude",
    ] {
        report_tool(tool, false);
    }

    section("Genasis assets");
    report_path(&project_root.join(CONFIG_FILE_NAME), "genasis.toml");
    report_path(&project_root.join("GENASIS.md"), "GENASIS.md");
    report_path(
        &project_root.join("docs").join("design-system.md"),
        "docs/design-system.md",
    );
    report_path(
        &project_root.join(".claude").join("agents"),
        ".claude/agents/",
    );

    section("Genasis config");
    let cfg_path = project_root.join(CONFIG_FILE_NAME);
    match Config::load(&cfg_path) {
        Ok(cfg) => {
            println!("  project.name = {}", cfg.project.name);
            println!("  project.domain = {}", cfg.project.domain);
            if let Some(p) = &cfg.plane {
                println!("  plane.flavor = {} ({})", p.flavor, p.url);
            } else {
                println!("  [plane] missing");
            }
            if let Some(m) = &cfg.mattermost {
                println!("  mattermost.flavor = {} ({})", m.flavor, m.url);
            } else {
                println!("  [mattermost] missing");
            }
            if let Some(d) = &cfg.db {
                println!(
                    "  db.driver = {}, migration_tool = {}",
                    d.driver, d.migration_tool
                );
            } else {
                println!("  [db] missing");
            }
        }
        Err(e) => println!("  could not load config: {e}"),
    }

    section("Environment secrets");
    for k in [
        "PLANE_API_KEY",
        "MM_ADMIN_TOKEN",
        "PLANE_TOKEN_PM",
        "PLANE_TOKEN_FRONTEND",
        "MM_TOKEN_PM",
    ] {
        let present = std::env::var(k).is_ok();
        println!("  {k}: {}", if present { "set ✓" } else { "unset" });
    }

    Ok(())
}

fn resolve_project_root(arg: Option<&std::path::Path>) -> Result<PathBuf> {
    if let Some(p) = arg {
        return Ok(p.canonicalize()?);
    }
    let cwd = std::env::current_dir()?;
    if let Some(cfg) = Config::discover(&cwd) {
        if let Some(parent) = cfg.parent() {
            return Ok(parent.to_path_buf());
        }
    }
    Ok(cwd)
}

fn section(title: &str) {
    println!("\n• {title}");
}

fn report_tool(name: &str, required: bool) {
    let label = if required { "required" } else { "optional" };
    match which::which(name) {
        Ok(p) => println!("  {name} ({label}): {} ✓", p.display()),
        Err(_) => {
            let icon = if required { "✗ MISSING" } else { "missing" };
            println!("  {name} ({label}): {icon}");
        }
    }
}

fn report_path(p: &std::path::Path, label: &str) {
    if p.exists() {
        println!("  {label}: present ✓");
    } else {
        println!("  {label}: missing");
    }
}
