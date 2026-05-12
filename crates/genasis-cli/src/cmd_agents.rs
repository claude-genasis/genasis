//! `genasis agents` subcommand — manage the agents catalog.
//!
//! ADR-011: agents catalog is fetched from GitHub Releases, cached locally,
//! and version-pinned in `genasis.toml [agents].version`.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

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
    let version = std::env::var("GENASIS_AGENTS_VERSION").unwrap_or_else(|_| "1.0.0".to_string());
    let registry = std::env::var("GENASIS_AGENTS_REGISTRY")
        .unwrap_or_else(|_| "https://github.com/claude-genasis/genasis/releases".to_string());
    let cache_dir = std::env::var("GENASIS_AGENTS_CACHE_DIR").unwrap_or_default();
    Ok((version, registry, cache_dir))
}

/// v0.5.6 D-002: resolve the agents catalog index for `list` / `browse` /
/// `status` commands. Fallback chain:
///   1. `./agents/index.json` (present only when the user clones the
///      genasis repo itself — the published binary's release tarball
///      does NOT include this file)
///   2. `<cache>/v<ver>/index.json` (the agents-pool release will
///      eventually ship this — currently absent from `agents-v1.0.0`)
///   3. **build from cache base/ frontmatters** — walks every `.md`
///      under `<cache>/v<ver>/base/`, parses YAML frontmatter for
///      `name`, `description`, `category`, `tags`, and synthesises the
///      `{agents, categories, presets}` shape the commands expect.
///
/// This makes `genasis agents list / browse / status` work against the
/// stock release-tarball cache without a separate `agents-v1.0.1`
/// drop. When the catalog eventually ships its own `index.json` the
/// fallback becomes dead code.
fn load_catalog_index(version: &str, cache_override: &str) -> Result<serde_json::Value> {
    use genasis_core::frontmatter;

    // 1. Project-local override
    let local = std::path::Path::new("agents/index.json");
    if local.exists() {
        let s = std::fs::read_to_string(local).context("read agents/index.json")?;
        return Ok(serde_json::from_str(&s)?);
    }

    let cache_dir = cache::cache_dir(version, cache_override)?;

    // 2. Cache-shipped index
    let cache_idx = cache_dir.join("index.json");
    if cache_idx.exists() {
        let s = std::fs::read_to_string(&cache_idx)
            .with_context(|| format!("read cached {}", cache_idx.display()))?;
        return Ok(serde_json::from_str(&s)?);
    }

    // 3. Build from base/ frontmatters (current v1.0.0 reality)
    let base_dir = cache_dir.join("base");
    if !base_dir.is_dir() {
        anyhow::bail!(
            "catalog cache not populated at {}. Run `genasis agents fetch` first.",
            cache_dir.display()
        );
    }

    let mut agents: Vec<serde_json::Value> = Vec::new();
    let mut categories_seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for entry in
        std::fs::read_dir(&base_dir).with_context(|| format!("read_dir {}", base_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        let raw = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let split = frontmatter::split(&raw);
        let fm = match split.frontmatter {
            Some(f) => f,
            None => continue,
        };
        let name = frontmatter::read_scalar(fm.raw, "name")
            .map(str::to_string)
            .or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .map(str::to_string)
            })
            .unwrap_or_default();
        if name.is_empty() {
            continue;
        }
        let description = frontmatter::read_scalar(fm.raw, "description")
            .unwrap_or("")
            .to_string();
        let category = frontmatter::read_scalar(fm.raw, "category")
            .unwrap_or("uncategorised")
            .to_string();
        categories_seen.insert(category.clone());

        // Tags rarely appear as a single-line scalar in real
        // frontmatters; treat absence as empty.
        let tags_raw = frontmatter::read_scalar(fm.raw, "tags").unwrap_or("");
        let tags: Vec<serde_json::Value> = if tags_raw.starts_with('[') && tags_raw.ends_with(']') {
            tags_raw[1..tags_raw.len() - 1]
                .split(',')
                .map(|s| s.trim().trim_matches(['\'', '"']).to_string())
                .filter(|s| !s.is_empty())
                .map(serde_json::Value::String)
                .collect()
        } else {
            Vec::new()
        };

        agents.push(serde_json::json!({
            "name": name,
            "description": description,
            "category": category,
            "tags": tags,
        }));
    }

    agents.sort_by(|a, b| {
        let an = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let bn = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
        an.cmp(bn)
    });

    let categories: Vec<serde_json::Value> = categories_seen
        .into_iter()
        .map(|c| {
            serde_json::json!({
                "id": c,
                "name": c,
                "description": "",
            })
        })
        .collect();

    Ok(serde_json::json!({
        "agents": agents,
        "categories": categories,
        "presets": {},
        "_source": "synthesised-from-cache-base-frontmatters",
    }))
}

fn cmd_browse() -> Result<()> {
    use dialoguer::{theme::ColorfulTheme, FuzzySelect, MultiSelect};

    let (version, _, cache_override) = resolve_config()?;
    let index = load_catalog_index(&version, &cache_override)?;

    let categories = index
        .get("categories")
        .and_then(|c| c.as_array())
        .context("invalid index.json")?;
    let agents = index
        .get("agents")
        .and_then(|a| a.as_array())
        .context("invalid index.json")?;

    let theme = ColorfulTheme::default();

    // Step 1: Select category
    let cat_labels: Vec<String> = categories
        .iter()
        .map(|c| {
            let name = c.get("name").and_then(|n| n.as_str()).unwrap_or("?");
            let desc = c.get("description").and_then(|d| d.as_str()).unwrap_or("");
            format!("{name:<24} {desc}")
        })
        .collect();

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
        categories
            .get(cat_idx - 1)
            .and_then(|c| c.get("id"))
            .and_then(|id| id.as_str())
    };

    let filtered_agents: Vec<&serde_json::Value> = agents
        .iter()
        .filter(|a| {
            if let Some(cat) = category_filter {
                a.get("category").and_then(|c| c.as_str()) == Some(cat)
            } else {
                true
            }
        })
        .collect();

    if filtered_agents.is_empty() {
        println!("No agents in this category.");
        return Ok(());
    }

    // Step 3: Multi-select agents to install
    let agent_labels: Vec<String> = filtered_agents
        .iter()
        .map(|a| {
            let name = a.get("name").and_then(|n| n.as_str()).unwrap_or("?");
            let desc = a.get("description").and_then(|d| d.as_str()).unwrap_or("");
            format!("{name:<24} {desc}")
        })
        .collect();

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

    // Ensure catalog is cached (fetch if needed)
    ensure_catalog_cached(&version, &registry_url, &cache_override)?;

    if let Some(preset_name) = preset {
        return install_preset(&preset_name, &version, &cache_override);
    }

    let agent_name = name.context(
        "specify an agent name or --preset. Run `genasis agents list` to see available agents.",
    )?;

    install_single_agent(&agent_name, &version, &cache_override)
}

fn ensure_catalog_cached(version: &str, registry_url: &str, cache_override: &str) -> Result<()> {
    if cache::is_cached(version, cache_override)? {
        return Ok(());
    }
    println!("Fetching agents catalog v{version}...");
    let tarball = registry::fetch_tarball(registry_url, version)
        .context("failed to fetch agents catalog — check network and version")?;
    cache::store_tarball(version, cache_override, &tarball)?;
    println!("  ✓ Cached agents catalog v{version}");
    Ok(())
}

fn install_single_agent(agent_name: &str, version: &str, cache_override: &str) -> Result<()> {
    let dir = cache::cache_dir(version, cache_override)?;
    let source = dir.join("base").join(format!("{agent_name}.md"));

    if !source.exists() {
        anyhow::bail!(
            "agent '{agent_name}' not found in catalog v{version}. Run `genasis agents list` to see available."
        );
    }

    let agents_dir = std::path::Path::new(".claude/agents");
    std::fs::create_dir_all(agents_dir)?;
    let target = agents_dir.join(format!("{agent_name}.md"));

    if target.exists() {
        println!("  ⚠ {agent_name}.md already exists. Skipping (use `remove` first to replace).");
        return Ok(());
    }

    std::fs::copy(&source, &target)?;
    println!("  ✓ Installed {agent_name} → {}", target.display());
    println!("  ℹ Run `genasis attach` to inject the Plane/MM overlay protocol.");
    Ok(())
}

fn install_preset(preset_name: &str, version: &str, cache_override: &str) -> Result<()> {
    // v0.5.6 D-002: presets come from whichever source actually
    // exposes a `presets` block — the synthesised index from cache
    // base/ frontmatters does NOT (frontmatters don't carry preset
    // membership). Use the unified loader; if presets are absent the
    // user gets a clear error pointing at the catalog refresh.
    let presets_json = load_catalog_index(version, cache_override)?;

    let presets = presets_json
        .get("presets")
        .and_then(|p| p.as_object())
        .filter(|p| !p.is_empty())
        .context(
            "no presets defined in catalog. \
             The current catalog (v1.0.0) ships agents but no preset \
             definitions — install individual agents by name (e.g. \
             `genasis agents install frontend-developer`). Presets \
             land in `agents-v1.0.1`.",
        )?;

    let preset = presets.get(preset_name).context(format!(
        "preset '{preset_name}' not found. Available: {}",
        presets.keys().cloned().collect::<Vec<_>>().join(", ")
    ))?;

    let agents = preset
        .get("agents")
        .and_then(|a| a.as_array())
        .context("preset has no agents list")?;

    let desc = preset
        .get("description")
        .and_then(|d| d.as_str())
        .unwrap_or("");
    println!("Installing preset '{preset_name}': {desc}");
    println!("  Agents: {}\n", agents.len());

    let mut installed = 0;
    for agent_val in agents {
        let name = agent_val.as_str().unwrap_or("");
        if name.is_empty() {
            continue;
        }
        match install_single_agent(name, version, cache_override) {
            Ok(()) => installed += 1,
            Err(e) => eprintln!("  ✗ {name}: {e}"),
        }
    }
    println!(
        "\n  ✓ Installed {installed}/{} agents from preset '{preset_name}'",
        agents.len()
    );
    println!("  ℹ Run `genasis attach` to inject overlays.");
    Ok(())
}

fn cmd_list(category: Option<String>, search: Option<String>) -> Result<()> {
    let (version, _, cache_override) = resolve_config()?;
    let index = load_catalog_index(&version, &cache_override)?;
    let agents = index
        .get("agents")
        .and_then(|a| a.as_array())
        .context("invalid index.json: missing agents array")?;

    println!("=== Available Agents ===\n");

    for agent in agents {
        let name = agent.get("name").and_then(|n| n.as_str()).unwrap_or("?");
        let desc = agent
            .get("description")
            .and_then(|d| d.as_str())
            .unwrap_or("");
        let cat = agent.get("category").and_then(|c| c.as_str()).unwrap_or("");
        let tags: Vec<&str> = agent
            .get("tags")
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
        anyhow::bail!(
            "agent '{name}' not installed ({} does not exist)",
            target.display()
        );
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
        let dir = cache::cache_dir(&version, &cache_override)?;
        let count = std::fs::read_dir(dir.join("base"))
            .map(|rd| rd.filter_map(|e| e.ok()).count())
            .unwrap_or(0);
        println!("  {count} agents available. Run `genasis agents browse` to install.");
        return Ok(());
    }

    println!("Fetching agents catalog v{version}...");
    let tarball = registry::fetch_tarball(&registry_url, &version)
        .context("failed to fetch agents catalog — check network and version")?;
    let dir = cache::store_tarball(&version, &cache_override, &tarball)?;
    let count = std::fs::read_dir(dir.join("base"))
        .map(|rd| rd.filter_map(|e| e.ok()).count())
        .unwrap_or(0);
    println!("  ✓ Cached {count} agents at {}", dir.display());
    println!("  Run `genasis agents browse` to install.");
    Ok(())
}

fn cmd_status() -> Result<()> {
    let (pinned, registry_url, cache_override) = resolve_config()?;

    println!("Registry: {registry_url}");
    println!("Pinned version: {pinned}");

    match load_catalog_index(&pinned, &cache_override) {
        Ok(index) => {
            let count = index
                .get("agents")
                .and_then(|a| a.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            let source = index
                .get("_source")
                .and_then(|s| s.as_str())
                .map(|s| format!(" ({s})"))
                .unwrap_or_default();
            println!("Index: {count} agents available{source}");
        }
        Err(_) => println!("Index: not available — run `genasis agents fetch`"),
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
