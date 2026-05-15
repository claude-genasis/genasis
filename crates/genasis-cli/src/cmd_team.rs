//! `genasis team …` — post-provision team-member management.
//!
//! ADR-019 §6: the initial `genasis provision` only sets up the team
//! once. Real teams grow over time — a new collaborator joins, the
//! PM decides to hire an additional `designer` agent, someone leaves.
//! Rather than re-running `provision` (which would attempt to recreate
//! everything), users reach for `genasis team` to mutate one member
//! at a time.
//!
//! Three operations:
//! - `genasis team add human "Name <email>"` — invite a new human
//!   to Plane + Mattermost, append to `.env.local`, update
//!   `genasis.toml [[humans]]`.
//! - `genasis team add agent <role>` — hire an additional agent.
//!   Role can be one of the canonical 10 not currently in the team,
//!   or a brand new role name (we'll create the user + PAT same way).
//! - `genasis team list` — show the current roster (humans + agents)
//!   straight from `genasis.toml`.
//! - `genasis team remove human <email>` / `team remove agent <role>` —
//!   deactivate the corresponding Plane + Mattermost account and
//!   drop the row from `genasis.toml` / `.env.local`.
//!
//! All operations are idempotent: re-running `add human` with an
//! email already on the roster is a no-op (with an informational
//! message), and `remove` of a missing member is also a no-op.

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};

use crate::cmd_provision::{parse_human_spec, HumanSpec};

#[derive(Parser, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub command: TeamCmd,
}

#[derive(Subcommand, Debug)]
pub enum TeamCmd {
    /// Add a human collaborator or hire an additional agent.
    Add(AddArgs),
    /// Remove a human or retire an agent.
    Remove(RemoveArgs),
    /// Print the current team roster (humans + agents) from
    /// `genasis.toml` plus per-member health (Plane account active?
    /// Mattermost account active?).
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
    /// `"Name <email@x.com>"` (same as `genasis provision --humans`),
    /// or a bare `email@x.com` for which we'll derive the display
    /// name from the local-part.
    Human {
        /// `"Name <email@x.com>"` or just `email@x.com`.
        spec: String,
    },
    /// Hire an additional agent. Role can be one of the canonical
    /// roles (pm/frontend/backend/devops/designer/qa/planner/architect/
    /// code-reviewer/security) not currently in the team, or any new
    /// custom name. Creates the Plane + Mattermost user, issues PATs,
    /// and writes them to `.env.local`.
    Agent {
        /// Role identifier — used as the env-var suffix
        /// (`PLANE_AGENT_TOKEN_<ROLE>`) and the agent email
        /// local-part (`<role>-<team-slug>@genasis.bot`).
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
    /// Deactivate a human's Plane + Mattermost account and remove
    /// them from `genasis.toml`. The history (issues created, posts
    /// authored) is preserved in both backends.
    Human {
        /// The exact email currently on the team's roster.
        email: String,
    },
    /// Retire an agent. The Plane + Mattermost user is deactivated
    /// (history preserved) and the row + env vars are removed.
    Agent {
        /// The role identifier as it appears in `genasis.toml`.
        role: String,
    },
}

pub async fn run(args: Args) -> Result<()> {
    match args.command {
        TeamCmd::Add(a) => match a.target {
            AddTarget::Human { spec } => add_human(parse_human_spec(&spec)?).await,
            AddTarget::Agent { role } => add_agent(&role).await,
        },
        TeamCmd::Remove(r) => match r.target {
            RemoveTarget::Human { email } => remove_human(&email).await,
            RemoveTarget::Agent { role } => remove_agent(&role).await,
        },
        TeamCmd::List => list_roster().await,
    }
}

async fn add_human(_h: HumanSpec) -> Result<()> {
    bail!(
        "`genasis team add human` is scaffolded but not yet implemented — the \
         underlying Plane + Mattermost REST adapters land in the next alpha. \
         Run `genasis provision` first if you haven't, then re-try this \
         command when alpha.27+ ships."
    );
}

async fn add_agent(_role: &str) -> Result<()> {
    bail!(
        "`genasis team add agent` is scaffolded but not yet implemented — the \
         underlying Plane + Mattermost REST adapters land in the next alpha."
    );
}

async fn remove_human(_email: &str) -> Result<()> {
    bail!(
        "`genasis team remove human` is scaffolded but not yet implemented — \
         lands with the same alpha that ships the `add` operations."
    );
}

async fn remove_agent(_role: &str) -> Result<()> {
    bail!(
        "`genasis team remove agent` is scaffolded but not yet implemented — \
         lands with the same alpha that ships the `add` operations."
    );
}

async fn list_roster() -> Result<()> {
    bail!(
        "`genasis team list` is scaffolded but not yet implemented — lands \
         once the `genasis.toml [[humans]] / [[agents]]` schema is finalised \
         in the upcoming alpha that ships real provisioning."
    );
}
