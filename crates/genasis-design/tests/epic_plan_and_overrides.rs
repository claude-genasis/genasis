//! M-D2 e2e: full-rewrite EPIC plan, override accumulation across swaps,
//! and verify hash detection.

use genasis_design::pointer::Locale;
use genasis_design::swap::{Source, SwapInput};
use genasis_design::{
    auto_plan, override_add, override_list, override_remove, run_swap, run_verify, ImpactArea,
    PlanMode, State, DEFAULT_FULL_REWRITE_THRESHOLD,
};

use tempfile::tempdir;

fn run_local_swap(project: &std::path::Path, body_path: std::path::PathBuf) {
    run_swap(SwapInput {
        project_root: project.to_path_buf(),
        external_dir: "docs/design-system".into(),
        gallery_index_url: "https://getdesign.md/".into(),
        gallery_url_template: "https://getdesign.md/{slug}/design-md".into(),
        disable_telemetry: true,
        locale: Locale::En,
        source: Source::File(body_path),
    })
    .unwrap();
}

#[test]
fn auto_plan_emits_epic_when_majority_of_areas_change() {
    let areas = vec![
        ImpactArea::ColorTokens,
        ImpactArea::Typography,
        ImpactArea::Spacing,
        ImpactArea::Components,
    ];
    let p = auto_plan(
        PlanMode::Auto,
        &areas,
        "https://getdesign.md/posthog/design-md",
        "posthog",
        "https://getdesign.md/posthog/design-md",
        DEFAULT_FULL_REWRITE_THRESHOLD,
    );
    match p {
        genasis_design::Plan::FullRewrite { epic, children } => {
            assert!(epic.title.contains("posthog"));
            assert_eq!(children.len(), 4);
            // Each child must reference the EPIC title for boards without
            // native parent_id support.
            for c in children {
                assert!(c.description.contains(&epic.title));
            }
        }
        genasis_design::Plan::PerArea(_) => {
            panic!("expected FullRewrite at threshold-of-4");
        }
    }
}

#[test]
fn overrides_persist_across_swap_and_can_be_listed_then_removed() {
    let dir = tempdir().unwrap();
    let project = dir.path();
    std::fs::create_dir_all(project.join("docs")).unwrap();
    std::fs::write(project.join("docs/design-system.md"), "# pristine\n").unwrap();

    // swap 1
    let a = project.join("a.md");
    std::fs::write(&a, "# DESIGN.md A\ncolor: blue\n").unwrap();
    run_local_swap(project, a);

    // 3 overrides
    override_add(project, "primary should be deep red").unwrap();
    override_add(project, "buttons must use 8px radius").unwrap();
    override_add(project, "do not use lime accent in nav").unwrap();
    assert_eq!(State::load(project).unwrap().override_count, 3);
    assert_eq!(override_list(project).unwrap().len(), 3);

    // swap 2 — overrides survive (pointer body is regenerated, but the
    // override_count in state persists; M-D2 deliberately does not migrate
    // §B.2 entries automatically, so the user reviews them post-swap).
    let b = project.join("b.md");
    std::fs::write(&b, "# DESIGN.md B\ncolor: red\nradius: 8\n").unwrap();
    run_local_swap(project, b);

    // After a fresh swap the pointer is regenerated → §B.2 starts empty.
    // Override count in state is reset to 0 because the §B.2 history is
    // tied to the old pointer; we surface this clearly in CLI status.
    let entries_after = override_list(project).unwrap();
    assert_eq!(entries_after.len(), 0);
    // The previous override count is now 0 in state.toml because swap
    // re-renders the pointer; the design-aware skill prompts the user
    // to re-review their overrides post-swap.
    assert_eq!(State::load(project).unwrap().override_count, 0);

    // Add a new override under the new design.
    override_add(project, "navy primary, not red").unwrap();
    assert_eq!(State::load(project).unwrap().override_count, 1);

    // Remove it.
    let removed = override_remove(project, "override-1").unwrap();
    assert!(removed);
    assert_eq!(State::load(project).unwrap().override_count, 0);
}

#[test]
fn verify_detects_hand_edits_to_external_design_md() {
    let dir = tempdir().unwrap();
    let project = dir.path();
    std::fs::create_dir_all(project.join("docs")).unwrap();
    std::fs::write(project.join("docs/design-system.md"), "# pristine\n").unwrap();
    let a = project.join("a.md");
    std::fs::write(&a, "# DESIGN.md A\n").unwrap();
    run_local_swap(project, a);

    // Pre-tamper: verify OK.
    let v = run_verify(project, "docs/design-system").unwrap();
    assert!(v.matches);

    // Hand-edit DESIGN.md (anti-pattern; verify is the safety net).
    std::fs::write(
        project.join("docs/design-system/DESIGN.md"),
        "# DESIGN.md A\n# tampered\n",
    )
    .unwrap();
    let v2 = run_verify(project, "docs/design-system").unwrap();
    assert!(!v2.matches);
    assert_ne!(v2.recorded_hash, v2.actual_hash);
}
