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

    /// Override the 10-agent default. Comma-separated role names.
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

    // M-v6.x: real Plane + Mattermost provisioners land in their own
    // commits — this scaffolding commit only validates the plan and
    // wires the clap surface so the next change has a clean target.
    bail!(
        "live provisioning not implemented yet — re-run with `--dry-run` to \
         preview the plan, or wait for the upcoming alpha that ships the \
         Plane + Mattermost REST adapters."
    );
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
    println!("  mode       : {}", if dry_run { "DRY-RUN" } else { "LIVE" });
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

    let team_slug = slugify_abbrev(&team_name);
    let app_slug = slugify_abbrev(&app_name);

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

    let agents: Vec<String> = args
        .agents
        .clone()
        .unwrap_or_else(|| DEFAULT_AGENTS.iter().map(|s| s.to_string()).collect());
    if agents.is_empty() {
        bail!("--agents must list at least one role; omit the flag to use the 10-agent default");
    }

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
    let entries: Vec<Entry> = serde_json::from_str(&body)
        .with_context(|| format!("--humans-file {} must be a JSON array of {{name,email}}", path.display()))?;
    entries
        .into_iter()
        .map(|e| {
            if !is_plausible_email(&e.email) {
                bail!("entry in {} has unparseable email {:?}", path.display(), e.email);
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

/// Derive a Plane / Mattermost username suggestion from an email.
/// The REST adapters take this as a starting point and append a
/// numeric suffix on collision.
pub fn derive_username(email: &str) -> String {
    let local = email.split('@').next().unwrap_or(email);
    local
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '_' })
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
