use anyhow::Result;
use clap::{Parser, Subcommand};

mod cmd_attach;
mod cmd_db;
mod cmd_design;
mod cmd_detach;
mod cmd_doctor;
mod cmd_init;
mod cmd_mm;
mod cmd_monitor;
mod cmd_plane;
mod cmd_upgrade;
mod cmd_version;
mod tui_attach;

#[derive(Parser)]
#[command(name = "genasis", version, about = "Bolt-on agentic team layer")]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Bootstrap a blank project: ECC team + overlay + Plane/MM provisioning
    Init(cmd_init::Args),
    /// Attach overlay onto an existing agentic team
    Attach(cmd_attach::Args),
    /// Remove the overlay (marker fences only)
    Detach(cmd_detach::Args),
    /// Verify environment, tools, and configuration
    Doctor(cmd_doctor::Args),
    /// Upgrade overlay to a newer template version
    Upgrade(cmd_upgrade::Args),
    /// Design-system hot-swap orchestration
    Design(cmd_design::Args),
    /// Database operations (read-only query, migrate, diff, status, doctor)
    Db(cmd_db::Args),
    /// Plane API thin wrapper (debug)
    Plane(cmd_plane::Args),
    /// Mattermost API thin wrapper (debug)
    Mm(cmd_mm::Args),
    /// Ratatui TUI for sprint/tokens/agents/deploy/network/logs
    Monitor(cmd_monitor::Args),
    /// Print version metadata
    Version(cmd_version::Args),
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let cli = Cli::parse();
    match cli.command {
        Cmd::Init(a) => cmd_init::run(a).await,
        Cmd::Attach(a) => cmd_attach::run(a).await,
        Cmd::Detach(a) => cmd_detach::run(a).await,
        Cmd::Doctor(a) => cmd_doctor::run(a).await,
        Cmd::Upgrade(a) => cmd_upgrade::run(a).await,
        Cmd::Design(a) => cmd_design::run(a).await,
        Cmd::Db(a) => cmd_db::run(a).await,
        Cmd::Plane(a) => cmd_plane::run(a).await,
        Cmd::Mm(a) => cmd_mm::run(a).await,
        Cmd::Monitor(a) => cmd_monitor::run(a).await,
        Cmd::Version(a) => cmd_version::run(a).await,
    }
}

fn init_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).with_target(false).init();
}
