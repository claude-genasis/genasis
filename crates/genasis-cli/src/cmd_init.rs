use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use genasis_core::config::{Config, CONFIG_FILE_NAME};
use genasis_providers::{mattermost, plane};

#[derive(Parser, Debug)]
pub struct Args {
    /// Project root. Defaults to the current working directory.
    #[arg(long, value_name = "DIR")]
    pub project: Option<PathBuf>,

    /// Stop after pinging Plane / Mattermost — do not provision anything.
    #[arg(long)]
    pub probe_only: bool,
}

pub async fn run(args: Args) -> Result<()> {
    let project_root = resolve_project_root(args.project.as_deref())?;
    let cfg_path = project_root.join(CONFIG_FILE_NAME);
    let cfg = if cfg_path.is_file() {
        Config::load(&cfg_path)?
    } else {
        anyhow::bail!(
            "no genasis.toml at {} — copy templates/genasis.toml.tera and fill it in",
            cfg_path.display()
        );
    };

    let plane_cfg = cfg.plane.as_ref().context("[plane] section missing")?;
    let mm_cfg = cfg.mattermost.as_ref().context("[mattermost] section missing")?;

    let plane_token = std::env::var("PLANE_API_KEY")
        .or_else(|_| std::env::var("PLANE_TOKEN_PM"))
        .context("PLANE_API_KEY (or PLANE_TOKEN_PM) not set in environment")?;
    let mm_token = std::env::var("MM_ADMIN_TOKEN")
        .context("MM_ADMIN_TOKEN not set in environment")?;

    let plane_flavor = plane::FlavorChoice::parse(&plane_cfg.flavor)?;
    let mm_flavor = mattermost::FlavorChoice::parse(&mm_cfg.flavor)?;

    println!("→ resolving Plane flavor ({})…", plane_cfg.flavor);
    let plane_client = plane::build(plane_flavor, &plane_cfg.url, &plane_cfg.workspace_slug, &plane_token).await?;
    let plane_health = plane_client.health().await?;
    println!("  plane health: {}", short_json(&plane_health));

    println!("→ resolving Mattermost flavor ({})…", mm_cfg.flavor);
    let mm_client = mattermost::build(mm_flavor, &mm_cfg.url, &mm_token).await?;
    let mm_ping = mm_client.ping().await?;
    println!("  mattermost ping: {}", short_json(&mm_ping));

    if args.probe_only {
        println!("\nprobe-only: skipping provisioning. Re-run without --probe-only when ready.");
        return Ok(());
    }

    println!("\n→ ensuring Plane project ({})…", cfg.project.name);
    let project_id = plane_client
        .ensure_project(&cfg.project.name, &slug_to_identifier(&cfg.project.name))
        .await?;
    println!("  plane project_id = {project_id}");

    let labels = [
        ("BUG", "#FF0000"),
        ("FEATURE", "#22C55E"),
        ("IMPROVEMENT", "#3B82F6"),
        ("QUESTION", "#EAB308"),
    ];
    for (name, color) in labels {
        let l = plane_client.ensure_label(&project_id, name, color).await?;
        println!("  plane label {name} → {}", l.id);
    }

    let scrum_channel = format!("scrum-{}", cfg.project.name);
    println!("\n→ ensuring Mattermost #{scrum_channel}…");
    let team_id = std::env::var("MM_TEAM_ID").unwrap_or_default();
    if !team_id.is_empty() {
        let ch = mm_client
            .ensure_channel(&team_id, &scrum_channel, &scrum_channel)
            .await?;
        println!("  mm channel = {} ({})", ch.name, ch.id);
    } else {
        println!("  (skipped: MM_TEAM_ID not in environment — set it before running cmd_init for full provisioning)");
    }

    println!("\nnext: run `genasis attach` to bolt the overlay onto your existing agents.");
    Ok(())
}

fn resolve_project_root(arg: Option<&std::path::Path>) -> Result<PathBuf> {
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

fn slug_to_identifier(name: &str) -> String {
    let upper: String = name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect();
    if upper.len() >= 3 {
        upper.chars().take(4).collect()
    } else {
        format!("{upper:0<3}")
    }
}

fn short_json(v: &serde_json::Value) -> String {
    let s = v.to_string();
    if s.len() > 200 {
        format!("{}…", &s[..200])
    } else {
        s
    }
}
