//! M19.4 — README §CLI Reference debug commands.
//!
//! Covers `genasis debug {status,log,collect,reset}` against a freshly
//! provisioned project. The submit subcommand lands under M16 (PR-only
//! channel per ADR-012 §8) and is exercised separately.

use std::fs;

use genasis_e2e::{cli_with_catalog, mock_agents_catalog, scratch_project, seed_blank};
use predicates::prelude::*;

#[test]
fn debug_status_reports_zero_drift_after_fresh_attach() {
    let (_g, project) = scratch_project();
    seed_blank(&project);
    let catalog = mock_agents_catalog();
    cli_with_catalog(&catalog)
        .args(["--non-interactive", "--yes", "bootstrap", "--lang", "en"])
        .arg("--project")
        .arg(&project)
        .assert()
        .success();

    cli_with_catalog(&catalog)
        .args(["--non-interactive", "--yes", "debug", "status"])
        .arg("--project")
        .arg(&project)
        .assert()
        .success()
        .stdout(predicate::str::contains("drift: 0 files"));
}

#[test]
fn debug_status_flags_modified_files() {
    let (_g, project) = scratch_project();
    seed_blank(&project);
    let catalog = mock_agents_catalog();
    cli_with_catalog(&catalog)
        .args(["--non-interactive", "--yes", "bootstrap", "--lang", "en"])
        .arg("--project")
        .arg(&project)
        .assert()
        .success();

    let frontend = project.join(".claude/agents/frontend.md");
    let body = fs::read_to_string(&frontend).unwrap();
    fs::write(&frontend, format!("{body}\n# user edit\n")).unwrap();

    cli_with_catalog(&catalog)
        .args(["--non-interactive", "--yes", "debug", "status"])
        .arg("--project")
        .arg(&project)
        .assert()
        .success()
        .stdout(predicate::str::contains("modified"))
        .stdout(predicate::str::contains("frontend.md"));
}

#[test]
fn debug_collect_to_stdout_emits_anonymised_json() {
    let (_g, project) = scratch_project();
    seed_blank(&project);
    let catalog = mock_agents_catalog();
    cli_with_catalog(&catalog)
        .args(["--non-interactive", "--yes", "bootstrap", "--lang", "en"])
        .arg("--project")
        .arg(&project)
        .assert()
        .success();

    // Inject a secret-shaped line into a managed file so we can verify
    // that strip_secrets() redacted it.
    let frontend = project.join(".claude/agents/frontend.md");
    let body = fs::read_to_string(&frontend).unwrap();
    fs::write(
        &frontend,
        format!("{body}\nMM_ADMIN_TOKEN=should-be-redacted\n# user note\n"),
    )
    .unwrap();

    let assert = cli_with_catalog(&catalog)
        .args(["--non-interactive", "--yes", "debug", "collect", "--stdout"])
        .arg("--project")
        .arg(&project)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("collect must emit valid JSON");
    assert_eq!(parsed["schema_version"], "1");
    assert!(parsed["project_hash"].as_str().unwrap().len() == 16);
    assert!(parsed["entries"].as_array().unwrap().len() >= 1);
    assert!(
        !stdout.contains("should-be-redacted"),
        "secrets must not appear in collect output"
    );
}

#[test]
fn debug_reset_clears_pending_drift() {
    let (_g, project) = scratch_project();
    seed_blank(&project);
    let catalog = mock_agents_catalog();
    cli_with_catalog(&catalog)
        .args(["--non-interactive", "--yes", "bootstrap", "--lang", "en"])
        .arg("--project")
        .arg(&project)
        .assert()
        .success();

    let frontend = project.join(".claude/agents/frontend.md");
    let body = fs::read_to_string(&frontend).unwrap();
    fs::write(&frontend, format!("{body}\n# edit\n")).unwrap();

    cli_with_catalog(&catalog)
        .args(["--non-interactive", "--yes", "debug", "reset"])
        .arg("--project")
        .arg(&project)
        .assert()
        .success()
        .stdout(predicate::str::contains("0 drift"));

    cli_with_catalog(&catalog)
        .args(["--non-interactive", "--yes", "debug", "status"])
        .arg("--project")
        .arg(&project)
        .assert()
        .success()
        .stdout(predicate::str::contains("drift: 0 files"));
}

#[test]
fn debug_log_reports_absent_when_no_log_file() {
    let (_g, project) = scratch_project();
    seed_blank(&project);

    cli_with_catalog(&mock_agents_catalog())
        .args(["--non-interactive", "--yes", "debug", "log"])
        .args(["--project"])
        .arg(&project)
        .assert()
        .success()
        .stdout(predicate::str::contains("no drift-log"));
}
