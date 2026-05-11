//! Sanity check against the real v1.0.0 catalog tarball shipped at
//! `agents-pool/release-assets/agents-v1.0.0.tar.gz`. Demonstrates that
//! the alias-walk + graceful-skip refactor lets bootstrap survive a
//! partial catalog (which is what real users actually get when they
//! download v1.0.0 from GitHub Releases).
//!
//! Skipped automatically when the tarball is not present (e.g. on CI
//! runs of the public repo where the agents-pool submodule is empty).

use std::path::PathBuf;

use genasis_overlay::{plan_bootstrap, BootstrapAction, BootstrapOptions, Role};
use genasis_templates::{store_tarball, AgentStore};

fn tarball_path() -> Option<PathBuf> {
    let candidate = std::env::current_dir()
        .ok()?
        .ancestors()
        .find(|p| {
            p.join("agents-pool/release-assets/agents-v1.0.0.tar.gz")
                .is_file()
        })?
        .join("agents-pool/release-assets/agents-v1.0.0.tar.gz");
    Some(candidate)
}

#[test]
fn bootstrap_against_real_v1_catalog_completes_without_error() {
    let Some(tarball) = tarball_path() else {
        eprintln!("skipped: agents-pool tarball not available");
        return;
    };

    // Stage into a fresh cache root using the same path resolution as
    // the real CLI (override_dir wins → cache_dir = <root>/v1.0.0/).
    let cache_root = tempfile::tempdir().unwrap();
    let raw = std::fs::read(&tarball).unwrap();
    let cache_dir = store_tarball("1.0.0", cache_root.path().to_str().unwrap(), &raw).unwrap();

    let store = AgentStore::from_dir(cache_dir).unwrap();
    let project = tempfile::tempdir().unwrap();
    let plan = plan_bootstrap(project.path(), &BootstrapOptions::default(), &store).unwrap();

    // Every role gets a decision — no early Err.
    assert_eq!(plan.changes.len(), Role::ALL.len());

    // The v1.0.0 catalog ships canonical filenames for every role we
    // care about (planner / architect / code-reviewer as plain slugs,
    // and frontend-developer / backend-developer / qa-tester / etc. via
    // aliases). With the alias-walk refactor every role must resolve.
    let missing_slugs: Vec<&str> = plan
        .changes
        .iter()
        .filter_map(|c| {
            if matches!(c.action, BootstrapAction::Missing { .. }) {
                Some(c.role.slug())
            } else {
                None
            }
        })
        .collect();
    assert!(
        missing_slugs.is_empty(),
        "roles still missing after alias walk: {missing_slugs:?}"
    );

    // Spot-check the resolution path: Role::Frontend should pick up
    // `frontend-developer.md` from the v1.0.0 tarball (no `frontend.md`
    // exists in that release).
    let frontend = plan
        .changes
        .iter()
        .find(|c| c.role == Role::Frontend)
        .unwrap();
    match &frontend.action {
        BootstrapAction::Create { source_alias, .. } => {
            assert_eq!(
                source_alias, "frontend-developer",
                "Role::Frontend should resolve to frontend-developer.md in v1.0.0"
            );
        }
        other => panic!("expected Create for Role::Frontend, got {other:?}"),
    }
}
