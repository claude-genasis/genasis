//! `genasis.toml` schema, discovery, load/save.
//!
//! Discovery walks up from the current working directory looking for
//! `genasis.toml`. The first match wins and its parent dir becomes the
//! "project root" for subsequent overlay/db operations.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::fs::{atomic_write, read_to_string_optional};

pub const CONFIG_FILE_NAME: &str = "genasis.toml";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub project: ProjectConfig,
    pub plane: Option<PlaneConfig>,
    pub mattermost: Option<MattermostConfig>,
    pub db: Option<DbConfig>,
    pub deploy: Option<DeployConfig>,
    pub token_economics: Option<TokenEconomicsConfig>,
    pub i18n: Option<I18nConfig>,
}

/// Locale configuration recorded by `genasis init` / `attach` and read by
/// every subsequent invocation. See blueprint §19.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I18nConfig {
    /// Active language of the installed agent context (.claude/agents/*
    /// fence bodies, .claude/genasis/{skills,commands,hooks}/, GENASIS.md).
    /// "en" or "ko".
    pub active: String,

    /// Language used to render marker-fence bodies. Normally equal to
    /// `active`; carried separately so M12.5 `genasis lang switch` can
    /// stage a transition atomically.
    #[serde(default)]
    pub fence_lang: String,

    /// Language for CLI / TUI runtime output. Distinct from `active` so
    /// an English-context project can still surface Korean diagnostics
    /// to its operator.
    #[serde(default)]
    pub cli_lang: String,

    /// On-disk reference docs in additional languages. They live under
    /// `docs/genasis-i18n-reference/<lang>/` and are NEVER `@import`'d
    /// into agent context.
    #[serde(default)]
    pub reference_langs: Vec<String>,

    /// Provenance: how the active language was chosen. Diagnostic only.
    /// "flag" | "prompt" | "lang_env" | "default".
    #[serde(default)]
    pub selected_via: String,
}

impl Default for I18nConfig {
    fn default() -> Self {
        Self {
            active: "en".into(),
            fence_lang: "en".into(),
            cli_lang: "en".into(),
            reference_langs: Vec::new(),
            selected_via: "default".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectConfig {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaneConfig {
    pub url: String,
    /// "upstream" | "agent-aware" | "auto"
    #[serde(default = "default_flavor")]
    pub flavor: String,
    pub workspace_slug: String,
    #[serde(default)]
    pub project_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MattermostConfig {
    pub url: String,
    #[serde(default = "default_flavor")]
    pub flavor: String,
    pub team_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConfig {
    /// "postgres" | "mysql" | "sqlite" | "duckdb"
    pub driver: String,
    /// Connection string or file path
    pub url: String,
    /// "atlas" | "drizzle-kit" | "raw"
    #[serde(default = "default_migration_tool")]
    pub migration_tool: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeployConfig {
    #[serde(default)]
    pub build: Option<String>,
    #[serde(default)]
    pub cmd_dev: Option<String>,
    #[serde(default)]
    pub cmd_prod: Option<String>,
    #[serde(default)]
    pub rollback: Option<String>,
    #[serde(default)]
    pub dev_url: Option<String>,
    #[serde(default)]
    pub prod_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenEconomicsConfig {
    #[serde(default = "default_trim_threshold_kb")]
    pub trim_threshold_kb: u32,
}

fn default_flavor() -> String {
    "auto".to_string()
}
fn default_migration_tool() -> String {
    "atlas".to_string()
}
fn default_trim_threshold_kb() -> u32 {
    32
}

impl Config {
    /// Walk up from `start` looking for `genasis.toml`. Returns the resolved
    /// path or `None` if no config exists between `start` and the filesystem
    /// root.
    pub fn discover(start: &Path) -> Option<PathBuf> {
        let mut cur: Option<&Path> = Some(start);
        while let Some(d) = cur {
            let candidate = d.join(CONFIG_FILE_NAME);
            if candidate.is_file() {
                return Some(candidate);
            }
            cur = d.parent();
        }
        None
    }

    /// Load from explicit path.
    pub fn load(path: &Path) -> Result<Self> {
        let body = read_to_string_optional(path)?
            .ok_or_else(|| Error::Config(format!("config not found: {}", path.display())))?;
        Self::parse(&body)
    }

    pub fn parse(s: &str) -> Result<Self> {
        toml::from_str(s).map_err(Error::from)
    }

    pub fn to_toml_string(&self) -> Result<String> {
        toml::to_string_pretty(self).map_err(|e| Error::Config(format!("toml encode: {e}")))
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        atomic_write(path, self.to_toml_string()?.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn parse_minimal_with_defaults() {
        let s = r#"
            [project]
            name = "demo"
            domain = "example.com"
        "#;
        let cfg = Config::parse(s).unwrap();
        assert_eq!(cfg.project.name, "demo");
        assert!(cfg.plane.is_none());
    }

    #[test]
    fn round_trip_through_disk() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("genasis.toml");
        let mut cfg = Config::default();
        cfg.project.name = "demo".into();
        cfg.project.domain = "example.com".into();
        cfg.plane = Some(PlaneConfig {
            url: "https://plane.example.com".into(),
            flavor: "auto".into(),
            workspace_slug: "demo-ws".into(),
            project_id: None,
        });
        cfg.save(&path).unwrap();

        let reread = Config::load(&path).unwrap();
        assert_eq!(reread.project.name, "demo");
        assert_eq!(
            reread.plane.as_ref().unwrap().url,
            "https://plane.example.com"
        );
        assert_eq!(reread.plane.as_ref().unwrap().flavor, "auto");
    }

    #[test]
    fn discover_walks_up() {
        let dir = tempdir().unwrap();
        let nested = dir.path().join("a/b/c");
        std::fs::create_dir_all(&nested).unwrap();
        let cfg_path = dir.path().join("genasis.toml");
        std::fs::write(&cfg_path, "[project]\nname = \"x\"\n").unwrap();

        let found = Config::discover(&nested).unwrap();
        assert_eq!(found, cfg_path);
    }

    #[test]
    fn discover_returns_none_when_absent() {
        let dir = tempdir().unwrap();
        assert!(Config::discover(dir.path()).is_none());
    }
}
