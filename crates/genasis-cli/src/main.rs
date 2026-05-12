use anyhow::Result;
use clap::{Parser, Subcommand};

mod cmd_agents;
mod cmd_attach;
mod cmd_bootstrap;
mod cmd_db;
mod cmd_debug;
mod cmd_design;
mod cmd_detach;
mod cmd_doctor;
mod cmd_example;
mod cmd_humans;
mod cmd_init;
mod cmd_lang;
mod cmd_listen;
mod cmd_mm;
mod cmd_monitor;
mod cmd_plane;
mod cmd_trial;
mod cmd_upgrade;
mod cmd_version;
mod lang_prompt;
mod tui_attach;

#[derive(Parser)]
#[command(name = "genasis", version, about = "Bolt-on agentic team layer")]
struct Cli {
    /// Locale for CLI/TUI output (en|ko). Overrides $GENASIS_LANG and $LANG.
    #[arg(long, global = true, value_name = "LANG", env = "GENASIS_LANG_FLAG")]
    pub lang: Option<String>,

    /// Skip interactive prompts; use defaults where possible.
    #[arg(long, global = true)]
    pub non_interactive: bool,

    /// Auto-accept confirmation prompts.
    #[arg(long = "yes", short = 'y', global = true)]
    pub assume_yes: bool,

    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// **Primary entry point.** Bootstrap a blank project: base agents +
    /// overlay (commands/hooks/skills) + Plane/MM provisioning. Modes:
    /// `--trial` (zero-setup), `--bootstrap` (scaffold only), bare
    /// (real Plane/MM provisioning against pre-filled `genasis.toml`).
    Init(cmd_init::Args),
    /// **[Advanced]** Scaffold canonical 10-role base agent files into
    /// `.claude/agents/` (green-field projects). `genasis init` already
    /// runs this in `--trial` / `--bootstrap` mode; call it directly
    /// only when you want to scaffold without the rest of init.
    /// Auto-chains to `attach` unless `--no-attach-after`. ADR-010 §3.
    Bootstrap(cmd_bootstrap::Args),
    /// **[Advanced]** Attach overlay onto an existing agentic team.
    /// `genasis init` already runs this for `--trial` / `--bootstrap`;
    /// call directly when you've hand-authored `.claude/agents/` and
    /// want overlay fences + commands/hooks/skills + GENASIS.md added
    /// without re-scaffolding the bases. Also supports `--lang` to
    /// switch the active agent-context language (deprecates
    /// `genasis lang switch`).
    Attach(cmd_attach::Args),
    /// Remove the overlay (marker fences only)
    Detach(cmd_detach::Args),
    /// Verify environment, tools, and configuration
    Doctor(cmd_doctor::Args),
    /// **[Deprecated v0.5.3]** Upgrade overlay to a newer template
    /// version. Prefer `genasis attach --upgrade` — same effect, one
    /// less command to remember. This subcommand will be removed in
    /// v0.7.0.
    Upgrade(cmd_upgrade::Args),
    /// Design-system hot-swap orchestration
    Design(cmd_design::Args),
    /// Database operations (read-only query, migrate, diff, status, doctor)
    Db(cmd_db::Args),
    /// **[Deprecated v0.5.3]** Plane API thin wrapper. Prefer
    /// `genasis doctor --probe-plane` for connectivity checks. Will
    /// be removed in v0.7.0.
    Plane(cmd_plane::Args),
    /// **[Deprecated v0.5.3]** Mattermost API thin wrapper. Prefer
    /// `genasis doctor --probe-mm` for connectivity checks. Will be
    /// removed in v0.7.0.
    Mm(cmd_mm::Args),
    /// Ratatui TUI for sprint/tokens/agents/deploy/network/logs
    Monitor(cmd_monitor::Args),
    /// Print version metadata
    Version(cmd_version::Args),
    /// **[Deprecated v0.5.3]** Inspect or change the active agent-context
    /// language. Prefer `genasis attach --lang=<en|ko>` — same effect,
    /// one less command. Will be removed in v0.7.0.
    Lang(cmd_lang::Args),
    /// Drop a sample document (PRD/design-system/PRD2) into the project
    /// root so the agentic team has something immediately actionable.
    Example(cmd_example::Args),
    /// Browse, install, list, or remove agents from the catalog (ADR-011)
    Agents(cmd_agents::Args),
    /// Inspect drift, collect anonymised patches, reset baseline (ADR-012)
    Debug(cmd_debug::Args),
    /// Manage human team-member roster: list, add, edit, remove,
    /// and provision into Mattermost + Plane (ADR-014).
    Humans(cmd_humans::Args),
    /// Flip the trial-app team's `app_status` to `'complete'` so the
    /// ShowcasePanel unlocks for the user. Alias for the legacy
    /// `genasis trial publish` (which still works, see `trial`).
    Publish(cmd_trial::PublishArgs),
    /// **[Deprecated v0.5.3]** Operate on the trial-app companion
    /// (ADR-017). The only sub today (`publish`) is now available
    /// at the top level as `genasis publish`. This namespace will be
    /// removed in v0.7.0.
    Trial(cmd_trial::Args),
    /// Reactive bridge daemon — subscribe to the trial-app SSE stream
    /// (or the Mattermost-bridge equivalent — TODO) and spawn
    /// `claude --print` for every human-authored chat message so an
    /// agent persona auto-responds and transitions the related kanban
    /// card. Implements genesis §0 (Mattermost+Plane only) + §28
    /// Mattermost Bridge in the trial flavor.
    Listen(cmd_listen::Args),
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let cli = Cli::parse();
    // Resolve locale before dispatching so every cmd_*.rs sees a primed
    // rust-i18n bundle. genasis.toml [i18n] cli_lang is wired in M12.4
    // when we touch the config schema.
    let resolved = genasis_i18n::resolve(cli.lang.as_deref(), None);
    genasis_i18n::install(resolved.lang);
    tracing::debug!(
        lang = %resolved.lang,
        source = resolved.source.label(),
        "i18n locale installed"
    );
    match cli.command {
        Cmd::Init(a) => {
            cmd_init::run_with_globals(a, cli.lang.clone(), cli.non_interactive, cli.assume_yes)
                .await
        }
        Cmd::Bootstrap(a) => {
            cmd_bootstrap::run(a, cli.lang.clone(), cli.non_interactive, cli.assume_yes).await
        }
        Cmd::Attach(a) => {
            cmd_attach::pub_run(a, cli.lang.clone(), cli.non_interactive, cli.assume_yes).await
        }
        Cmd::Detach(a) => cmd_detach::run(a).await,
        Cmd::Doctor(a) => cmd_doctor::run(a).await,
        Cmd::Upgrade(a) => {
            eprintln!(
                "  note: `genasis upgrade` is deprecated in v0.5.3 — same effect via \
                 `genasis attach --upgrade`. This subcommand will be removed in v0.7.0."
            );
            cmd_upgrade::run(a).await
        }
        Cmd::Design(a) => cmd_design::run(a).await,
        Cmd::Db(a) => cmd_db::run(a).await,
        Cmd::Plane(a) => {
            eprintln!(
                "  note: `genasis plane` is deprecated in v0.5.3 — for connectivity \
                 checks use `genasis doctor --probe-plane`. This subcommand will be \
                 removed in v0.7.0."
            );
            cmd_plane::run(a).await
        }
        Cmd::Mm(a) => {
            eprintln!(
                "  note: `genasis mm` is deprecated in v0.5.3 — for connectivity \
                 checks use `genasis doctor --probe-mm`. This subcommand will be \
                 removed in v0.7.0."
            );
            cmd_mm::run(a).await
        }
        Cmd::Monitor(a) => cmd_monitor::run(a).await,
        Cmd::Version(a) => cmd_version::run(a).await,
        Cmd::Lang(a) => {
            eprintln!(
                "  note: `genasis lang` is deprecated in v0.5.3 — switch language via \
                 `genasis attach --lang=<en|ko>`. This subcommand will be removed in v0.7.0."
            );
            cmd_lang::run(a, cli.non_interactive, cli.assume_yes).await
        }
        Cmd::Example(a) => cmd_example::run(a),
        Cmd::Agents(a) => cmd_agents::run(a),
        Cmd::Debug(a) => cmd_debug::run(a),
        Cmd::Humans(a) => cmd_humans::run(a).await,
        Cmd::Publish(a) => cmd_trial::run_publish_with_project(a).await,
        Cmd::Trial(a) => {
            eprintln!(
                "  note: `genasis trial publish` is deprecated in v0.5.3 — use \
                 `genasis publish` (top-level). The `trial` namespace will be removed \
                 in v0.7.0."
            );
            cmd_trial::run(a).await
        }
        Cmd::Listen(a) => cmd_listen::run(a).await,
    }
}

fn init_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).with_target(false).init();
}
