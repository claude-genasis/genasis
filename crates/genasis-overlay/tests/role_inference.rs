//! Cross-crate test: every documented role slug round-trips through
//! `infer_from_name`.

use genasis_overlay::{infer_from_name, Classified, Role};

#[test]
fn every_role_slug_is_recognised() {
    for r in Role::ALL {
        let slug = r.slug();
        assert_eq!(infer_from_name(slug), Classified::Known(*r));
    }
}

#[test]
fn case_insensitive_matching() {
    assert_eq!(
        infer_from_name("FRONTEND"),
        Classified::Known(Role::Frontend)
    );
    assert_eq!(
        infer_from_name("Architect"),
        Classified::Known(Role::Architect)
    );
}

#[test]
fn unknown_names_become_custom() {
    match infer_from_name("loop-operator") {
        Classified::Custom(s) => assert_eq!(s, "loop-operator"),
        other => panic!("expected custom, got {other:?}"),
    }
}
