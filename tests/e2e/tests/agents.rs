//! M19.2 — README §CLI Reference agents marketplace commands.
//!
//! Covers:
//! 1. `genasis agents list`              — filter by category / search
//! 2. `genasis agents installed`         — what's in this project
//! 3. `genasis agents install <name>`    — single fetch from cache
//! 4. `genasis agents install --preset`  — preset team install
//! 5. `genasis agents remove <name>`     — uninstall
//! 6. `genasis agents browse`            — TUI smoke (errors when index missing)
//!
//! `agents list/browse` reads `./agents/index.json` from the current
//! working directory; `install` copies from the cached catalog at
//! `<cache>/v<version>/base/<name>.md`. We seed a mock catalog with
//! matching files so installs succeed offline.

use std::fs;
use std::path::Path;

use genasis_e2e::{cli_with_catalog, mock_agents_catalog, scratch_project};
use predicates::prelude::*;

const MOCK_INDEX_JSON: &str = r#"{
  "version": "0.0.1-e2e",
  "categories": [
    { "id": "core", "name": "Core", "description": "Essential roles" },
    { "id": "infra", "name": "Infra", "description": "Operations" }
  ],
  "agents": [
    { "name": "frontend", "description": "Frontend developer", "category": "core", "tags": ["ui","react"] },
    { "name": "backend",  "description": "Backend developer",  "category": "core", "tags": ["api"] },
    { "name": "devops",   "description": "DevOps engineer",    "category": "infra", "tags": ["k8s"] }
  ],
  "presets": {
    "web-app": {
      "description": "Web application team",
      "agents": ["frontend", "backend"]
    }
  }
}"#;

fn seed_index(project: &Path) {
    let agents_dir = project.join("agents");
    fs::create_dir_all(&agents_dir).unwrap();
    fs::write(agents_dir.join("index.json"), MOCK_INDEX_JSON).unwrap();
}

#[test]
fn agents_list_renders_all_three_agents() {
    let (_g, project) = scratch_project();
    seed_index(&project);
    let catalog = mock_agents_catalog();

    cli_with_catalog(&catalog)
        .current_dir(&project)
        .args(["agents", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("frontend"))
        .stdout(predicate::str::contains("backend"))
        .stdout(predicate::str::contains("devops"));
}

#[test]
fn agents_list_filters_by_category() {
    let (_g, project) = scratch_project();
    seed_index(&project);
    let catalog = mock_agents_catalog();

    cli_with_catalog(&catalog)
        .current_dir(&project)
        .args(["agents", "list", "--category", "core"])
        .assert()
        .success()
        .stdout(predicate::str::contains("frontend"))
        .stdout(predicate::str::contains("backend"))
        .stdout(predicate::str::contains("devops").not());
}

#[test]
fn agents_list_filters_by_search() {
    let (_g, project) = scratch_project();
    seed_index(&project);
    let catalog = mock_agents_catalog();

    cli_with_catalog(&catalog)
        .current_dir(&project)
        .args(["agents", "list", "--search", "k8s"])
        .assert()
        .success()
        .stdout(predicate::str::contains("devops"))
        .stdout(predicate::str::contains("frontend").not());
}

#[test]
fn agents_installed_reports_empty_when_no_dir() {
    let (_g, project) = scratch_project();
    let catalog = mock_agents_catalog();

    cli_with_catalog(&catalog)
        .current_dir(&project)
        .args(["agents", "installed"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No agents installed"));
}

#[test]
fn agents_install_copies_from_catalog_then_installed_lists_it() {
    let (_g, project) = scratch_project();
    seed_index(&project);
    let catalog = mock_agents_catalog();

    cli_with_catalog(&catalog)
        .current_dir(&project)
        .args(["agents", "install", "frontend"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Installed frontend"));

    assert!(project.join(".claude/agents/frontend.md").is_file());

    cli_with_catalog(&catalog)
        .current_dir(&project)
        .args(["agents", "installed"])
        .assert()
        .success()
        .stdout(predicate::str::contains("frontend"))
        .stdout(predicate::str::contains("1 agent"));
}

#[test]
fn agents_install_preset_installs_all_member_agents() {
    let (_g, project) = scratch_project();
    seed_index(&project);
    let catalog = mock_agents_catalog();

    cli_with_catalog(&catalog)
        .current_dir(&project)
        .args(["agents", "install", "--preset", "web-app"])
        .assert()
        .success()
        .stdout(predicate::str::contains("frontend"))
        .stdout(predicate::str::contains("backend"));

    assert!(project.join(".claude/agents/frontend.md").is_file());
    assert!(project.join(".claude/agents/backend.md").is_file());
}

#[test]
fn agents_remove_deletes_the_file() {
    let (_g, project) = scratch_project();
    seed_index(&project);
    let catalog = mock_agents_catalog();

    cli_with_catalog(&catalog)
        .current_dir(&project)
        .args(["agents", "install", "frontend"])
        .assert()
        .success();
    assert!(project.join(".claude/agents/frontend.md").is_file());

    cli_with_catalog(&catalog)
        .current_dir(&project)
        .args(["agents", "remove", "frontend"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed frontend"));
    assert!(!project.join(".claude/agents/frontend.md").exists());
}

#[test]
fn agents_remove_unknown_errors() {
    let (_g, project) = scratch_project();
    let catalog = mock_agents_catalog();

    cli_with_catalog(&catalog)
        .current_dir(&project)
        .args(["agents", "remove", "ghost"])
        .assert()
        .failure();
}

#[test]
fn agents_browse_without_index_errors_cleanly() {
    // Smoke for the TUI entry point: when the index isn't present in
    // the cwd, browse must fail with a deterministic message instead
    // of trying to read stdin.
    let (_g, project) = scratch_project();
    let catalog = mock_agents_catalog();

    cli_with_catalog(&catalog)
        .current_dir(&project)
        .args(["agents", "browse"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("agents/index.json"));
}

// Note: `agents status` is intentionally omitted — it is a debug aid not
// listed in README §CLI Reference, and the current implementation calls
// reqwest::blocking inside the tokio runtime which conflicts with
// `#[tokio::main]`. Tracked as a follow-up; not on the v0.1.0 critical
// path.
