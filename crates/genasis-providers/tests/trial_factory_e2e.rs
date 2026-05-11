//! End-to-end test for the trial routing path.
//!
//! Verifies that `factory::build(Trial, .., Some(&trial_cfg))` returns a
//! provider whose calls actually land on the trial-app HTTP endpoints —
//! i.e. the `[trial]` section is the source of truth, not `mm_cfg.url` /
//! `plane_cfg.url`.
//!
//! Marked `#[ignore]` because it requires a running trial-app. Run with:
//!
//! ```sh
//! TRIAL_BASE=http://localhost:3000 \
//! TRIAL_SECRET=trialsecret \
//!   cargo test -p genasis-providers --test trial_factory_e2e -- --ignored --nocapture
//! ```

use genasis_core::config::TrialConfig;
use genasis_providers::{mattermost, plane};

fn trial_cfg() -> TrialConfig {
    TrialConfig {
        enabled: true,
        url: std::env::var("TRIAL_BASE").unwrap_or_else(|_| "http://localhost:3000".into()),
        shared_secret: std::env::var("TRIAL_SECRET").unwrap_or_else(|_| "trialsecret".into()),
        team_token: std::env::var("TRIAL_TEAM_TOKEN").ok(),
    }
}

#[tokio::test]
#[ignore]
async fn factory_routes_plane_trial_to_trial_app() {
    let t = trial_cfg();
    // Pass deliberately wrong base_url/api_key — factory must ignore them
    // for the Trial flavor and use [trial] instead.
    let client = plane::build(
        plane::FlavorChoice::Trial,
        "http://NOT-USED:9999",
        "ignored-ws",
        "ignored-key",
        Some(&t),
    )
    .await
    .expect("factory build should succeed");

    let project = client
        .ensure_project("Factory E2E", "FACTORYE2E")
        .await
        .expect("ensure_project against trial-app");
    let issue = client
        .create_issue(&project, "Routed via factory", "")
        .await
        .expect("create_issue");
    assert!(!issue.id.is_empty());
}

#[tokio::test]
#[ignore]
async fn factory_routes_mattermost_trial_to_trial_app() {
    let t = trial_cfg();
    let client = mattermost::build(
        mattermost::FlavorChoice::Trial,
        "http://NOT-USED:9999",
        "ignored-token",
        Some(&t),
    )
    .await
    .expect("factory build should succeed");

    let ch = client
        .ensure_channel("ignored-team", "scrum-factory-e2e", "Factory E2E Scrum")
        .await
        .expect("ensure_channel against trial-app");
    let post = client
        .post_root(&ch.id, "Hello via factory routing")
        .await
        .expect("post_root");
    assert!(!post.id.is_empty());
}

#[tokio::test]
async fn factory_rejects_trial_when_section_missing_or_disabled() {
    // Both providers must surface a clear Config error rather than
    // attempting the HTTP call against a stale base_url. Use match
    // arms because trait-object Results don't implement Debug.
    let mut disabled = trial_cfg();
    disabled.enabled = false;

    match plane::build(
        plane::FlavorChoice::Trial,
        "http://wrong",
        "ws",
        "key",
        Some(&disabled),
    )
    .await
    {
        Err(e) => assert!(
            format!("{e}").contains("enabled"),
            "expected enabled-false error, got: {e}"
        ),
        Ok(_) => panic!("expected error"),
    }

    match mattermost::build(
        mattermost::FlavorChoice::Trial,
        "http://wrong",
        "tok",
        Some(&disabled),
    )
    .await
    {
        Err(e) => assert!(
            format!("{e}").contains("enabled"),
            "expected enabled-false error, got: {e}"
        ),
        Ok(_) => panic!("expected error"),
    }

    match plane::build(
        plane::FlavorChoice::Trial,
        "http://wrong",
        "ws",
        "key",
        None,
    )
    .await
    {
        Err(e) => assert!(
            format!("{e}").contains("trial"),
            "expected missing-section error, got: {e}"
        ),
        Ok(_) => panic!("expected error"),
    }

    match mattermost::build(mattermost::FlavorChoice::Trial, "http://wrong", "tok", None).await {
        Err(e) => assert!(
            format!("{e}").contains("trial"),
            "expected missing-section error, got: {e}"
        ),
        Ok(_) => panic!("expected error"),
    }
}
