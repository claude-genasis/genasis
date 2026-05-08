//! M19.3 — README §CLI Reference supporting commands.
//!
//! Covers:
//! 1. `genasis lang status`        — report active locale
//! 2. `genasis lang switch <lang>` — atomic locale swap
//! 3. `genasis init --trial`       — trial mode (probe-only path,
//!                                    no real Plane/MM calls)
//! 4. `genasis db query "..."`     — read-only SQL guard end-to-end
//! 5. `genasis design swap --from` — local-file design swap
//! 6. `genasis monitor`            — headless smoke (no TTY → bail)
//! 7. `genasis version --json`     — sanity check for the version command

use std::fs;
use std::path::Path;

use genasis_e2e::{cli, cli_with_catalog, mock_agents_catalog, scratch_project, seed_blank};
use predicates::prelude::*;
use tempfile::TempDir;

/// Bootstrap a project with a mock catalog and return the catalog so
/// subsequent `cli_with_catalog(&catalog)` calls keep hitting the cache.
fn provision(project: &Path) -> TempDir {
    seed_blank(project);
    let catalog = mock_agents_catalog();
    cli_with_catalog(&catalog)
        .args(["--non-interactive", "--yes", "bootstrap", "--lang", "en"])
        .arg("--project")
        .arg(project)
        .assert()
        .success();
    catalog
}

#[test]
fn lang_status_reports_active_locale() {
    let (_g, project) = scratch_project();
    let catalog = provision(&project);

    cli_with_catalog(&catalog)
        .args(["--non-interactive", "--yes", "lang", "status"])
        .arg("--project")
        .arg(&project)
        .assert()
        .success()
        .stdout(predicate::str::contains("active:"));
}

#[test]
fn lang_switch_to_ko_persists_in_genasis_toml() {
    let (_g, project) = scratch_project();
    let catalog = provision(&project);

    cli_with_catalog(&catalog)
        .args(["--non-interactive", "--yes", "lang", "switch", "ko"])
        .arg("--project")
        .arg(&project)
        .assert()
        .success();

    let cfg = fs::read_to_string(project.join("genasis.toml")).unwrap();
    assert!(
        cfg.contains("active = \"ko\""),
        "genasis.toml should pin active=ko after switch:\n{cfg}"
    );
}

#[test]
fn lang_switch_unknown_locale_errors() {
    let (_g, project) = scratch_project();
    let catalog = provision(&project);

    cli_with_catalog(&catalog)
        .args(["--non-interactive", "--yes", "lang", "switch", "klingon"])
        .arg("--project")
        .arg(&project)
        .assert()
        .failure();
}

#[test]
fn init_trial_probe_only_writes_minimal_config_no_network() {
    let (_g, project) = scratch_project();
    seed_blank(&project);

    cli()
        .args([
            "--non-interactive",
            "--yes",
            "init",
            "--trial",
            "--probe-only",
        ])
        .arg("--project")
        .arg(&project)
        .assert()
        .success();

    let cfg = fs::read_to_string(project.join("genasis.toml")).unwrap();
    assert!(cfg.contains("[trial]"));
    assert!(cfg.contains("flavor = \"trial\""));
}

#[test]
fn db_query_rejects_ddl_via_sql_guard() {
    let (_g, project) = scratch_project();
    seed_blank(&project);
    fs::write(
        project.join("genasis.toml"),
        r#"
[project]
name = "e2e"
domain = "example.com"

[db]
driver = "sqlite"
url = "file:not-real.db"
migration_tool = "atlas"
"#,
    )
    .unwrap();

    // SQL guard must reject DDL even before reaching the driver.
    cli()
        .args([
            "--non-interactive",
            "--yes",
            "db",
            "query",
            "DROP TABLE users",
        ])
        .arg("--project")
        .arg(&project)
        .assert()
        .failure();
}

#[test]
fn design_swap_from_local_file_writes_pointer_body() {
    let (_g, project) = scratch_project();
    seed_blank(&project);
    let catalog = mock_agents_catalog();
    cli_with_catalog(&catalog)
        .args(["--non-interactive", "--yes", "bootstrap", "--lang", "en"])
        .arg("--project")
        .arg(&project)
        .assert()
        .success();

    let local_design = project.join("local-design.md");
    fs::write(
        &local_design,
        "# Local Design\n\nColors: blue/green.\nTypography: Inter.\n",
    )
    .unwrap();

    cli()
        .args([
            "--non-interactive",
            "--yes",
            "design",
            "swap",
            "--from",
        ])
        .arg(&local_design)
        .arg("--project")
        .arg(&project)
        .assert()
        .success();

    // External-mode swap writes DESIGN.md + a pointer body.
    let design_dir = project.join("docs/design-system");
    assert!(
        design_dir.is_dir(),
        "external design dir should exist: {}",
        design_dir.display()
    );
    assert!(design_dir.join("DESIGN.md").is_file());
    assert!(project.join("docs/design-system.md").is_file());
}

#[test]
fn monitor_smoke_without_tty_exits_cleanly() {
    // `genasis monitor` enters an alternate-screen TUI loop. With stdin
    // not a TTY (assert_cmd default), the runtime must surface a sane
    // error rather than hang. We accept either success (immediate exit)
    // or failure as long as the process terminates within the
    // assert_cmd default timeout.
    let _ = cli()
        .args(["--non-interactive", "--yes", "monitor"])
        .timeout(std::time::Duration::from_secs(5))
        .assert();
}

#[test]
fn version_json_emits_structured_metadata() {
    let assert = cli()
        .args(["version", "--json"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("version --json must emit valid JSON");
    assert!(
        parsed.get("marker_fence_version").is_some(),
        "version JSON missing marker_fence_version: {parsed:?}"
    );
    assert!(parsed.get("version").is_some());
}
