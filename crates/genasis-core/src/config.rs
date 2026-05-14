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
    pub design: Option<DesignConfig>,
    /// Trial bridge — points the Plane and Mattermost providers at a
    /// running trial-app instead of real Plane/MM servers, so users can
    /// exercise the agentic workflow without installing either tool.
    pub trial: Option<TrialConfig>,
    /// Human team members enrolled in this project. Their messages on
    /// the Mattermost scrum channel are treated by agents as binding
    /// stakeholder requirements (see ADR-014). Provisioned in Plane and
    /// Mattermost by `genasis init` / `genasis humans sync`.
    #[serde(default, rename = "humans")]
    pub humans: Vec<HumanEntry>,
}

/// One human member of the project — captured in `genasis.toml` under
/// `[[humans]]`. Identity in external systems (Mattermost user_id, Plane
/// user_id) is stored separately in `.genasis/humans.lock.toml` to keep
/// `genasis.toml` free of provisioning side-effects.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HumanEntry {
    /// Display name (used for Mattermost first/last name and Plane label).
    pub name: String,
    /// Lowercase email — primary key for idempotent provisioning.
    pub email: String,
    /// Stakeholder role label, free-form. Common values: "stakeholder",
    /// "pm-human", "reviewer", "designer-human". Agents use this to
    /// prioritise: stakeholder > pm-human > reviewer.
    #[serde(default = "default_human_role")]
    pub role: String,
    /// Optional Mattermost username override. If empty, derived from
    /// the local part of `email` (sanitised to MM constraints).
    #[serde(default)]
    pub mm_username: String,
    /// Optional locale hint ("en" / "ko") for system messages directed
    /// at this human. Falls back to the project i18n.cli_lang.
    #[serde(default)]
    pub locale: String,
}

impl HumanEntry {
    /// Returns the Mattermost username that should be used when
    /// provisioning this human. If `mm_username` is set, returns it;
    /// otherwise derives from the local part of the email address.
    pub fn effective_mm_username(&self) -> String {
        if !self.mm_username.is_empty() {
            return self.mm_username.clone();
        }
        derive_mm_username(&self.email)
    }
}

fn default_human_role() -> String {
    "stakeholder".to_string()
}

/// Lowercase-kebab slug of a free-form name. Used to derive Plane
/// project slugs and Mattermost channel slugs from human-typed
/// project names (e.g. "Marketing Squad" → "marketing-squad").
/// Collapses runs of non-alphanumeric characters into single hyphens
/// and strips leading/trailing hyphens. Empty input returns "team",
/// so callers can rely on a non-empty result.
pub fn slugify(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut prev_dash = true; // suppress leading dash
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "team".to_string()
    } else {
        out
    }
}

/// Generate a 32-char hex token for trial-mode tenancy keys. Combines
/// nanosecond time, process id, and a per-process atomic counter,
/// hashed through SHA-256. Unpredictable enough to avoid concurrent
/// collisions on the shared trial-app instance, but is **not** a
/// cryptographic auth secret — the trial-app additionally enforces
/// `[trial].shared_secret` for server-to-server callers.
pub fn random_team_token() -> String {
    use sha2::{Digest, Sha256};
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let pid = std::process::id();
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);

    let mut hasher = Sha256::new();
    hasher.update(nanos.to_le_bytes());
    hasher.update(pid.to_le_bytes());
    hasher.update(n.to_le_bytes());
    let digest = hasher.finalize();
    hex::encode(&digest[..16])
}

/// Sanitise an email's local-part into a Mattermost-compatible username.
/// MM usernames are 3-22 chars, lowercase a-z 0-9 . - _.
pub fn derive_mm_username(email: &str) -> String {
    // Local part, then strip Gmail-style "+tag" suffix.
    let local = email.split('@').next().unwrap_or(email);
    let local = local.split('+').next().unwrap_or(local);
    let mut s: String = local
        .chars()
        .map(|c| c.to_ascii_lowercase())
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
        .collect();
    if s.len() < 3 {
        s.push_str("-hu");
    }
    if s.len() > 22 {
        s.truncate(22);
    }
    s
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrialConfig {
    /// Whether the trial bridge is active. When `true`, the `flavor =
    /// "trial"` setting on `[plane]` / `[mattermost]` becomes the
    /// effective backend; when `false`, the trial-app is ignored even if
    /// `flavor = "trial"`.
    #[serde(default)]
    pub enabled: bool,
    /// Base URL of the trial-app. Defaults to the operator-hosted
    /// public instance at `https://mmplane-trial.realstory.blog`.
    /// Override only if you self-host the trial-app at a different
    /// location (rare — the trial flow is meant to be zero-setup).
    #[serde(default = "default_trial_url")]
    pub url: String,
    /// Shared secret sent in the `X-Genasis-Trial-Secret` header by
    /// server-to-server callers (the genasis Rust providers). Must
    /// match `TRIAL_SHARED_SECRET` on the trial-app side.
    /// Empty value = browser-tab same-origin enforcement only; set
    /// this when calling the trial-app from a non-browser caller.
    #[serde(default)]
    pub shared_secret: String,
    /// Per-team isolation key for the trial-app's multi-tenant sim
    /// (ADR-016). A 32-char hex string generated by
    /// `genasis init --trial`. Empty / missing resolves to
    /// `DEFAULT_TEAM_TOKEN` on the trial-app side, preserving
    /// single-tenant behaviour for configs that pre-date ADR-016.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team_token: Option<String>,
}

/// Token written to `[trial].team_token` and accepted by the trial-app
/// as the "no specific tenant" sentinel — used for the legacy hosted
/// demo where browser tabs arrive without any genasis state to
/// reference.
pub const DEFAULT_TEAM_TOKEN: &str = "default";

fn default_trial_url() -> String {
    "https://mmplane-trial.realstory.blog".to_string()
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
    /// Human-readable project name shown in Plane UI and the trial-app
    /// kanban header. Defaults to `[project].name` when absent — see
    /// `Config::derive_naming_defaults` (ADR-016).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MattermostConfig {
    pub url: String,
    #[serde(default = "default_flavor")]
    pub flavor: String,
    pub team_name: String,
    /// Channels provisioned for this team. The first entry whose `key`
    /// equals `"scrum"` is the binding-stakeholder channel (ADR-014).
    /// Empty in legacy configs — `Config::derive_naming_defaults`
    /// synthesises a single `scrum` channel from `[project].name`
    /// (ADR-016).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub channels: Vec<MattermostChannel>,
}

/// One Mattermost channel under `[mattermost].channels`. Defined per
/// ADR-016 §1 to replace the prior "#scrum-{project_name}" string
/// convention with structured data the CLI, agents, and trial-app can
/// all read without reparsing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MattermostChannel {
    /// Stable lookup key used by templates and agent prompts ("scrum",
    /// "design", "design-incoming"). Unique per team.
    pub key: String,
    /// Mattermost channel slug. Conventionally `{key}-{project_slug}`
    /// when synthesised by `derive_naming_defaults`, but may be any
    /// MM-valid name once the operator edits it by hand.
    pub name: String,
    /// Human-readable display name shown in Mattermost UI and the
    /// trial-app sidebar.
    pub display_name: String,
}

impl MattermostChannel {
    /// Synthesise a "scrum" channel from a project name. Used by
    /// `derive_naming_defaults` so legacy configs without a
    /// `[mattermost].channels` array continue to expose at least the
    /// binding-stakeholder channel by ADR-014's convention.
    pub fn scrum_from_project(project_name: &str) -> Self {
        let slug = slugify(project_name);
        Self {
            key: "scrum".to_string(),
            name: format!("scrum-{slug}"),
            display_name: format!("{project_name} — Scrum"),
        }
    }
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

/// `[design]` — external design provider integration (M-D1+).
///
/// `add_command` is a templated shell command run when `genasis design swap
/// <slug>` is invoked. Placeholders `{slug}` and `{out}` are substituted.
/// The default delegates to `getdesign` (npm) but a self-hosted gallery can
/// replace it without code changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignConfig {
    /// Gallery landing URL shown in CLI/TUI prose.
    #[serde(default = "default_gallery_index_url")]
    pub gallery_index_url: String,

    /// Per-slug preview URL template. `{slug}` is substituted.
    #[serde(default = "default_gallery_url_template")]
    pub gallery_url_template: String,

    /// Shell command template for the npx-style fetch path. `{slug}` and
    /// `{out}` are substituted.
    #[serde(default = "default_add_command")]
    pub add_command: String,

    /// When true, `genasis design swap` exports `GETDESIGN_DISABLE_TELEMETRY=1`
    /// before invoking `add_command`. Default: true.
    #[serde(default = "default_disable_telemetry")]
    pub disable_telemetry: bool,

    /// Directory (relative to project root) that holds the active external
    /// `DESIGN.md` plus state/backup. Default: `docs/design-system`.
    #[serde(default = "default_external_dir")]
    pub external_dir: String,
}

impl Default for DesignConfig {
    fn default() -> Self {
        Self {
            gallery_index_url: default_gallery_index_url(),
            gallery_url_template: default_gallery_url_template(),
            add_command: default_add_command(),
            disable_telemetry: default_disable_telemetry(),
            external_dir: default_external_dir(),
        }
    }
}

fn default_gallery_index_url() -> String {
    "https://getdesign.md/".to_string()
}
fn default_gallery_url_template() -> String {
    "https://getdesign.md/{slug}/design-md".to_string()
}
fn default_add_command() -> String {
    "npx getdesign@latest add {slug} --force --out {out}".to_string()
}
fn default_disable_telemetry() -> bool {
    true
}
fn default_external_dir() -> String {
    "docs/design-system".to_string()
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

    /// D-072 + D-095: walk-up + BFS walk-down (up to depth 2). The
    /// plain `discover` only walks up the directory chain; that matches
    /// the typical "I'm somewhere inside the sandbox" workflow but
    /// breaks the **testbed pattern** where the user does `mkdir testbed
    /// && cd testbed && genasis init --trial example-app` and runs
    /// commands (`genasis monitor`, `genasis listen`) from the testbed
    /// root — the sandbox is one directory down at
    /// `testbed/example-app/genasis.toml` which walk-up never reaches.
    ///
    /// D-095 extends the walk-down from 1 step to 2 steps so the user
    /// can run `genasis monitor` from `/work/agenteams` (grand-parent
    /// of the sandbox) and still find `team-ex/quiz-21/genasis.toml`.
    /// We BFS so the shallower sandbox always wins on ties (closer to
    /// where the user actually invoked the command).
    pub fn discover_or_descend(start: &Path) -> Option<PathBuf> {
        if let Some(found) = Self::discover(start) {
            return Some(found);
        }
        const MAX_DEPTH: usize = 2;
        let mut queue: std::collections::VecDeque<(PathBuf, usize)> =
            std::collections::VecDeque::new();
        queue.push_back((start.to_path_buf(), 0));
        while let Some((dir, depth)) = queue.pop_front() {
            let entries = match std::fs::read_dir(&dir) {
                Ok(e) => e,
                Err(_) => continue,
            };
            let mut child_dirs: Vec<PathBuf> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.is_dir())
                .filter(|p| {
                    p.file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| {
                            !n.starts_with('.')
                                && n != "node_modules"
                                && n != "target"
                                && n != "build"
                                && n != "dist"
                                && n != ".next"
                        })
                        .unwrap_or(false)
                })
                .collect();
            child_dirs.sort();
            // Pass 1: any immediate child with a config wins (shallowest).
            for child in &child_dirs {
                let candidate = child.join(CONFIG_FILE_NAME);
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
            // Pass 2: enqueue children for deeper search if depth allows.
            if depth + 1 < MAX_DEPTH {
                for child in child_dirs {
                    queue.push_back((child, depth + 1));
                }
            }
        }
        None
    }

    /// Load from explicit path.
    pub fn load(path: &Path) -> Result<Self> {
        let body = read_to_string_optional(path)?
            .ok_or_else(|| Error::Config(format!("config not found: {}", path.display())))?;
        let cfg = Self::parse(&body)?;
        cfg.validate_trial()?;
        Ok(cfg)
    }

    pub fn parse(s: &str) -> Result<Self> {
        toml::from_str(s).map_err(Error::from)
    }

    /// Fill in identifier fields that legacy configs (pre-ADR-016)
    /// leave blank. Idempotent — running it twice yields the same
    /// result, and any value already present is preserved.
    ///
    /// Specifically:
    /// 1. `[plane].project_name` defaults to `[project].name` so the
    ///    trial-app kanban header and Plane UI both have something
    ///    human-readable to show.
    /// 2. `[mattermost].channels` synthesises a single `scrum` channel
    ///    from the project name, matching ADR-014's
    ///    "#scrum-{project_name}" convention.
    ///
    /// Callers that mutate `[project].name` or
    /// `[plane].project_name` should re-run this method so the
    /// derived channel slug stays in sync.
    pub fn derive_naming_defaults(&mut self) {
        let project_name = if self.project.name.is_empty() {
            "team".to_string()
        } else {
            self.project.name.clone()
        };

        if let Some(plane) = self.plane.as_mut() {
            if plane.project_name.is_none() {
                plane.project_name = Some(project_name.clone());
            }
        }

        let display_basis = self
            .plane
            .as_ref()
            .and_then(|p| p.project_name.clone())
            .unwrap_or_else(|| project_name.clone());

        if let Some(mm) = self.mattermost.as_mut() {
            if mm.channels.is_empty() {
                mm.channels
                    .push(MattermostChannel::scrum_from_project(&display_basis));
            }
        }
    }

    /// Effective tenancy token for the trial-app. Returns
    /// `[trial].team_token` when present and non-empty, else
    /// `DEFAULT_TEAM_TOKEN`. Callers (the Rust providers, the trial
    /// factory) use this to fill the `X-Genasis-Team-Token` header.
    pub fn effective_team_token(&self) -> &str {
        self.trial
            .as_ref()
            .and_then(|t| t.team_token.as_deref())
            .filter(|s| !s.is_empty())
            .unwrap_or(DEFAULT_TEAM_TOKEN)
    }

    /// Look up a Mattermost channel by `key` after defaults have been
    /// derived. Returns `None` when the project has no Mattermost
    /// section or no channel with that key.
    pub fn mattermost_channel(&self, key: &str) -> Option<&MattermostChannel> {
        self.mattermost
            .as_ref()
            .and_then(|m| m.channels.iter().find(|c| c.key == key))
    }

    /// Cross-section validation: when `[plane]` or `[mattermost]` declare
    /// `flavor = "trial"`, the `[trial]` section must exist and be
    /// `enabled = true`. Catches misconfigurations early so the operator
    /// gets a clear message instead of a runtime HTTP failure against the
    /// wrong endpoint.
    pub fn validate_trial(&self) -> Result<()> {
        let plane_trial = self
            .plane
            .as_ref()
            .map(|p| p.flavor == "trial")
            .unwrap_or(false);
        let mm_trial = self
            .mattermost
            .as_ref()
            .map(|m| m.flavor == "trial")
            .unwrap_or(false);
        if !plane_trial && !mm_trial {
            return Ok(());
        }
        match self.trial.as_ref() {
            None => Err(Error::Config(
                "flavor = \"trial\" set on [plane] or [mattermost] but [trial] section missing"
                    .into(),
            )),
            Some(t) if !t.enabled => Err(Error::Config(
                "flavor = \"trial\" set on [plane] or [mattermost] but [trial] enabled = false"
                    .into(),
            )),
            Some(_) => Ok(()),
        }
    }

    pub fn to_toml_string(&self) -> Result<String> {
        toml::to_string_pretty(self).map_err(|e| Error::Config(format!("toml encode: {e}")))
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        atomic_write(path, self.to_toml_string()?.as_bytes())
    }

    /// Insert or replace a human entry, keyed by lowercase email.
    /// Returns true if a new entry was added, false if an existing
    /// entry was updated in place.
    pub fn upsert_human(&mut self, mut entry: HumanEntry) -> bool {
        entry.email = entry.email.trim().to_ascii_lowercase();
        if let Some(slot) = self
            .humans
            .iter_mut()
            .find(|h| h.email.eq_ignore_ascii_case(&entry.email))
        {
            *slot = entry;
            false
        } else {
            self.humans.push(entry);
            true
        }
    }

    /// Remove a human entry by email (case-insensitive). Returns true
    /// if an entry was removed.
    pub fn remove_human(&mut self, email: &str) -> bool {
        let want = email.trim().to_ascii_lowercase();
        let before = self.humans.len();
        self.humans.retain(|h| !h.email.eq_ignore_ascii_case(&want));
        self.humans.len() != before
    }

    /// Lookup a human by email (case-insensitive).
    pub fn find_human(&self, email: &str) -> Option<&HumanEntry> {
        let want = email.trim().to_ascii_lowercase();
        self.humans
            .iter()
            .find(|h| h.email.eq_ignore_ascii_case(&want))
    }
}

/// Side-channel store for provisioning artefacts (Mattermost user_id,
/// Plane user_id, temp passwords flagged for first-login change).
/// Lives at `.genasis/humans.lock.toml` and is gitignored by default
/// because it contains transient credentials.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct HumansLock {
    pub entries: Vec<HumanLockEntry>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct HumanLockEntry {
    pub email: String,
    pub mm_user_id: String,
    pub mm_username: String,
    pub plane_user_id: String,
    /// Temporary password issued at provision time. Cleared once the
    /// human logs in for the first time and changes it.
    pub mm_temp_password: String,
    /// ISO-8601 UTC timestamp of last successful provision.
    pub provisioned_at: String,
}

impl HumansLock {
    pub const FILE_NAME: &'static str = ".genasis/humans.lock.toml";

    pub fn load(project_root: &Path) -> Result<Self> {
        let p = project_root.join(Self::FILE_NAME);
        match read_to_string_optional(&p)? {
            Some(body) => {
                toml::from_str(&body).map_err(|e| Error::Config(format!("humans.lock.toml: {e}")))
            }
            None => Ok(Self::default()),
        }
    }

    pub fn save(&self, project_root: &Path) -> Result<()> {
        let p = project_root.join(Self::FILE_NAME);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).map_err(Error::from)?;
        }
        let body = toml::to_string_pretty(self)
            .map_err(|e| Error::Config(format!("humans.lock encode: {e}")))?;
        atomic_write(&p, body.as_bytes())
    }

    pub fn upsert(&mut self, entry: HumanLockEntry) {
        let email = entry.email.to_ascii_lowercase();
        if let Some(slot) = self
            .entries
            .iter_mut()
            .find(|e| e.email.eq_ignore_ascii_case(&email))
        {
            *slot = entry;
        } else {
            self.entries.push(entry);
        }
    }

    pub fn remove(&mut self, email: &str) -> bool {
        let before = self.entries.len();
        self.entries
            .retain(|e| !e.email.eq_ignore_ascii_case(email.trim()));
        self.entries.len() != before
    }

    pub fn find(&self, email: &str) -> Option<&HumanLockEntry> {
        self.entries
            .iter()
            .find(|e| e.email.eq_ignore_ascii_case(email.trim()))
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
            project_name: None,
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

    #[test]
    fn validate_trial_passes_when_no_trial_flavor() {
        let s = r#"
            [project]
            name = "demo"

            [plane]
            url = "https://plane.example"
            workspace_slug = "ws"
            flavor = "auto"

            [mattermost]
            url = "https://mm.example"
            team_name = "team"
            flavor = "auto"
        "#;
        Config::parse(s).unwrap().validate_trial().unwrap();
    }

    #[test]
    fn validate_trial_fails_when_trial_flavor_but_section_missing() {
        let s = r#"
            [project]
            name = "demo"

            [plane]
            url = "http://localhost:3000"
            workspace_slug = "trial"
            flavor = "trial"

            [mattermost]
            url = "http://localhost:3000"
            team_name = "trial"
            flavor = "auto"
        "#;
        let err = Config::parse(s).unwrap().validate_trial().unwrap_err();
        match err {
            Error::Config(msg) => assert!(msg.contains("trial"), "msg = {msg}"),
            other => panic!("expected Config error, got {other:?}"),
        }
    }

    #[test]
    fn validate_trial_fails_when_enabled_false() {
        let s = r#"
            [project]
            name = "demo"

            [mattermost]
            url = "http://localhost:3000"
            team_name = "trial"
            flavor = "trial"

            [trial]
            enabled = false
            url = "http://localhost:3000"
            shared_secret = ""
        "#;
        let err = Config::parse(s).unwrap().validate_trial().unwrap_err();
        match err {
            Error::Config(msg) => assert!(msg.contains("enabled"), "msg = {msg}"),
            other => panic!("expected Config error, got {other:?}"),
        }
    }

    #[test]
    fn validate_trial_passes_when_trial_enabled() {
        let s = r#"
            [project]
            name = "demo"

            [plane]
            url = "http://localhost:3000"
            workspace_slug = "trial"
            flavor = "trial"

            [mattermost]
            url = "http://localhost:3000"
            team_name = "trial"
            flavor = "trial"

            [trial]
            enabled = true
            url = "http://localhost:3000"
            shared_secret = ""
        "#;
        Config::parse(s).unwrap().validate_trial().unwrap();
    }

    #[test]
    fn parses_humans_array() {
        let s = r#"
            [project]
            name = "demo"

            [[humans]]
            name = "Bravo"
            email = "gnoopy@gmail.com"

            [[humans]]
            name = "Mia"
            email = "mia@example.com"
            role = "reviewer"
            mm_username = "mia.r"
            locale = "en"
        "#;
        let cfg = Config::parse(s).unwrap();
        assert_eq!(cfg.humans.len(), 2);
        assert_eq!(cfg.humans[0].email, "gnoopy@gmail.com");
        assert_eq!(cfg.humans[0].role, "stakeholder", "default role applied");
        assert_eq!(cfg.humans[1].role, "reviewer");
        assert_eq!(cfg.humans[1].mm_username, "mia.r");
    }

    #[test]
    fn upsert_replaces_existing_email_case_insensitive() {
        let mut cfg = Config::default();
        let added = cfg.upsert_human(HumanEntry {
            name: "First".into(),
            email: "Foo@Bar.com".into(),
            role: "stakeholder".into(),
            mm_username: String::new(),
            locale: String::new(),
        });
        assert!(added);
        let added2 = cfg.upsert_human(HumanEntry {
            name: "Replaced".into(),
            email: "FOO@bar.COM".into(),
            role: "reviewer".into(),
            mm_username: String::new(),
            locale: String::new(),
        });
        assert!(!added2, "second upsert should replace not add");
        assert_eq!(cfg.humans.len(), 1);
        assert_eq!(cfg.humans[0].name, "Replaced");
        assert_eq!(cfg.humans[0].email, "foo@bar.com");
    }

    #[test]
    fn remove_human_is_idempotent() {
        let mut cfg = Config::default();
        cfg.upsert_human(HumanEntry {
            name: "A".into(),
            email: "a@x.com".into(),
            role: "stakeholder".into(),
            mm_username: String::new(),
            locale: String::new(),
        });
        assert!(cfg.remove_human("A@x.COM"));
        assert!(!cfg.remove_human("A@x.COM"));
        assert!(cfg.humans.is_empty());
    }

    #[test]
    fn derive_mm_username_strips_unsupported_chars() {
        assert_eq!(derive_mm_username("gnoopy@gmail.com"), "gnoopy");
        assert_eq!(derive_mm_username("a.b+tag@example.com"), "a.b");
        assert_eq!(derive_mm_username("xy@x.com"), "xy-hu");
    }

    #[test]
    fn slugify_basic_cases() {
        assert_eq!(slugify("Marketing Squad"), "marketing-squad");
        assert_eq!(slugify("  Trial Demo!! "), "trial-demo");
        assert_eq!(slugify("---x---"), "x");
        assert_eq!(slugify(""), "team");
        assert_eq!(slugify("팀"), "team", "non-ascii falls through to fallback");
        assert_eq!(slugify("Foo_Bar.baz"), "foo-bar-baz");
    }

    #[test]
    fn random_team_token_is_32_hex_chars() {
        let t = random_team_token();
        assert_eq!(t.len(), 32, "token = {t}");
        assert!(t.chars().all(|c| c.is_ascii_hexdigit()), "token = {t}");
        let t2 = random_team_token();
        assert_ne!(t, t2, "two consecutive tokens collided: {t} vs {t2}");
    }

    #[test]
    fn derive_naming_defaults_fills_blank_project_name() {
        let mut cfg = Config::default();
        cfg.project.name = "Marketing Squad".into();
        cfg.plane = Some(PlaneConfig {
            url: "u".into(),
            flavor: "auto".into(),
            workspace_slug: "ws".into(),
            project_id: None,
            project_name: None,
        });
        cfg.derive_naming_defaults();
        assert_eq!(
            cfg.plane.as_ref().unwrap().project_name.as_deref(),
            Some("Marketing Squad")
        );
    }

    #[test]
    fn derive_naming_defaults_synthesises_scrum_channel() {
        let mut cfg = Config::default();
        cfg.project.name = "Marketing Squad".into();
        cfg.mattermost = Some(MattermostConfig {
            url: "u".into(),
            flavor: "auto".into(),
            team_name: "marketing".into(),
            channels: vec![],
        });
        cfg.derive_naming_defaults();
        let channels = &cfg.mattermost.as_ref().unwrap().channels;
        assert_eq!(channels.len(), 1);
        assert_eq!(channels[0].key, "scrum");
        assert_eq!(channels[0].name, "scrum-marketing-squad");
        assert_eq!(channels[0].display_name, "Marketing Squad — Scrum");
    }

    #[test]
    fn derive_naming_defaults_is_idempotent_and_preserves_explicit_values() {
        let mut cfg = Config::default();
        cfg.project.name = "Squad A".into();
        cfg.plane = Some(PlaneConfig {
            url: "u".into(),
            flavor: "auto".into(),
            workspace_slug: "a".into(),
            project_id: None,
            project_name: Some("Squad Alpha (Pretty)".into()),
        });
        cfg.mattermost = Some(MattermostConfig {
            url: "u".into(),
            flavor: "auto".into(),
            team_name: "a".into(),
            channels: vec![MattermostChannel {
                key: "design".into(),
                name: "design-x".into(),
                display_name: "Design X".into(),
            }],
        });
        cfg.derive_naming_defaults();
        cfg.derive_naming_defaults();
        assert_eq!(
            cfg.plane.as_ref().unwrap().project_name.as_deref(),
            Some("Squad Alpha (Pretty)"),
            "pre-set project_name preserved"
        );
        assert_eq!(
            cfg.mattermost.as_ref().unwrap().channels.len(),
            1,
            "non-empty channel list left alone"
        );
        assert_eq!(cfg.mattermost.as_ref().unwrap().channels[0].key, "design");
    }

    #[test]
    fn effective_team_token_fallback_and_override() {
        let mut cfg = Config::default();
        assert_eq!(cfg.effective_team_token(), DEFAULT_TEAM_TOKEN);

        cfg.trial = Some(TrialConfig {
            enabled: true,
            url: "u".into(),
            shared_secret: String::new(),
            team_token: Some(String::new()),
        });
        assert_eq!(
            cfg.effective_team_token(),
            DEFAULT_TEAM_TOKEN,
            "empty token treated as fallback"
        );

        cfg.trial.as_mut().unwrap().team_token = Some("abc123".into());
        assert_eq!(cfg.effective_team_token(), "abc123");
    }

    #[test]
    fn mattermost_channel_lookup_by_key() {
        let cfg = Config {
            mattermost: Some(MattermostConfig {
                url: "u".into(),
                flavor: "auto".into(),
                team_name: "t".into(),
                channels: vec![
                    MattermostChannel {
                        key: "scrum".into(),
                        name: "scrum-x".into(),
                        display_name: "X — Scrum".into(),
                    },
                    MattermostChannel {
                        key: "design".into(),
                        name: "design-x".into(),
                        display_name: "X — Design".into(),
                    },
                ],
            }),
            ..Default::default()
        };
        assert_eq!(cfg.mattermost_channel("scrum").unwrap().name, "scrum-x");
        assert_eq!(cfg.mattermost_channel("design").unwrap().name, "design-x");
        assert!(cfg.mattermost_channel("nonexistent").is_none());
    }

    #[test]
    fn channels_round_trip_through_toml() {
        let s = r#"
            [project]
            name = "Marketing Squad"

            [mattermost]
            url = "https://mm.example"
            team_name = "marketing"
            flavor = "auto"
            channels = [
              { key = "scrum",  name = "scrum-marketing-squad",  display_name = "Marketing Squad — Scrum" },
              { key = "design", name = "design-marketing-squad", display_name = "Marketing Squad — Design" },
            ]
        "#;
        let cfg = Config::parse(s).unwrap();
        let channels = &cfg.mattermost.as_ref().unwrap().channels;
        assert_eq!(channels.len(), 2);
        assert_eq!(channels[0].key, "scrum");
        assert_eq!(channels[1].name, "design-marketing-squad");

        let encoded = cfg.to_toml_string().unwrap();
        let reparsed = Config::parse(&encoded).unwrap();
        assert_eq!(reparsed.mattermost.as_ref().unwrap().channels.len(), 2);
    }

    #[test]
    fn humans_lock_round_trip() {
        let dir = tempdir().unwrap();
        let mut lock = HumansLock::default();
        lock.upsert(HumanLockEntry {
            email: "a@x.com".into(),
            mm_user_id: "uid-1".into(),
            mm_username: "a".into(),
            plane_user_id: "plane-1".into(),
            mm_temp_password: "secret".into(),
            provisioned_at: "2026-05-10T00:00:00Z".into(),
        });
        lock.save(dir.path()).unwrap();
        let reloaded = HumansLock::load(dir.path()).unwrap();
        assert_eq!(reloaded.entries.len(), 1);
        assert_eq!(reloaded.find("A@x.com").unwrap().mm_user_id, "uid-1");
    }
}
