//! M-D1 e2e: pristine → swap (slug-A via --from) → swap (slug-B via --from)
//! → restore round-trip. Verifies state file lifecycle, pristine.bak,
//! pointer body shape, sha256 hash, and archive directory creation.
//!
//! Uses the `--from <path>` source so the test does not depend on `npx` or
//! the network.

use genasis_design::pointer::Locale;
use genasis_design::swap::{Source, SwapInput};
use genasis_design::{run_restore, run_swap, sha256_hex, Mode, State};

use tempfile::tempdir;

fn fixed_input(
    project_root: &std::path::Path,
    source_file: std::path::PathBuf,
) -> SwapInput {
    SwapInput {
        project_root: project_root.to_path_buf(),
        external_dir: "docs/design-system".into(),
        gallery_index_url: "https://getdesign.md/".into(),
        gallery_url_template: "https://getdesign.md/{slug}/design-md".into(),
        disable_telemetry: true,
        locale: Locale::En,
        source: Source::File(source_file),
    }
}

#[test]
fn pristine_to_external_to_external_to_pristine_round_trip() {
    let dir = tempdir().unwrap();
    let project = dir.path();
    let docs = project.join("docs");
    std::fs::create_dir_all(&docs).unwrap();

    let pristine = "# my pristine design system\n\n## Tokens\ncolor: green\n";
    std::fs::write(docs.join("design-system.md"), pristine).unwrap();

    // Initial state: load returns pristine default (no file).
    let s0 = State::load(project).unwrap();
    assert_eq!(s0.mode, Mode::Pristine);

    // ── Swap 1: pristine → external (slug-A) ──────────────────────────
    let a_body = "# DESIGN.md (slug A)\ncolor: blue\n";
    let a_path = project.join("a-spec.md");
    std::fs::write(&a_path, a_body).unwrap();
    let s1 = run_swap(fixed_input(project, a_path)).unwrap();

    assert_eq!(s1.new_state.mode, Mode::External);
    assert_eq!(s1.new_state.slug, "a-spec");
    assert_eq!(s1.new_state.template_hash, sha256_hex(a_body));
    assert_eq!(s1.new_state.previous_slug, "");
    assert_eq!(
        s1.new_state.gallery_preview,
        "https://getdesign.md/a-spec/design-md"
    );
    assert!(s1.pristine_backup_path.is_some());
    let bak = s1.pristine_backup_path.unwrap();
    assert_eq!(std::fs::read_to_string(&bak).unwrap(), pristine);

    // Pointer body must reference §A path, hash, slug.
    let ptr1 = std::fs::read_to_string(docs.join("design-system.md")).unwrap();
    assert!(ptr1.contains("docs/design-system/DESIGN.md"));
    assert!(ptr1.contains(&format!("sha256:{}", s1.new_state.template_hash)));
    assert!(ptr1.contains("a-spec"));
    // External-mode markers
    assert!(ptr1.contains("§A. 1st-class truth"));
    assert!(ptr1.contains("§B. User overrides"));
    assert!(ptr1.contains("§C. Operator manual"));

    // ── Swap 2: external (slug-A) → external (slug-B) ─────────────────
    let b_body = "# DESIGN.md (slug B)\ncolor: red\nradius: 8\n";
    let b_path = project.join("b-spec.md");
    std::fs::write(&b_path, b_body).unwrap();
    let s2 = run_swap(fixed_input(project, b_path)).unwrap();

    assert_eq!(s2.new_state.mode, Mode::External);
    assert_eq!(s2.new_state.slug, "b-spec");
    assert_eq!(s2.new_state.previous_slug, "a-spec");
    assert_eq!(s2.new_state.template_hash, sha256_hex(b_body));
    // Previous body is overwritten in the external dir.
    assert_eq!(
        std::fs::read_to_string(s2.design_md_path.clone()).unwrap(),
        b_body
    );
    // No new pristine.bak because we were already in external mode —
    // the existing pristine.bak from swap 1 is preserved.
    assert!(s2.pristine_backup_path.is_none());
    let bak2 = project.join("docs/design-system/pristine.bak");
    assert!(bak2.is_file());
    assert_eq!(std::fs::read_to_string(&bak2).unwrap(), pristine);

    // State file reflects the latest swap.
    let loaded = State::load(project).unwrap();
    assert_eq!(loaded.slug, "b-spec");
    assert_eq!(loaded.previous_slug, "a-spec");

    // ── Restore: external → pristine ──────────────────────────────────
    let r = run_restore(project, "docs/design-system").unwrap();
    assert!(r.design_system_md_restored);
    // Pristine body is back.
    assert_eq!(
        std::fs::read_to_string(docs.join("design-system.md")).unwrap(),
        pristine
    );
    // Archive directory exists; external dir is gone.
    assert!(r.archive_dir.is_dir());
    assert!(!project.join("docs/design-system").exists());
    // Archive contains the previous DESIGN.md and pristine.bak.
    assert!(r.archive_dir.join("DESIGN.md").is_file());
    assert!(r.archive_dir.join("pristine.bak").is_file());
    // State file is gone — load returns pristine default.
    let s3 = State::load(project).unwrap();
    assert_eq!(s3.mode, Mode::Pristine);

    // ── Restore again should refuse (already pristine) ────────────────
    let err = run_restore(project, "docs/design-system").unwrap_err();
    assert!(err.to_string().contains("already pristine"));
}
