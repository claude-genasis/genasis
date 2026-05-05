//! `genasis design restore` — return a project from external mode to its
//! pristine `docs/design-system.md` body.
//!
//! Behaviour:
//!  1. read `.design-state.toml`. If `mode = pristine`, refuse with a
//!     clear message — there is nothing to restore.
//!  2. if `<external_dir>/pristine.bak` exists, copy it back to
//!     `docs/design-system.md` (atomic write).
//!  3. move the entire `<external_dir>/` to
//!     `docs/design-system.archive-<unix-ts>/` so the user can audit or
//!     recover. This is intentionally non-destructive — the old DESIGN.md
//!     and any overrides survive.
//!  4. delete `.design-state.toml` (absence == pristine).

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use genasis_core::error::{Error, Result};
use genasis_core::fs::{atomic_write, read_to_string_optional};

use crate::mode::{Mode, State};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreOutcome {
    pub previous_state: State,
    pub archive_dir: PathBuf,
    pub design_system_md_restored: bool,
}

pub fn run(project_root: &std::path::Path, external_dir: &str) -> Result<RestoreOutcome> {
    let state = State::load(project_root)?;
    if state.mode == Mode::Pristine {
        return Err(Error::Config(
            "design is already pristine — nothing to restore".to_string(),
        ));
    }

    let external_dir_abs = project_root.join(external_dir);
    let pristine_bak = external_dir_abs.join("pristine.bak");
    let pointer_path = project_root.join("docs").join("design-system.md");

    let restored = if let Some(body) = read_to_string_optional(&pristine_bak)? {
        atomic_write(&pointer_path, body.as_bytes())?;
        true
    } else {
        // No backup recorded — leave the pointer body in place but warn the
        // caller. CLI surface translates this into a yellow "no backup
        // found" notice.
        false
    };

    // Archive the external dir.
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let archive_dir = project_root
        .join("docs")
        .join(format!("design-system.archive-{ts}"));
    if external_dir_abs.exists() {
        std::fs::rename(&external_dir_abs, &archive_dir)?;
    }

    // Drop the state file. Pristine is "no state".
    let state_path = State::path_in(project_root);
    if state_path.exists() {
        std::fs::remove_file(&state_path)?;
    }

    Ok(RestoreOutcome {
        previous_state: state,
        archive_dir,
        design_system_md_restored: restored,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pointer::Locale;
    use crate::swap::{self, Source, SwapInput};
    use tempfile::tempdir;

    #[test]
    fn restore_returns_to_pristine_body() {
        let dir = tempdir().unwrap();
        let docs = dir.path().join("docs");
        std::fs::create_dir_all(&docs).unwrap();
        let pristine_body = "# my pristine design system\ntoken: 1\n";
        std::fs::write(docs.join("design-system.md"), pristine_body).unwrap();

        let local = dir.path().join("brand.md");
        std::fs::write(&local, "# external\n").unwrap();
        swap::run(SwapInput {
            project_root: dir.path().to_path_buf(),
            external_dir: "docs/design-system".into(),
            gallery_index_url: "https://getdesign.md/".into(),
            gallery_url_template: "https://getdesign.md/{slug}/design-md".into(),
            disable_telemetry: true,
            locale: Locale::En,
            source: Source::File(local),
        })
        .unwrap();

        // Sanity — we are now external.
        assert_eq!(State::load(dir.path()).unwrap().mode, Mode::External);

        let out = run(dir.path(), "docs/design-system").unwrap();
        assert!(out.design_system_md_restored);

        // Pristine body is back.
        assert_eq!(
            std::fs::read_to_string(docs.join("design-system.md")).unwrap(),
            pristine_body
        );
        // External dir was moved to archive (not deleted).
        assert!(!dir.path().join("docs/design-system").exists());
        assert!(out.archive_dir.is_dir());
        // State file is gone.
        assert!(!State::path_in(dir.path()).exists());
        // Loading state after restore returns pristine default.
        assert_eq!(State::load(dir.path()).unwrap().mode, Mode::Pristine);
    }

    #[test]
    fn restore_in_pristine_errors() {
        let dir = tempdir().unwrap();
        let err = run(dir.path(), "docs/design-system").unwrap_err();
        assert!(err.to_string().contains("already pristine"));
    }
}
