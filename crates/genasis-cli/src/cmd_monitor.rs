use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {}

pub async fn run(_args: Args) -> Result<()> {
    genasis_monitor::app::run().await.map_err(Into::into)
}
