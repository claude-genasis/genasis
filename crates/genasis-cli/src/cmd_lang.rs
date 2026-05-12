//! `genasis lang status` / `genasis lang switch <lang>` — manage the
//! agent-context language after install.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use genasis_core::config::{Config, I18nConfig, CONFIG_FILE_NAME};
use genasis_i18n::{tr_args, Lang};
use genasis_templates::SUPPORTED_LANGS;

#[derive(Parser, Debug)]
pub struct Args {
    /// Project root. Defaults to the current working directory.
    #[arg(long, value_name = "DIR", global = true)]
    pub project: Option<PathBuf>,

    #[command(subcommand)]
    pub op: LangOp,
}

#[derive(Subcommand, Debug)]
pub enum LangOp {
    /// Print the current active language plus available locales.
    Status,
    /// Atomically swap the active agent-context language.
    Switch {
        /// Target locale (en|ko).
        lang: String,
    },
}

pub async fn run(args: Args, _non_interactive: bool, _assume_yes: bool) -> Result<()> {
    let project_root = resolve_project_root(args.project.as_deref())?;
    match args.op {
        LangOp::Status => status(&project_root),
        LangOp::Switch { lang } => switch(&project_root, &lang),
    }
}

fn status(project_root: &std::path::Path) -> Result<()> {
    let cfg_path = project_root.join(CONFIG_FILE_NAME);
    let cfg = if cfg_path.is_file() {
        Config::load(&cfg_path)?
    } else {
        Config::default()
    };
    let i18n = cfg.i18n.unwrap_or_default();
    println!(
        "active: {}  cli_lang: {}  fence_lang: {}",
        i18n.active, i18n.cli_lang, i18n.fence_lang
    );
    println!("selected_via: {}", i18n.selected_via);
    if i18n.reference_langs.is_empty() {
        println!("reference_langs: (none)");
    } else {
        println!("reference_langs: {}", i18n.reference_langs.join(", "));
    }
    println!("available locales: {}", SUPPORTED_LANGS.join(", "));
    Ok(())
}

fn switch(project_root: &std::path::Path, raw: &str) -> Result<()> {
    let target = Lang::parse(raw).with_context(|| {
        format!(
            "unknown language `{raw}` (allowed: {})",
            SUPPORTED_LANGS.join(", ")
        )
    })?;
    let cfg_path = project_root.join(CONFIG_FILE_NAME);
    let mut cfg = if cfg_path.is_file() {
        Config::load(&cfg_path)?
    } else {
        Config::default()
    };
    let current_active = cfg
        .i18n
        .as_ref()
        .map(|i| i.active.clone())
        .unwrap_or_else(|| "en".to_string());
    if current_active == target.code() {
        println!(
            "{}",
            tr_args("lang.switch.no_op", &[("lang", target.native_name())])
        );
        return Ok(());
    }

    println!(
        "{}",
        tr_args(
            "lang.switch.starting",
            &[("from", &current_active), ("to", target.code())]
        )
    );

    // Re-attach with the new fence language. `cmd_attach::pub_run` walks
    // the agent files and rewrites every fence body from the new locale's
    // template tree. We pass --non-interactive + --yes to skip the prompt,
    // because `lang switch` is itself an explicit user choice.
    let attach_args = crate::cmd_attach::Args {
        project: Some(project_root.to_path_buf()),
        dry_run: false,
        diff: false,
        force: true, // overwrite even Pristine fences from the previous locale
        fence_version: "1.0".to_string(),
        reference_docs: cfg
            .i18n
            .as_ref()
            .map(|i| i.reference_langs.clone())
            .unwrap_or_default(),
        upgrade: false,
    };
    // We synchronously block on the attach future; the caller is already
    // in tokio::main scope, so we drive it through a fresh handle.
    futures_block_on(crate::cmd_attach::pub_run(
        attach_args,
        Some(target.code().to_string()),
        true,
        true,
    ))?;

    // Refresh the i18n block after attach has rewritten genasis.toml.
    cfg = Config::load(&cfg_path)?;
    cfg.i18n = Some(I18nConfig {
        active: target.code().to_string(),
        fence_lang: target.code().to_string(),
        cli_lang: target.code().to_string(),
        reference_langs: cfg
            .i18n
            .as_ref()
            .map(|i| i.reference_langs.clone())
            .unwrap_or_default(),
        selected_via: "switch".into(),
    });
    cfg.save(&cfg_path)?;

    println!(
        "{}",
        tr_args(
            "lang.switch.success",
            &[("from", &current_active), ("to", target.code())]
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

/// Block on a future from synchronous code that is already inside the
/// outer tokio runtime. Uses `tokio::task::block_in_place` + a current
/// handle.
fn futures_block_on<F: std::future::Future<Output = Result<()>>>(fut: F) -> Result<()> {
    let handle = tokio::runtime::Handle::current();
    tokio::task::block_in_place(|| handle.block_on(fut))
}
