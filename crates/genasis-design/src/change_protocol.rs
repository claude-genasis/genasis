//! 5-phase design hot-swap orchestrator.
//!
//! Phases (per blueprint.md §7.2):
//!  1. Snapshot existing design-system.md.
//!  2. Run extractor (delegated to designer agent skill).
//!  3. Diff and categorise impacted areas.
//!  4. Emit IMPROVEMENT issues (one per area) — *plan* only; the caller
//!     hands these to the Plane provider.
//!  5. Tag and announce on Mattermost (the caller does this).
//!
//! The orchestrator is pure (no IO except snapshot + write). It returns
//! a [`SwapOutcome`] describing what changed, leaving the side effects
//! (Plane create_issue, Mattermost post_root, git tag) to the caller.

use std::path::Path;

use serde::{Deserialize, Serialize};

use genasis_core::error::Result;

use crate::diff::{changed_areas, ImpactArea};
use crate::extractor::{snapshot_existing, write_design_system, ExtractInput};
use crate::ticket_emitter::{plan as plan_issues, PlannedIssue};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwapPhase {
    Snapshot,
    Extract,
    Diff,
    Plan,
    Announce,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapOutcome {
    pub reference_url: String,
    pub areas: Vec<ImpactArea>,
    pub planned_issues: Vec<PlannedIssue>,
    pub previous_present: bool,
}

/// Run the swap with a *new* design-system.md body that has already been
/// produced by an external skill. The orchestrator persists the new body,
/// computes the diff, and returns the planned issues.
pub fn run(project_root: &Path, reference_url: &str, new_body: &str) -> Result<SwapOutcome> {
    // Phase 1 — snapshot existing.
    let snapshot = snapshot_existing(&ExtractInput {
        reference_url: reference_url.to_string(),
        project_root: project_root.to_path_buf(),
    })?;

    // Phase 3 — diff.
    let areas = changed_areas(&snapshot.current, new_body);

    // Phase 2 finishing — persist the new body.
    write_design_system(project_root, new_body)?;

    // Phase 4 — plan issues.
    let planned_issues = plan_issues(&areas, reference_url);

    Ok(SwapOutcome {
        reference_url: reference_url.into(),
        areas,
        planned_issues,
        previous_present: snapshot.previous.is_some(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn run_writes_new_body_and_plans_issues() {
        let dir = tempdir().unwrap();
        let docs = dir.path().join("docs");
        fs::create_dir_all(&docs).unwrap();
        fs::write(
            docs.join("design-system.md"),
            "color-primary: red\nfont-size: 14px\n",
        )
        .unwrap();

        let new = "color-primary: blue\nfont-size: 14px\n";
        let outcome = run(dir.path(), "https://gumroad.com", new).unwrap();
        assert!(outcome.previous_present);
        assert!(outcome.areas.contains(&ImpactArea::ColorTokens));
        assert!(!outcome.planned_issues.is_empty());
        // File on disk is the new body.
        assert_eq!(
            fs::read_to_string(docs.join("design-system.md")).unwrap(),
            new
        );
    }
}
