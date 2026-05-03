//! Integration test for `.env.agents` round-tripping with comments and
//! quoting preserved.

use genasis_core::env::EnvFile;

#[test]
fn full_round_trip_preserves_humans_comments() {
    let raw = "\
# Plane
PLANE_URL=\"https://plane.example.com\"
PLANE_WORKSPACE_SLUG=demo

# Per-agent tokens
PLANE_TOKEN_FRONTEND=plane_api_xxx
PLANE_TOKEN_BACKEND=plane_api_yyy

# Mattermost
MM_URL=\"https://mm.example.com\"
MM_TEAM_NAME=demo-team
";
    let env = EnvFile::from_str(raw).unwrap();
    assert_eq!(env.to_string(), raw, "round-trip must be byte-identical");
    assert_eq!(env.get("PLANE_TOKEN_FRONTEND"), Some("plane_api_xxx"));
}

#[test]
fn upgrade_path_overwrites_one_key_without_disturbing_neighbours() {
    let raw = "\
A=1
# B is special
B=\"two words\"
C=3
";
    let mut env = EnvFile::from_str(raw).unwrap();
    env.set("B", "ten words and # special");
    let s = env.to_string();
    // A and C are intact in their original positions.
    assert!(s.contains("A=1\n"));
    assert!(s.contains("C=3\n"));
    // B was rewritten with quoting.
    assert!(s.contains("B=\""));
    assert!(s.contains("# B is special"));
}
