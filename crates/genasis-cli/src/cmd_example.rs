//! `genasis example {prd|design|prd2}` — drop a sample document into the
//! current project so tutorials and onboarding flows have something
//! immediately actionable for the agentic team to chew on.
//!
//! As of ADR-017 the PRD output is locale-aware: a project initialised
//! in Korean (`[i18n].active = "ko"`) gets `prd.ko.md`; English projects
//! get `prd.en.md`. The two PRDs describe the same reference app — the
//! "I Am a Claude Code Expert" / "나는 Claude Code 전문가" quiz that
//! the trial-app's showcase panel will reveal once agents publish
//! completion via `genasis trial publish`.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use clap::{Args as ClapArgs, Subcommand};

use genasis_core::config::{Config, CONFIG_FILE_NAME};

const TEMPLATE_PRD_EN: &str = include_str!("../templates/examples/prd.en.md");
const TEMPLATE_PRD_KO: &str = include_str!("../templates/examples/prd.ko.md");
const TEMPLATE_DESIGN: &str = include_str!("../templates/examples/design-system.md");
const TEMPLATE_PRD2: &str = include_str!("../templates/examples/prd2.md");

#[derive(ClapArgs, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub kind: Kind,

    /// Project root. Defaults to the current working directory.
    #[arg(long, value_name = "DIR", global = true)]
    pub project: Option<PathBuf>,

    /// Overwrite an existing destination file.
    #[arg(long, global = true)]
    pub force: bool,

    /// Force a specific locale for the generated content. When omitted,
    /// the CLI reads `[i18n].active` from the project's `genasis.toml`
    /// (falling back to `"en"`). Accepts `en` or `ko`.
    #[arg(long, global = true, value_name = "LANG")]
    pub lang: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Kind {
    /// Write a sample PRD.md to the project root.
    Prd,
    /// Write a sample design-system.md to the project root.
    Design,
    /// Write a sample PRD2.md (feature expansion) to the project root.
    Prd2,
}

pub fn run(args: Args) -> Result<()> {
    let root = if let Some(p) = args.project.as_deref() {
        if !p.exists() {
            fs::create_dir_all(p)
                .with_context(|| format!("create --project dir {}", p.display()))?;
        }
        p.canonicalize()
            .with_context(|| format!("canonicalize {}", p.display()))?
    } else {
        std::env::current_dir()?
    };

    let lang = resolve_lang(&root, args.lang.as_deref());

    let (filename, body) = match args.kind {
        Kind::Prd => ("PRD.md", prd_template_for(&lang)),
        Kind::Design => ("design-system.md", TEMPLATE_DESIGN),
        Kind::Prd2 => ("PRD2.md", TEMPLATE_PRD2),
    };

    let dest = root.join(filename);
    write_template(&dest, body, args.force)?;
    println!("→ wrote {} ({})", dest.display(), lang);
    Ok(())
}

/// Pick the PRD body for the active locale. Defaults to English for
/// any unrecognised value so a typo never wedges the command.
fn prd_template_for(lang: &str) -> &'static str {
    match lang {
        "ko" => TEMPLATE_PRD_KO,
        _ => TEMPLATE_PRD_EN,
    }
}

/// Lookup order for the locale, per ADR-017:
///   1. explicit `--lang` flag (highest priority)
///   2. `[i18n].active` in the project's `genasis.toml`
///   3. fallback to `"en"` so projects with no config still succeed
///
/// The `[i18n].active` field is the same value `genasis init` writes
/// for every other locale-aware artifact (agent prompts, GENASIS.md,
/// slash commands), so the PRD now follows that convention.
fn resolve_lang(project_root: &Path, flag: Option<&str>) -> String {
    if let Some(v) = flag {
        let v = v.trim().to_ascii_lowercase();
        if matches!(v.as_str(), "ko" | "en") {
            return v;
        }
    }
    let cfg_path = project_root.join(CONFIG_FILE_NAME);
    if cfg_path.is_file() {
        if let Ok(cfg) = Config::load(&cfg_path) {
            if let Some(i18n) = cfg.i18n.as_ref() {
                let active = i18n.active.trim().to_ascii_lowercase();
                if matches!(active.as_str(), "ko" | "en") {
                    return active;
                }
            }
        }
    }
    "en".to_string()
}

fn write_template(dest: &Path, body: &str, force: bool) -> Result<()> {
    if dest.exists() && !force {
        return Err(anyhow!(
            "{} already exists (use --force to overwrite)",
            dest.display()
        ));
    }
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create parent dir {}", parent.display()))?;
    }
    fs::write(dest, body).with_context(|| format!("write {}", dest.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn templates_have_content() {
        assert!(TEMPLATE_PRD_EN.contains("I Am a Claude Code Expert"));
        assert!(TEMPLATE_PRD_KO.contains("나는 Claude Code 전문가"));
        assert!(TEMPLATE_DESIGN.contains("# Design System"));
        assert!(TEMPLATE_PRD2.contains("# PRD2"));
    }

    fn write_config_with_lang(dir: &Path, active: &str) {
        let body = format!(
            r#"
[project]
name = "demo"

[i18n]
active = "{active}"
fence_lang = "{active}"
cli_lang = "{active}"
selected_via = "test"
"#
        );
        fs::write(dir.join(CONFIG_FILE_NAME), body).unwrap();
    }

    #[test]
    fn prd_emits_english_when_active_lang_en() {
        let tmp = TempDir::new().unwrap();
        write_config_with_lang(tmp.path(), "en");
        run(Args {
            kind: Kind::Prd,
            project: Some(tmp.path().to_path_buf()),
            force: false,
            lang: None,
        })
        .unwrap();
        let body = fs::read_to_string(tmp.path().join("PRD.md")).unwrap();
        assert!(
            body.contains("I Am a Claude Code Expert"),
            "expected English PRD:\n{body}"
        );
    }

    #[test]
    fn prd_emits_korean_when_active_lang_ko() {
        let tmp = TempDir::new().unwrap();
        write_config_with_lang(tmp.path(), "ko");
        run(Args {
            kind: Kind::Prd,
            project: Some(tmp.path().to_path_buf()),
            force: false,
            lang: None,
        })
        .unwrap();
        let body = fs::read_to_string(tmp.path().join("PRD.md")).unwrap();
        assert!(
            body.contains("나는 Claude Code 전문가"),
            "expected Korean PRD:\n{body}"
        );
    }

    #[test]
    fn explicit_lang_flag_overrides_config() {
        let tmp = TempDir::new().unwrap();
        write_config_with_lang(tmp.path(), "ko");
        run(Args {
            kind: Kind::Prd,
            project: Some(tmp.path().to_path_buf()),
            force: false,
            lang: Some("en".into()),
        })
        .unwrap();
        let body = fs::read_to_string(tmp.path().join("PRD.md")).unwrap();
        assert!(body.contains("I Am a Claude Code Expert"));
    }

    #[test]
    fn missing_config_falls_back_to_english() {
        let tmp = TempDir::new().unwrap();
        run(Args {
            kind: Kind::Prd,
            project: Some(tmp.path().to_path_buf()),
            force: false,
            lang: None,
        })
        .unwrap();
        let body = fs::read_to_string(tmp.path().join("PRD.md")).unwrap();
        assert!(body.contains("I Am a Claude Code Expert"));
    }

    #[test]
    fn refuses_to_overwrite_without_force() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("PRD.md");
        fs::write(&path, "preexisting").unwrap();
        let args = Args {
            kind: Kind::Prd,
            project: Some(tmp.path().to_path_buf()),
            force: false,
            lang: Some("en".into()),
        };
        let err = run(args).unwrap_err().to_string();
        assert!(err.contains("already exists"));
        assert_eq!(fs::read_to_string(&path).unwrap(), "preexisting");
    }

    #[test]
    fn force_overwrites() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("PRD.md");
        fs::write(&path, "preexisting").unwrap();
        run(Args {
            kind: Kind::Prd,
            project: Some(tmp.path().to_path_buf()),
            force: true,
            lang: Some("en".into()),
        })
        .unwrap();
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.contains("I Am a Claude Code Expert"));
    }

    #[test]
    fn each_kind_writes_correct_filename() {
        let tmp = TempDir::new().unwrap();
        for (kind, filename) in [
            (Kind::Prd, "PRD.md"),
            (Kind::Design, "design-system.md"),
            (Kind::Prd2, "PRD2.md"),
        ] {
            run(Args {
                kind,
                project: Some(tmp.path().to_path_buf()),
                force: false,
                lang: Some("en".into()),
            })
            .unwrap();
            assert!(tmp.path().join(filename).exists());
        }
    }
}
