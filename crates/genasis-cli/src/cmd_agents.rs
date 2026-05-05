//! `genasis agents` subcommand — manage the agents catalog.
//!
//! ADR-011: agents catalog is fetched from GitHub Releases, cached locally,
//! and version-pinned in `genasis.toml [agents].version`.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use genasis_i18n::tr;
use genasis_templates::{cache, registry, store::AgentStore};

#[derive(Parser, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub command: AgentsCommand,
}

#[derive(Subcommand, Debug)]
pub enum AgentsCommand {
    /// Fetch the agents catalog (download + cache).
    Fetch {
        /// Version to fetch. Defaults to the version pinned in genasis.toml.
        #[arg(long)]
        version: Option<String>,
    },
    /// Show pinned version, cached versions, and latest available.
    Status,
    /// Fetch the latest version and update genasis.toml pin.
    Update,
    /// List agents in the current catalog.
    List,
}

pub fn run(args: Args) -> Result<()> {
    match args.command {
        AgentsCommand::Fetch { version } => cmd_fetch(version),
        AgentsCommand::Status => cmd_status(),
        AgentsCommand::Update => cmd_update(),
        AgentsCommand::List => cmd_list(),
    }
}

fn resolve_config() -> Result<(String, String, String)> {
    // Try to load genasis.toml from current dir for [agents] config.
    // Defaults if not found.
    let version = std::env::var("GENASIS_AGENTS_VERSION")
        .unwrap_or_else(|_| "1.0.0".to_string());
    let registry = std::env::var("GENASIS_AGENTS_REGISTRY").unwrap_or_else(|_| {
        "https://github.com/claude-genasis/genasis/releases".to_string()
    });
    let cache_dir = std::env::var("GENASIS_AGENTS_CACHE_DIR").unwrap_or_default();
    Ok((version, registry, cache_dir))
}

fn cmd_fetch(version_override: Option<String>) -> Result<()> {
    let (pinned, registry_url, cache_override) = resolve_config()?;
    let version = version_override.unwrap_or(pinned);

    if cache::is_cached(&version, &cache_override)? {
        println!("agents catalog v{version} already cached.");
        return Ok(());
    }

    println!("Fetching agents catalog v{version} from registry...");
    let tarball = registry::fetch_tarball(&registry_url, &version)
        .context("failed to fetch agents catalog")?;
    let dir = cache::store_tarball(&version, &cache_override, &tarball)?;
    println!("Cached at: {}", dir.display());
    Ok(())
}

fn cmd_status() -> Result<()> {
    let (pinned, registry_url, cache_override) = resolve_config()?;

    println!("Pinned version: {pinned}");
    let cached = cache::is_cached(&pinned, &cache_override)?;
    println!("Cached: {}", if cached { "yes" } else { "no" });

    let all_cached = cache::list_cached(&cache_override)?;
    if !all_cached.is_empty() {
        println!("All cached versions: {}", all_cached.join(", "));
    }

    match registry::check_latest(&registry_url) {
        Ok(latest) => {
            println!("Latest available: {latest}");
            if latest != pinned {
                println!("  → Update available! Run `genasis agents update`.");
            }
        }
        Err(e) => println!("Could not check latest: {e}"),
    }
    Ok(())
}

fn cmd_update() -> Result<()> {
    let (_pinned, registry_url, cache_override) = resolve_config()?;

    println!("Checking latest agents catalog version...");
    let latest = registry::check_latest(&registry_url)
        .context("failed to check latest version")?;

    if cache::is_cached(&latest, &cache_override)? {
        println!("agents catalog v{latest} already cached.");
    } else {
        println!("Fetching agents catalog v{latest}...");
        let tarball = registry::fetch_tarball(&registry_url, &latest)?;
        cache::store_tarball(&latest, &cache_override, &tarball)?;
        println!("Cached v{latest}.");
    }

    println!("\nUpdate genasis.toml [agents].version to \"{latest}\" to pin this version.");
    // TODO: auto-update genasis.toml when config write support is added.
    Ok(())
}

fn cmd_list() -> Result<()> {
    let (pinned, _registry_url, cache_override) = resolve_config()?;

    if !cache::is_cached(&pinned, &cache_override)? {
        anyhow::bail!(
            "agents catalog v{pinned} not cached. Run `genasis agents fetch` first."
        );
    }

    let dir = cache::cache_dir(&pinned, &cache_override)?;
    let store = AgentStore::from_dir(dir)?;
    let agents = store.list_base_agents()?;

    println!("Agents catalog v{pinned} — {} base agents:", agents.len());
    for name in &agents {
        println!("  • {}", name.trim_end_matches(".md"));
    }
    Ok(())
}
