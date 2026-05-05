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
    /// Browse and install agents interactively (TUI).
    Browse,
    /// Install a specific agent by name (or a preset).
    Install {
        /// Agent name (e.g., "frontend-developer") or --preset flag.
        name: Option<String>,
        /// Install a preset group (e.g., "web-app", "mobile", "full-stack").
        #[arg(long)]
        preset: Option<String>,
    },
    /// List available agents from the index.
    List {
        /// Filter by category (e.g., "core", "mobile", "infra").
        #[arg(long)]
        category: Option<String>,
        /// Search by keyword in name/description/tags.
        #[arg(long)]
        search: Option<String>,
    },
    /// Show installed agents in current project.
    Installed,
    /// Fetch/update the agents index + cache.
    Fetch {
        /// Version to fetch. Defaults to latest.
        #[arg(long)]
        version: Option<String>,
    },
    /// Show registry status (pinned version, cached, latest).
    Status,
    /// Remove an installed agent from current project.
    Remove {
        /// Agent name to remove.
        name: String,
    },
}

pub fn run(args: Args) -> Result<()> {
    match args.command {
        AgentsCommand::Browse => cmd_browse(),
        AgentsCommand::Install { name, preset } => cmd_install(name, preset),
        AgentsCommand::List { category, search } => cmd_list(category, search),
        AgentsCommand::Installed => cmd_installed(),
        AgentsCommand::Fetch { version } => cmd_fetch(version),
        AgentsCommand::Status => cmd_status(),
        AgentsCommand::Remove { name } => cmd_remove(name),
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

fn cmd_browse() -> Result<()> {
    use dialoguer::{FuzzySelect, MultiSelect, theme::ColorfulTheme};

    let index_path = std::path::Path::new("agents/index.json");
    let index_content = if index_path.exists() {
        std::fs::read_to_string(index_path)?
    } else {
        anyhow::bail!("agents/index.json not found. Run `genasis agents fetch` first.");
    };
    let index: serde_json::Value = serde_json::from_str(&index_content)?;

    let categories = index.get("categories").and_then(|c| c.as_array())
        .context("invalid index.json")?;
    let agents = index.get("agents").and_then(|a| a.as_array())
        .context("invalid index.json")?;

    let theme = ColorfulTheme::default();

    // Step 1: Select category
    let cat_labels: Vec<String> = categories.iter().map(|c| {
        let name = c.get("name").and_then(|n| n.as_str()).unwrap_or("?");
        let desc = c.get("description").and_then(|d| d.as_str()).unwrap_or("");
        format!("{name:<24} {desc}")
    }).collect();

    let mut cat_labels_with_all = vec!["All categories".to_string()];
    cat_labels_with_all.extend(cat_labels);

    let cat_idx = FuzzySelect::with_theme(&theme)
        .with_prompt("Select a category (type to filter)")
        .items(&cat_labels_with_all)
        .default(0)
        .interact()?;

    // Step 2: Filter agents by selected category
    let category_filter: Option<&str> = if cat_idx == 0 {
        None
    } else {
        categories.get(cat_idx - 1)
            .and_then(|c| c.get("id"))
            .and_then(|id| id.as_str())
    };

    let filtered_agents: Vec<&serde_json::Value> = agents.iter().filter(|a| {
        if let Some(cat) = category_filter {
            a.get("category").and_then(|c| c.as_str()) == Some(cat)
        } else {
            true
        }
    }).collect();

    if filtered_agents.is_empty() {
        println!("No agents in this category.");
        return Ok(());
    }

    // Step 3: Multi-select agents to install
    let agent_labels: Vec<String> = filtered_agents.iter().map(|a| {
        let name = a.get("name").and_then(|n| n.as_str()).unwrap_or("?");
        let desc = a.get("description").and_then(|d| d.as_str()).unwrap_or("");
        format!("{name:<24} {desc}")
    }).collect();

    let selections = MultiSelect::with_theme(&theme)
        .with_prompt("Select agents to install (Space to toggle, Enter to confirm)")
        .items(&agent_labels)
        .interact()?;

    if selections.is_empty() {
        println!("No agents selected.");
        return Ok(());
    }

    // Step 4: Install selected agents
    let (version, registry_url, _cache) = resolve_config()?;
    for idx in selections {
        let agent = filtered_agents[idx];
        let name = agent.get("name").and_then(|n| n.as_str()).unwrap_or("?");
        println!("\nInstalling {name}...");
        if let Err(e) = cmd_install(Some(name.to_string()), None) {
            eprintln!("  ✗ Failed to install {name}: {e}");
        }
    }

    Ok(())
}

fn cmd_install(name: Option<String>, preset: Option<String>) -> Result<()> {
    let (version, registry_url, cache_override) = resolve_config()?;

    if let Some(preset_name) = preset {
        println!("Installing preset: {preset_name}");
        // TODO: read index.json presets, resolve agent list, install each
        println!("TODO: implement preset install (read index.json → batch install)");
        return Ok(());
    }

    let agent_name = name.context(
        "specify an agent name or --preset. Run `genasis agents list` to see available agents."
    )?;

    println!("Installing agent: {agent_name}...");

    // Fetch individual agent .md from release assets
    let download_url = format!(
        "{}/download/agents-v{}/{}.md",
        registry_url.trim_end_matches('/'),
        version,
        agent_name
    );

    let client = reqwest::blocking::Client::builder()
        .user_agent("genasis-cli")
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let resp = client.get(&download_url).send()
        .with_context(|| format!("failed to fetch agent {agent_name}"))?;

    if !resp.status().is_success() {
        anyhow::bail!(
            "agent '{}' not found in registry (status {}). Run `genasis agents list` to see available.",
            agent_name, resp.status()
        );
    }

    let body = resp.text()?;

    // Write to .claude/agents/<name>.md
    let agents_dir = std::path::Path::new(".claude/agents");
    std::fs::create_dir_all(agents_dir)?;
    let target = agents_dir.join(format!("{agent_name}.md"));

    if target.exists() {
        println!("  ⚠ {agent_name}.md already exists. Skipping (use --force to overwrite).");
        return Ok(());
    }

    std::fs::write(&target, &body)?;
    println!("  ✓ Installed {agent_name} → {}", target.display());
    println!("  ℹ Run `genasis attach` to inject the Plane/MM overlay protocol.");
    Ok(())
}

fn cmd_list(category: Option<String>, search: Option<String>) -> Result<()> {
    // Read index.json from local agents/ dir or from cache
    let index_path = std::path::Path::new("agents/index.json");
    let index_content = if index_path.exists() {
        std::fs::read_to_string(index_path)?
    } else {
        // TODO: fetch index from registry if not local
        anyhow::bail!("agents/index.json not found. Run `genasis agents fetch` first.");
    };

    let index: serde_json::Value = serde_json::from_str(&index_content)?;
    let agents = index.get("agents").and_then(|a| a.as_array())
        .context("invalid index.json: missing agents array")?;

    println!("=== Available Agents ===\n");

    for agent in agents {
        let name = agent.get("name").and_then(|n| n.as_str()).unwrap_or("?");
        let desc = agent.get("description").and_then(|d| d.as_str()).unwrap_or("");
        let cat = agent.get("category").and_then(|c| c.as_str()).unwrap_or("");
        let tags: Vec<&str> = agent.get("tags")
            .and_then(|t| t.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();

        // Filter by category
        if let Some(ref cat_filter) = category {
            if cat != cat_filter.as_str() {
                continue;
            }
        }

        // Filter by search
        if let Some(ref q) = search {
            let q_lower = q.to_lowercase();
            let matches = name.to_lowercase().contains(&q_lower)
                || desc.to_lowercase().contains(&q_lower)
                || tags.iter().any(|t| t.to_lowercase().contains(&q_lower));
            if !matches {
                continue;
            }
        }

        println!("  {:<22} {}", name, desc);
    }

    println!("\nInstall: genasis agents install <name>");
    println!("Preset:  genasis agents install --preset web-app");
    Ok(())
}

fn cmd_installed() -> Result<()> {
    let agents_dir = std::path::Path::new(".claude/agents");
    if !agents_dir.is_dir() {
        println!("No agents installed (.claude/agents/ does not exist).");
        return Ok(());
    }
    let mut count = 0;
    for entry in std::fs::read_dir(agents_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(".md") && !name.starts_with('.') {
            println!("  • {}", name.trim_end_matches(".md"));
            count += 1;
        }
    }
    println!("\n{count} agent(s) installed.");
    Ok(())
}

fn cmd_remove(name: String) -> Result<()> {
    let target = std::path::Path::new(".claude/agents").join(format!("{name}.md"));
    if !target.exists() {
        anyhow::bail!("agent '{name}' not installed ({} does not exist)", target.display());
    }
    std::fs::remove_file(&target)?;
    println!("  ✓ Removed {name}");
    Ok(())
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
    let (pinned, registry_url, _cache_override) = resolve_config()?;

    println!("Registry: {registry_url}");
    println!("Pinned version: {pinned}");

    // Check index.json locally
    let index_path = std::path::Path::new("agents/index.json");
    if index_path.exists() {
        let content = std::fs::read_to_string(index_path)?;
        let index: serde_json::Value = serde_json::from_str(&content)?;
        let count = index.get("agents").and_then(|a| a.as_array()).map(|a| a.len()).unwrap_or(0);
        println!("Index: {count} agents available");
    } else {
        println!("Index: not found locally");
    }

    // Check installed
    let agents_dir = std::path::Path::new(".claude/agents");
    if agents_dir.is_dir() {
        let installed = std::fs::read_dir(agents_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().ends_with(".md"))
            .count();
        println!("Installed: {installed} agent(s)");
    } else {
        println!("Installed: none (.claude/agents/ not found)");
    }

    match registry::check_latest(&registry_url) {
        Ok(latest) => {
            println!("Latest release: agents-v{latest}");
            if latest != pinned {
                println!("  → Update available! Run `genasis agents fetch --version {latest}`");
            }
        }
        Err(e) => println!("Could not check latest: {e}"),
    }
    Ok(())
}
