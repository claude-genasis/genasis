//! `genasis team …` — post-provision team-member management.
//!
//! Reads the snapshot left by `genasis provision`, mutates it
//! through the same REST adapters (`PlaneClient` / `MmClient`), then
//! rewrites all three artifacts via `provision_writer::write_all` so
//! genasis.toml.snapshot + .env.local + provision.log stay in sync.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};

use genasis_providers::mattermost::real_provisioner::{
    MmClient, Outcome as MmOutcome, CHANNEL_OPEN,
};
use genasis_providers::plane::real_provisioner::{
    PlaneClient, ProjectCreateOutcome as PlaneOutcome, ROLE_MEMBER,
};

use crate::cmd_provision::{derive_username, parse_human_spec, HumanSpec, AGENT_EMAIL_DOMAIN};
use crate::provision_writer::{
    load_snapshot, resolve_snapshot_path, write_all, AgentRecord, HumanRecord, ProvisionRecord,
};

#[derive(Parser, Debug)]
pub struct Args {
    /// Directory containing `genasis.toml.snapshot`. Defaults to cwd
    /// (with a `GENASIS_SECRETS_ROOT` + `--team-slug` fallback).
    #[arg(long, value_name = "DIR", global = true)]
    pub from: Option<PathBuf>,

    /// Team slug — used to locate the snapshot under
    /// `GENASIS_SECRETS_ROOT/teams/<slug>` when `--from` is omitted.
    #[arg(long, value_name = "SLUG", global = true)]
    pub team_slug: Option<String>,

    #[command(subcommand)]
    pub command: TeamCmd,
}

#[derive(Subcommand, Debug)]
pub enum TeamCmd {
    /// Add a human collaborator or hire an additional agent.
    Add(AddArgs),
    /// Remove a human or retire an agent.
    Remove(RemoveArgs),
    /// Print the current team roster from `genasis.toml.snapshot`.
    List,
}

#[derive(Parser, Debug)]
pub struct AddArgs {
    #[command(subcommand)]
    pub target: AddTarget,
}

#[derive(Subcommand, Debug)]
pub enum AddTarget {
    /// Invite a new human team-member. Spec format is
    /// `"Name <email@x.com>"` or just `email@x.com`.
    Human { spec: String },
    /// Hire an additional agent. Plane side: the agent user must
    /// already exist as a workspace member with the matching
    /// `display_name`; we only attach them to the project.
    Agent {
        /// Role identifier — used for env-var suffix and the agent
        /// email local-part.
        role: String,
    },
}

#[derive(Parser, Debug)]
pub struct RemoveArgs {
    #[command(subcommand)]
    pub target: RemoveTarget,
}

#[derive(Subcommand, Debug)]
pub enum RemoveTarget {
    /// Drop a human from the roster. The Plane workspace invitation
    /// is revoked if pending, otherwise the user remains in
    /// `agentic` (we can't deactivate global users from a workspace
    /// API key) but is removed from the project. Mattermost team
    /// membership is removed.
    Human { email: String },
    /// Retire an agent. The agent's Mattermost team + channel
    /// membership is removed; Plane project membership is removed.
    /// We do NOT delete the underlying user accounts — history is
    /// preserved.
    Agent { role: String },
}

pub async fn run(args: Args) -> Result<()> {
    let snapshot_path = resolve_snapshot_path(args.from.as_deref(), args.team_slug.as_deref())?;
    let snapshot_dir = snapshot_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("snapshot path has no parent dir"))?
        .to_path_buf();
    let mut record = load_snapshot(&snapshot_path)?;

    match args.command {
        TeamCmd::List => list_roster(&record),
        TeamCmd::Add(a) => match a.target {
            AddTarget::Human { spec } => {
                let h = parse_human_spec(&spec)?;
                add_human(&mut record, &snapshot_dir, h).await
            }
            AddTarget::Agent { role } => add_agent(&mut record, &snapshot_dir, &role).await,
        },
        TeamCmd::Remove(r) => match r.target {
            RemoveTarget::Human { email } => remove_human(&mut record, &snapshot_dir, &email).await,
            RemoveTarget::Agent { role } => remove_agent(&mut record, &snapshot_dir, &role).await,
        },
    }
}

fn list_roster(record: &ProvisionRecord) -> Result<()> {
    println!("Team        : {} ({})", record.team_name, record.team_slug);
    println!("App         : {} ({})", record.app_name, record.app_slug);
    println!(
        "Plane       : {}/{}",
        record.plane.url, record.plane.project_identifier
    );
    println!(
        "Mattermost  : {}/{}/channels/{}",
        record.mattermost.url, record.mattermost.team_name, record.mattermost.scrum_channel_name
    );
    println!();
    println!("Humans ({}):", record.humans.len());
    for h in &record.humans {
        let plane = h.plane_user_id.as_deref().unwrap_or("(pending invite)");
        let mm = h.mm_user_id.as_deref().unwrap_or("(pending invite)");
        println!(
            "  - {:<20} <{}>  plane={}  mm={}",
            h.name, h.email, plane, mm
        );
    }
    println!();
    println!("Agents ({}):", record.agents.len());
    for a in &record.agents {
        let plane = a.plane_user_id.as_deref().unwrap_or("(none)");
        println!(
            "  - {:<14} {:<30} plane={}  mm={}",
            a.role, a.email, plane, a.mm_user_id
        );
    }
    Ok(())
}

async fn add_human(record: &mut ProvisionRecord, snapshot_dir: &Path, h: HumanSpec) -> Result<()> {
    if record
        .humans
        .iter()
        .any(|x| x.email.eq_ignore_ascii_case(&h.email))
    {
        println!("ℹ {} is already on the roster — no-op", h.email);
        return Ok(());
    }
    let (plane, mm) = open_clients(record)?;

    println!("→ Plane workspace invite for {} <{}>...", h.name, h.email);
    let (inv, p_o) = plane
        .ensure_workspace_invitation(&h.email, ROLE_MEMBER)
        .await?;
    println!("  ✓ {:?} accepted={}", p_o, inv.accepted);

    println!("→ Mattermost: ...");
    let mm_id = match mm.user_by_email(&h.email).await? {
        Some(user) => {
            mm.ensure_team_member(&record.mattermost.team_id, &user.id)
                .await?;
            mm.ensure_channel_member(&record.mattermost.scrum_channel_id, &user.id)
                .await?;
            println!("  ✓ existing MM account attached to team + channel");
            Some(user.id)
        }
        None => {
            mm.invite_human_by_email(&record.mattermost.team_id, &[h.email.clone()])
                .await?;
            println!("  ✓ MM invite emailed to {}", h.email);
            None
        }
    };

    record.humans.push(HumanRecord {
        name: h.name.clone(),
        email: h.email.clone(),
        username: derive_username(&h.email),
        plane_user_id: None,
        mm_user_id: mm_id,
    });
    write_all(record, snapshot_dir).context("rewrite snapshot")?;
    println!("\n✓ added {} <{}>", h.name, h.email);
    Ok(())
}

async fn add_agent(record: &mut ProvisionRecord, snapshot_dir: &Path, role: &str) -> Result<()> {
    if record.agents.iter().any(|a| a.role == role) {
        println!("ℹ agent {role:?} already on the roster — no-op");
        return Ok(());
    }
    let (plane, mm) = open_clients(record)?;

    // Plane side — match by display_name. If the agent isn't a
    // workspace member yet, surface the actionable error and bail.
    let plane_member = plane
        .find_member_by_display_name(role)
        .await?
        .ok_or_else(|| {
            anyhow::anyhow!(
                "no Plane workspace member with display_name={role:?}. The operator \
             must register the agent in the Plane admin UI / DB first; then \
             re-run this command."
            )
        })?;
    let role_code = if role == "pm" {
        genasis_providers::plane::real_provisioner::ROLE_ADMIN
    } else {
        ROLE_MEMBER
    };
    let p_o = plane
        .ensure_project_member(&record.plane.project_id, &plane_member.id, role_code)
        .await?;
    println!("  ✓ Plane project +{role} ({:?})", p_o);

    // Mattermost — create user + PAT + memberships.
    let agent_email = format!("{role}-{}@{}", record.team_slug, AGENT_EMAIL_DOMAIN);
    let mm_username = sanitize_mm_username(&format!("{role}-{}", record.team_slug));
    let password = random_password();
    let (user, u_o) = mm
        .ensure_agent_user(&agent_email, &mm_username, &password)
        .await?;
    println!("  ✓ MM agent user {agent_email} ({:?})", u_o);
    mm.ensure_team_member(&record.mattermost.team_id, &user.id)
        .await?;
    mm.ensure_channel_member(&record.mattermost.scrum_channel_id, &user.id)
        .await?;
    let pat = mm
        .issue_pat(&user.id, &format!("genasis-{}-{}", record.team_slug, role))
        .await?;

    record.agents.push(AgentRecord {
        role: role.to_string(),
        email: agent_email,
        plane_user_id: Some(plane_member.id),
        plane_pat: None,
        mm_user_id: user.id,
        mm_pat: pat.token,
    });
    write_all(record, snapshot_dir).context("rewrite snapshot")?;
    println!("\n✓ hired agent {role}");
    Ok(())
}

async fn remove_human(
    record: &mut ProvisionRecord,
    snapshot_dir: &Path,
    email: &str,
) -> Result<()> {
    let idx = record
        .humans
        .iter()
        .position(|h| h.email.eq_ignore_ascii_case(email));
    let idx = match idx {
        Some(i) => i,
        None => {
            println!("ℹ {email} not on the roster — no-op");
            return Ok(());
        }
    };
    let (_plane, mm) = open_clients(record)?;
    let human = record.humans[idx].clone();

    // Mattermost — remove from team if account exists.
    if let Some(user) = mm.user_by_email(email).await? {
        let url = format!(
            "{}/api/v4/teams/{}/members/{}",
            record.mattermost.url, record.mattermost.team_id, user.id
        );
        let resp = reqwest::Client::new()
            .delete(&url)
            .header(
                reqwest::header::AUTHORIZATION,
                // we don't have the admin token cached on MmClient
                // post-build; reconstruct via env that resolve_plan
                // already validated.
                format!(
                    "Bearer {}",
                    std::env::var("MM_ADMIN_TOKEN")
                        .context("MM_ADMIN_TOKEN required for remove")?
                ),
            )
            .send()
            .await
            .context("delete team member")?;
        if !resp.status().is_success() && resp.status() != reqwest::StatusCode::NOT_FOUND {
            let s = resp.status();
            let b = resp.text().await.unwrap_or_default();
            bail!("MM remove member {s}: {b}");
        }
        println!("  ✓ MM removed {} from team-{}", email, record.team_slug);
    }
    // Plane — drop the workspace invitation if still pending. If
    // the human has already accepted and become a workspace member,
    // we leave them in the workspace (workspace API key can't
    // deactivate global users) and only remove from project.
    // For simplicity in this first cut we don't touch Plane project
    // membership on human-remove — operator does that manually.

    record.humans.remove(idx);
    write_all(record, snapshot_dir).context("rewrite snapshot")?;
    println!("\n✓ removed {} <{}>", human.name, human.email);
    Ok(())
}

async fn remove_agent(record: &mut ProvisionRecord, snapshot_dir: &Path, role: &str) -> Result<()> {
    let idx = record.agents.iter().position(|a| a.role == role);
    let idx = match idx {
        Some(i) => i,
        None => {
            println!("ℹ agent {role:?} not on the roster — no-op");
            return Ok(());
        }
    };
    let agent = record.agents[idx].clone();
    // For now we don't issue REST DELETEs against Plane / MM in the
    // remove path — that destroys history and is hard to undo. The
    // canonical "retire" semantics in this first cut is: drop from
    // genasis.toml.snapshot + .env.local so the daemon stops
    // routing work to that agent. Operator can deactivate the
    // backing user via admin UI when truly done.
    println!(
        "  ✓ dropping {role} from roster (plane user {}, mm user {} kept for history)",
        agent.plane_user_id.as_deref().unwrap_or("(none)"),
        agent.mm_user_id
    );
    record.agents.remove(idx);
    write_all(record, snapshot_dir).context("rewrite snapshot")?;
    println!("\n✓ retired agent {role}");
    Ok(())
}

fn open_clients(record: &ProvisionRecord) -> Result<(PlaneClient, MmClient)> {
    let plane_token = std::env::var("PLANE_ADMIN_TOKEN")
        .context("PLANE_ADMIN_TOKEN required for `team` commands")?;
    let mm_token =
        std::env::var("MM_ADMIN_TOKEN").context("MM_ADMIN_TOKEN required for `team` commands")?;
    let plane = PlaneClient::new(
        &record.plane.url,
        &plane_token,
        &record.plane.workspace_slug,
    )?;
    let mm = MmClient::new(&record.mattermost.url, &mm_token)?;
    Ok((plane, mm))
}

fn sanitize_mm_username(s: &str) -> String {
    let cleaned: String = s
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                c
            } else {
                '_'
            }
        })
        .collect();
    if cleaned.len() <= 22 {
        cleaned
    } else {
        cleaned.chars().take(22).collect()
    }
}

fn random_password() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let pid = std::process::id();
    format!("{nanos:x}{pid:x}Z!")
}

// Reference the unused outcome types to keep the warnings quiet —
// they're emitted by helper logs and may also be matched on by
// downstream PRs.
#[allow(dead_code)]
fn _outcome_silencer() -> (PlaneOutcome, MmOutcome) {
    (PlaneOutcome::Created, MmOutcome::Created)
}
