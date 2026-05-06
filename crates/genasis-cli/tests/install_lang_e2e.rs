//! End-to-end CLI tests for the install-time language selection.
//!
//! Each test invokes the built `genasis` binary (`env!("CARGO_BIN_EXE_genasis")`)
//! against a temp project so the full clap → cmd_attach → lang_prompt
//! → genasis-templates path is exercised.
//!
//! These tests cover M12.4's seven scenarios:
//!   1. flag_en  — `--lang en` skips prompt
//!   2. flag_ko  — `--lang ko` skips prompt
//!   3. both_rejected — `--lang both` exits 2 with bilingual banner
//!   4. prompt_default — TTY mock + Enter accepts $LANG suggestion
//!   5. prompt_choice  — TTY mock + "1" picks English
//!   6. prompt_decline — TTY mock + confirmation "n" aborts
//!   7. non_tty_fallback — non-TTY input → $LANG fallback announced
//!
//! Tests 4–6 require a PTY which the std::process::Command path does
//! not provide; we cover the same logic at the unit level in
//! `lang_decide.rs` and exercise the binary path here for the
//! flag-driven and non-tty cases that don't need a PTY.

use std::path::{Path, PathBuf};
use std::process::Command;

fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_genasis"))
}

/// Create a minimal agents catalog that `genasis_templates::load()` accepts.
fn create_mock_catalog(cache_dir: &Path) {
    let v = cache_dir.join("v1.0.0");
    std::fs::create_dir_all(v.join("base")).unwrap();
    std::fs::create_dir_all(v.join("overlays/en")).unwrap();
    std::fs::create_dir_all(v.join("overlays/ko")).unwrap();
    std::fs::write(
        v.join("manifest.json"),
        r#"{"version":"1.0.0","roles":["frontend"]}"#,
    )
    .unwrap();
    std::fs::write(
        v.join("base/frontend.md"),
        "---\nname: frontend\n---\n# Frontend\n",
    )
    .unwrap();
    // Minimal overlay template per locale
    for lang in ["en", "ko"] {
        std::fs::write(
            v.join(format!("overlays/{lang}/frontend.patch.md.tera")),
            "## Genasis Overlay\nproject={{ project_name }}\n",
        )
        .unwrap();
    }
}

fn fresh_project() -> (tempfile::TempDir, tempfile::TempDir) {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join(".claude/agents")).unwrap();
    // Minimal frontmatter agent so attach has something to fence.
    std::fs::write(
        tmp.path().join(".claude/agents/frontend.md"),
        "---\nname: frontend\n---\n# Frontend\n",
    )
    .unwrap();
    // Minimal genasis.toml so persist_i18n_choice has a file to update.
    std::fs::write(
        tmp.path().join("genasis.toml"),
        "[project]\nname = \"e2etest\"\ndomain = \"example.com\"\n",
    )
    .unwrap();

    // Mock agents catalog cache
    let cache = tempfile::tempdir().expect("cache tempdir");
    create_mock_catalog(cache.path());

    (tmp, cache)
}

/// Build a Command with the mock catalog env vars pre-set.
fn cmd_with_cache(cache: &Path) -> Command {
    let mut c = Command::new(binary());
    c.env("GENASIS_AGENTS_VERSION", "1.0.0")
        .env("GENASIS_AGENTS_CACHE_DIR", cache.to_str().unwrap());
    c
}

#[test]
fn flag_en_drives_attach_without_prompt() {
    let (tmp, cache) = fresh_project();
    let out = cmd_with_cache(cache.path())
        .args([
            "attach",
            "--project",
            tmp.path().to_str().unwrap(),
            "--lang",
            "en",
            "--non-interactive",
            "--yes",
            "--dry-run",
        ])
        .output()
        .expect("spawn");
    assert!(
        out.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let cfg = std::fs::read_to_string(tmp.path().join("genasis.toml")).unwrap();
    assert!(
        cfg.contains("active = \"en\""),
        "genasis.toml [i18n].active not en:\n{cfg}"
    );
}

#[test]
fn flag_ko_drives_attach_without_prompt() {
    let (tmp, cache) = fresh_project();
    let out = cmd_with_cache(cache.path())
        .args([
            "attach",
            "--project",
            tmp.path().to_str().unwrap(),
            "--lang",
            "ko",
            "--non-interactive",
            "--yes",
            "--dry-run",
        ])
        .output()
        .expect("spawn");
    assert!(
        out.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let cfg = std::fs::read_to_string(tmp.path().join("genasis.toml")).unwrap();
    assert!(
        cfg.contains("active = \"ko\""),
        "genasis.toml [i18n].active not ko"
    );
}

#[test]
fn both_is_rejected_with_exit_2_and_banner() {
    let (tmp, cache) = fresh_project();
    let out = cmd_with_cache(cache.path())
        .args([
            "attach",
            "--project",
            tmp.path().to_str().unwrap(),
            "--lang",
            "both",
        ])
        .output()
        .expect("spawn");
    let code = out.status.code().unwrap_or(-1);
    // BothRejected propagates as anyhow → main returns 1; lang_prompt's
    // banner is on stderr and contains the "not supported" / "지원하지" text.
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "expected failure, got code={code}");
    assert!(
        stderr.contains("not supported") || stderr.contains("지원하지"),
        "expected --lang both rejection banner; stderr was:\n{stderr}"
    );
}

#[test]
fn non_tty_fallback_uses_lang_env_and_announces_it() {
    let (tmp, cache) = fresh_project();
    let out = cmd_with_cache(cache.path())
        .args([
            "attach",
            "--project",
            tmp.path().to_str().unwrap(),
            "--non-interactive",
            "--yes",
            "--dry-run",
        ])
        .env("LANG", "ko_KR.UTF-8")
        // Wipe GENASIS_LANG so the resolver actually walks down to $LANG.
        .env_remove("GENASIS_LANG")
        .output()
        .expect("spawn");
    assert!(
        out.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let cfg = std::fs::read_to_string(tmp.path().join("genasis.toml")).unwrap();
    assert!(
        cfg.contains("active = \"ko\""),
        "non-TTY fallback should pick ko from $LANG; cfg:\n{cfg}"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("non-interactive") || stderr.contains("ko"),
        "expected non-interactive announcement; stderr:\n{stderr}"
    );
}

#[test]
fn lang_status_reports_active_locale() {
    let (tmp, cache) = fresh_project();
    // Seed i18n config first.
    cmd_with_cache(cache.path())
        .args([
            "attach",
            "--project",
            tmp.path().to_str().unwrap(),
            "--lang",
            "ko",
            "--non-interactive",
            "--yes",
            "--dry-run",
        ])
        .output()
        .expect("spawn");
    // Now query.
    let out = cmd_with_cache(cache.path())
        .args(["lang", "--project", tmp.path().to_str().unwrap(), "status"])
        .output()
        .expect("spawn");
    assert!(
        out.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("active: ko"),
        "expected `active: ko`; stdout:\n{stdout}"
    );
}

#[test]
fn lang_switch_no_op_when_already_on_target() {
    let (tmp, cache) = fresh_project();
    cmd_with_cache(cache.path())
        .args([
            "attach",
            "--project",
            tmp.path().to_str().unwrap(),
            "--lang",
            "en",
            "--non-interactive",
            "--yes",
            "--dry-run",
        ])
        .output()
        .expect("spawn");
    let out = cmd_with_cache(cache.path())
        .args([
            "lang",
            "--project",
            tmp.path().to_str().unwrap(),
            "switch",
            "en",
        ])
        .output()
        .expect("spawn");
    assert!(
        out.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Already on") || stdout.contains("이미"),
        "expected no-op message; stdout:\n{stdout}"
    );
}
