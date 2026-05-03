use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    /// Emit JSON instead of a plain one-liner.
    #[arg(long)]
    pub json: bool,
}

pub async fn run(args: Args) -> Result<()> {
    let info = serde_json::json!({
        "name": env!("CARGO_PKG_NAME"),
        "version": env!("CARGO_PKG_VERSION"),
        "git_sha": option_env!("GENASIS_GIT_SHA").unwrap_or("unknown"),
        "build_profile": if cfg!(debug_assertions) { "debug" } else { "release" },
        "marker_fence_version": "1.0",
    });
    if args.json {
        println!("{}", serde_json::to_string_pretty(&info)?);
    } else {
        println!(
            "genasis {} ({}, fence v{})",
            info["version"].as_str().unwrap_or("?"),
            info["build_profile"].as_str().unwrap_or("?"),
            info["marker_fence_version"].as_str().unwrap_or("?"),
        );
    }
    Ok(())
}
