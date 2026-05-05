//! `docs/.design-state.toml` — single source of truth for which design is
//! active and how it got there.
//!
//! M-D1: written by `genasis design swap` / `restore`, read by every other
//! `cmd_design` subcommand and the monitor "Design" widget. Never edited by
//! humans (locking convention enforced via comment header on serialise).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use genasis_core::error::{Error, Result};
use genasis_core::fs::{atomic_write, read_to_string_optional};

pub const STATE_FILE_NAME: &str = ".design-state.toml";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// `docs/design-system.md` body is the truth (no external delegation).
    Pristine,
    /// `docs/design-system.md` is a pointer; `<external_dir>/DESIGN.md` is
    /// the active reference.
    External,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Pristine
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub mode: Mode,

    /// External slug (e.g. "posthog"). Empty in pristine mode.
    #[serde(default)]
    pub slug: String,

    /// Provenance string. `getdesign/<slug>` for npx path,
    /// `file:<absolute-path>` for `--from` path. Empty in pristine.
    #[serde(default)]
    pub source: String,

    /// Verbatim shell command that was run (for reproducibility / audit).
    #[serde(default)]
    pub source_command: String,

    /// SHA-256 of the active `DESIGN.md` body (lowercase hex). Empty in
    /// pristine mode.
    #[serde(default)]
    pub template_hash: String,

    /// ISO-8601 UTC timestamp of the most recent swap.
    #[serde(default)]
    pub applied_at: String,

    /// Slug of the previously active external design, if any. Persisted
    /// across swaps so users can see "I was on apple, switched to posthog".
    #[serde(default)]
    pub previous_slug: String,

    /// Per-slug preview URL. Resolved at swap time from `[design]
    /// gallery_url_template`.
    #[serde(default)]
    pub gallery_preview: String,

    /// Gallery landing URL. Snapshot of `[design] gallery_index_url` at
    /// swap time.
    #[serde(default)]
    pub gallery_index: String,

    /// Number of user-override entries currently accumulated under §B of
    /// the pointer body. Bumped by `genasis design override add`.
    #[serde(default)]
    pub override_count: u32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            mode: Mode::Pristine,
            slug: String::new(),
            source: String::new(),
            source_command: String::new(),
            template_hash: String::new(),
            applied_at: String::new(),
            previous_slug: String::new(),
            gallery_preview: String::new(),
            gallery_index: String::new(),
            override_count: 0,
        }
    }
}

impl State {
    /// Path to the state file given the project root.
    pub fn path_in(project_root: &Path) -> PathBuf {
        project_root.join("docs").join(STATE_FILE_NAME)
    }

    /// Load the state file. Returns `Mode::Pristine` default if the file is
    /// missing — the absence of state is itself the canonical pristine signal.
    pub fn load(project_root: &Path) -> Result<Self> {
        let path = Self::path_in(project_root);
        match read_to_string_optional(&path)? {
            Some(body) => toml::from_str(&body)
                .map_err(|e| Error::Config(format!("parse {}: {e}", path.display()))),
            None => Ok(Self::default()),
        }
    }

    /// Atomically persist the state file with a do-not-edit header.
    pub fn save(&self, project_root: &Path) -> Result<()> {
        let path = Self::path_in(project_root);
        let body = toml::to_string_pretty(self)
            .map_err(|e| Error::Config(format!("encode design state: {e}")))?;
        let with_header = format!(
            "# docs/.design-state.toml — managed by `genasis design`. Do not edit by hand.\n# `genasis design swap` / `restore` / `override` rewrite this file.\n\n{body}"
        );
        atomic_write(&path, with_header.as_bytes())
    }
}

/// Convenience: SHA-256 hex digest of a UTF-8 body.
pub fn sha256_hex(body: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(body.as_bytes());
    hex::encode(digest)
}

/// Convenience: ISO-8601 UTC timestamp `YYYY-MM-DDTHH:MM:SSZ`.
pub fn iso8601_now() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn round_trip_through_disk() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("docs")).unwrap();
        let s = State {
            mode: Mode::External,
            slug: "posthog".into(),
            source: "getdesign/posthog".into(),
            source_command: "npx getdesign add posthog".into(),
            template_hash: "abcd".into(),
            applied_at: "2026-05-04T10:00:00Z".into(),
            previous_slug: "apple".into(),
            gallery_preview: "https://getdesign.md/posthog/design-md".into(),
            gallery_index: "https://getdesign.md/".into(),
            override_count: 2,
        };
        s.save(dir.path()).unwrap();
        let r = State::load(dir.path()).unwrap();
        assert_eq!(r.mode, Mode::External);
        assert_eq!(r.slug, "posthog");
        assert_eq!(r.override_count, 2);
    }

    #[test]
    fn missing_state_is_pristine() {
        let dir = tempdir().unwrap();
        let s = State::load(dir.path()).unwrap();
        assert_eq!(s.mode, Mode::Pristine);
        assert!(s.slug.is_empty());
    }

    #[test]
    fn header_present_after_save() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("docs")).unwrap();
        let s = State::default();
        s.save(dir.path()).unwrap();
        let body = std::fs::read_to_string(State::path_in(dir.path())).unwrap();
        assert!(body.starts_with("# docs/.design-state.toml"));
    }

    #[test]
    fn sha256_is_stable_lowercase_hex() {
        let h = sha256_hex("hello");
        assert_eq!(h.len(), 64);
        assert!(h
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }
}
