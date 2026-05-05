//! Local cache management for the agents catalog.
//!
//! Default location: `~/.cache/genasis/agents/v{version}/`
//! Override via `genasis.toml [agents].cache_dir`.

use std::fs;
use std::io::Read as IoRead;
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use genasis_core::error::{Error, Result};

/// Resolve the cache directory for a given version.
///
/// If `override_dir` is non-empty, use that. Otherwise default to
/// `~/.cache/genasis/agents/v{version}/`.
pub fn cache_dir(version: &str, override_dir: &str) -> Result<PathBuf> {
    if !override_dir.is_empty() {
        return Ok(PathBuf::from(override_dir).join(format!("v{version}")));
    }
    let base = dirs::cache_dir()
        .ok_or_else(|| Error::Config("cannot determine home cache directory".into()))?;
    Ok(base
        .join("genasis")
        .join("agents")
        .join(format!("v{version}")))
}

/// Check whether a version is already cached (directory exists + manifest.json present).
pub fn is_cached(version: &str, override_dir: &str) -> Result<bool> {
    let dir = cache_dir(version, override_dir)?;
    Ok(dir.join("manifest.json").is_file())
}

/// Store a downloaded tarball (gzipped tar) into the cache.
///
/// Extracts the tarball contents into `cache_dir(version)/`.
pub fn store_tarball(version: &str, override_dir: &str, tarball: &[u8]) -> Result<PathBuf> {
    let dir = cache_dir(version, override_dir)?;
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }
    fs::create_dir_all(&dir)?;

    let decoder = GzDecoder::new(tarball);
    let mut archive = tar::Archive::new(decoder);
    archive
        .unpack(&dir)
        .map_err(|e| Error::Config(format!("failed to extract agents tarball: {e}")))?;

    // Verify manifest exists after extraction.
    if !dir.join("manifest.json").is_file() {
        return Err(Error::Config(format!(
            "extracted tarball missing manifest.json in {}",
            dir.display()
        )));
    }
    Ok(dir)
}

/// List all cached versions (directory names matching `v*`).
pub fn list_cached(override_dir: &str) -> Result<Vec<String>> {
    let base = if !override_dir.is_empty() {
        PathBuf::from(override_dir)
    } else {
        let cache = dirs::cache_dir()
            .ok_or_else(|| Error::Config("cannot determine home cache directory".into()))?;
        cache.join("genasis").join("agents")
    };
    if !base.is_dir() {
        return Ok(Vec::new());
    }
    let mut versions = Vec::new();
    for entry in fs::read_dir(&base)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('v') {
                versions.push(name[1..].to_string());
            }
        }
    }
    versions.sort();
    Ok(versions)
}

/// Remove a cached version.
pub fn remove_cached(version: &str, override_dir: &str) -> Result<()> {
    let dir = cache_dir(version, override_dir)?;
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }
    Ok(())
}
