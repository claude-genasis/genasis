use genasis_providers::{mattermost, plane};

#[test]
fn plane_flavor_choice_parses_known_strings() {
    assert!(matches!(
        plane::FlavorChoice::parse("upstream").unwrap(),
        plane::FlavorChoice::Upstream
    ));
    assert!(matches!(
        plane::FlavorChoice::parse("agent-aware").unwrap(),
        plane::FlavorChoice::AgentAware
    ));
    assert!(matches!(
        plane::FlavorChoice::parse("auto").unwrap(),
        plane::FlavorChoice::Auto
    ));
    assert!(plane::FlavorChoice::parse("not-a-flavor").is_err());
}

#[test]
fn mm_flavor_choice_parses_known_strings() {
    assert!(matches!(
        mattermost::FlavorChoice::parse("upstream").unwrap(),
        mattermost::FlavorChoice::Upstream
    ));
    assert!(matches!(
        mattermost::FlavorChoice::parse("agent-aware").unwrap(),
        mattermost::FlavorChoice::AgentAware
    ));
    assert!(matches!(
        mattermost::FlavorChoice::parse("auto").unwrap(),
        mattermost::FlavorChoice::Auto
    ));
    assert!(mattermost::FlavorChoice::parse("xx").is_err());
}
