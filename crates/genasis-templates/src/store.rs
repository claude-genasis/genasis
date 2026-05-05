//! AgentStore — disk-based template/agent loader.
//!
//! Replaces the old `include_dir!()` static `TEMPLATES: Dir<'_>`.
//! Consumers (merger, bootstrap) get an `&AgentStore` reference and use
//! the same `get_file` / `get_dir_files` interface.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use genasis_core::error::{Error, Result};

use crate::cache;
use crate::registry;

/// A loaded agents catalog from the local cache.
///
/// Holds the cache directory path and provides file access methods
/// analogous to the old `include_dir::Dir` interface.
pub struct AgentStore {
    root: PathBuf,
}

impl AgentStore {
    /// Open an already-cached catalog directory.
    pub fn from_dir(dir: PathBuf) -> Result<Self> {
        if !dir.join("manifest.json").is_file() {
            return Err(Error::Config(format!(
                "not a valid agents catalog (missing manifest.json): {}",
                dir.display()
            )));
        }
        Ok(Self { root: dir })
    }

    /// Root directory of the loaded catalog.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Read a file by relative path (e.g., `"base/frontend-developer.md"`).
    pub fn get_file(&self, relative: &str) -> Option<String> {
        let path = self.root.join(relative);
        fs::read_to_string(&path).ok()
    }

    /// List files in a subdirectory matching a suffix filter.
    ///
    /// Returns `(filename, contents)` pairs. Useful for loading all
    /// overlay `.tera` files from `overlays/en/`.
    pub fn get_dir_files(&self, subdir: &str, suffix: &str) -> Result<Vec<(String, String)>> {
        let dir = self.root.join(subdir);
        if !dir.is_dir() {
            return Err(Error::Overlay(format!(
                "catalog subdirectory missing: {subdir}"
            )));
        }
        let mut files = Vec::new();
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(suffix) {
                let contents = fs::read_to_string(entry.path())?;
                files.push((name, contents));
            }
        }
        files.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(files)
    }

    /// List all base agent filenames (without path prefix).
    pub fn list_base_agents(&self) -> Result<Vec<String>> {
        let dir = self.root.join("base");
        if !dir.is_dir() {
            return Ok(Vec::new());
        }
        let mut names = Vec::new();
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".md") && !name.starts_with('.') {
                names.push(name);
            }
        }
        names.sort();
        Ok(names)
    }

    /// Load the manifest.json contents as a parsed JSON value.
    pub fn manifest(&self) -> Result<serde_json::Value> {
        let path = self.root.join("manifest.json");
        let raw = fs::read_to_string(&path)?;
        serde_json::from_str(&raw).map_err(|e| Error::Config(format!("manifest parse: {e}")))
    }
}

/// High-level load function: ensure the pinned version is cached, then open it.
///
/// If the version is not cached and `auto_fetch` is true, fetches from the
/// registry. If `auto_fetch` is false and not cached, returns an error
/// guiding the user to run `genasis agents fetch`.
pub fn load(
    version: &str,
    registry_url: &str,
    cache_override: &str,
    auto_fetch: bool,
) -> Result<AgentStore> {
    if cache::is_cached(version, cache_override)? {
        let dir = cache::cache_dir(version, cache_override)?;
        return AgentStore::from_dir(dir);
    }

    if !auto_fetch {
        return Err(Error::Config(format!(
            "agents catalog v{version} not cached. Run `genasis agents fetch` first, \
             or set [agents].auto_check = true in genasis.toml."
        )));
    }

    // Auto-fetch from registry.
    let tarball = registry::fetch_tarball(registry_url, version)?;
    let dir = cache::store_tarball(version, cache_override, &tarball)?;
    AgentStore::from_dir(dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn create_mock_catalog(dir: &Path) {
        fs::create_dir_all(dir.join("base")).unwrap();
        fs::create_dir_all(dir.join("overlays/en")).unwrap();
        fs::write(
            dir.join("manifest.json"),
            r#"{"version":"0.0.1","roles":[]}"#,
        )
        .unwrap();
        fs::write(
            dir.join("base/frontend-developer.md"),
            "---\nname: frontend-developer\n---\n# Frontend\n",
        )
        .unwrap();
        fs::write(
            dir.join("overlays/en/frontend.patch.md.tera"),
            "## overlay\n{{ project_name }}\n",
        )
        .unwrap();
    }

    #[test]
    fn from_dir_valid() {
        let d = tempdir().unwrap();
        create_mock_catalog(d.path());
        let store = AgentStore::from_dir(d.path().to_path_buf()).unwrap();
        assert_eq!(store.root(), d.path());
    }

    #[test]
    fn from_dir_invalid_no_manifest() {
        let d = tempdir().unwrap();
        let err = AgentStore::from_dir(d.path().to_path_buf()).unwrap_err();
        assert!(format!("{err:?}").contains("manifest.json"));
    }

    #[test]
    fn get_file_reads_content() {
        let d = tempdir().unwrap();
        create_mock_catalog(d.path());
        let store = AgentStore::from_dir(d.path().to_path_buf()).unwrap();
        let content = store.get_file("base/frontend-developer.md").unwrap();
        assert!(content.contains("name: frontend-developer"));
    }

    #[test]
    fn get_dir_files_with_suffix() {
        let d = tempdir().unwrap();
        create_mock_catalog(d.path());
        let store = AgentStore::from_dir(d.path().to_path_buf()).unwrap();
        let files = store.get_dir_files("overlays/en", ".patch.md.tera").unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].0, "frontend.patch.md.tera");
        assert!(files[0].1.contains("{{ project_name }}"));
    }

    #[test]
    fn list_base_agents_returns_md_files() {
        let d = tempdir().unwrap();
        create_mock_catalog(d.path());
        let store = AgentStore::from_dir(d.path().to_path_buf()).unwrap();
        let agents = store.list_base_agents().unwrap();
        assert_eq!(agents, vec!["frontend-developer.md"]);
    }
}
