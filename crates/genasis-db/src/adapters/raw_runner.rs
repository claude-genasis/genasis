//! Raw SQL runner for DuckDB and other engines without a declarative
//! migration tool.
//!
//! Layout: `<project>/db/migrations/<timestamp>__<slug>.up.sql`
//! and a matching `.down.sql`. Application is sequential by filename.
//! State is tracked in a single `genasis_migrations` table created on
//! first run.

use std::path::{Path, PathBuf};

use genasis_core::error::{Error, Result};

pub async fn apply(project_root: &Path) -> Result<String> {
    let dir = migrations_dir(project_root);
    if !dir.is_dir() {
        return Err(Error::Db(format!(
            "raw_runner: missing {} — create db/migrations/ with *.up.sql files",
            dir.display()
        )));
    }
    let mut entries: Vec<_> = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension().and_then(|s| s.to_str()) == Some("sql")
                && p.file_name()
                    .and_then(|s| s.to_str())
                    .map(|n| n.ends_with(".up.sql"))
                    .unwrap_or(false)
        })
        .collect();
    entries.sort();

    let plan = entries
        .iter()
        .map(|p| format!(" - {}", p.display()))
        .collect::<Vec<_>>()
        .join("\n");
    Ok(format!(
        "raw_runner plan ({} migration(s)):\n{plan}\n\n[note] M5 lays the groundwork; runtime SQL execution \
         lands once the DuckDB Rust binding is approved (see ADR-004).",
        entries.len()
    ))
}

pub async fn diff(project_root: &Path) -> Result<String> {
    apply(project_root).await
}

fn migrations_dir(project_root: &Path) -> PathBuf {
    project_root.join("db").join("migrations")
}
