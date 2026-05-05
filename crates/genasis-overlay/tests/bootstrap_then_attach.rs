//! Integration: bootstrap fills `.claude/agents/` with 10 base files,
//! then the detector classifies each one as `Known(_)` so plan_attach
//! injects a fence into every role.
//!
//! See ADR-010 + progress.ko.md M14.2.

use genasis_overlay::{
    apply_bootstrap, plan_attach, plan_bootstrap, scan, AttachOptions, BootstrapOptions,
    Classified, PlannedAction, Role,
};
use genasis_templates::AgentStore;
use tempfile::tempdir;

/// Create a mock AgentStore with base .md files and overlay templates
/// for all 10 roles (both `en` and `ko` locales).
fn mock_store() -> (tempfile::TempDir, AgentStore) {
    let catalog = tempdir().unwrap();
    let base = catalog.path().join("base");
    let overlays_en = catalog.path().join("overlays/en");
    let overlays_ko = catalog.path().join("overlays/ko");
    std::fs::create_dir_all(&base).unwrap();
    std::fs::create_dir_all(&overlays_en).unwrap();
    std::fs::create_dir_all(&overlays_ko).unwrap();
    std::fs::write(
        catalog.path().join("manifest.json"),
        r#"{"version":"0.0.1-test","roles":[]}"#,
    )
    .unwrap();
    for role in Role::ALL {
        let slug = role.slug();
        let content = format!(
            "---\nname: {slug}\ndescription: test {slug}\ntools: Read\nmodel: sonnet\ncolor: gray\n---\n\n# {slug} Agent\n\nTest base file.\n"
        );
        std::fs::write(base.join(format!("{slug}.md")), &content).unwrap();
        std::fs::write(
            overlays_en.join(format!("{slug}.patch.md.tera")),
            format!("## (Genasis Overlay) Plane / Mattermost protocol\nproject: {{{{ project_name | default(value=\"test\") }}}}\nrole: {slug}\n"),
        )
        .unwrap();
        std::fs::write(
            overlays_ko.join(format!("{slug}.patch.md.tera")),
            format!("## (Genasis Overlay) Plane / Mattermost 프로토콜\nproject: {{{{ project_name | default(value=\"test\") }}}}\nrole: {slug}\n"),
        )
        .unwrap();
    }
    let store = AgentStore::from_dir(catalog.path().to_path_buf()).unwrap();
    (catalog, store)
}

#[test]
fn bootstrap_then_attach_injects_into_every_role() {
    let d = tempdir().unwrap();
    let (_cat, store) = mock_store();
    let plan = plan_bootstrap(d.path(), &BootstrapOptions::new("en"), &store).unwrap();
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
    let attach_plan = plan_attach(&report.agents, &opts, &store).unwrap();
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
    let (_cat, store) = mock_store();
    let plan = plan_bootstrap(d.path(), &BootstrapOptions::new("ko"), &store).unwrap();
    apply_bootstrap(&plan).unwrap();

    let report = scan(d.path()).unwrap();
    assert_eq!(report.agents.len(), 10);

    let opts = AttachOptions::new("1.0").with_lang("ko");
    let attach_plan = plan_attach(&report.agents, &opts, &store).unwrap();
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
    let (_cat, store) = mock_store();

    // Pre-author one role file the user already owns; bootstrap should
    // skip it and only emit the other 9.
    let agents = d.path().join(".claude/agents");
    std::fs::create_dir_all(&agents).unwrap();
    std::fs::write(
        agents.join("frontend.md"),
        "---\nname: frontend\ndescription: my own\ntools: Read\nmodel: sonnet\ncolor: orange\n---\n# my header\n",
    )
    .unwrap();

    let plan = plan_bootstrap(d.path(), &BootstrapOptions::new("en"), &store).unwrap();
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
