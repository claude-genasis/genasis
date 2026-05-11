use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use genasis_core::config::{Config, DesignConfig, I18nConfig, CONFIG_FILE_NAME};
use genasis_i18n::tr_args;
use genasis_overlay::{plan_attach, scan, summary, unified_diff, AttachOptions};

use crate::lang_prompt;

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

    /// Additional language(s) to keep on disk as reference docs (not
    /// `@import`'d). Repeatable.
    #[arg(long = "reference-docs", value_name = "LANG")]
    pub reference_docs: Vec<String>,
}

#[allow(dead_code)]
pub async fn run(args: Args) -> Result<()> {
    pub_run(args, None, false, false).await
}

pub async fn pub_run(
    args: Args,
    lang_flag: Option<String>,
    non_interactive: bool,
    assume_yes: bool,
) -> Result<()> {
    // Resolve install language. Interactive prompt fires when no flag and
    // stdin is a TTY; otherwise falls back to $LANG.
    let decision = lang_prompt::decide(lang_flag.as_deref(), non_interactive, assume_yes)?;
    tracing::info!(
        install_lang = %decision.lang,
        via = decision.via.label(),
        "attach: language decided"
    );

    let project_root = resolve_project_root(args.project.as_deref())?;
    tracing::info!(project_root = %project_root.display(), "attach: scanning agents");

    // Persist the locale choice into genasis.toml [i18n].
    persist_i18n_choice(&project_root, decision, &args.reference_docs)?;
    // Seed `[design]` with default getdesign URLs the first time we attach.
    // Phase D — the gallery is replaceable later via genasis.toml edits.
    seed_design_defaults(&project_root)?;

    let report = scan(&project_root)?;
    if !report.skipped.is_empty() {
        for (path, why) in &report.skipped {
            tracing::warn!(path = %path.display(), reason = %why, "skipped agent");
        }
    }
    // ADR-010 §3: if no agents are present at all, surface the bootstrap
    // entry point instead of silently doing nothing.
    if report.agents.is_empty() && report.skipped.is_empty() {
        eprintln!("{}", genasis_i18n::tr("bootstrap.no_agents_hint"));
    }

    let context = build_context(&project_root)?;
    let opts = AttachOptions {
        fence_version: args.fence_version.clone(),
        context,
        force: args.force,
        lang: decision.lang.code().to_string(),
    };

    // ADR-011: Load agents catalog from cache (auto-fetch if [agents].auto_check).
    let agents_cfg = load_agents_config(&project_root);
    let store = genasis_templates::load(
        &agents_cfg.version,
        &agents_cfg.registry,
        &agents_cfg.cache_dir,
        agents_cfg.auto_check,
    )?;
    write_reference_docs(&project_root, &args.reference_docs, decision.lang, &store)?;
    let plan = plan_attach(&report.agents, &opts, &store)?;

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
        anyhow::bail!(
            "{}",
            tr_args("attach.refused", &[("count", &refused.to_string())])
        );
    }

    let applied = genasis_overlay::apply(&plan)?;
    println!(
        "\n{}",
        tr_args(
            "attach.wrote_summary",
            &[
                ("count", &applied.written.len().to_string()),
                ("backups", &applied.backups.len().to_string()),
            ]
        )
    );

    // v0.5.2 — install the overlay artifacts the README has always
    // promised (GENASIS.md + .claude/genasis/{commands,hooks,skills}/)
    // but `apply()` never produced. Field testing surfaced that
    // attach was writing only the agent fence, leaving sprint-start
    // / issue-done / db-migrate slash commands and session hooks
    // entirely unscaffolded — even though the catalog tarball
    // already ships them. Fix is non-destructive (overwrites on
    // re-attach so user-edited bodies need backup; matches how
    // agent fences are handled today).
    let install_count =
        install_genasis_overlay_artifacts(&project_root, &store, decision.lang.code())
            .unwrap_or_else(|e| {
                tracing::warn!(reason = %e, "overlay artifacts install failed");
                eprintln!("  ⚠ failed to install commands/hooks/skills: {e}");
                0
            });
    if install_count > 0 {
        println!(
            "  + {} overlay artifact(s) under .claude/genasis/ + GENASIS.md",
            install_count
        );
    }

    // M15.2 — refresh `.claude/genasis/.manifest.json` so the next CLI
    // invocation can detect drift against this canonical state.
    if let Err(e) = update_manifest_after_apply(&project_root, &applied, decision.lang.code()) {
        tracing::warn!(reason = %e, "manifest refresh failed after attach");
    }

    Ok(())
}

/// Install the GENASIS.md protocol contract + slash commands + hooks
/// + skills that the catalog ships under `<lang>/GENASIS.md.tera`,
/// `commands/*.tera`, `hooks/*.tera`, and `skills/*.tera` (Issue #11
/// from the v0.5.0 field test log). Returns the count of files
/// written so the CLI can report it.
///
/// The v1.0.0 catalog's templates do not contain Tera variables —
/// they're pre-rendered protocol text — so we pass them through
/// `Tera::one_off` with the standard project-name / project-domain
/// context anyway. Future catalogs that DO want variables get them
/// automatically via the same context.
///
/// Hooks ending in `.sh` are chmod'd 0755 on Unix so Claude Code's
/// PostToolUse / SessionStart hook runner can execute them directly.
fn install_genasis_overlay_artifacts(
    project_root: &std::path::Path,
    store: &genasis_templates::AgentStore,
    lang_code: &str,
) -> Result<usize> {
    use std::fs;

    let ctx_value = build_context(project_root)?;
    let ctx = tera::Context::from_value(ctx_value)
        .map_err(|e| anyhow::anyhow!("build Tera context: {e}"))?;
    let mut written: usize = 0;

    // GENASIS.md at project root — the protocol contract that
    // CLAUDE.md @imports. The catalog only ships `en/GENASIS.md.tera`
    // and `ko/GENASIS.md.tera`, so fall back to English if the
    // active lang isn't present.
    let genasis_md_template = store
        .get_file(&format!("{lang_code}/GENASIS.md.tera"))
        .or_else(|| store.get_file("en/GENASIS.md.tera"));
    if let Some(body) = genasis_md_template {
        let rendered = tera::Tera::one_off(&body, &ctx, true)
            .map_err(|e| anyhow::anyhow!("render GENASIS.md: {e}"))?;
        let target = project_root.join("GENASIS.md");
        fs::write(&target, rendered).with_context(|| format!("write {}", target.display()))?;
        written += 1;
    }

    for (subdir, out_subdir) in [
        ("commands", ".claude/genasis/commands"),
        ("hooks", ".claude/genasis/hooks"),
        ("skills", ".claude/genasis/skills"),
    ] {
        let out_dir = project_root.join(out_subdir);
        fs::create_dir_all(&out_dir).with_context(|| format!("create {}", out_dir.display()))?;
        let files = match store.get_dir_files(subdir, ".tera") {
            Ok(f) => f,
            Err(_) => continue, // empty / missing subdir is fine
        };
        for (name, body) in files {
            let rendered = tera::Tera::one_off(&body, &ctx, true)
                .map_err(|e| anyhow::anyhow!("render {name}: {e}"))?;
            let out_name = name.strip_suffix(".tera").unwrap_or(&name);
            let target = out_dir.join(out_name);
            fs::write(&target, rendered).with_context(|| format!("write {}", target.display()))?;
            #[cfg(unix)]
            if out_name.ends_with(".sh") {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(&target, fs::Permissions::from_mode(0o755));
            }
            written += 1;
        }
    }

    Ok(written)
}

fn update_manifest_after_apply(
    project_root: &std::path::Path,
    applied: &genasis_overlay::AppliedReport,
    lang_code: &str,
) -> Result<()> {
    use genasis_core::manifest::{hash_file, FileEntry, Manifest};

    let mut manifest = Manifest::load(project_root)
        .ok()
        .flatten()
        .unwrap_or_else(|| Manifest::new(env!("CARGO_PKG_VERSION")));
    manifest.lang = lang_code.to_string();
    manifest.attached_at = chrono::Utc::now().to_rfc3339();

    for written_path in &applied.written {
        let rel = match written_path.strip_prefix(project_root) {
            Ok(r) => r.to_string_lossy().into_owned(),
            Err(_) => continue,
        };
        let sha = hash_file(written_path)?
            .ok_or_else(|| anyhow::anyhow!("hash_file returned None for written path"))?;
        manifest.files.insert(
            rel,
            FileEntry {
                sha256: sha,
                ..Default::default()
            },
        );
    }
    manifest.save(project_root)?;
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

/// Persist the chosen language into `genasis.toml [i18n]`. If the file
/// does not exist yet (blank-project case), write a minimal scaffold so
/// later commands can rely on it.
fn persist_i18n_choice(
    project_root: &std::path::Path,
    decision: lang_prompt::Decision,
    reference_docs: &[String],
) -> Result<()> {
    let cfg_path = project_root.join(CONFIG_FILE_NAME);
    let mut cfg = if cfg_path.is_file() {
        Config::load(&cfg_path)?
    } else {
        Config::default()
    };
    cfg.i18n = Some(I18nConfig {
        active: decision.lang.code().to_string(),
        fence_lang: decision.lang.code().to_string(),
        cli_lang: decision.lang.code().to_string(),
        reference_langs: reference_docs.iter().cloned().collect(),
        selected_via: decision.via.label().to_string(),
    });
    if cfg_path.is_file() {
        cfg.save(&cfg_path)?;
    } else {
        // Scaffold-only write — leaves the rest of the config defaulted.
        // Real provisioning (`genasis init`) will populate the rest.
        cfg.save(&cfg_path)?;
    }
    Ok(())
}

/// Write `[design]` defaults to `genasis.toml` if absent. Phase D —
/// non-interactive: the user can edit `gallery_index_url`, `add_command`,
/// or any other field after the fact to point at a self-hosted gallery.
/// Existing `[design]` config is preserved (idempotent).
fn seed_design_defaults(project_root: &std::path::Path) -> Result<()> {
    let cfg_path = project_root.join(CONFIG_FILE_NAME);
    let mut cfg = if cfg_path.is_file() {
        Config::load(&cfg_path)?
    } else {
        Config::default()
    };
    if cfg.design.is_none() {
        cfg.design = Some(DesignConfig::default());
        cfg.save(&cfg_path)?;
    }
    Ok(())
}

/// Materialise reference-doc trees under
/// `docs/genasis-i18n-reference/<lang>/`. These files are NOT loaded by
/// Claude — they are operator-facing reference copies of the protocol.
fn write_reference_docs(
    project_root: &std::path::Path,
    reference_langs: &[String],
    active: genasis_i18n::Lang,
    store: &genasis_templates::AgentStore,
) -> Result<()> {
    use genasis_templates::SUPPORTED_LANGS;
    if reference_langs.is_empty() {
        return Ok(());
    }
    let base = project_root.join("docs").join("genasis-i18n-reference");
    for raw in reference_langs {
        let lang_code = raw.to_ascii_lowercase();
        if !SUPPORTED_LANGS.contains(&lang_code.as_str()) {
            tracing::warn!(
                lang = %lang_code,
                "unknown --reference-docs language; skipping"
            );
            continue;
        }
        if lang_code == active.code() {
            tracing::debug!(
                lang = %lang_code,
                "skipping reference-docs for active language"
            );
            continue;
        }
        let dir = base.join(&lang_code);
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("create reference-docs dir: {}", dir.display()))?;
        // Only the GENASIS.md contract makes sense as a reference; per-role
        // overlays would need template variables that only attach knows.
        // ADR-011: read from AgentStore on disk (get_lang removed with
        // include_dir migration).
        if let Some(body) = store.get_file(&format!("{lang_code}/GENASIS.md.tera")) {
            let target = dir.join("GENASIS.md");
            std::fs::write(&target, body).with_context(|| format!("write {}", target.display()))?;
        }
    }
    Ok(())
}

/// Load [agents] config from genasis.toml, or return defaults.
struct AgentsConfig {
    version: String,
    registry: String,
    cache_dir: String,
    auto_check: bool,
}

fn load_agents_config(project_root: &std::path::Path) -> AgentsConfig {
    let cfg_path = project_root.join(CONFIG_FILE_NAME);
    let _cfg = cfg_path
        .is_file()
        .then(|| Config::load(&cfg_path).ok())
        .flatten();

    // TODO: read from _cfg.agents once Config struct gains [agents] section.
    AgentsConfig {
        version: std::env::var("GENASIS_AGENTS_VERSION").unwrap_or_else(|_| "1.0.0".to_string()),
        registry: std::env::var("GENASIS_AGENTS_REGISTRY")
            .unwrap_or_else(|_| "https://github.com/claude-genasis/genasis/releases".to_string()),
        cache_dir: std::env::var("GENASIS_AGENTS_CACHE_DIR").unwrap_or_default(),
        auto_check: true,
    }
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
