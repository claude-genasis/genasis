//! `genasis humans` — CRUD over the `[[humans]]` roster in
//! `genasis.toml`, plus `sync` which calls Mattermost / Plane to
//! provision (or re-confirm) every entry.
//!
//! See ADR-014. The CLI exists alongside the TUI wizard's Humans step
//! so scripts and CI can manipulate the roster non-interactively.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use genasis_core::config::{Config, HumanEntry, HumansLock, CONFIG_FILE_NAME};
use genasis_providers::mattermost::{self, HumanUserSpec};
use genasis_providers::plane::user_provisioner::{
    provision as plane_provision, HumanRequest as PlaneHumanRequest, ProvisionInput,
};

#[derive(Parser, Debug)]
pub struct Args {
    /// Project root. Defaults to the current working directory's
    /// `genasis.toml` walk.
    #[arg(long, value_name = "DIR", global = true)]
    pub project: Option<PathBuf>,

    #[command(subcommand)]
    pub action: Action,
}

#[derive(Subcommand, Debug)]
pub enum Action {
    /// List the project's human members.
    List,
    /// Add or replace a human (idempotent on email).
    Add {
        #[arg(long)]
        email: String,
        #[arg(long)]
        name: String,
        #[arg(long, default_value = "stakeholder")]
        role: String,
        #[arg(long, default_value = "")]
        mm_username: String,
        #[arg(long, default_value = "")]
        locale: String,
        /// After updating `genasis.toml`, immediately call provisioning.
        #[arg(long)]
        sync: bool,
    },
    /// Edit only the fields that are passed; unspecified fields are
    /// left untouched.
    Edit {
        #[arg(long)]
        email: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        role: Option<String>,
        #[arg(long)]
        mm_username: Option<String>,
        #[arg(long)]
        locale: Option<String>,
        #[arg(long)]
        sync: bool,
    },
    /// Remove a human by email. Does NOT delete the underlying
    /// Mattermost / Plane account — use the upstream tool's UI for
    /// that.
    Remove {
        #[arg(long)]
        email: String,
    },
    /// Provision every roster entry into Mattermost + Plane (idempotent).
    Sync {
        /// Run only the Mattermost half.
        #[arg(long)]
        mm_only: bool,
        /// Run only the Plane half.
        #[arg(long)]
        plane_only: bool,
        /// Print the plan without making changes.
        #[arg(long)]
        dry_run: bool,
    },
}

pub async fn run(args: Args) -> Result<()> {
    let project_root = resolve_root(args.project.as_deref())?;
    let cfg_path = project_root.join(CONFIG_FILE_NAME);
    let mut cfg = if cfg_path.is_file() {
        Config::load(&cfg_path)?
    } else {
        anyhow::bail!(
            "no genasis.toml at {} — run `genasis init` first",
            cfg_path.display()
        );
    };

    match args.action {
        Action::List => {
            list_humans(&cfg, &project_root)?;
        }
        Action::Add {
            email,
            name,
            role,
            mm_username,
            locale,
            sync,
        } => {
            let entry = HumanEntry {
                name,
                email: email.trim().to_ascii_lowercase(),
                role,
                mm_username,
                locale,
            };
            let added = cfg.upsert_human(entry.clone());
            cfg.save(&cfg_path)?;
            println!(
                "{} {} ({})",
                if added { "+ added" } else { "~ replaced" },
                entry.email,
                entry.name
            );
            if sync {
                sync_humans(&cfg, &project_root, false, false, false).await?;
            }
        }
        Action::Edit {
            email,
            name,
            role,
            mm_username,
            locale,
            sync,
        } => {
            let key = email.trim().to_ascii_lowercase();
            let mut current = cfg
                .find_human(&key)
                .cloned()
                .with_context(|| format!("no human with email `{key}`"))?;
            if let Some(n) = name {
                current.name = n;
            }
            if let Some(r) = role {
                current.role = r;
            }
            if let Some(u) = mm_username {
                current.mm_username = u;
            }
            if let Some(l) = locale {
                current.locale = l;
            }
            cfg.upsert_human(current.clone());
            cfg.save(&cfg_path)?;
            println!("~ edited {} ({})", current.email, current.name);
            if sync {
                sync_humans(&cfg, &project_root, false, false, false).await?;
            }
        }
        Action::Remove { email } => {
            let removed = cfg.remove_human(&email);
            if removed {
                cfg.save(&cfg_path)?;
                let mut lock = HumansLock::load(&project_root)?;
                if lock.remove(&email) {
                    lock.save(&project_root)?;
                }
                println!("- removed {email}");
            } else {
                println!("(no-op) no entry for {email}");
            }
        }
        Action::Sync {
            mm_only,
            plane_only,
            dry_run,
        } => {
            sync_humans(&cfg, &project_root, mm_only, plane_only, dry_run).await?;
        }
    }

    Ok(())
}

fn list_humans(cfg: &Config, project_root: &Path) -> Result<()> {
    let lock = HumansLock::load(project_root).unwrap_or_default();
    if cfg.humans.is_empty() {
        println!("(empty) no humans registered. Add one with:");
        println!("  genasis humans add --email you@example.com --name 'You'");
        return Ok(());
    }
    println!(
        "{:<24} {:<28} {:<14} {:<14} {}",
        "name", "email", "role", "mm_username", "provisioned"
    );
    for h in &cfg.humans {
        let provisioned = lock
            .find(&h.email)
            .map(|e| {
                if !e.mm_user_id.is_empty() && !e.plane_user_id.is_empty() {
                    "mm+plane"
                } else if !e.mm_user_id.is_empty() {
                    "mm"
                } else if !e.plane_user_id.is_empty() {
                    "plane"
                } else {
                    "no"
                }
            })
            .unwrap_or("no");
        let mm_un = if h.mm_username.is_empty() {
            h.effective_mm_username()
        } else {
            h.mm_username.clone()
        };
        println!(
            "{:<24} {:<28} {:<14} {:<14} {}",
            truncate(&h.name, 24),
            truncate(&h.email, 28),
            truncate(&h.role, 14),
            truncate(&mm_un, 14),
            provisioned
        );
    }
    Ok(())
}

async fn sync_humans(
    cfg: &Config,
    project_root: &Path,
    mm_only: bool,
    plane_only: bool,
    dry_run: bool,
) -> Result<()> {
    if cfg.humans.is_empty() {
        println!("(no-op) no humans to sync");
        return Ok(());
    }

    if mm_only && plane_only {
        anyhow::bail!("--mm-only and --plane-only are mutually exclusive");
    }

    println!(
        "→ syncing {} human(s){}",
        cfg.humans.len(),
        if dry_run { " (dry-run)" } else { "" }
    );
    let mut lock = HumansLock::load(project_root)?;

    if !plane_only {
        sync_humans_mattermost(cfg, &mut lock, dry_run).await?;
    }
    if !mm_only {
        sync_humans_plane(cfg, &mut lock, dry_run).await?;
    }

    if !dry_run {
        lock.save(project_root)?;
        println!(
            "  wrote {}",
            project_root.join(HumansLock::FILE_NAME).display()
        );
    }
    Ok(())
}

async fn sync_humans_mattermost(cfg: &Config, lock: &mut HumansLock, dry_run: bool) -> Result<()> {
    let Some(mm_cfg) = cfg.mattermost.as_ref() else {
        println!("  ⚠ [mattermost] section missing — skipping MM provisioning");
        return Ok(());
    };
    let mm_flavor = mattermost::FlavorChoice::parse(&mm_cfg.flavor)?;
    // Trial flavor draws URL/secret from [trial]; real flavors require
    // MM_ADMIN_TOKEN.
    let mm_token = if mm_flavor == mattermost::FlavorChoice::Trial {
        String::new()
    } else {
        match std::env::var("MM_ADMIN_TOKEN") {
            Ok(t) => t,
            Err(_) => {
                println!("  ⚠ MM_ADMIN_TOKEN not set — skipping MM provisioning");
                return Ok(());
            }
        }
    };
    let team_id = std::env::var("MM_TEAM_ID").ok();
    let client = mattermost::build(mm_flavor, &mm_cfg.url, &mm_token, cfg.trial.as_ref()).await?;

    println!("  Mattermost ({}):", mm_cfg.url);
    for h in &cfg.humans {
        let username = h.effective_mm_username();
        let display = if h.name.is_empty() {
            username.clone()
        } else {
            h.name.clone()
        };
        if dry_run {
            println!(
                "    plan: ensure_human_user email={} username={username}",
                h.email
            );
            continue;
        }
        let spec = HumanUserSpec {
            email: h.email.clone(),
            username: username.clone(),
            display_name: display.clone(),
            first_name: h.name.clone(),
            last_name: String::new(),
            locale: h.locale.clone(),
        };
        match client.ensure_human_user(&spec, team_id.as_deref()).await {
            Ok(refout) => {
                let was_new = refout.temp_password.is_some();
                println!(
                    "    {} {} → user_id={}{}",
                    if was_new { "+ created" } else { "= existing" },
                    h.email,
                    refout.user_id,
                    if was_new {
                        format!(" temp_password=<recorded in {}>", HumansLock::FILE_NAME)
                    } else {
                        String::new()
                    }
                );
                let mut entry = lock.find(&h.email).cloned().unwrap_or_default();
                entry.email = h.email.clone();
                entry.mm_user_id = refout.user_id;
                entry.mm_username = refout.username;
                if let Some(p) = refout.temp_password {
                    entry.mm_temp_password = p;
                }
                entry.provisioned_at = now_iso();
                lock.upsert(entry);
            }
            Err(e) => {
                println!("    ✗ {} → {e}", h.email);
            }
        }
    }
    Ok(())
}

async fn sync_humans_plane(cfg: &Config, lock: &mut HumansLock, dry_run: bool) -> Result<()> {
    let Some(plane_cfg) = cfg.plane.as_ref() else {
        println!("  ⚠ [plane] section missing — skipping Plane provisioning");
        return Ok(());
    };
    let admin_email = std::env::var("PLANE_ADMIN_EMAIL").unwrap_or_default();
    let admin_password = std::env::var("PLANE_ADMIN_PASSWORD").unwrap_or_default();
    if admin_email.is_empty() || admin_password.is_empty() {
        println!(
            "  ⚠ PLANE_ADMIN_EMAIL / PLANE_ADMIN_PASSWORD not set — skipping Plane provisioning"
        );
        return Ok(());
    }

    let humans: Vec<PlaneHumanRequest> = cfg
        .humans
        .iter()
        .map(|h| PlaneHumanRequest {
            name: h.name.clone(),
            email: h.email.clone(),
            role: h.role.clone(),
            plane_role: "Member".to_string(),
        })
        .collect();

    println!("  Plane ({}):", plane_cfg.url);
    if dry_run {
        for h in &humans {
            println!(
                "    plan: invite {} as Member of {}",
                h.email, plane_cfg.workspace_slug
            );
        }
        return Ok(());
    }

    let script = locate_plane_script()?;
    let input = ProvisionInput {
        plane_url: plane_cfg.url.clone(),
        workspace_slug: plane_cfg.workspace_slug.clone(),
        admin_email,
        admin_password,
        agents: Vec::new(),
        humans,
    };
    let out = plane_provision(&script, &input).await?;
    if out.status == "error" {
        println!(
            "    ✗ provisioner error: {}",
            out.error.unwrap_or_else(|| "unknown".into())
        );
        return Ok(());
    }
    if out.status == "stub" {
        println!(
            "    ~ provisioner stub (Playwright UI port pending) — recorded {} placeholder ID(s)",
            out.humans.len()
        );
    }
    for h in out.humans {
        let mut entry = lock.find(&h.email).cloned().unwrap_or_default();
        entry.email = h.email.clone();
        entry.plane_user_id = h.user_id.clone();
        entry.provisioned_at = now_iso();
        lock.upsert(entry);
        println!(
            "    {} {} → plane_user_id={}",
            if h.status == "joined" { "=" } else { "+" },
            h.email,
            h.user_id
        );
    }
    Ok(())
}

fn locate_plane_script() -> Result<PathBuf> {
    // 1. Allow override.
    if let Ok(p) = std::env::var("GENASIS_PLANE_SCRIPT") {
        let pb = PathBuf::from(p);
        if pb.is_file() {
            return Ok(pb);
        }
    }
    // 2. Probe alongside the binary (release layout).
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let cand = parent.join("scripts/provision-plane-users.mjs");
            if cand.is_file() {
                return Ok(cand);
            }
        }
    }
    // 3. Repo layout (development).
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/provision-plane-users.mjs");
    if repo.is_file() {
        return Ok(repo);
    }
    anyhow::bail!("provision-plane-users.mjs not found; set GENASIS_PLANE_SCRIPT")
}

fn resolve_root(arg: Option<&Path>) -> Result<PathBuf> {
    if let Some(p) = arg {
        return p
            .canonicalize()
            .with_context(|| format!("--project path does not exist: {}", p.display()));
    }
    let cwd = std::env::current_dir()?;
    if let Some(cfg) = Config::discover(&cwd) {
        if let Some(parent) = cfg.parent() {
            return Ok(parent.to_path_buf());
        }
    }
    Ok(cwd)
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(n.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}

fn now_iso() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_keeps_short() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn truncate_marks_overflow() {
        let t = truncate("abcdefghij", 5);
        assert_eq!(t.chars().count(), 5);
        assert!(t.ends_with('…'));
    }

    #[test]
    fn iso_timestamp_is_well_formed() {
        let ts = now_iso();
        assert!(ts.ends_with('Z'));
        assert_eq!(ts.len(), 20);
        assert_eq!(&ts[4..5], "-");
        assert_eq!(&ts[7..8], "-");
        assert_eq!(&ts[10..11], "T");
    }
}
