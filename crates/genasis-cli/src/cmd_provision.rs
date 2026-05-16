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

use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Input, MultiSelect};

use anyhow::{anyhow, bail, Context, Result};
use clap::Parser;

use genasis_core::slug::slugify_abbrev;
use genasis_providers::mattermost::real_provisioner::{
    MmClient, Outcome as MmOutcome, CHANNEL_OPEN,
};
use genasis_providers::plane::real_provisioner::{
    PlaneClient, ProjectCreateOutcome, ROLE_ADMIN, ROLE_MEMBER,
};

use crate::provision_writer::{
    write_all, AgentRecord, HumanRecord, MattermostRecord, PlaneRecord, ProvisionRecord,
};

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
    /// 5-char slug used everywhere downstream. Prompted interactively
    /// if omitted.
    #[arg(long, value_name = "NAME")]
    pub team: Option<String>,

    /// Human-readable app/project name. Prompted interactively if
    /// omitted.
    #[arg(long, value_name = "NAME")]
    pub app: Option<String>,

    /// Human collaborators in `"Name <email>"` form, comma-separated.
    /// The email's local-part seeds the Plane / Mattermost username,
    /// so make sure each email is the one the user actually owns.
    ///
    /// Example:
    /// `--humans "Bravo Kim <gnoopy@gmail.com>,Alice <alice@x.com>"`
    ///
    /// Prompted interactively (one at a time) if neither this nor
    /// `--humans-file` is given.
    #[arg(long, value_name = "SPECS", value_delimiter = ',')]
    pub humans: Vec<String>,

    /// Alternative batch input — JSON file shaped as
    /// `[{"name":"Bravo Kim","email":"gnoopy@gmail.com"}, ...]`. Use
    /// this from CI / scripts instead of fighting the `--humans` shell
    /// quoting.
    #[arg(long, value_name = "PATH")]
    pub humans_file: Option<PathBuf>,

    /// Pin the team slug instead of deriving it from `--team`. Use
    /// when the automatic 5-char abbreviation collides with an
    /// existing team or you want a specific identifier.
    #[arg(long, value_name = "SLUG")]
    pub team_slug: Option<String>,

    /// Pin the app/project slug instead of deriving it from `--app`.
    /// Same rationale as `--team-slug`.
    #[arg(long, value_name = "SLUG")]
    pub app_slug: Option<String>,

    /// Override agent roster. Comma-separated role names. When
    /// omitted, the CLI auto-detects from `<output>/.claude/agents/`
    /// (the agents already installed in the project) and falls back
    /// to the 10-agent canonical default for greenfield projects.
    #[arg(long, value_name = "ROLES", value_delimiter = ',')]
    pub agents: Option<Vec<String>>,

    /// Path to write `genasis.toml` + `.env.local`. Defaults to the
    /// current working directory.
    #[arg(long, value_name = "DIR")]
    pub output: Option<PathBuf>,

    /// Force non-interactive mode. Required values must come from
    /// flags / env / `--humans-file`; missing required values cause
    /// an immediate error instead of a stdin prompt. Use this in
    /// CI / automation.
    #[arg(long)]
    pub non_interactive: bool,

    /// Dry-run — compute slugs, print intended actions, but make no
    /// HTTP calls and write no files.
    #[arg(long)]
    pub dry_run: bool,
}

/// One human team-member spec: a display name and an email. The
/// email's local-part is what we'll suggest as the Plane / Mattermost
/// username (the REST adapters add a numeric suffix on conflict).
#[derive(Debug, Clone)]
pub struct HumanSpec {
    pub name: String,
    pub email: String,
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
    pub humans: Vec<HumanSpec>,
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
    print_plan(&plan, args.dry_run);

    if args.dry_run {
        println!("  [dry-run] skipping all API calls + file writes.");
        return Ok(());
    }

    if !args.non_interactive {
        let answer = prompt_required("Proceed with live provisioning? [y/N]", false)?;
        if !matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes") {
            bail!("aborted by user");
        }
    }

    // Try to load an existing snapshot at the target output dir so
    // we can hand the REST adapters ownership-assertion ids
    // (Option A in the slug-collision plan — a re-run from the
    // same secrets dir reuses our own resources; a brand-new run
    // with the same slug as somebody else's team rejects loudly).
    let prior_snapshot = {
        let snapshot_dir = if let Ok(root) = std::env::var("GENASIS_SECRETS_ROOT") {
            std::path::PathBuf::from(root)
                .join("teams")
                .join(&plan.team_slug)
        } else {
            plan.output_dir.clone()
        };
        let snapshot_path = snapshot_dir.join("genasis.toml.snapshot");
        if snapshot_path.is_file() {
            match crate::provision_writer::load_snapshot(&snapshot_path) {
                Ok(r) => Some(r),
                Err(e) => {
                    eprintln!(
                        "⚠ existing snapshot at {} failed to parse — ignoring: {e}",
                        snapshot_path.display()
                    );
                    None
                }
            }
        } else {
            None
        }
    };
    let expected_plane_project_id = prior_snapshot.as_ref().map(|r| r.plane.project_id.clone());
    let expected_mm_team_id = prior_snapshot
        .as_ref()
        .map(|r| r.mattermost.team_id.clone());
    let expected_mm_channel_id = prior_snapshot
        .as_ref()
        .map(|r| r.mattermost.scrum_channel_id.clone());

    let plane = PlaneClient::new(&plan.plane.url, &plan.plane.admin_token, "agentic")
        .context("build Plane client")?;
    let mm = MmClient::new(&plan.mattermost.url, &plan.mattermost.admin_token)
        .context("build Mattermost client")?;

    // Step 0 — auth probes. Fail fast before any side effects.
    println!("→ Plane: probing admin/...");
    let plane_me = plane.whoami().await.context("Plane whoami")?;
    println!(
        "  ✓ Plane sees you as {} <{}>",
        plane_me.display_name, plane_me.email
    );

    println!("→ Mattermost: probing admin...");
    let mm_me = mm.whoami().await.context("Mattermost whoami")?;
    println!(
        "  ✓ Mattermost sees you as {} <{}>",
        mm_me.username, mm_me.email
    );

    // Step 1 — Plane project + agent membership.
    let project_name = if plan.team_slug == plan.app_slug {
        plan.app_name.clone()
    } else {
        format!("{} — {}", plan.team_name, plan.app_name)
    };
    let project_identifier = uppercase_identifier(&plan.app_slug);
    println!("→ Plane: ensuring project {project_name:?} identifier={project_identifier}...");
    let (project, p_outcome) = plane
        .ensure_project(
            &project_name,
            &project_identifier,
            expected_plane_project_id.as_deref(),
        )
        .await?;
    println!("  ✓ project_id={} ({:?})", project.id, p_outcome);

    // Resolve agent users by display_name match against existing
    // workspace members. Plane CE has no REST user-create endpoint
    // accessible via workspace-scoped API keys; agents are expected
    // to be pre-registered out-of-band.
    let mut plane_agents: Vec<(
        String,
        Option<genasis_providers::plane::real_provisioner::WorkspaceMember>,
    )> = Vec::new();
    for role in &plan.agents {
        let m = plane.find_member_by_display_name(role).await?;
        if m.is_none() {
            println!(
                "  ⚠ Plane workspace has no agent user with display_name={role:?} — \
                 operator must register it once (admin UI / DB) before this agent can \
                 act on Plane issues."
            );
        }
        plane_agents.push((role.clone(), m));
    }

    // Attach each known agent + the inviting human to the project.
    for (role, member) in &plane_agents {
        if let Some(m) = member {
            let role_code = if role == "pm" {
                ROLE_ADMIN
            } else {
                ROLE_MEMBER
            };
            let o = plane
                .ensure_project_member(&project.id, &m.id, role_code)
                .await?;
            println!("  ✓ Plane project +{role} ({:?})", o);
        }
    }

    // Step 2 — Human invitations on Plane (workspace-level — Plane CE
    // has no direct project invite).
    let mut human_records: Vec<HumanRecord> = Vec::new();
    for h in &plan.humans {
        let (inv, o) = plane
            .ensure_workspace_invitation(&h.email, ROLE_MEMBER)
            .await?;
        let id = if inv.accepted {
            Some(inv.id.clone())
        } else {
            None
        };
        println!(
            "  ✓ Plane invite for {} <{}> ({:?}) accepted={}",
            h.name, h.email, o, inv.accepted
        );
        human_records.push(HumanRecord {
            name: h.name.clone(),
            email: h.email.clone(),
            username: derive_username(&h.email),
            plane_user_id: id,
            mm_user_id: None, // populated below
        });
    }

    // Step 3 — Mattermost team + scrum channel.
    let mm_team_name = format!("team-{}", plan.team_slug);
    println!("→ Mattermost: ensuring team {mm_team_name:?}...");
    let (team, t_o) = mm
        .ensure_team(
            &mm_team_name,
            &plan.team_name,
            expected_mm_team_id.as_deref(),
        )
        .await?;
    println!("  ✓ team_id={} ({:?})", team.id, t_o);

    let scrum_name = format!("scrum-{}", plan.app_slug);
    let scrum_display = format!("Scrum — {}", plan.app_name);
    println!("→ Mattermost: ensuring channel {scrum_name:?}...");
    let (channel, c_o) = mm
        .ensure_channel(
            &team.id,
            &scrum_name,
            &scrum_display,
            CHANNEL_OPEN,
            expected_mm_channel_id.as_deref(),
        )
        .await?;
    println!("  ✓ channel_id={} ({:?})", channel.id, c_o);

    // Step 4 — Mattermost agent users + PATs + team/channel membership.
    let mut agent_records: Vec<AgentRecord> = Vec::new();
    for (role, plane_member) in &plane_agents {
        let agent_email = format!("{role}-{}@genasis.bot", plan.team_slug);
        let agent_username = sanitize_mm_username(&format!("{role}-{}", plan.team_slug));
        let password = random_password();
        let (user, u_o) = mm
            .ensure_agent_user(&agent_email, &agent_username, &password)
            .await?;
        println!("  ✓ MM agent user {agent_email} ({:?})", u_o);

        let m_o = mm.ensure_team_member(&team.id, &user.id).await?;
        println!("    → team member ({:?})", m_o);
        let c_o = mm.ensure_channel_member(&channel.id, &user.id).await?;
        println!("    → channel member ({:?})", c_o);

        let pat = mm
            .issue_pat(&user.id, &format!("genasis-{}-{}", plan.team_slug, role))
            .await?;

        agent_records.push(AgentRecord {
            role: role.clone(),
            email: agent_email,
            plane_user_id: plane_member.as_ref().map(|m| m.id.clone()),
            plane_pat: None, // workspace-shared PAT model — no per-team PAT
            mm_user_id: user.id,
            mm_pat: pat.token,
        });
    }

    // Step 5 — Mattermost: invite each human by email + add existing
    // accounts to the team / channel.
    for h in human_records.iter_mut() {
        if let Some(user) = mm.user_by_email(&h.email).await? {
            mm.ensure_team_member(&team.id, &user.id).await?;
            mm.ensure_channel_member(&channel.id, &user.id).await?;
            h.mm_user_id = Some(user.id);
            println!(
                "  ✓ MM human {} already has account — added to team",
                h.email
            );
        } else {
            mm.invite_human_by_email(&team.id, &[h.email.clone()])
                .await?;
            println!("  ✓ MM invite emailed to {}", h.email);
        }
    }

    // Step 6 — write outputs.
    let record = ProvisionRecord {
        team_name: plan.team_name.clone(),
        team_slug: plan.team_slug.clone(),
        app_name: plan.app_name.clone(),
        app_slug: plan.app_slug.clone(),
        plane: PlaneRecord {
            url: plan.plane.url.clone(),
            workspace_slug: "agentic".to_string(),
            project_id: project.id.clone(),
            project_identifier: project.identifier.clone(),
            project_name: project.name.clone(),
        },
        mattermost: MattermostRecord {
            url: plan.mattermost.url.clone(),
            team_id: team.id.clone(),
            team_name: team.name.clone(),
            scrum_channel_id: channel.id.clone(),
            scrum_channel_name: channel.name.clone(),
        },
        humans: human_records,
        agents: agent_records,
    };
    let out_dir = write_all(&record, &plan.output_dir).context("write provision outputs")?;

    println!();
    println!("──────────────────────────────────────────────");
    println!("  ✓ provision complete");
    println!("──────────────────────────────────────────────");
    println!("  artifacts: {}", out_dir.display());
    println!("    - .env.local");
    println!("    - genasis.toml.snapshot");
    println!("    - provision.log");
    println!();
    println!(
        "  Plane project   : {}/{}",
        plan.plane.url, project.identifier
    );
    println!(
        "  Mattermost team : {}/{}/channels/{}",
        plan.mattermost.url, team.name, channel.name
    );
    println!();
    println!("  Next: `cd <project-dir> && genasis listen --real` to start the daemon.");

    // Suppress unused-variant warning for outcome captures.
    let _ = (ProjectCreateOutcome::Created, MmOutcome::Created);
    Ok(())
}

fn uppercase_identifier(slug: &str) -> String {
    let upper: String = slug
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect();
    // Plane caps identifier at ~12 chars; truncate defensively.
    upper.chars().take(12).collect()
}

fn sanitize_mm_username(s: &str) -> String {
    // Mattermost requires usernames to be 3-22 chars, lowercase
    // alphanumerics + `.`, `_`, `-`. We've already lowercased; just
    // strip anything else and clamp the length.
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
    // 24-char random hex. We never store this — agents authenticate
    // via the PAT — but Mattermost still requires the user-create
    // call to supply one.
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let pid = std::process::id();
    let mut s = format!("{nanos:x}{pid:x}");
    // Mattermost requires at least one number and lowercase — our
    // hex output covers both — but also needs an uppercase + symbol
    // when password complexity is enabled. Suffix with a constant
    // satisfies the policy without bloating randomness materially.
    s.push_str("Z!");
    s
}

fn print_plan(plan: &ResolvedProvisionPlan, dry_run: bool) {
    println!("──────────────────────────────────────────────");
    println!("  genasis provision — plan");
    println!("──────────────────────────────────────────────");
    println!("  team       : {} ({})", plan.team_name, plan.team_slug);
    println!("  app        : {} ({})", plan.app_name, plan.app_slug);
    println!("  humans     :");
    for h in &plan.humans {
        let username = derive_username(&h.email);
        println!("    - {} <{}>  → username {}", h.name, h.email, username);
    }
    println!("  agents     :");
    for agent in &plan.agents {
        println!(
            "    - {:<14} → {}-{}@{}",
            agent, agent, plan.team_slug, AGENT_EMAIL_DOMAIN
        );
    }
    println!("  plane URL  : {}", plan.plane.url);
    println!("  mm URL     : {}", plan.mattermost.url);
    println!("  output dir : {}", plan.output_dir.display());
    println!(
        "  mode       : {}",
        if dry_run { "DRY-RUN" } else { "LIVE" }
    );
    println!();
}

fn resolve_plan(args: &Args) -> Result<ResolvedProvisionPlan> {
    // 1) team / app names. Either come from flags or, in interactive
    //    mode, are prompted from stdin.
    let team_name = match &args.team {
        Some(t) if !t.trim().is_empty() => t.trim().to_string(),
        _ => prompt_required("Team name (e.g. 'Marketing Squad')", args.non_interactive)?,
    };
    let app_name = match &args.app {
        Some(a) if !a.trim().is_empty() => a.trim().to_string(),
        _ => prompt_required(
            "App / project name (e.g. 'Quiz Demo')",
            args.non_interactive,
        )?,
    };

    // Slugs default to the abbreviation of the human-readable name
    // but the caller can override either with the explicit flag.
    let team_slug = match &args.team_slug {
        Some(s) if !s.trim().is_empty() => s.trim().to_ascii_lowercase(),
        _ => slugify_abbrev(&team_name),
    };
    let app_slug = match &args.app_slug {
        Some(s) if !s.trim().is_empty() => s.trim().to_ascii_lowercase(),
        _ => slugify_abbrev(&app_name),
    };

    // 2) humans. Three input paths in priority order: --humans-file
    //    (JSON batch) > --humans (inline `"Name <email>"`) > stdin
    //    prompts. The empty default is rejected — every team needs at
    //    least one human owner.
    let humans = if let Some(path) = &args.humans_file {
        parse_humans_file(path)?
    } else if !args.humans.is_empty() {
        args.humans
            .iter()
            .map(|spec| parse_human_spec(spec))
            .collect::<Result<Vec<_>>>()?
    } else {
        prompt_humans(args.non_interactive)?
    };
    if humans.is_empty() {
        bail!("at least one human team-member is required (Plane + Mattermost both need a real account to own the team)");
    }

    let output_dir = match &args.output {
        Some(p) => p.clone(),
        None => std::env::current_dir()
            .context("unable to read current working directory for --output default")?,
    };

    // Agent roster resolution — priority order:
    //   1. `--agents` flag (explicit override).
    //   2. `<output_dir>/.claude/agents/*.md` (auto-detect what the
    //      project already has installed via `genasis init` /
    //      `bootstrap` / `agents install`). Backup files
    //      (*.genasis.bak.*) are skipped.
    //   3. The 10-agent canonical default (greenfield).
    let agents: Vec<String> = if let Some(explicit) = &args.agents {
        explicit.clone()
    } else if let Some(detected) = detect_agents_from_dir(&output_dir) {
        if detected.is_empty() {
            DEFAULT_AGENTS.iter().map(|s| s.to_string()).collect()
        } else {
            detected
        }
    } else {
        DEFAULT_AGENTS.iter().map(|s| s.to_string()).collect()
    };
    if agents.is_empty() {
        bail!("--agents must list at least one role; omit the flag to use the auto-detected or 10-agent default");
    }

    let plane = read_endpoint("PLANE_URL", "PLANE_ADMIN_TOKEN")
        .context("Plane admin credentials missing from environment")?;
    let mattermost = read_endpoint("MM_URL", "MM_ADMIN_TOKEN")
        .context("Mattermost admin credentials missing from environment")?;

    Ok(ResolvedProvisionPlan {
        team_name,
        team_slug,
        app_name,
        app_slug,
        humans,
        agents,
        output_dir,
        plane,
        mattermost,
    })
}

/// Parse one `--humans` token of the form `"Bravo Kim <gnoopy@gmail.com>"`.
/// A bare email like `gnoopy@gmail.com` is also accepted — the name
/// defaults to the local-part with first letter uppercased.
pub fn parse_human_spec(spec: &str) -> Result<HumanSpec> {
    let s = spec.trim();
    if s.is_empty() {
        bail!("empty human spec");
    }
    if let Some(open) = s.find('<') {
        let close = s.rfind('>').ok_or_else(|| {
            anyhow!("human spec {s:?} has '<' but no closing '>' — expected `Name <email>` form")
        })?;
        if close <= open {
            bail!("human spec {s:?} has '>' before '<' — expected `Name <email>` form");
        }
        let name = s[..open].trim().to_string();
        let email = s[open + 1..close].trim().to_string();
        if name.is_empty() {
            bail!("human spec {s:?} is missing the display name before '<'");
        }
        if !is_plausible_email(&email) {
            bail!("human spec {s:?} has an unparseable email {email:?}");
        }
        Ok(HumanSpec { name, email })
    } else if is_plausible_email(s) {
        let local = s.split('@').next().unwrap_or(s);
        let mut chars = local.chars();
        let name = match chars.next() {
            Some(c) => c.to_uppercase().chain(chars).collect(),
            None => local.to_string(),
        };
        Ok(HumanSpec {
            name,
            email: s.to_string(),
        })
    } else {
        bail!("human spec {s:?} should be either `Name <email@x.com>` or just `email@x.com`")
    }
}

fn parse_humans_file(path: &PathBuf) -> Result<Vec<HumanSpec>> {
    let body = std::fs::read_to_string(path)
        .with_context(|| format!("read --humans-file {}", path.display()))?;
    #[derive(serde::Deserialize)]
    struct Entry {
        name: String,
        email: String,
    }
    let entries: Vec<Entry> = serde_json::from_str(&body).with_context(|| {
        format!(
            "--humans-file {} must be a JSON array of {{name,email}}",
            path.display()
        )
    })?;
    entries
        .into_iter()
        .map(|e| {
            if !is_plausible_email(&e.email) {
                bail!(
                    "entry in {} has unparseable email {:?}",
                    path.display(),
                    e.email
                );
            }
            Ok(HumanSpec {
                name: e.name,
                email: e.email,
            })
        })
        .collect()
}

fn prompt_humans(non_interactive: bool) -> Result<Vec<HumanSpec>> {
    if non_interactive {
        bail!(
            "--non-interactive set but no --humans / --humans-file given. \
             Provide at least one human team-member."
        );
    }
    println!("Add human team-members. Press Enter on a blank name to stop.");
    let mut out: Vec<HumanSpec> = Vec::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    loop {
        let idx = out.len() + 1;
        let name = read_line(&mut handle, &format!("  [{idx}] Name (blank = done): "))?;
        if name.trim().is_empty() {
            if out.is_empty() {
                println!("  (no humans added — you need at least one. Try again.)");
                continue;
            }
            break;
        }
        let email = read_line(&mut handle, &format!("       Email for {name}: "))?;
        let email = email.trim().to_string();
        if !is_plausible_email(&email) {
            println!("       {email:?} doesn't look like an email — try again.");
            continue;
        }
        out.push(HumanSpec {
            name: name.trim().to_string(),
            email,
        });
    }
    Ok(out)
}

fn prompt_required(label: &str, non_interactive: bool) -> Result<String> {
    if non_interactive {
        bail!("--non-interactive set but `{label}` not given as a flag");
    }
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    loop {
        let line = read_line(&mut handle, &format!("{label}: "))?;
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
        println!("  (this value is required — try again)");
    }
}

fn read_line<R: BufRead>(r: &mut R, prompt: &str) -> Result<String> {
    print!("{prompt}");
    io::stdout().flush().ok();
    let mut buf = String::new();
    r.read_line(&mut buf).context("read stdin")?;
    Ok(buf)
}

/// Lightweight email shape check — we don't need RFC 5322 here, just
/// enough to catch obvious typos like a missing `@`. Plane and
/// Mattermost both validate properly on their own side.
fn is_plausible_email(s: &str) -> bool {
    let s = s.trim();
    let parts: Vec<&str> = s.splitn(2, '@').collect();
    if parts.len() != 2 {
        return false;
    }
    let (local, domain) = (parts[0], parts[1]);
    !local.is_empty() && domain.contains('.') && !domain.starts_with('.')
}

/// Inspect `<dir>/.claude/agents/` and return the list of agent
/// role names found there (filename stem of every `.md` file,
/// excluding genasis backup files). Returns `None` if the directory
/// doesn't exist — caller treats that as "fall back to defaults".
///
/// Filenames are mapped straight to role names — `code-reviewer.md`
/// becomes `code-reviewer`, `pm.md` becomes `pm`. The `.genasis.bak.*`
/// backups that overlay produces are skipped so we don't
/// double-count.
pub fn detect_agents_from_dir(dir: &std::path::Path) -> Option<Vec<String>> {
    let agents_dir = dir.join(".claude").join("agents");
    if !agents_dir.is_dir() {
        return None;
    }
    let mut roles: Vec<String> = Vec::new();
    let entries = std::fs::read_dir(&agents_dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };
        // Skip backups like `pm.md.genasis.bak.1778787786`.
        if name.contains(".genasis.bak.") {
            continue;
        }
        if let Some(stem) = name.strip_suffix(".md") {
            if !stem.is_empty() && !roles.iter().any(|r| r == stem) {
                roles.push(stem.to_string());
            }
        }
    }
    roles.sort();
    Some(roles)
}

/// Derive a Plane / Mattermost username suggestion from an email.
/// The REST adapters take this as a starting point and append a
/// numeric suffix on collision.
pub fn derive_username(email: &str) -> String {
    let local = email.split('@').next().unwrap_or(email);
    local
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_spec_name_and_email() {
        let h = parse_human_spec("Bravo Kim <gnoopy@gmail.com>").unwrap();
        assert_eq!(h.name, "Bravo Kim");
        assert_eq!(h.email, "gnoopy@gmail.com");
    }

    #[test]
    fn human_spec_bare_email_derives_name() {
        let h = parse_human_spec("alice@example.com").unwrap();
        assert_eq!(h.name, "Alice");
        assert_eq!(h.email, "alice@example.com");
    }

    #[test]
    fn human_spec_rejects_missing_angle_close() {
        assert!(parse_human_spec("Alice <alice@x.com").is_err());
    }

    #[test]
    fn human_spec_rejects_bad_email() {
        assert!(parse_human_spec("Bad <not-an-email>").is_err());
        assert!(parse_human_spec("just-a-word").is_err());
    }

    #[test]
    fn username_lowercases_and_replaces_punctuation() {
        assert_eq!(derive_username("GNoopy@gmail.com"), "gnoopy");
        assert_eq!(derive_username("first.last+tag@x.com"), "first_last_tag");
        assert_eq!(derive_username("UPPER@x.com"), "upper");
    }

    #[test]
    fn detect_agents_from_claude_agents_dir() {
        let dir = tempfile::tempdir().unwrap();
        let agents = dir.path().join(".claude").join("agents");
        std::fs::create_dir_all(&agents).unwrap();
        std::fs::write(agents.join("pm.md"), "stub").unwrap();
        std::fs::write(agents.join("code-reviewer.md"), "stub").unwrap();
        std::fs::write(agents.join("designer.md"), "stub").unwrap();
        // Backup files must be ignored.
        std::fs::write(agents.join("pm.md.genasis.bak.123"), "stub").unwrap();
        // Non-md files must be ignored.
        std::fs::write(agents.join("README"), "stub").unwrap();

        let detected = detect_agents_from_dir(dir.path()).unwrap();
        assert_eq!(detected, vec!["code-reviewer", "designer", "pm"]);
    }

    #[test]
    fn detect_agents_returns_none_when_dir_missing() {
        let dir = tempfile::tempdir().unwrap();
        assert!(detect_agents_from_dir(dir.path()).is_none());
    }

    #[test]
    fn email_validator_basic_cases() {
        assert!(is_plausible_email("a@b.co"));
        assert!(!is_plausible_email("no-at"));
        assert!(!is_plausible_email("a@b"));
        assert!(!is_plausible_email("@b.co"));
        assert!(!is_plausible_email("a@.co"));
    }
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
