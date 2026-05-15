//! `genasis provision` — automate Plane + Mattermost setup for a real
//! agentic team.
//!
//! ADR-019: replaces the per-environment Python script the user
//! previously suggested. Single Rust binary, same flow on the
//! operator-hosted instance (plane.realstory.blog / mm.realstory.blog)
//! and on a self-host docker-compose stack (`localhost:8080` /
//! `localhost:8065`).
//!
//! User decisions (this cycle's plan):
//! - 5-char abbreviation slug; Hangul translated via local `claude`
//!   CLI before abbreviating (see `genasis_core::slug`).
//! - 10 default agents: pm, frontend, backend, devops, designer, qa,
//!   planner, architect, code-reviewer, security.
//! - Agent email pattern: `<role>-<team_slug>@genasis.bot`.
//! - Plane: try per-team workspace; on permission failure fall back
//!   to a shared `agentic` workspace + `<team-slug>-<app-slug>`
//!   project name.
//! - Mattermost: one team + one `scrum-<app-slug>` channel per
//!   request.
//! - Output: `genasis.toml` (identifiers only) + `.env.local`
//!   (per-agent tokens, chmod 600).
//! - No rollback on partial failure — idempotent re-run picks up
//!   where we left off.

use std::path::PathBuf;

use anyhow::{anyhow, bail, Context, Result};
use clap::Parser;

use genasis_core::slug::slugify_abbrev;

/// Canonical 10-agent default. Order is significant: PM lands first
/// in the env file so the daemon can read it without scanning the
/// whole list.
pub const DEFAULT_AGENTS: &[&str] = &[
    "pm",
    "frontend",
    "backend",
    "devops",
    "designer",
    "qa",
    "planner",
    "architect",
    "code-reviewer",
    "security",
];

/// Domain hard-coded per user decision. Both Plane and Mattermost
/// accept synthetic email domains for bot users; this isolates the
/// fake addresses we mint from any real domain the operator owns.
pub const AGENT_EMAIL_DOMAIN: &str = "genasis.bot";

#[derive(Parser, Debug)]
pub struct Args {
    /// Human-readable team name. Translated + abbreviated to a
    /// 5-char slug used everywhere downstream.
    #[arg(long, value_name = "NAME")]
    pub team: String,

    /// Human-readable app/project name.
    #[arg(long, value_name = "NAME")]
    pub app: String,

    /// Comma-separated list of human collaborator emails. Each will
    /// be invited as a Member to both Plane and Mattermost.
    #[arg(long, value_name = "EMAILS", value_delimiter = ',')]
    pub humans: Vec<String>,

    /// Override the 10-agent default. Comma-separated role names.
    #[arg(long, value_name = "ROLES", value_delimiter = ',')]
    pub agents: Option<Vec<String>>,

    /// Path to write `genasis.toml` + `.env.local`. Defaults to the
    /// current working directory.
    #[arg(long, value_name = "DIR")]
    pub output: Option<PathBuf>,

    /// Dry-run — compute slugs, print intended actions, but make no
    /// HTTP calls and write no files.
    #[arg(long)]
    pub dry_run: bool,
}

/// Resolved + sanitised inputs ready for both provisioners. Built
/// once at the top of `run()` and threaded through the rest of the
/// flow so we never re-derive slugs.
#[derive(Debug, Clone)]
pub struct ResolvedProvisionPlan {
    pub team_name: String,
    pub team_slug: String,
    pub app_name: String,
    pub app_slug: String,
    pub humans: Vec<String>,
    pub agents: Vec<String>,
    pub output_dir: PathBuf,

    pub plane: TargetEndpoint,
    pub mattermost: TargetEndpoint,
}

/// One concrete admin endpoint (Plane or Mattermost). Filled from
/// environment variables — the caller must export them before
/// running `genasis provision`. Documented as a struct so the missing-
/// env error path can name them precisely.
#[derive(Debug, Clone)]
pub struct TargetEndpoint {
    pub url: String,
    pub admin_token: String,
}

pub async fn run(args: Args) -> Result<()> {
    let plan = resolve_plan(&args)?;
    println!("──────────────────────────────────────────────");
    println!("  genasis provision — plan");
    println!("──────────────────────────────────────────────");
    println!("  team       : {} ({})", plan.team_name, plan.team_slug);
    println!("  app        : {} ({})", plan.app_name, plan.app_slug);
    println!("  humans     : {}", plan.humans.join(", "));
    println!("  agents     : {}", plan.agents.join(", "));
    println!("  plane URL  : {}", plan.plane.url);
    println!("  mm URL     : {}", plan.mattermost.url);
    println!("  output dir : {}", plan.output_dir.display());
    println!("  mode       : {}", if args.dry_run { "DRY-RUN" } else { "LIVE" });
    println!();
    for agent in &plan.agents {
        println!(
            "    agent {:<14} → {}-{}@{}",
            agent, agent, plan.team_slug, AGENT_EMAIL_DOMAIN
        );
    }
    println!();

    if args.dry_run {
        println!("  [dry-run] skipping all API calls + file writes.");
        return Ok(());
    }

    // M-v6.x: real Plane + Mattermost provisioners land in their own
    // commits — this scaffolding commit only validates the plan and
    // wires the clap surface so the next change has a clean target.
    bail!(
        "live provisioning not implemented yet — re-run with `--dry-run` to \
         preview the plan, or wait for the upcoming alpha that ships the \
         Plane + Mattermost REST adapters."
    );
}

fn resolve_plan(args: &Args) -> Result<ResolvedProvisionPlan> {
    let team_name = args.team.trim().to_string();
    let app_name = args.app.trim().to_string();
    if team_name.is_empty() {
        bail!("--team must be a non-empty string");
    }
    if app_name.is_empty() {
        bail!("--app must be a non-empty string");
    }
    if args.humans.is_empty() {
        bail!("at least one --humans email is required (this is who Plane + Mattermost get invited as)");
    }

    let team_slug = slugify_abbrev(&team_name);
    let app_slug = slugify_abbrev(&app_name);

    let agents: Vec<String> = args
        .agents
        .clone()
        .unwrap_or_else(|| DEFAULT_AGENTS.iter().map(|s| s.to_string()).collect());

    let output_dir = match &args.output {
        Some(p) => p.clone(),
        None => std::env::current_dir()
            .context("unable to read current working directory for --output default")?,
    };

    let plane = read_endpoint("PLANE_URL", "PLANE_ADMIN_TOKEN")
        .context("Plane admin credentials missing from environment")?;
    let mattermost = read_endpoint("MM_URL", "MM_ADMIN_TOKEN")
        .context("Mattermost admin credentials missing from environment")?;

    Ok(ResolvedProvisionPlan {
        team_name,
        team_slug,
        app_name,
        app_slug,
        humans: args.humans.clone(),
        agents,
        output_dir,
        plane,
        mattermost,
    })
}

fn read_endpoint(url_var: &str, token_var: &str) -> Result<TargetEndpoint> {
    let url = std::env::var(url_var).map_err(|_| {
        anyhow!(
            "env var ${url_var} is required — set it to the target instance \
             (e.g. https://plane.realstory.blog or http://localhost:8080 \
             for a docker-compose self-host)"
        )
    })?;
    let admin_token = std::env::var(token_var).map_err(|_| {
        anyhow!(
            "env var ${token_var} is required — generate an admin API token \
             from the instance admin UI and export it before running \
             `genasis provision`"
        )
    })?;
    if url.trim().is_empty() {
        bail!("env var ${url_var} is set but empty");
    }
    if admin_token.trim().is_empty() {
        bail!("env var ${token_var} is set but empty");
    }
    Ok(TargetEndpoint {
        url: url.trim_end_matches('/').to_string(),
        admin_token,
    })
}
