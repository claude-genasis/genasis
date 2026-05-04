//! Filesystem primitives: atomic write, snapshot/backup, transactional rename.
//!
//! These are deliberately small and synchronous — the overlay engine writes
//! one file at a time and does not need async IO. Async paths use these via
//! `tokio::task::spawn_blocking` if needed.
//!
//! Atomicity strategy:
//! - Write to a sibling temp file (`<name>.<pid>.<rand>.tmp`).
//! - `fsync` the temp file.
//! - `rename` the temp file over the target (POSIX atomic on the same fs).
//! - Best-effort `fsync` of the parent directory so the rename is durable.
//!
//! Snapshot strategy:
//! - Copy `path` to `path.with_extension("genasis.bak.<timestamp>")`.
//! - Returns the backup path so callers can record it for rollback.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{Error, Result};

/// Atomically write `content` to `path`. The parent directory must exist.
pub fn atomic_write(path: &Path, content: &[u8]) -> Result<()> {
    let parent = path.parent().ok_or_else(|| {
        Error::Config(format!(
            "atomic_write: path has no parent: {}",
            path.display()
        ))
    })?;
    if !parent.as_os_str().is_empty() {
        std::fs::create_dir_all(parent)?;
    }

    let tmp = tmp_sibling(path);
    {
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)?;
        f.write_all(content)?;
        f.flush()?;
        // Best-effort fsync; ignore platforms that don't support it well.
        let _ = f.sync_all();
    }

    if let Err(e) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e.into());
    }

    // Best-effort directory fsync so the rename survives a crash.
    if let Ok(dir) = std::fs::File::open(parent) {
        let _ = dir.sync_all();
    }
    Ok(())
}

fn tmp_sibling(path: &Path) -> PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let pid = std::process::id();
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "tmp".to_string());
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    parent.join(format!(".{name}.{pid}.{ts}.tmp"))
}

/// Copy `path` to `path.<ext>.genasis.bak.<unix-ts>` and return the backup path.
/// If `path` does not exist, returns `Ok(None)`.
pub fn snapshot(path: &Path) -> Result<Option<PathBuf>> {
    if !path.exists() {
        return Ok(None);
    }
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let mut backup = path.as_os_str().to_owned();
    backup.push(format!(".genasis.bak.{ts}"));
    let backup_path = PathBuf::from(backup);
    std::fs::copy(path, &backup_path)?;
    Ok(Some(backup_path))
}

/// Read a UTF-8 file or return `Ok(None)` if it does not exist.
pub fn read_to_string_optional(path: &Path) -> Result<Option<String>> {
    match std::fs::read_to_string(path) {
        Ok(s) => Ok(Some(s)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn atomic_write_creates_then_replaces() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("nested/dir/file.txt");
        atomic_write(&p, b"first").unwrap();
        assert_eq!(fs::read_to_string(&p).unwrap(), "first");

        atomic_write(&p, b"second").unwrap();
        assert_eq!(fs::read_to_string(&p).unwrap(), "second");

        // No leftover tmp files in the parent dir.
        let stray = fs::read_dir(p.parent().unwrap())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().ends_with(".tmp"))
            .count();
        assert_eq!(stray, 0);
    }

    #[test]
    fn snapshot_copies_existing_file() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("a.md");
        fs::write(&p, "alpha").unwrap();
        let backup = snapshot(&p).unwrap().unwrap();
        assert_eq!(fs::read_to_string(&backup).unwrap(), "alpha");
        assert_ne!(backup, p);
    }

    #[test]
    fn snapshot_missing_returns_none() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("missing.md");
        assert!(snapshot(&p).unwrap().is_none());
    }

    #[test]
    fn read_optional_handles_missing() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("missing.md");
        assert!(read_to_string_optional(&p).unwrap().is_none());
    }
}
