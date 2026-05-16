//! Smoke test for `plane::real_provisioner` against a live workspace.
//!
//! Run with:
//!   PLANE_URL=https://plane.realstory.blog \
//!   PLANE_API_KEY=... \
//!   PLANE_WORKSPACE_SLUG=agentic \
//!     cargo run -p genasis-providers --example plane_smoke
//!
//! Exercises: whoami → list members → idempotent project create →
//! idempotent project-member add → cleanup (delete the test project).
//! Does NOT touch agent users or human invitations — that's covered by
//! the live cmd_provision flow in PR-4.

use anyhow::{Context, Result};

use genasis_providers::plane::real_provisioner::{PlaneClient, ProjectCreateOutcome, ROLE_MEMBER};

#[tokio::main]
async fn main() -> Result<()> {
    let url = std::env::var("PLANE_URL").context("PLANE_URL")?;
    let key = std::env::var("PLANE_API_KEY").context("PLANE_API_KEY")?;
    let ws = std::env::var("PLANE_WORKSPACE_SLUG").context("PLANE_WORKSPACE_SLUG")?;
    let c = PlaneClient::new(&url, &key, &ws)?;

    println!("→ whoami…");
    let me = c.whoami().await?;
    println!("  ✓ {} <{}>", me.display_name, me.email);

    println!("→ list workspace members…");
    let members = c.list_workspace_members().await?;
    println!("  ✓ {} members", members.len());

    println!("→ find pm agent by display_name…");
    let pm = c.find_member_by_display_name("pm").await?;
    if let Some(pm) = &pm {
        println!("  ✓ pm = {} ({})", pm.email, pm.id);
    } else {
        println!("  (no pm agent registered yet — operator must add one first)");
    }

    println!("→ ensure project (GINT) — first call should Create…");
    let (proj, outcome) = c
        .ensure_project("Genasis Integration Test", "GINT", None)
        .await?;
    println!("  ✓ project = {} ({:?}) id={}", proj.name, outcome, proj.id);

    println!("→ ensure project (GINT) — second call with expected_id should Reuse…");
    let (_proj2, outcome2) = c
        .ensure_project("Genasis Integration Test", "GINT", Some(&proj.id))
        .await?;
    println!("  ✓ outcome = {:?}", outcome2);
    assert_eq!(
        outcome2,
        ProjectCreateOutcome::Reused,
        "second ensure_project should reuse"
    );

    if let Some(pm) = pm {
        println!("→ add pm as project member — should Create…");
        let r = c
            .ensure_project_member(&proj.id, &pm.id, ROLE_MEMBER)
            .await?;
        println!("  ✓ outcome = {:?}", r);

        println!("→ re-add pm — should Reuse…");
        let r2 = c
            .ensure_project_member(&proj.id, &pm.id, ROLE_MEMBER)
            .await?;
        println!("  ✓ outcome = {:?}", r2);
    }

    println!("\nALL OK — clean up the GINT project from Plane UI when done.");
    Ok(())
}
