use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use genasis_core::config::{Config, CONFIG_FILE_NAME};
use genasis_i18n::{tr, tr_args};
use genasis_providers::{mattermost, plane};

#[derive(Parser, Debug)]
pub struct Args {
    /// Project root. Defaults to the current working directory.
    #[arg(long, value_name = "DIR")]
    pub project: Option<PathBuf>,

    /// Stop after pinging Plane / Mattermost — do not provision anything.
    #[arg(long)]
    pub probe_only: bool,

    /// Bootstrap in trial mode: skip real Plane/MM provisioning, write a
    /// minimal `genasis.toml` with `[trial]` enabled, and offer to open
    /// the trial-app demo at http://localhost:3000.
    #[arg(long)]
    pub trial: bool,

    /// Alias for `genasis bootstrap` — scaffold canonical 10-role base
    /// agent files when `.claude/agents/` is empty, then auto-chain into
    /// `attach`. ADR-010 §3 decision (b)+(d).
    #[arg(long)]
    pub bootstrap: bool,

    /// Comma-separated subset of roles for `--bootstrap`. Forwarded to
    /// `cmd_bootstrap`.
    #[arg(long, value_name = "LIST")]
    pub roles: Option<String>,
}

pub async fn run(args: Args) -> Result<()> {
    run_with_globals(args, None, false, false).await
}

pub async fn run_with_globals(
    args: Args,
    lang_flag: Option<String>,
    non_interactive: bool,
    assume_yes: bool,
) -> Result<()> {
    if args.bootstrap {
        let bootstrap_args = crate::cmd_bootstrap::Args {
            project: args.project.clone(),
            roles: args.roles.clone(),
            no_attach_after: false,
            dry_run: false,
        };
        return crate::cmd_bootstrap::run(bootstrap_args, lang_flag, non_interactive, assume_yes)
            .await;
    }
    if args.trial {
        return run_trial(args).await;
    }
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
    let mm_cfg = cfg
        .mattermost
        .as_ref()
        .context("[mattermost] section missing")?;

    let plane_token = std::env::var("PLANE_API_KEY")
        .or_else(|_| std::env::var("PLANE_TOKEN_PM"))
        .context("PLANE_API_KEY (or PLANE_TOKEN_PM) not set in environment")?;
    let mm_token =
        std::env::var("MM_ADMIN_TOKEN").context("MM_ADMIN_TOKEN not set in environment")?;

    let plane_flavor = plane::FlavorChoice::parse(&plane_cfg.flavor)?;
    let mm_flavor = mattermost::FlavorChoice::parse(&mm_cfg.flavor)?;

    println!(
        "{}",
        tr_args("init.resolving_plane", &[("flavor", &plane_cfg.flavor)])
    );
    let plane_client = plane::build(
        plane_flavor,
        &plane_cfg.url,
        &plane_cfg.workspace_slug,
        &plane_token,
    )
    .await?;
    let plane_health = plane_client.health().await?;
    println!("  plane health: {}", short_json(&plane_health));

    println!(
        "{}",
        tr_args("init.resolving_mm", &[("flavor", &mm_cfg.flavor)])
    );
    let mm_client = mattermost::build(mm_flavor, &mm_cfg.url, &mm_token).await?;
    let mm_ping = mm_client.ping().await?;
    println!("  mattermost ping: {}", short_json(&mm_ping));

    if args.probe_only {
        println!("\n{}", tr("init.probe_only_skip"));
        return Ok(());
    }

    println!(
        "\n{}",
        tr_args("init.ensure_project", &[("name", &cfg.project.name)])
    );
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
    println!(
        "\n{}",
        tr_args("init.ensure_channel", &[("channel", &scrum_channel)])
    );
    let team_id = std::env::var("MM_TEAM_ID").unwrap_or_default();
    if !team_id.is_empty() {
        let ch = mm_client
            .ensure_channel(&team_id, &scrum_channel, &scrum_channel)
            .await?;
        println!("  mm channel = {} ({})", ch.name, ch.id);
    } else {
        println!("  {}", tr("init.mm_team_id_missing"));
    }

    println!("\n{}", tr("init.next_step"));
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

const TRIAL_CONFIG_TEMPLATE: &str = r#"# Genasis trial-mode config — generated by `genasis init --trial`.
# The trial-app stands in as a Plane + Mattermost simulator so the
# agentic workflow can be exercised without installing either tool.

[project]
name = "trial-demo"

[plane]
url = "http://localhost:3000"
workspace_slug = "trial"
flavor = "trial"

[mattermost]
url = "http://localhost:3000"
team_name = "trial"
flavor = "trial"

[trial]
enabled = true
url = "http://localhost:3000"
shared_secret = ""
"#;

async fn run_trial(args: Args) -> Result<()> {
    use std::fs;
    use std::process::{Command, Stdio};

    let project_root = if let Some(p) = args.project.as_deref() {
        if !p.exists() {
            fs::create_dir_all(p)
                .with_context(|| format!("create --project dir {}", p.display()))?;
        }
        p.canonicalize()
            .with_context(|| format!("canonicalize {}", p.display()))?
    } else {
        std::env::current_dir()?
    };

    println!("→ Initializing trial-mode project at {}", project_root.display());

    let cfg_path = project_root.join(CONFIG_FILE_NAME);
    if !cfg_path.exists() {
        fs::write(&cfg_path, TRIAL_CONFIG_TEMPLATE)
            .with_context(|| format!("write {}", cfg_path.display()))?;
        println!("  wrote {}", cfg_path.display());
    } else {
        println!("  {} already exists — leaving as-is", cfg_path.display());
    }

    fs::create_dir_all(project_root.join(".claude/agents")).ok();
    fs::create_dir_all(project_root.join(".genasis")).ok();

    if args.probe_only {
        println!("\n--probe-only set — skipping launch prompt");
        return Ok(());
    }

    let assume_yes = std::env::var("GENASIS_TRIAL_AUTOLAUNCH")
        .ok()
        .as_deref()
        == Some("1");
    let launch = if assume_yes {
        true
    } else {
        match dialoguer::Confirm::new()
            .with_prompt("Launch trial demo app?")
            .default(true)
            .interact()
        {
            Ok(b) => b,
            Err(_) => false,
        }
    };
    if !launch {
        println!(
            "\n(Skipped) Start the trial-app yourself with `npm --prefix /work/genasis/trial-app run dev` and open http://localhost:3000"
        );
        return Ok(());
    }

    let cmd_str = std::env::var("TRIAL_APP_BIN")
        .unwrap_or_else(|_| "npm --prefix /work/genasis/trial-app run start".to_string());

    println!("→ Spawning trial-app: {cmd_str}");
    let mut cmd = Command::new("sh");
    cmd.arg("-c")
        .arg(&cmd_str)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let child = cmd
        .spawn()
        .with_context(|| format!("spawn `{cmd_str}`"))?;
    let pid = child.id();
    let pid_path = project_root.join(".genasis/trial.pid");
    fs::write(&pid_path, pid.to_string()).ok();
    println!(
        "  trial-app PID {pid} (recorded at {})",
        pid_path.display()
    );

    let url = "http://localhost:3000";
    if let Some(opener) = pick_browser_opener() {
        let _ = Command::new(opener)
            .arg(url)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
    }

    println!("\nTrial app running on {url} — Ctrl+C the spawned process to stop, or `kill $(cat .genasis/trial.pid)`.");
    println!(
        "Open the three tabs:\n  체험하기      — scripted 8-step agent sprint\n  라이브 트라이얼 — kanban + chat that mirrors agent calls live\n  신청하기      — request a hosted Plane + Mattermost trial environment"
    );
    Ok(())
}

fn pick_browser_opener() -> Option<&'static str> {
    if cfg!(target_os = "macos") {
        Some("open")
    } else if cfg!(target_os = "windows") {
        None
    } else if which::which("xdg-open").is_ok() {
        Some("xdg-open")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn run_trial_writes_config_and_skips_launch_in_probe_only() {
        let tmp = TempDir::new().unwrap();
        let args = Args {
            project: Some(tmp.path().to_path_buf()),
            probe_only: true,
            trial: true,
            bootstrap: false,
            roles: None,
        };
        run_trial(args).await.expect("trial probe_only succeeds");
        let cfg = std::fs::read_to_string(tmp.path().join(CONFIG_FILE_NAME)).unwrap();
        assert!(cfg.contains("[trial]"));
        assert!(cfg.contains("enabled = true"));
        assert!(cfg.contains("flavor = \"trial\""));
    }

    #[tokio::test]
    async fn run_trial_does_not_overwrite_existing_config() {
        let tmp = TempDir::new().unwrap();
        let cfg_path = tmp.path().join(CONFIG_FILE_NAME);
        std::fs::write(&cfg_path, "# existing config\n").unwrap();
        let args = Args {
            project: Some(tmp.path().to_path_buf()),
            probe_only: true,
            trial: true,
            bootstrap: false,
            roles: None,
        };
        run_trial(args).await.unwrap();
        let cfg = std::fs::read_to_string(&cfg_path).unwrap();
        assert_eq!(cfg, "# existing config\n");
    }
}
