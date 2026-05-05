use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};

use genasis_core::config::{Config, DesignConfig, CONFIG_FILE_NAME};
use genasis_design::{
    auto_plan, override_add, override_list, override_remove, run_legacy_swap, run_restore,
    run_swap, run_verify, swap::SwapInput, Locale, Mode, Plan, PlanMode, State, SwapSource,
    DEFAULT_FULL_REWRITE_THRESHOLD,
};
use genasis_i18n::{tr, tr_args};

#[derive(Parser, Debug)]
pub struct Args {
    /// Project root. Defaults to the current working directory.
    #[arg(long, value_name = "DIR", global = true)]
    pub project: Option<PathBuf>,

    #[command(subcommand)]
    pub op: DesignOp,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum CliPlanMode {
    Auto,
    PerArea,
    FullRewrite,
}

impl From<CliPlanMode> for PlanMode {
    fn from(m: CliPlanMode) -> Self {
        match m {
            CliPlanMode::Auto => PlanMode::Auto,
            CliPlanMode::PerArea => PlanMode::PerArea,
            CliPlanMode::FullRewrite => PlanMode::FullRewrite,
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum DesignOp {
    /// Swap to an external design system. Three shapes:
    /// `swap <slug>`              — fetch via `[design].add_command` (default: npx getdesign).
    /// `swap --from <path>`       — copy a local spec file (no network).
    /// `swap <url> --body <path>` — legacy M7 path (extractor wrote the new body).
    Swap {
        /// Slug or reference URL. Mutually exclusive with --from.
        #[arg(value_name = "SLUG_OR_URL")]
        target: Option<String>,
        /// Path to a local design specification (`--from <path>`). Skips npx.
        #[arg(long, value_name = "PATH")]
        from: Option<PathBuf>,
        /// Legacy M7: pre-rendered design-system.md body. Implies the
        /// 5-phase change_protocol pipeline.
        #[arg(long, value_name = "PATH")]
        body: Option<PathBuf>,
        /// Force telemetry on for this invocation (overrides genasis.toml).
        #[arg(long)]
        telemetry: bool,
        /// EPIC vs per-area planning mode. Auto picks based on the
        /// `>= threshold of 7` heuristic (default 4).
        #[arg(long, value_enum, default_value_t = CliPlanMode::Auto)]
        plan: CliPlanMode,
    },
    /// Print the current design status (mode, slug, overrides, preview URL).
    Status,
    /// Restore the project from external mode back to its pristine
    /// `docs/design-system.md` body.
    Restore,
    /// Re-hash the active external DESIGN.md and compare to the recorded
    /// `template_hash`. Reports OK or "tampered".
    Verify,
    /// Manage user-override entries accumulated under §B.2.
    Override {
        #[command(subcommand)]
        op: OverrideOp,
    },
}

#[derive(Subcommand, Debug)]
pub enum OverrideOp {
    /// Append a new override entry. Pre-condition: the agent has already
    /// surfaced any §A conflict and the user said yes.
    Add {
        /// Body of the override (single string).
        text: String,
    },
    /// List override entries currently in §B.2.
    List,
    /// Remove an override entry by id (e.g. `override-2`).
    Remove {
        /// Override id.
        id: String,
    },
}

pub async fn run(args: Args) -> Result<()> {
    let project_root = resolve_project_root(args.project.as_deref())?;
    let cfg = load_config_or_default(&project_root)?;
    let design_cfg = cfg.design.clone().unwrap_or_default();
    let locale = Locale::from_active(&active_locale(&cfg));

    match args.op {
        DesignOp::Swap {
            target,
            from,
            body,
            telemetry,
            plan,
        } => {
            run_swap_op(
                &project_root,
                &design_cfg,
                locale,
                target,
                from,
                body,
                telemetry,
                plan.into(),
            )
            .await
        }
        DesignOp::Status => run_status(&project_root, &design_cfg).await,
        DesignOp::Restore => run_restore_op(&project_root, &design_cfg).await,
        DesignOp::Verify => run_verify_op(&project_root, &design_cfg).await,
        DesignOp::Override { op } => run_override_op(&project_root, op).await,
    }
}

async fn run_swap_op(
    project_root: &std::path::Path,
    design_cfg: &DesignConfig,
    locale: Locale,
    target: Option<String>,
    from: Option<PathBuf>,
    body: Option<PathBuf>,
    telemetry: bool,
    plan_mode: PlanMode,
) -> Result<()> {
    // Legacy M7 path: `swap <url> --body <path>` — extractor produced the
    // body, we run the 5-phase change_protocol.
    if let (Some(url), Some(body_path)) = (target.as_ref(), body.as_ref()) {
        let new_body = std::fs::read_to_string(body_path)
            .with_context(|| format!("read --body: {}", body_path.display()))?;
        let outcome = run_legacy_swap(project_root, url, &new_body)?;
        println!("{}", tr_args("design.swap.header", &[("url", url)]));
        println!(
            "  {}",
            tr_args(
                "design.swap.previous_present",
                &[("value", &outcome.previous_present.to_string())],
            )
        );
        println!(
            "  {}",
            tr_args(
                "design.swap.impacted_areas",
                &[("count", &outcome.areas.len().to_string())],
            )
        );
        for a in &outcome.areas {
            println!("    - {:?}", a);
        }
        println!("  {}", tr("design.swap.planned_issues"));
        for issue in &outcome.planned_issues {
            println!("    - [{}] {}", issue.label, issue.title);
        }
        println!("\n{}", tr("design.swap.next_step_1"));
        println!("{}", tr("design.swap.next_step_2"));
        return Ok(());
    }

    // Phase D path: slug or --from.
    let source = match (target, from) {
        (Some(slug), None) => SwapSource::Slug {
            slug,
            add_command: design_cfg.add_command.clone(),
        },
        (None, Some(path)) => SwapSource::File(path),
        (Some(_), Some(_)) => {
            anyhow::bail!("pass either <slug> or --from <path>, not both");
        }
        (None, None) => {
            anyhow::bail!("nothing to swap — pass a slug, --from <path>, or <url> --body <path>");
        }
    };

    let disable_telemetry = if telemetry {
        false
    } else {
        design_cfg.disable_telemetry
    };

    let input = SwapInput {
        project_root: project_root.to_path_buf(),
        external_dir: design_cfg.external_dir.clone(),
        gallery_index_url: design_cfg.gallery_index_url.clone(),
        gallery_url_template: design_cfg.gallery_url_template.clone(),
        disable_telemetry,
        locale,
        source: source.clone(),
    };

    match &source {
        SwapSource::Slug { slug, add_command } => {
            let cmd = add_command.replace("{slug}", slug).replace(
                "{out}",
                &project_root
                    .join(&design_cfg.external_dir)
                    .join("DESIGN.md")
                    .display()
                    .to_string(),
            );
            println!("{}", tr_args("design.swap.delegating", &[("cmd", &cmd)]));
        }
        SwapSource::File(path) => {
            println!(
                "{}",
                tr_args(
                    "design.swap.from_local",
                    &[("path", &path.display().to_string())],
                )
            );
        }
    }

    let outcome = run_swap(input).context("design swap failed")?;

    if let Some(bak) = &outcome.pristine_backup_path {
        println!(
            "{}",
            tr_args(
                "design.swap.pristine_backed_up",
                &[("path", &bak.display().to_string())],
            )
        );
    }
    let hash_short =
        &outcome.new_state.template_hash[..outcome.new_state.template_hash.len().min(12)];
    println!(
        "{}",
        tr_args(
            "design.swap.design_md_written",
            &[
                ("path", &outcome.design_md_path.display().to_string()),
                ("bytes", &outcome.design_md_size.to_string()),
                ("hash_short", hash_short),
            ],
        )
    );
    println!(
        "{}",
        tr_args(
            "design.swap.pointer_written",
            &[("path", &outcome.pointer_path.display().to_string())],
        )
    );
    println!(
        "{}",
        tr_args(
            "design.swap.state_updated",
            &[("slug", &outcome.new_state.slug)],
        )
    );

    // Plan EPIC vs per-area issues. Diff the *previous external body* (or
    // pristine body) against the new external body to compute changed areas.
    let prev_body = if outcome.previous_state.mode == Mode::External {
        let prev_design_md = project_root
            .join(&design_cfg.external_dir)
            .join("DESIGN.md");
        std::fs::read_to_string(&prev_design_md).unwrap_or_default()
    } else if let Some(bak) = &outcome.pristine_backup_path {
        std::fs::read_to_string(bak).unwrap_or_default()
    } else {
        String::new()
    };
    let new_body = std::fs::read_to_string(&outcome.design_md_path).unwrap_or_default();
    // For accurate diff after the swap (DESIGN.md is now the new body), we
    // approximate "changed areas" by diffing prev vs new directly.
    let areas = genasis_design::changed_areas(&prev_body, &new_body);
    let reference_url = outcome.new_state.gallery_preview.clone();
    let plan = auto_plan(
        plan_mode,
        &areas,
        &reference_url,
        &outcome.new_state.slug,
        &outcome.new_state.gallery_preview,
        DEFAULT_FULL_REWRITE_THRESHOLD,
    );
    print_plan(&plan, areas.len());

    // Mattermost announcement template (caller is responsible for posting —
    // typically via `cmd_init`'s provider client; we emit the canned body).
    println!("\n{}", tr("design.swap.mattermost_template_header"));
    let prev_label = if outcome.previous_state.mode == Mode::External {
        outcome.previous_state.slug.clone()
    } else {
        "pristine".to_string()
    };
    println!(
        "{}",
        tr_args(
            "design.swap.mattermost_template_body",
            &[
                ("from", &prev_label),
                ("to", &outcome.new_state.slug),
                ("preview_url", &outcome.new_state.gallery_preview),
                ("issue_count", &plan.issue_count().to_string()),
            ],
        )
    );

    println!("\n{}", tr("design.swap.post_swap_header"));
    println!("{}", tr("design.swap.post_swap_1"));
    println!(
        "{}",
        tr_args(
            "design.swap.post_swap_2",
            &[("preview_url", &outcome.new_state.gallery_preview)],
        )
    );
    println!(
        "{}",
        tr_args(
            "design.swap.post_swap_3",
            &[("gallery_url", &outcome.new_state.gallery_index)],
        )
    );
    println!("{}", tr("design.swap.post_swap_4"));

    Ok(())
}

fn print_plan(plan: &Plan, area_count: usize) {
    match plan {
        Plan::PerArea(items) => {
            println!(
                "\n{}",
                tr_args(
                    "design.plan.per_area_header",
                    &[("count", &items.len().to_string())],
                )
            );
            for it in items {
                println!("    - [{}] {}", it.label, it.title);
            }
        }
        Plan::FullRewrite { epic, children } => {
            println!(
                "\n{}",
                tr_args(
                    "design.plan.full_rewrite_header",
                    &[
                        ("areas", &area_count.to_string()),
                        ("threshold", &DEFAULT_FULL_REWRITE_THRESHOLD.to_string()),
                    ],
                )
            );
            println!("    [{}] {}", epic.label, epic.title);
            for c in children {
                println!("      ├─ [{}] {}", c.label, c.title);
            }
        }
    }
}

async fn run_status(project_root: &std::path::Path, _design_cfg: &DesignConfig) -> Result<()> {
    let state = State::load(project_root)?;
    match state.mode {
        Mode::Pristine => {
            println!("{}", tr("design.status.mode_pristine"));
            let target = project_root.join("docs").join("design-system.md");
            if let Ok(meta) = std::fs::metadata(&target) {
                println!(
                    "{}",
                    tr_args(
                        "design.status.file_size",
                        &[("bytes", &meta.len().to_string())],
                    )
                );
            } else {
                println!("{}", tr("design.status.missing"));
            }
        }
        Mode::External => {
            println!(
                "{}",
                tr_args(
                    "design.status.mode_external",
                    &[("slug", &state.slug), ("applied_at", &state.applied_at)],
                )
            );
            println!(
                "{}",
                tr_args("design.status.source", &[("source", &state.source)])
            );
            println!(
                "{}",
                tr_args(
                    "design.status.overrides",
                    &[("count", &state.override_count.to_string())],
                )
            );
            println!(
                "{}",
                tr_args("design.status.preview", &[("url", &state.gallery_preview)])
            );
            println!(
                "{}",
                tr_args("design.status.gallery", &[("url", &state.gallery_index)])
            );
        }
    }
    Ok(())
}

async fn run_restore_op(
    project_root: &std::path::Path,
    design_cfg: &DesignConfig,
) -> Result<()> {
    let outcome =
        run_restore(project_root, &design_cfg.external_dir).context("design restore failed")?;
    println!(
        "{}",
        tr_args(
            "design.restore.archived",
            &[("path", &outcome.archive_dir.display().to_string())],
        )
    );
    if outcome.design_system_md_restored {
        println!("{}", tr("design.restore.body_restored"));
    } else {
        println!("{}", tr("design.restore.no_backup"));
    }
    println!("{}", tr("design.restore.state_cleared"));
    Ok(())
}

async fn run_verify_op(
    project_root: &std::path::Path,
    design_cfg: &DesignConfig,
) -> Result<()> {
    let outcome =
        run_verify(project_root, &design_cfg.external_dir).context("design verify failed")?;
    match outcome.mode {
        Mode::Pristine => {
            println!("{}", tr("design.verify.pristine_skip"));
        }
        Mode::External => {
            if outcome.matches {
                println!(
                    "{}",
                    tr_args(
                        "design.verify.ok",
                        &[(
                            "hash_short",
                            &outcome.actual_hash[..outcome.actual_hash.len().min(12)],
                        )],
                    )
                );
            } else {
                eprintln!(
                    "{}",
                    tr_args(
                        "design.verify.tampered",
                        &[
                            (
                                "expected",
                                &outcome.recorded_hash[..outcome.recorded_hash.len().min(12)],
                            ),
                            (
                                "actual",
                                &outcome.actual_hash[..outcome.actual_hash.len().min(12)],
                            ),
                        ],
                    )
                );
                anyhow::bail!("design verify: hash mismatch");
            }
        }
    }
    Ok(())
}

async fn run_override_op(project_root: &std::path::Path, op: OverrideOp) -> Result<()> {
    match op {
        OverrideOp::Add { text } => {
            let entry = override_add(project_root, &text).context("override add failed")?;
            println!(
                "{}",
                tr_args(
                    "design.override.added",
                    &[
                        ("id", &entry.id),
                        ("applied_at", &entry.applied_at),
                    ],
                )
            );
            println!("    {}", entry.body);
        }
        OverrideOp::List => {
            let entries = override_list(project_root).context("override list failed")?;
            if entries.is_empty() {
                println!("{}", tr("design.override.list_empty"));
            } else {
                println!(
                    "{}",
                    tr_args(
                        "design.override.list_header",
                        &[("count", &entries.len().to_string())],
                    )
                );
                for e in entries {
                    println!("  {} @ {}", e.id, e.applied_at);
                    for line in e.body.lines() {
                        println!("      {line}");
                    }
                }
            }
        }
        OverrideOp::Remove { id } => {
            let removed = override_remove(project_root, &id).context("override remove failed")?;
            if removed {
                println!("{}", tr_args("design.override.removed", &[("id", &id)]));
            } else {
                println!("{}", tr_args("design.override.not_found", &[("id", &id)]));
            }
        }
    }
    Ok(())
}

fn active_locale(cfg: &Config) -> String {
    cfg.i18n
        .as_ref()
        .map(|i| i.active.clone())
        .unwrap_or_else(|| "en".to_string())
}

fn load_config_or_default(project_root: &std::path::Path) -> Result<Config> {
    let path = project_root.join(CONFIG_FILE_NAME);
    if path.is_file() {
        Ok(Config::load(&path)?)
    } else {
        Ok(Config::default())
    }
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
