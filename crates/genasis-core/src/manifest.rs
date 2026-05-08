//! Manifest of every Genasis-managed file in a project, written
//! after `attach` / `bootstrap` / `init` apply succeeds. The manifest
//! is the source of truth for drift detection (M15) — we hash each
//! managed file at apply time and re-hash on every CLI invocation to
//! surface user edits as `DriftEntry`s.
//!
//! Layout: `.claude/genasis/.manifest.json`
//!
//! Schema (all fields optional in the entries map for forward compat):
//! ```json
//! {
//!   "genasis_version": "0.0.1",
//!   "agents_catalog_version": "1.0.0",
//!   "attached_at": "2026-05-08T12:00:00Z",
//!   "lang": "en",
//!   "files": {
//!     ".claude/agents/frontend.md": {
//!       "sha256": "<full-file-hash>",
//!       "fence_sha256": "<fence-only-hash>",
//!       "template_source": "agents-pool@frontend.patch.md.tera"
//!     }
//!   }
//! }
//! ```
//!
//! ADR-012: contributors only ever submit `debug-history/patches/*.patch.json`
//! files to the genasis repo, which means `manifest::compare()` results
//! must be deterministic and self-contained — no path leakage, no
//! environment-specific timestamps in the diff itself.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;

pub const MANIFEST_RELATIVE_PATH: &str = ".claude/genasis/.manifest.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Manifest {
    pub genasis_version: String,
    #[serde(default)]
    pub agents_catalog_version: String,
    #[serde(default)]
    pub attached_at: String,
    #[serde(default)]
    pub lang: String,
    pub files: BTreeMap<String, FileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct FileEntry {
    pub sha256: String,
    #[serde(default)]
    pub fence_sha256: String,
    #[serde(default)]
    pub template_source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriftKind {
    /// File present in manifest, gone from disk.
    Removed,
    /// File on disk, not in the manifest.
    Added,
    /// File hash differs from the recorded one.
    Modified,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriftEntry {
    pub file: String,
    pub kind: DriftKind,
    pub recorded_hash: Option<String>,
    pub actual_hash: Option<String>,
}

impl Manifest {
    pub fn new(genasis_version: impl Into<String>) -> Self {
        Self {
            genasis_version: genasis_version.into(),
            agents_catalog_version: String::new(),
            attached_at: String::new(),
            lang: String::new(),
            files: BTreeMap::new(),
        }
    }

    pub fn load(project_root: &Path) -> Result<Option<Self>> {
        let path = project_root.join(MANIFEST_RELATIVE_PATH);
        if !path.is_file() {
            return Ok(None);
        }
        let body = std::fs::read_to_string(&path)?;
        let m: Manifest = serde_json::from_str(&body)?;
        Ok(Some(m))
    }

    pub fn save(&self, project_root: &Path) -> Result<()> {
        let path = project_root.join(MANIFEST_RELATIVE_PATH);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let body = serde_json::to_string_pretty(self)?;
        crate::fs::atomic_write(&path, body.as_bytes())?;
        Ok(())
    }
}

/// SHA-256 of the file contents at `path`. Returns `None` if the file
/// is missing.
pub fn hash_file(path: &Path) -> Result<Option<String>> {
    if !path.is_file() {
        return Ok(None);
    }
    let bytes = std::fs::read(path)?;
    Ok(Some(sha256_hex(&bytes)))
}

/// Build a manifest entry for a given file relative to the project
/// root. Caller fills `template_source` and `fence_sha256` if known.
pub fn entry_for(project_root: &Path, rel_path: &str) -> Result<Option<FileEntry>> {
    let abs = project_root.join(rel_path);
    let Some(sha) = hash_file(&abs)? else {
        return Ok(None);
    };
    Ok(Some(FileEntry {
        sha256: sha,
        ..Default::default()
    }))
}

/// Compare a manifest against the project's current disk state.
/// Only files listed in the manifest are considered "managed"; we
/// do not try to enumerate `.claude/genasis/` blindly.
pub fn compare(manifest: &Manifest, project_root: &Path) -> Result<Vec<DriftEntry>> {
    let mut out = Vec::new();
    for (rel, entry) in &manifest.files {
        let abs = project_root.join(rel);
        match hash_file(&abs)? {
            None => out.push(DriftEntry {
                file: rel.clone(),
                kind: DriftKind::Removed,
                recorded_hash: Some(entry.sha256.clone()),
                actual_hash: None,
            }),
            Some(actual) if actual == entry.sha256 => {
                // Pristine — no drift entry.
            }
            Some(actual) => out.push(DriftEntry {
                file: rel.clone(),
                kind: DriftKind::Modified,
                recorded_hash: Some(entry.sha256.clone()),
                actual_hash: Some(actual),
            }),
        }
    }
    Ok(out)
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut s = String::with_capacity(64);
    for b in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut s, "{b:02x}");
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn save_then_load_round_trips() {
        let dir = tempdir().unwrap();
        let mut m = Manifest::new("0.0.1");
        m.files.insert(
            ".claude/agents/frontend.md".into(),
            FileEntry {
                sha256: "abc".into(),
                ..Default::default()
            },
        );
        m.save(dir.path()).unwrap();
        let loaded = Manifest::load(dir.path()).unwrap().unwrap();
        assert_eq!(loaded, m);
    }

    #[test]
    fn load_returns_none_when_absent() {
        let dir = tempdir().unwrap();
        assert!(Manifest::load(dir.path()).unwrap().is_none());
    }

    #[test]
    fn compare_flags_modified_files() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("a.md");
        std::fs::write(&target, "v1").unwrap();
        let mut m = Manifest::new("0.0.1");
        m.files.insert(
            "a.md".into(),
            FileEntry {
                sha256: hash_file(&target).unwrap().unwrap(),
                ..Default::default()
            },
        );
        // No drift while the bytes match.
        assert!(compare(&m, dir.path()).unwrap().is_empty());
        // Mutate.
        std::fs::write(&target, "v2").unwrap();
        let drift = compare(&m, dir.path()).unwrap();
        assert_eq!(drift.len(), 1);
        assert_eq!(drift[0].kind, DriftKind::Modified);
    }

    #[test]
    fn compare_flags_removed_files() {
        let dir = tempdir().unwrap();
        let mut m = Manifest::new("0.0.1");
        m.files.insert(
            "a.md".into(),
            FileEntry {
                sha256: "abc".into(),
                ..Default::default()
            },
        );
        let drift = compare(&m, dir.path()).unwrap();
        assert_eq!(drift.len(), 1);
        assert_eq!(drift[0].kind, DriftKind::Removed);
    }

    #[test]
    fn hash_file_returns_none_for_missing() {
        let dir = tempdir().unwrap();
        assert!(hash_file(&dir.path().join("nonexistent.md")).unwrap().is_none());
    }
}
