//! Live smoke test for `mattermost::real_provisioner`.
//!
//! Run with:
//!   MM_URL=https://mm.realstory.blog \
//!   MM_ADMIN_TOKEN=... \
//!     cargo run -p genasis-providers --example mm_smoke

use anyhow::{Context, Result};

use genasis_providers::mattermost::real_provisioner::{MmClient, Outcome, CHANNEL_OPEN};

#[tokio::main]
async fn main() -> Result<()> {
    let url = std::env::var("MM_URL").context("MM_URL")?;
    let token = std::env::var("MM_ADMIN_TOKEN").context("MM_ADMIN_TOKEN")?;
    let c = MmClient::new(&url, &token)?;

    println!("→ whoami…");
    let me = c.whoami().await?;
    println!("  ✓ {} <{}> roles={}", me.username, me.email, me.roles);

    println!("→ ensure team team-genasis-smoke — should Create…");
    let (team, o1) = c
        .ensure_team("team-genasis-smoke", "Genasis Smoke", None)
        .await?;
    println!("  ✓ team id={} outcome={:?}", team.id, o1);

    println!("→ ensure team with matching expected_id — should Reuse…");
    let (_, o2) = c
        .ensure_team("team-genasis-smoke", "Genasis Smoke", Some(&team.id))
        .await?;
    assert_eq!(o2, Outcome::Reused);
    println!("  ✓ outcome={:?}", o2);

    println!("→ ensure channel scrum-genasis-smoke — should Create…");
    let (chan, o3) = c
        .ensure_channel(
            &team.id,
            "scrum-genasis-smoke",
            "Scrum Smoke",
            CHANNEL_OPEN,
            None,
        )
        .await?;
    println!("  ✓ channel id={} outcome={:?}", chan.id, o3);

    println!("→ ensure channel with matching expected_id — should Reuse…");
    let (_, o4) = c
        .ensure_channel(
            &team.id,
            "scrum-genasis-smoke",
            "Scrum Smoke",
            CHANNEL_OPEN,
            Some(&chan.id),
        )
        .await?;
    assert_eq!(o4, Outcome::Reused);
    println!("  ✓ outcome={:?}", o4);

    println!("→ ensure agent user smoke-pm@genasis.bot — should Create…");
    let (agent, o5) = c
        .ensure_agent_user("smoke-pm@genasis.bot", "smoke-pm-gsmk", "Tmp-Pass-123456")
        .await?;
    println!("  ✓ user id={} outcome={:?}", agent.id, o5);

    println!("→ ensure agent — should Reuse…");
    let (_, o6) = c
        .ensure_agent_user("smoke-pm@genasis.bot", "smoke-pm-gsmk", "Tmp-Pass-123456")
        .await?;
    assert_eq!(o6, Outcome::Reused);

    println!("→ add agent to team — should Create…");
    let o7 = c.ensure_team_member(&team.id, &agent.id).await?;
    println!("  ✓ outcome={:?}", o7);

    println!("→ add agent to team again — should Reuse…");
    let o8 = c.ensure_team_member(&team.id, &agent.id).await?;
    assert_eq!(o8, Outcome::Reused);

    println!("→ add agent to scrum channel…");
    let _ = c.ensure_channel_member(&chan.id, &agent.id).await?;

    println!("→ issue PAT for agent…");
    let pat = c.issue_pat(&agent.id, "genasis-smoke").await?;
    println!("  ✓ PAT id={} len(token)={}", pat.id, pat.token.len());

    println!(
        "\nALL OK — clean up `team-genasis-smoke` and `smoke-pm@genasis.bot` from MM admin UI."
    );
    Ok(())
}
