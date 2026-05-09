//! Shared helpers for the README-parity E2E suite (M19).
//!
//! Each spec file under `tests/` exercises one command from
//! `README.md §CLI Reference`. The default backend for any command
//! touching Plane / Mattermost is the `trial` flavor (the trial-app
//! HTTP forwarder); M20's nightly workflow re-runs the same suite
//! against real Plane + Mattermost via `servers/docker-compose.yml`.

#![allow(dead_code)]

use std::path::{Path, PathBuf};

use assert_cmd::Command;
use tempfile::TempDir;

pub fn workspace_root() -> PathBuf {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_dir
        .ancestors()
        .find(|p| p.join("Cargo.toml").is_file() && p.join("crates").is_dir())
        .map(|p| p.to_path_buf())
        .unwrap_or(crate_dir)
}

/// The release-or-debug `genasis` binary as built by Cargo.
pub fn cli() -> Command {
    Command::cargo_bin("genasis").expect("cargo bin `genasis` is built")
}

/// A scratch project directory that lives only as long as the test.
pub fn scratch_project() -> (TempDir, PathBuf) {
    let tmp = TempDir::new().expect("tempdir");
    let path = tmp.path().canonicalize().expect("canonicalize");
    (tmp, path)
}

/// Seed a project root with a minimal `README.md` so it looks like a
/// real repo.
pub fn seed_blank(project: &Path) {
    std::fs::write(project.join("README.md"), "# scratch project\n").unwrap();
}

/// Seed an arbitrary `genasis.toml` for commands that require a
/// config to load.
pub fn seed_genasis_toml(project: &Path, body: &str) {
    std::fs::write(project.join("genasis.toml"), body).unwrap();
}

/// Materialise a deterministic mock agents catalog under
/// `<dir>/v{CATALOG_VERSION}/` so commands that load `AgentStore` can
/// run without reaching out to GitHub Releases. Returns the dir guard
/// so it lives for the duration of the test.
///
/// Mirrors the structure produced by `agents-pool/scripts/publish.sh`:
/// `manifest.json`, `base/<role>.md` (10 roles), `overlays/{en,ko}/<role>.patch.md.tera`.
pub const CATALOG_VERSION: &str = "0.0.1-e2e";
const CANONICAL_ROLES: &[&str] = &[
    "pm",
    "planner",
    "architect",
    "frontend",
    "backend",
    "qa",
    "designer",
    "security",
    "devops",
    "code-reviewer",
];

pub fn mock_agents_catalog() -> TempDir {
    let dir = TempDir::new().expect("tempdir for catalog");
    let root = dir.path().join(format!("v{CATALOG_VERSION}"));
    let base = root.join("base");
    let overlays_en = root.join("overlays/en");
    let overlays_ko = root.join("overlays/ko");
    std::fs::create_dir_all(&base).unwrap();
    std::fs::create_dir_all(&overlays_en).unwrap();
    std::fs::create_dir_all(&overlays_ko).unwrap();
    std::fs::write(
        root.join("manifest.json"),
        format!(r#"{{"version":"{CATALOG_VERSION}","roles":[]}}"#),
    )
    .unwrap();
    for role in CANONICAL_ROLES {
        std::fs::write(
            base.join(format!("{role}.md")),
            format!(
                "---\n\
                 name: {role}\n\
                 description: e2e mock {role}\n\
                 tools: Read, Write, Edit\n\
                 model: sonnet\n\
                 color: gray\n\
                 ---\n\
                 # {role}\n\nMock body for E2E tests.\n"
            ),
        )
        .unwrap();
        std::fs::write(
            overlays_en.join(format!("{role}.patch.md.tera")),
            format!("## {role} overlay (en)\nproject: {{{{ project_name | default(value=\"e2e\") }}}}\n"),
        )
        .unwrap();
        std::fs::write(
            overlays_ko.join(format!("{role}.patch.md.tera")),
            format!("## {role} 오버레이 (ko)\n프로젝트: {{{{ project_name | default(value=\"e2e\") }}}}\n"),
        )
        .unwrap();
    }
    dir
}

/// Wrap a [`Command`] with the env vars that point at a mock catalog,
/// keeping the catalog tempdir alive until the call returns.
pub fn cli_with_catalog(catalog: &TempDir) -> Command {
    let mut c = cli();
    c.env("GENASIS_AGENTS_VERSION", CATALOG_VERSION)
        .env("GENASIS_AGENTS_CACHE_DIR", catalog.path())
        .env("GENASIS_AGENTS_AUTO_CHECK", "false");
    c
}

/// Run `genasis bootstrap` against `project` against a mock catalog.
pub fn provision_default_project(project: &Path) -> TempDir {
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
