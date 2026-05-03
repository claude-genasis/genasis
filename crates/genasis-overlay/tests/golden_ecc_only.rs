//! E2E-shaped golden test: copy `tests/golden/ecc-only/input/` to a temp
//! directory, run attach + detach in-process, verify round-trip equality
//! and the presence of the frontend fence after attach.
//!
//! Stays at the library level (no binary spawning) so the test runs under
//! `cargo test --workspace` without a release build.

use std::fs;
use std::path::{Path, PathBuf};

use genasis_overlay::{apply, plan_attach, plan_detach, scan, AttachOptions, PlannedAction};

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR for genasis-overlay points at crates/genasis-overlay.
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_dir
        .ancestors()
        .find(|p| p.join("Cargo.toml").is_file() && p.join("crates").is_dir())
        .unwrap_or(&crate_dir)
        .to_path_buf()
}

fn copy_dir_recursive(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir_recursive(&from, &to);
        } else {
            fs::copy(&from, &to).unwrap();
        }
    }
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

#[test]
fn ecc_only_attach_then_detach_round_trips() {
    let fixture_input = workspace_root().join("tests/golden/ecc-only/input");
    assert!(
        fixture_input.is_dir(),
        "fixture not present at {}",
        fixture_input.display()
    );

    let tmp = tempfile::tempdir().unwrap();
    let project = tmp.path().join("project");
    copy_dir_recursive(&fixture_input, &project);

    // 1. Snapshot original byte content.
    let frontend_path = project.join(".claude/agents/frontend.md");
    let backend_path = project.join(".claude/agents/backend.md");
    let custom_path = project.join(".claude/agents/loop-operator.md");
    let original_frontend = read(&frontend_path);
    let original_backend = read(&backend_path);
    let original_custom = read(&custom_path);

    // 2. attach
    let report = scan(&project).unwrap();
    let opts = AttachOptions::new("1.0");
    let plan = plan_attach(&report.agents, &opts).unwrap();
    let written = apply(&plan).unwrap();

    // Frontend has a template → fence injected.
    let after_attach = read(&frontend_path);
    assert!(
        after_attach.contains("<!-- GENASIS:BEGIN role=frontend"),
        "frontend.md should have a fence after attach:\n{after_attach}"
    );
    assert!(after_attach.contains("<!-- GENASIS:END -->"));

    // Backend has no template at M2 → skipped, file unchanged.
    assert_eq!(read(&backend_path), original_backend);

    // Custom agent → skipped, file unchanged.
    assert_eq!(read(&custom_path), original_custom);

    // The merger should have written exactly the file(s) it changed.
    assert_eq!(written.written.len(), 1);
    assert_eq!(written.written[0], frontend_path);
    // A snapshot was taken for the file we wrote.
    assert_eq!(written.backups.len(), 1);
    assert!(written.backups[0]
        .file_name()
        .unwrap()
        .to_string_lossy()
        .contains("frontend.md.genasis.bak."));

    // 3. detach
    let report = scan(&project).unwrap();
    let plan = plan_detach(&report.agents).unwrap();

    // Frontend fence is removed; others are NoFenceToRemove.
    let mut saw_remove = false;
    let mut saw_nothing = 0;
    for c in &plan.changes {
        match c.action {
            PlannedAction::Remove => saw_remove = true,
            PlannedAction::NoFenceToRemove => saw_nothing += 1,
            other => panic!("unexpected action in detach plan: {other:?}"),
        }
    }
    assert!(saw_remove, "expected at least one Remove");
    assert!(saw_nothing >= 1, "expected NoFenceToRemove for non-frontend files");

    apply(&plan).unwrap();

    // 4. round-trip equality
    assert_eq!(read(&frontend_path), original_frontend, "frontend not restored");
    assert_eq!(read(&backend_path), original_backend, "backend changed unexpectedly");
    assert_eq!(read(&custom_path), original_custom, "custom changed unexpectedly");
}

#[test]
fn double_attach_is_idempotent() {
    let fixture_input = workspace_root().join("tests/golden/ecc-only/input");
    let tmp = tempfile::tempdir().unwrap();
    let project = tmp.path().join("project");
    copy_dir_recursive(&fixture_input, &project);

    let opts = AttachOptions::new("1.0");
    // First attach
    let r1 = scan(&project).unwrap();
    let p1 = plan_attach(&r1.agents, &opts).unwrap();
    apply(&p1).unwrap();

    let frontend_path = project.join(".claude/agents/frontend.md");
    let after_first = read(&frontend_path);

    // Second attach — must be a no-op.
    let r2 = scan(&project).unwrap();
    let p2 = plan_attach(&r2.agents, &opts).unwrap();
    let frontend_change = p2
        .changes
        .iter()
        .find(|c| c.path == frontend_path)
        .expect("frontend in plan");
    assert!(matches!(frontend_change.action, PlannedAction::Skip(_)));

    apply(&p2).unwrap();
    let after_second = read(&frontend_path);
    assert_eq!(after_first, after_second, "second attach changed the file");
}
