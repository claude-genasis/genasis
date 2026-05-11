//! `genasis bootstrap` — green-field scaffolding entry point.
//!
//! Per ADR-010 §3 (decision (b)+(d), ratified 2026-05-08), bootstrap is an
//! explicit subcommand rather than an automatic side-effect of `attach`.
//! It scaffolds the canonical 10-role base agent files into
//! `.claude/agents/` and, by default, chains into `cmd_attach` so the
//! marker-fence patches land in one shot. `--no-attach-after` separates
//! the two stages for callers (or tests) that want to inspect the
//! bootstrap output before any fence is injected.
//!
//! `genasis init --bootstrap` is an alias that delegates here so the two
//! flows stay byte-identical (M14.3).

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use genasis_i18n::tr_args;
use genasis_overlay::{apply_bootstrap, plan_bootstrap, BootstrapAction, BootstrapOptions, Role};

use crate::cmd_attach;
use crate::lang_prompt;

#[derive(Parser, Debug)]
pub struct Args {
    /// Project root. Defaults to the current working directory.
    #[arg(long, value_name = "DIR")]
    pub project: Option<PathBuf>,

    /// Comma-separated subset of roles to scaffold (default: all 10).
    /// Example: `--roles pm,frontend,backend,qa`.
    #[arg(long, value_name = "LIST")]
    pub roles: Option<String>,

    /// Skip the auto-chained `cmd_attach` call. Use this when you want
    /// to inspect the freshly written base files before the patch fence
    /// is injected.
    #[arg(long)]
    pub no_attach_after: bool,

    /// Print the planned actions without writing anything.
    #[arg(long)]
    pub dry_run: bool,
}

pub async fn run(
    args: Args,
    lang_flag: Option<String>,
    non_interactive: bool,
    assume_yes: bool,
) -> Result<()> {
    let project_root = resolve_project_root(args.project.as_deref())?;
    let decision = lang_prompt::decide(lang_flag.as_deref(), non_interactive, assume_yes)?;

    let roles = parse_roles(args.roles.as_deref())?;
    let opts = BootstrapOptions {
        lang: decision.lang.code().to_string(),
        roles,
    };

    // Load the agents catalog (auto-fetch is governed by genasis.toml /
    // GENASIS_AGENTS_AUTO_CHECK; we mirror cmd_attach's defaults so the
    // user sees a single resolution path).
    let agents_cfg = load_agents_config(&project_root);
    let store = genasis_templates::load(
        &agents_cfg.version,
        &agents_cfg.registry,
        &agents_cfg.cache_dir,
        agents_cfg.auto_check,
    )
    .context("load agents catalog for bootstrap")?;

    let plan = plan_bootstrap(&project_root, &opts, &store)?;

    let create_count = plan.creates().count();
    let skip_count = plan.skips().count();

    if create_count == 0 {
        println!(
            "{}",
            tr_args(
                "bootstrap.skipped_existing",
                &[("name", &format!("{skip_count} role(s)"))]
            )
        );
    } else {
        println!(
            "{}",
            tr_args(
                "bootstrap.scaffolded_summary",
                &[("count", &create_count.to_string())]
            )
        );
    }
    for change in plan.creates() {
        match &change.action {
            BootstrapAction::Create { source_alias, .. } if source_alias != change.role.slug() => {
                // Field alias was used because the canonical slug
                // wasn't shipped in the catalog (e.g. v1.0.0 has
                // `frontend-developer.md`, not `frontend.md`). Surface
                // it so the user can verify they got what they wanted.
                println!(
                    "  + {} (resolved via {}.md)",
                    change.path.display(),
                    source_alias
                );
            }
            _ => {
                println!("  + {}", change.path.display());
            }
        }
    }
    for change in plan.skips() {
        println!("  = {} (exists)", change.path.display());
    }
    // ADR-017 field-feedback: surface roles the catalog couldn't
    // satisfy instead of aborting hard. Users can patch the catalog
    // or hand-author the base file; bootstrap installs everything
    // else.
    let missing: Vec<_> = plan.missing().collect();
    if !missing.is_empty() {
        eprintln!(
            "\n⚠ {} role(s) had no catalog match — bootstrap skipped them:",
            missing.len()
        );
        for change in &missing {
            let tried = match &change.action {
                BootstrapAction::Missing { tried } => tried.join(", "),
                _ => String::new(),
            };
            eprintln!("  ! {} — tried {}", change.role.slug(), tried);
        }
        eprintln!(
            "  hint: hand-author `.claude/agents/<role>.md` or run `genasis agents update` for a newer catalog\n"
        );
    }

    if args.dry_run {
        return Ok(());
    }

    if create_count > 0 {
        let report = apply_bootstrap(&plan)?;
        tracing::info!(wrote = report.written.len(), "bootstrap apply complete");
    }

    if args.no_attach_after {
        println!("\n{}", tr_args("bootstrap.next_step", &[]));
        return Ok(());
    }

    // Auto-chain into cmd_attach so the marker fence lands in the
    // freshly written base files. We forward the same lang flag so the
    // overlay renders in the chosen locale.
    let attach_args = cmd_attach::Args {
        project: Some(project_root),
        dry_run: false,
        diff: false,
        force: false,
        fence_version: "1.0".to_string(),
        reference_docs: Vec::new(),
    };
    cmd_attach::pub_run(attach_args, lang_flag, non_interactive, assume_yes).await
}

fn resolve_project_root(arg: Option<&std::path::Path>) -> Result<PathBuf> {
    if let Some(p) = arg {
        // Allow scaffolding into a directory that does not yet exist —
        // M14 explicitly targets blank projects, so we create it lazily.
        if !p.exists() {
            std::fs::create_dir_all(p)
                .with_context(|| format!("create --project dir {}", p.display()))?;
        }
        return p
            .canonicalize()
            .with_context(|| format!("canonicalize {}", p.display()));
    }
    Ok(std::env::current_dir()?)
}

fn parse_roles(spec: Option<&str>) -> Result<Vec<Role>> {
    let Some(raw) = spec else {
        return Ok(Role::ALL.to_vec());
    };
    let mut out = Vec::new();
    for token in raw.split(',') {
        let trimmed = token.trim();
        if trimmed.is_empty() {
            continue;
        }
        let role = Role::ALL
            .iter()
            .copied()
            .find(|r| r.slug() == trimmed)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "unknown role slug: {trimmed}; valid: {}",
                    Role::ALL
                        .iter()
                        .map(|r| r.slug())
                        .collect::<Vec<_>>()
                        .join(",")
                )
            })?;
        out.push(role);
    }
    if out.is_empty() {
        return Ok(Role::ALL.to_vec());
    }
    Ok(out)
}

struct AgentsConfig {
    version: String,
    registry: String,
    cache_dir: String,
    auto_check: bool,
}

fn load_agents_config(_project_root: &std::path::Path) -> AgentsConfig {
    AgentsConfig {
        version: std::env::var("GENASIS_AGENTS_VERSION").unwrap_or_else(|_| "1.0.0".to_string()),
        registry: std::env::var("GENASIS_AGENTS_REGISTRY")
            .unwrap_or_else(|_| "https://github.com/claude-genasis/genasis/releases".to_string()),
        cache_dir: std::env::var("GENASIS_AGENTS_CACHE_DIR").unwrap_or_default(),
        auto_check: true,
    }
}

#[allow(dead_code)]
fn dead_code_silencer(_: &BootstrapAction) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_roles_default_returns_all_ten() {
        let v = parse_roles(None).unwrap();
        assert_eq!(v.len(), Role::ALL.len());
    }

    #[test]
    fn parse_roles_subset_filters_correctly() {
        let v = parse_roles(Some("pm,frontend, qa")).unwrap();
        let slugs: Vec<&str> = v.iter().map(|r| r.slug()).collect();
        assert_eq!(slugs, vec!["pm", "frontend", "qa"]);
    }

    #[test]
    fn parse_roles_empty_string_falls_back_to_all() {
        let v = parse_roles(Some("")).unwrap();
        assert_eq!(v.len(), Role::ALL.len());
    }

    #[test]
    fn parse_roles_unknown_slug_errors() {
        let err = parse_roles(Some("not-a-role")).unwrap_err();
        assert!(err.to_string().contains("unknown role slug"));
    }
}
