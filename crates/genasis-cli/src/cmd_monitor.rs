use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct Args {
    /// Project root containing `genasis.toml`. Defaults to walk-up from
    /// CWD. D-058: 사용자가 testbed 루트 (`/work/agenteams/team-ex`) 에서
    /// `genasis monitor` 를 실행하면 자식 sandbox 의 `genasis.toml` 을
    /// 못 찾아 모든 widget 이 빈 상태였음 — 그 경우 명시적으로 path 를
    /// 주거나 sandbox 안에서 실행해야 함.
    #[arg(long, value_name = "DIR")]
    pub project: Option<PathBuf>,
}

pub async fn run(args: Args) -> Result<()> {
    genasis_monitor::app::run(args.project)
        .await
        .map_err(Into::into)
}
