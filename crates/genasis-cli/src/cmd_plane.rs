use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use genasis_core::config::Config;
use genasis_providers::plane;

#[derive(Parser, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub op: PlaneOp,
}

#[derive(Subcommand, Debug)]
pub enum PlaneOp {
    /// Probe `<plane-url>/api/v1/health/` and print the response.
    Health,
}

pub async fn run(args: Args) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let cfg_path = Config::discover(&cwd).context("no genasis.toml in this tree")?;
    let cfg = Config::load(&cfg_path)?;
    let plane_cfg = cfg.plane.context("[plane] section missing")?;
    let token = std::env::var("PLANE_API_KEY").or_else(|_| std::env::var("PLANE_TOKEN_PM"))?;
    let flavor = plane::FlavorChoice::parse(&plane_cfg.flavor)?;
    let client = plane::build(flavor, &plane_cfg.url, &plane_cfg.workspace_slug, &token).await?;
    match args.op {
        PlaneOp::Health => {
            let v = client.health().await?;
            println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
        }
    }
    Ok(())
}
