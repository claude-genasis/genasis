use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use genasis_core::config::Config;
use genasis_providers::mattermost;

#[derive(Parser, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub op: MmOp,
}

#[derive(Subcommand, Debug)]
pub enum MmOp {
    /// Probe `<mm-url>/api/v4/system/ping` and print the response.
    Ping,
}

pub async fn run(args: Args) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let cfg_path = Config::discover(&cwd).context("no genasis.toml in this tree")?;
    let cfg = Config::load(&cfg_path)?;
    let mm_cfg = cfg.mattermost.context("[mattermost] section missing")?;
    let token = std::env::var("MM_ADMIN_TOKEN").context("MM_ADMIN_TOKEN unset")?;
    let flavor = mattermost::FlavorChoice::parse(&mm_cfg.flavor)?;
    let client = mattermost::build(flavor, &mm_cfg.url, &token).await?;
    match args.op {
        MmOp::Ping => {
            let v = client.ping().await?;
            println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
        }
    }
    Ok(())
}
