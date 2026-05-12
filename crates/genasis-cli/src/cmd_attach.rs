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

    /// Re-attach as an overlay upgrade (replaces the deprecated
    /// `genasis upgrade` subcommand). Currently a passthrough — the
    /// re-attach machinery is the upgrade — but the flag lets users
    /// signal intent and primes future versions to adopt a more
    /// conservative policy (e.g. preserve Tampered fences by default).
    #[arg(long)]
    pub upgrade: bool,
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
    // v0.5.4 (issue C3): the count returned here is the TOTAL number
    // of files actually written (GENASIS.md + commands/hooks/skills
    // + CLAUDE.md stub if absent). Previous releases concatenated
    // "+ GENASIS.md" to the log unconditionally; that lied when the
    // catalog didn't carry the template. install_genasis_overlay_artifacts
    // now surfaces its own warning when GENASIS.md isn't written, so
    // the summary line below just states the count.
    let install_count =
        install_genasis_overlay_artifacts(&project_root, &store, decision.lang.code())
            .unwrap_or_else(|e| {
                tracing::warn!(reason = %e, "overlay artifacts install failed");
                eprintln!("  ⚠ failed to install commands/hooks/skills: {e}");
                0
            });
    if install_count > 0 {
        println!(
            "  + {install_count} overlay file(s) under .claude/genasis/ (+ GENASIS.md / CLAUDE.md stub when applicable)"
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
/// `commands/*.tera`, `hooks/*.tera`, and `skills/*.tera`.
///
/// Bodies that contain real Tera tags (`{{` or `{%`) go through
/// `Tera::one_off`. Everything else is passed through verbatim —
/// shell scripts in particular use `${#var}` (length expansion) and
/// `${var:0:N}` (substring) syntax that collides with Tera's `{#`
/// comment marker. The v1.0.0 catalog's templates are all the
/// passthrough kind, but the next catalog refresh can introduce real
/// variables for any role without changing this code.
///
/// Per-file errors are surfaced as warnings and the loop continues —
/// one bad template never blocks the rest of the install (the v0.5.2
/// implementation's `?` propagation was the root cause of issue 가
/// from the v0.5.2 field test log, where 1 of 6 hooks tripped the
/// `{#body}` lex error and the other 5 never landed).
///
/// Hooks ending in `.sh` are chmod'd 0755 on Unix so Claude Code's
/// PostToolUse / SessionStart hook runner can execute them directly.
///
/// Additionally writes a minimal `CLAUDE.md` stub with an `@import
/// GENASIS.md` line if none exists at the project root (issue 마) —
/// without that line Claude Code never loads the protocol contract,
/// so every slash command and hook is effectively orphaned.
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
    // CLAUDE.md @imports. v0.5.4 (issue C3 from the v0.5.3 field
    // report): the v1.0.0 catalog tarball ships overlays/<lang>/
    // *.patch.md.tera but does NOT bundle a GENASIS.md.tera (the
    // contract source lives in the genasis repo's `agents/` dir,
    // not in the catalog). The previous code looked for it in the
    // catalog, found nothing, silently skipped — but the CLI still
    // printed "+ GENASIS.md" which was a flat-out lie.
    //
    // Fix: prefer catalog if a future release bundles it; otherwise
    // fall back to a binary-embedded copy of the same source. Track
    // the actual write so the log message reflects reality.
    let mut wrote_genasis_md = false;
    let genasis_md_body = store
        .get_file(&format!("{lang_code}/GENASIS.md.tera"))
        .or_else(|| store.get_file("en/GENASIS.md.tera"))
        .or_else(|| Some(GENASIS_MD_FALLBACK.to_string()));
    if let Some(body) = genasis_md_body {
        match render_template_body(&body, &ctx) {
            Ok(rendered) => {
                let target = project_root.join("GENASIS.md");
                fs::write(&target, rendered)
                    .with_context(|| format!("write {}", target.display()))?;
                wrote_genasis_md = true;
                written += 1;
            }
            Err(e) => {
                eprintln!("  ⚠ render GENASIS.md skipped: {e}");
            }
        }
    }
    // Surface a clear signal for the caller's summary line so the
    // "+ GENASIS.md" suffix isn't shown when the file wasn't
    // actually written (issue C3).
    if !wrote_genasis_md {
        eprintln!(
            "  ⚠ GENASIS.md not written (catalog missing template AND embedded fallback failed). \
             CLAUDE.md's `@import GENASIS.md` will be a broken link until the next attach."
        );
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
            let rendered = match render_template_body(&body, &ctx) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("  ⚠ {subdir}/{name} skipped: {e}");
                    continue;
                }
            };
            let out_name = name.strip_suffix(".tera").unwrap_or(&name);
            let target = out_dir.join(out_name);
            if let Err(e) = fs::write(&target, rendered) {
                eprintln!("  ⚠ write {} failed: {e}", target.display());
                continue;
            }
            #[cfg(unix)]
            if out_name.ends_with(".sh") {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(&target, fs::Permissions::from_mode(0o755));
            }
            written += 1;
        }
    }

    // CLAUDE.md stub (issue 마): without `@import GENASIS.md`, Claude
    // Code never reads the protocol contract and the slash commands +
    // hooks installed above become orphans. We only ever CREATE — if
    // the user already has a CLAUDE.md we leave it alone (they may
    // have other imports / project rules in it).
    let claude_md = project_root.join("CLAUDE.md");
    if !claude_md.exists() {
        let body = include_str!("../templates/claude_md_stub.md");
        fs::write(&claude_md, body).with_context(|| format!("write {}", claude_md.display()))?;
        written += 1;
    }

    Ok(written)
}

/// Binary-embedded GENASIS.md template body. The agents-v1.0.0
/// catalog tarball doesn't bundle this file, so v0.5.4 ships it
/// inside the genasis binary as a fallback. Once a future catalog
/// release (`agents-v1.1.0`) bundles `<lang>/GENASIS.md.tera`,
/// `install_genasis_overlay_artifacts` prefers the catalog copy.
const GENASIS_MD_FALLBACK: &str = include_str!("../../../agents/GENASIS.md.tera");

/// Per ADR-011 the catalog ships `.tera` files but most of them are
/// pre-rendered (no `{{ var }}` or `{% tag %}` syntax). Skipping the
/// Tera parser for those is the only way to avoid false positives on
/// bash `${#var}` length expansion (which the Tera lexer treats as
/// the start of a `{# comment #}` and then fails to find the close).
fn render_template_body(body: &str, ctx: &tera::Context) -> Result<String> {
    let has_tera_tags = body.contains("{{") || body.contains("{%");
    if !has_tera_tags {
        return Ok(body.to_string());
    }
    tera::Tera::one_off(body, ctx, true).map_err(|e| anyhow::anyhow!("{e}"))
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
    // v0.5.4 (issue S1): expose `project_slug` already-slugified so
    // overlay templates can use `{{ project_slug }}` directly when
    // they need a Mattermost-channel-safe identifier. The `slugify`
    // Tera filter is also registered (see merger.rs); templates can
    // use either form depending on readability.
    let project_slug = genasis_core::config::slugify(&cfg.project.name);
    Ok(serde_json::json!({
        "project_name": cfg.project.name,
        "project_slug": project_slug,
        "project_domain": cfg.project.domain,
        "plane_url": cfg.plane.as_ref().map(|p| p.url.clone()).unwrap_or_default(),
        "mm_url": cfg.mattermost.as_ref().map(|m| m.url.clone()).unwrap_or_default(),
        "plane_flavor": cfg.plane.as_ref().map(|p| p.flavor.clone()).unwrap_or_else(|| "auto".into()),
        "mm_flavor": cfg.mattermost.as_ref().map(|m| m.flavor.clone()).unwrap_or_else(|| "auto".into()),
    }))
}
