//! M14.4 — green-field bootstrap + attach round-trip.
//!
//! Layout:
//! - `tests/golden/blank/input/`   — empty mock project (just a README.md).
//! - `tests/golden/blank/expected/` — deterministic state after running
//!   `bootstrap` (creates 10 base agent files) followed by `attach`
//!   (injects the marker fence into each).
//!
//! Pass `BLESS=1` to overwrite `expected/` with the freshly produced
//! state — useful when role overlays change. Without that flag the test
//! diffs the live output against what is checked in.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use genasis_overlay::{
    apply, apply_bootstrap, plan_attach, plan_bootstrap, plan_detach, scan, AttachOptions,
    BootstrapOptions, PlannedAction, Role,
};
use genasis_templates::AgentStore;

fn workspace_root() -> PathBuf {
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

/// Mock catalog with deterministic base + overlay bodies for all 10 roles.
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
        // Frontmatter contract: name/description/tools/model/color.
        fs::write(
            base.join(format!("{slug}.md")),
            format!(
                "---\n\
                 name: {slug}\n\
                 description: M14 default {slug} role.\n\
                 tools: Read, Write, Edit\n\
                 model: sonnet\n\
                 color: gray\n\
                 ---\n\
                 # {slug}\n\
                 \n\
                 Default base body for the {slug} role. The marker fence below this\n\
                 line is owned by Genasis (`attach`/`upgrade` rewrites it). Anything\n\
                 outside the fence is yours to edit.\n"
            ),
        )
        .unwrap();
        fs::write(
            overlays_en.join(format!("{slug}.patch.md.tera")),
            format!(
                "## {slug} overlay\nproject: {{{{ project_name | default(value=\"blank-fixture\") }}}}\n"
            ),
        )
        .unwrap();
    }
    let store = AgentStore::from_dir(catalog.path().to_path_buf()).unwrap();
    (catalog, store)
}

fn list_files_sorted(root: &Path) -> Vec<PathBuf> {
    let mut out = BTreeSet::new();
    walk(root, root, &mut out);
    out.into_iter().collect()
}

fn walk(root: &Path, current: &Path, acc: &mut BTreeSet<PathBuf>) {
    for entry in fs::read_dir(current).unwrap() {
        let entry = entry.unwrap();
        let p = entry.path();
        if p.is_dir() {
            walk(root, &p, acc);
        } else {
            acc.insert(p.strip_prefix(root).unwrap().to_path_buf());
        }
    }
}

#[test]
fn blank_bootstrap_then_attach_creates_ten_fenced_agents() {
    let fixture_input = workspace_root().join("tests/golden/blank/input");
    assert!(
        fixture_input.is_dir(),
        "fixture not present at {}",
        fixture_input.display()
    );

    let tmp = tempfile::tempdir().unwrap();
    let project = tmp.path().join("project");
    copy_dir_recursive(&fixture_input, &project);

    // 1. bootstrap — should write 10 base agent files.
    let (_cat, store) = mock_store();
    let opts = BootstrapOptions::default();
    let plan = plan_bootstrap(&project, &opts, &store).unwrap();
    let creates = plan.creates().count();
    assert_eq!(creates, Role::ALL.len(), "expected 10 base files queued");
    let report = apply_bootstrap(&plan).unwrap();
    assert_eq!(report.written.len(), Role::ALL.len());

    // 2. attach — every base file gets a fence injected.
    let scanned = scan(&project).unwrap();
    assert_eq!(scanned.agents.len(), Role::ALL.len());
    let attach_opts = AttachOptions::new("1.0");
    let attach_plan = plan_attach(&scanned.agents, &attach_opts, &store).unwrap();
    let written = apply(&attach_plan).unwrap();
    assert_eq!(written.written.len(), Role::ALL.len());

    for role in Role::ALL {
        let path = project.join(format!(".claude/agents/{}.md", role.slug()));
        let body = read(&path);
        assert!(
            body.contains(&format!(
                "<!-- GENASIS:BEGIN role={}",
                role.slug()
            )),
            "missing fence in {}: {body}",
            path.display()
        );
        assert!(body.contains("<!-- GENASIS:END -->"));
    }

    // 3. detach must cleanly remove the fence (round-trip back to base).
    let scanned2 = scan(&project).unwrap();
    let detach_plan = plan_detach(&scanned2.agents).unwrap();
    let mut removes = 0;
    for c in &detach_plan.changes {
        if matches!(c.action, PlannedAction::Remove) {
            removes += 1;
        }
    }
    assert_eq!(removes, Role::ALL.len(), "every fence should be removable");
    apply(&detach_plan).unwrap();

    for role in Role::ALL {
        let path = project.join(format!(".claude/agents/{}.md", role.slug()));
        let body = read(&path);
        assert!(
            !body.contains("<!-- GENASIS:BEGIN"),
            "fence still present after detach in {}",
            path.display()
        );
    }
}

#[test]
fn blank_expected_snapshot_is_in_sync() {
    let fixture_input = workspace_root().join("tests/golden/blank/input");
    let expected_dir = workspace_root().join("tests/golden/blank/expected");

    let tmp = tempfile::tempdir().unwrap();
    let project = tmp.path().join("project");
    copy_dir_recursive(&fixture_input, &project);

    let (_cat, store) = mock_store();
    let opts = BootstrapOptions::default();
    let plan = plan_bootstrap(&project, &opts, &store).unwrap();
    apply_bootstrap(&plan).unwrap();
    let scanned = scan(&project).unwrap();
    let attach_plan = plan_attach(&scanned.agents, &AttachOptions::new("1.0"), &store).unwrap();
    apply(&attach_plan).unwrap();

    // Strip out backup files (.bak.<ts>) — those are runtime-only and
    // not part of the deterministic expected snapshot.
    let mut live_files: Vec<PathBuf> = list_files_sorted(&project)
        .into_iter()
        .filter(|p| !p.to_string_lossy().contains(".bak."))
        .collect();
    live_files.retain(|p| !p.starts_with("README.md") || p == Path::new("README.md"));

    if std::env::var("BLESS").is_ok() {
        // Refresh the snapshot.
        if expected_dir.exists() {
            fs::remove_dir_all(&expected_dir).unwrap();
        }
        for rel in &live_files {
            let src = project.join(rel);
            let dst = expected_dir.join(rel);
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::copy(&src, &dst).unwrap();
        }
        return;
    }

    let expected_files: Vec<PathBuf> = list_files_sorted(&expected_dir);
    assert!(
        !expected_files.is_empty(),
        "expected/ snapshot is empty — run with BLESS=1 to populate"
    );

    assert_eq!(
        expected_files, live_files,
        "file roster diverged from expected/ — rerun with BLESS=1 if intentional"
    );
    for rel in &expected_files {
        let exp = read(&expected_dir.join(rel));
        let live = read(&project.join(rel));
        assert_eq!(
            exp, live,
            "byte mismatch at {} — rerun with BLESS=1 if intentional",
            rel.display()
        );
    }
}
