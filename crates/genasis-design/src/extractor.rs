//! Wraps the `ui-style-extractor` skill that pulls a CSS palette from a
//! reference URL.
//!
//! Genasis does not re-implement CSS parsing — that lives in the designer
//! agent's `ui-style-extractor` skill. This module records the reference
//! URL and snapshots the existing design-system.md so the diff phase has
//! both before and after.

use std::path::Path;

use genasis_core::error::{Error, Result};
use genasis_core::fs as gfs;

#[derive(Debug, Clone)]
pub struct ExtractInput {
    pub reference_url: String,
    pub project_root: std::path::PathBuf,
}

#[derive(Debug, Clone)]
pub struct ExtractOutput {
    pub previous: Option<String>,
    pub current: String,
}

/// Snapshot the existing `docs/design-system.md` so the diff phase has
/// both versions to compare. Errors if the file is missing — the design
/// hot-swap requires a baseline.
pub fn snapshot_existing(input: &ExtractInput) -> Result<ExtractOutput> {
    let target = input.project_root.join("docs").join("design-system.md");
    let previous = gfs::read_to_string_optional(&target)?;
    let current = previous.clone().ok_or_else(|| {
        Error::Config(format!(
            "docs/design-system.md missing at {} — run `genasis attach` first",
            target.display()
        ))
    })?;
    Ok(ExtractOutput { previous, current })
}

/// Atomically write a fresh design-system.md (after the designer-agent
/// skill has produced new content).
pub fn write_design_system(project_root: &Path, body: &str) -> Result<()> {
    let target = project_root.join("docs").join("design-system.md");
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    gfs::atomic_write(&target, body.as_bytes())
}
