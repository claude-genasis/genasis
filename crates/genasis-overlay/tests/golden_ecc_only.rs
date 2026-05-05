//! E2E-shaped golden test: copy `tests/golden/ecc-only/input/` to a temp
//! directory, run attach + detach in-process, verify round-trip equality
//! and the presence of the frontend fence after attach.
//!
//! Stays at the library level (no binary spawning) so the test runs under
//! `cargo test --workspace` without a release build.

use std::fs;
use std::path::{Path, PathBuf};

use genasis_overlay::{apply, plan_attach, plan_detach, scan, AttachOptions, PlannedAction, Role};
use genasis_templates::AgentStore;

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

/// Create a mock AgentStore with overlay templates for all 10 roles.
fn mock_store() -> (tempfile::TempDir, AgentStore) {
    let catalog = tempfile::tempdir().unwrap();
    let base = catalog.path().join("base");
    let overlays_en = catalog.path().join("overlays/en");
    fs::create_dir_all(&base).unwrap();
    fs::create_dir_all(&overlays_en).unwrap();
    fs::write(
        catalog.path().join("manifest.json"),
        r#"{"version":"0.0.1-test","roles":[]}"#,
    )
    .unwrap();
    for role in Role::ALL {
        let slug = role.slug();
        fs::write(
            base.join(format!("{slug}.md")),
            format!("---\nname: {slug}\ndescription: test\ntools: Read\nmodel: sonnet\ncolor: gray\n---\n# {slug}\n"),
        )
        .unwrap();
        fs::write(
            overlays_en.join(format!("{slug}.patch.md.tera")),
            format!("## overlay for {slug}\nproject: {{{{ project_name | default(value=\"test\") }}}}\n"),
        )
        .unwrap();
    }
    let store = AgentStore::from_dir(catalog.path().to_path_buf()).unwrap();
    (catalog, store)
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
    let (_cat, store) = mock_store();
    let plan = plan_attach(&report.agents, &opts, &store).unwrap();
    let written = apply(&plan).unwrap();

    // Frontend has a template → fence injected.
    let after_attach = read(&frontend_path);
    assert!(
        after_attach.contains("<!-- GENASIS:BEGIN role=frontend"),
        "frontend.md should have a fence after attach:\n{after_attach}"
    );
    assert!(after_attach.contains("<!-- GENASIS:END -->"));

    // Backend has a template too (M6) → fence injected.
    let after_backend = read(&backend_path);
    assert!(
        after_backend.contains("<!-- GENASIS:BEGIN role=backend"),
        "backend.md should have a fence after attach"
    );

    // Custom agent → no template, skipped, file unchanged.
    assert_eq!(read(&custom_path), original_custom);

    // The merger writes one file per Known role with a template (frontend +
    // backend in this fixture).
    assert_eq!(written.written.len(), 2);
    assert!(written.written.contains(&frontend_path));
    assert!(written.written.contains(&backend_path));
    let _ = original_backend;
    // A snapshot was taken for each Known role we wrote.
    assert_eq!(written.backups.len(), 2);

    // 3. detach
    let report = scan(&project).unwrap();
    let plan = plan_detach(&report.agents).unwrap();

    // Frontend + backend fences removed; custom is NoFenceToRemove.
    let mut remove_count = 0;
    let mut saw_nothing = 0;
    for c in &plan.changes {
        match &c.action {
            PlannedAction::Remove => remove_count += 1,
            PlannedAction::NoFenceToRemove => saw_nothing += 1,
            other => panic!("unexpected action in detach plan: {other:?}"),
        }
    }
    assert_eq!(remove_count, 2, "expected exactly two Remove actions");
    assert!(
        saw_nothing >= 1,
        "expected NoFenceToRemove for the custom agent"
    );

    apply(&plan).unwrap();

    // 4. round-trip equality
    assert_eq!(
        read(&frontend_path),
        original_frontend,
        "frontend not restored"
    );
    assert_eq!(
        read(&backend_path),
        original_backend,
        "backend not restored"
    );
    assert_eq!(
        read(&custom_path),
        original_custom,
        "custom changed unexpectedly"
    );
}

#[test]
fn double_attach_is_idempotent() {
    let fixture_input = workspace_root().join("tests/golden/ecc-only/input");
    let tmp = tempfile::tempdir().unwrap();
    let project = tmp.path().join("project");
    copy_dir_recursive(&fixture_input, &project);

    let opts = AttachOptions::new("1.0");
    let (_cat, store) = mock_store();
    // First attach
    let r1 = scan(&project).unwrap();
    let p1 = plan_attach(&r1.agents, &opts, &store).unwrap();
    apply(&p1).unwrap();

    let frontend_path = project.join(".claude/agents/frontend.md");
    let after_first = read(&frontend_path);

    // Second attach — must be a no-op.
    let r2 = scan(&project).unwrap();
    let p2 = plan_attach(&r2.agents, &opts, &store).unwrap();
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
