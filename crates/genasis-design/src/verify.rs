//! `genasis design verify` — re-hash the active external `DESIGN.md` and
//! compare against `.design-state.toml.template_hash`. Detects accidental
//! human edits to the read-only external body, or filesystem corruption.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use genasis_core::error::{Error, Result};

use crate::mode::{sha256_hex, Mode, State};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyOutcome {
    pub mode: Mode,
    pub design_md_path: Option<PathBuf>,
    pub recorded_hash: String,
    pub actual_hash: String,
    pub matches: bool,
}

pub fn run(project_root: &std::path::Path, external_dir: &str) -> Result<VerifyOutcome> {
    let state = State::load(project_root)?;
    if state.mode == Mode::Pristine {
        return Ok(VerifyOutcome {
            mode: Mode::Pristine,
            design_md_path: None,
            recorded_hash: String::new(),
            actual_hash: String::new(),
            matches: true,
        });
    }
    let path = project_root.join(external_dir).join("DESIGN.md");
    let body = std::fs::read_to_string(&path).map_err(|e| {
        Error::Config(format!(
            "external DESIGN.md missing or unreadable at {} ({e})",
            path.display()
        ))
    })?;
    let actual_hash = sha256_hex(&body);
    let matches = actual_hash == state.template_hash;
    Ok(VerifyOutcome {
        mode: Mode::External,
        design_md_path: Some(path),
        recorded_hash: state.template_hash,
        actual_hash,
        matches,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pointer::Locale;
    use crate::swap::{self, Source, SwapInput};
    use tempfile::tempdir;

    fn setup_external(dir: &std::path::Path, body: &str) {
        let docs = dir.join("docs");
        std::fs::create_dir_all(&docs).unwrap();
        std::fs::write(docs.join("design-system.md"), "# pristine\n").unwrap();
        let local = dir.join("spec.md");
        std::fs::write(&local, body).unwrap();
        swap::run(SwapInput {
            project_root: dir.to_path_buf(),
            external_dir: "docs/design-system".into(),
            gallery_index_url: "https://getdesign.md/".into(),
            gallery_url_template: "https://getdesign.md/{slug}/design-md".into(),
            disable_telemetry: true,
            locale: Locale::En,
            source: Source::File(local),
        })
        .unwrap();
    }

    #[test]
    fn pristine_mode_always_matches() {
        let dir = tempdir().unwrap();
        let v = run(dir.path(), "docs/design-system").unwrap();
        assert_eq!(v.mode, Mode::Pristine);
        assert!(v.matches);
    }

    #[test]
    fn untampered_external_matches() {
        let dir = tempdir().unwrap();
        setup_external(dir.path(), "# original\ncolor: blue\n");
        let v = run(dir.path(), "docs/design-system").unwrap();
        assert_eq!(v.mode, Mode::External);
        assert!(v.matches);
        assert_eq!(v.recorded_hash, v.actual_hash);
    }

    #[test]
    fn tampered_external_detected() {
        let dir = tempdir().unwrap();
        setup_external(dir.path(), "# original\ncolor: blue\n");
        // Hand-edit DESIGN.md (anti-pattern the skill forbids — but verify
        // is the safety net).
        let path = dir.path().join("docs/design-system/DESIGN.md");
        std::fs::write(&path, "# tampered\ncolor: green\n").unwrap();
        let v = run(dir.path(), "docs/design-system").unwrap();
        assert_eq!(v.mode, Mode::External);
        assert!(!v.matches);
        assert_ne!(v.recorded_hash, v.actual_hash);
    }
}
