//! M19.1 — README §CLI Reference lifecycle commands.
//!
//! Covers, in dependency order:
//! 1. `genasis bootstrap` — green-field scaffold (M14)
//! 2. `genasis attach`     — overlay onto existing agents
//! 3. `genasis detach`     — fully reversible
//! 4. `genasis doctor`     — environment + bootstrap + i18n diagnostics
//! 5. `genasis upgrade`    — fence-version bump (idempotent on no-op)
//! 6. `genasis example`    — sample artifact emission
//!
//! Catalog dependency: each test seeds a deterministic mock agents
//! catalog under `$GENASIS_AGENTS_CACHE_DIR` so commands that load
//! `AgentStore` never reach GitHub Releases.

use genasis_e2e::{
    cli, cli_with_catalog, mock_agents_catalog, scratch_project, seed_blank,
};
use predicates::prelude::*;

#[test]
fn bootstrap_scaffolds_ten_agents_and_attaches_overlay() {
    let (_g, project) = scratch_project();
    seed_blank(&project);
    let catalog = mock_agents_catalog();

    cli_with_catalog(&catalog)
        .args(["--non-interactive", "--yes", "bootstrap", "--lang", "en"])
        .arg("--project")
        .arg(&project)
        .assert()
        .success()
        .stdout(predicate::str::contains("default agents created"));

    let agents_dir = project.join(".claude/agents");
    let md_files: Vec<_> = std::fs::read_dir(&agents_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|x| x.to_str())
                .map(|s| s == "md")
                .unwrap_or(false)
        })
        .collect();
    assert_eq!(
        md_files.len(),
        10,
        "expected 10 base agent .md files, got {}",
        md_files.len()
    );

    for entry in md_files {
        let path = entry.path();
        let body = std::fs::read_to_string(&path).unwrap();
        assert!(
            body.contains("<!-- GENASIS:BEGIN role="),
            "missing fence in {}",
            path.display()
        );
    }
}

#[test]
fn attach_then_detach_round_trips_byte_identical() {
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
    let after_attach = std::fs::read_to_string(&frontend).unwrap();
    assert!(after_attach.contains("<!-- GENASIS:BEGIN role=frontend"));

    cli_with_catalog(&catalog)
        .args(["--non-interactive", "--yes", "detach"])
        .arg("--project")
        .arg(&project)
        .assert()
        .success();

    let after_detach = std::fs::read_to_string(&frontend).unwrap();
    assert!(
        !after_detach.contains("<!-- GENASIS:BEGIN"),
        "fence still present after detach"
    );

    cli_with_catalog(&catalog)
        .args(["--non-interactive", "--yes", "attach"])
        .arg("--project")
        .arg(&project)
        .assert()
        .success();

    let after_reattach = std::fs::read_to_string(&frontend).unwrap();
    assert_eq!(
        after_attach, after_reattach,
        "re-attach was not idempotent"
    );
}

#[test]
fn attach_on_blank_project_emits_bootstrap_hint() {
    let (_g, project) = scratch_project();
    seed_blank(&project);
    std::fs::create_dir_all(project.join(".claude/agents")).unwrap();
    let catalog = mock_agents_catalog();

    cli_with_catalog(&catalog)
        .args(["--non-interactive", "--yes", "attach"])
        .arg("--project")
        .arg(&project)
        .assert()
        .stderr(predicate::str::contains("genasis bootstrap"));
}

#[test]
fn doctor_runs_on_a_provisioned_project() {
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
        .args(["--non-interactive", "--yes", "doctor"])
        .arg("--project")
        .arg(&project)
        .assert()
        .success()
        .stdout(predicate::str::contains("[bootstrap]"))
        .stdout(predicate::str::contains("agent files"));
}

#[test]
fn upgrade_is_idempotent_on_freshly_attached_project() {
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
    let before = std::fs::read_to_string(&frontend).unwrap();

    cli_with_catalog(&catalog)
        .args([
            "--non-interactive",
            "--yes",
            "upgrade",
            "--fence-version",
            "1.0",
        ])
        .arg("--project")
        .arg(&project)
        .assert()
        .success();

    let after = std::fs::read_to_string(&frontend).unwrap();
    assert_eq!(
        before, after,
        "upgrade to the same fence version should be a no-op"
    );
}

#[test]
fn example_emits_three_canonical_documents() {
    for (kind, filename) in [
        ("prd", "PRD.md"),
        ("design", "design-system.md"),
        ("prd2", "PRD2.md"),
    ] {
        let (_g, project) = scratch_project();
        cli()
            .args(["example", kind])
            .arg("--project")
            .arg(&project)
            .assert()
            .success();

        let target = project.join(filename);
        assert!(
            target.exists(),
            "expected {} after `genasis example {}`",
            target.display(),
            kind
        );
        assert!(
            std::fs::read_to_string(&target).unwrap().len() > 100,
            "{} is suspiciously short",
            filename
        );
    }
}
