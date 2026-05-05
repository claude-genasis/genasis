//! Integration: bootstrap fills `.claude/agents/` with 10 base files,
//! then the detector classifies each one as `Known(_)` so plan_attach
//! injects a fence into every role.
//!
//! See ADR-010 + progress.ko.md M14.2.

use genasis_overlay::{
    apply_bootstrap, plan_attach, plan_bootstrap, scan, AttachOptions, BootstrapOptions, Classified,
    PlannedAction, Role,
};
use tempfile::tempdir;

#[test]
fn bootstrap_then_attach_injects_into_every_role() {
    let d = tempdir().unwrap();
    let plan = plan_bootstrap(d.path(), &BootstrapOptions::new("en")).unwrap();
    apply_bootstrap(&plan).unwrap();

    let report = scan(d.path()).unwrap();
    assert_eq!(report.agents.len(), 10);
    assert!(report.skipped.is_empty());

    for agent in &report.agents {
        match &agent.classification {
            Classified::Known(_) => {}
            Classified::Custom(s) => panic!("base file classified as custom: {s}"),
        }
        assert!(
            !agent.has_existing_fence,
            "fresh base file should have no fence yet ({})",
            agent.path.display()
        );
    }

    let opts = AttachOptions::new("1.0").with_lang("en");
    let attach_plan = plan_attach(&report.agents, &opts).unwrap();
    let injects: Vec<_> = attach_plan
        .changes
        .iter()
        .filter(|c| matches!(c.action, PlannedAction::Inject))
        .collect();
    assert_eq!(injects.len(), 10, "every base role should accept a fence");
}

#[test]
fn bootstrap_ko_then_attach_ko_injects_korean_overlay() {
    let d = tempdir().unwrap();
    let plan = plan_bootstrap(d.path(), &BootstrapOptions::new("ko")).unwrap();
    apply_bootstrap(&plan).unwrap();

    let report = scan(d.path()).unwrap();
    assert_eq!(report.agents.len(), 10);

    let opts = AttachOptions::new("1.0").with_lang("ko");
    let attach_plan = plan_attach(&report.agents, &opts).unwrap();
    let injects = attach_plan
        .changes
        .iter()
        .filter(|c| matches!(c.action, PlannedAction::Inject))
        .count();
    assert_eq!(injects, 10);

    // Spot-check: the rendered Korean overlay must include the Korean
    // protocol header, not the English one.
    let backend = attach_plan
        .changes
        .iter()
        .find(|c| {
            c.path
                .file_name()
                .and_then(|s| s.to_str())
                .map(|n| n == "backend.md")
                .unwrap_or(false)
        })
        .expect("backend change present");
    assert!(
        backend.after.contains("Plane / Mattermost 프로토콜"),
        "Korean overlay header missing in attach output:\n{}",
        backend.after
    );
}

#[test]
fn bootstrap_partial_then_attach_handles_mix() {
    let d = tempdir().unwrap();

    // Pre-author one role file the user already owns; bootstrap should
    // skip it and only emit the other 9.
    let agents = d.path().join(".claude/agents");
    std::fs::create_dir_all(&agents).unwrap();
    std::fs::write(
        agents.join("frontend.md"),
        "---\nname: frontend\ndescription: my own\ntools: Read\nmodel: sonnet\ncolor: orange\n---\n# my header\n",
    )
    .unwrap();

    let plan = plan_bootstrap(d.path(), &BootstrapOptions::new("en")).unwrap();
    let creates: Vec<_> = plan.creates().collect();
    let skips: Vec<_> = plan.skips().collect();
    assert_eq!(creates.len(), 9);
    assert_eq!(skips.len(), 1);
    assert_eq!(skips[0].role, Role::Frontend);

    apply_bootstrap(&plan).unwrap();

    // The user-authored frontend.md must still be byte-identical.
    let raw = std::fs::read_to_string(agents.join("frontend.md")).unwrap();
    assert!(raw.contains("# my header"));
    assert!(raw.contains("description: my own"));
}
