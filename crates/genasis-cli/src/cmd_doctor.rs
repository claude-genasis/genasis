use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use genasis_core::config::{Config, CONFIG_FILE_NAME};
use genasis_i18n::{tr, tr_args};

#[derive(Parser, Debug)]
pub struct Args {
    /// Project root. Defaults to the current working directory.
    #[arg(long, value_name = "DIR")]
    pub project: Option<PathBuf>,
}

pub async fn run(args: Args) -> Result<()> {
    let project_root = resolve_project_root(args.project.as_deref())?;
    println!(
        "{} — project: {}",
        tr("doctor.header"),
        project_root.display()
    );

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

    section(&tr("doctor.design.section"));
    let cfg_path_design = project_root.join(CONFIG_FILE_NAME);
    let design_cfg = Config::load(&cfg_path_design)
        .ok()
        .and_then(|c| c.design)
        .unwrap_or_default();
    let state = genasis_design::State::load(&project_root)
        .ok()
        .unwrap_or_default();
    match state.mode {
        genasis_design::Mode::Pristine => {
            println!("  {}", tr("doctor.design.mode_pristine"));
            // npx is needed only when transitioning to external mode.
            match which::which("npx") {
                Ok(p) => println!("  npx (optional for slug swap): {} ✓", p.display()),
                Err(_) => println!("  {}", tr("doctor.design.npx_missing_optional")),
            }
        }
        genasis_design::Mode::External => {
            println!(
                "  {}",
                tr_args("doctor.design.mode_external", &[("slug", &state.slug)],)
            );
            // npx is required when external mode is in use (so subsequent
            // swaps can fetch new slugs).
            match which::which("npx") {
                Ok(p) => println!("  npx (required): {} ✓", p.display()),
                Err(_) => println!("  {}", tr("doctor.design.npx_missing_required")),
            }
            // Hash check: re-run verify and emit a single-line result.
            match genasis_design::run_verify(&project_root, &design_cfg.external_dir) {
                Ok(v) => {
                    if v.matches {
                        println!(
                            "  {}",
                            tr_args(
                                "doctor.design.verify_ok",
                                &[("hash_short", &v.actual_hash[..v.actual_hash.len().min(12)],)],
                            )
                        );
                    } else {
                        println!(
                            "  {}",
                            tr_args(
                                "doctor.design.verify_tampered",
                                &[
                                    (
                                        "expected",
                                        &v.recorded_hash[..v.recorded_hash.len().min(12)],
                                    ),
                                    ("actual", &v.actual_hash[..v.actual_hash.len().min(12)],),
                                ],
                            )
                        );
                    }
                }
                Err(e) => {
                    println!(
                        "  {}",
                        tr_args("doctor.design.verify_error", &[("reason", &e.to_string())])
                    );
                }
            }
            // Coherence: the pointer body must exist, and design-system/ dir must exist.
            let pointer = project_root.join("docs").join("design-system.md");
            let extdir = project_root.join(&design_cfg.external_dir);
            if !pointer.is_file() {
                println!("  {}", tr("doctor.design.pointer_missing"));
            }
            if !extdir.is_dir() {
                println!(
                    "  {}",
                    tr_args(
                        "doctor.design.extdir_missing",
                        &[("path", &extdir.display().to_string())],
                    )
                );
            }
            println!(
                "  overrides: {} | preview: {}",
                state.override_count, state.gallery_preview
            );
        }
    }

    section(&tr("doctor.i18n.section"));
    let resolved = genasis_i18n::resolve(None, None);
    println!(
        "  {}",
        tr_args(
            "doctor.i18n.runtime_locale",
            &[
                ("lang", resolved.lang.code()),
                ("source", resolved.source.label()),
            ]
        )
    );
    let cfg_path2 = project_root.join(CONFIG_FILE_NAME);
    if let Ok(cfg) = Config::load(&cfg_path2) {
        if let Some(i18n) = &cfg.i18n {
            println!(
                "  {}",
                tr_args("doctor.i18n.active_agent_locale", &[("lang", &i18n.active)])
            );
            if i18n.reference_langs.is_empty() {
                println!("  {}", tr("doctor.i18n.reference_docs_none"));
            } else {
                let langs = i18n.reference_langs.join(", ");
                println!(
                    "  {}",
                    tr_args("doctor.i18n.reference_docs_listed", &[("langs", &langs)])
                );
            }
        } else {
            println!("  [i18n] not configured (run `genasis attach --lang en|ko`)");
        }
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
