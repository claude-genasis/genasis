use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use genasis_core::config::{Config, I18nConfig, CONFIG_FILE_NAME};
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
    write_reference_docs(&project_root, &args.reference_docs, decision.lang)?;

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
        lang: decision.lang.code().to_string(),
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

/// Materialise reference-doc trees under
/// `docs/genasis-i18n-reference/<lang>/`. These files are NOT loaded by
/// Claude — they are operator-facing reference copies of the protocol.
fn write_reference_docs(
    project_root: &std::path::Path,
    reference_langs: &[String],
    active: genasis_i18n::Lang,
) -> Result<()> {
    use genasis_templates::{get_lang, SUPPORTED_LANGS};
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
        if let Some(body) = get_lang(&lang_code, "GENASIS.md.tera") {
            let target = dir.join("GENASIS.md");
            std::fs::write(&target, body).with_context(|| format!("write {}", target.display()))?;
        }
    }
    Ok(())
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
