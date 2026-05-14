//! `genasis listen` — reactive bridge daemon (genesis §0 + §28 등가물).
//!
//! Subcommand 매트릭스:
//!
//! | sub | 동작 |
//! |---|---|
//! | (none / `run`) | foreground 실행 — Ctrl-C 까지 |
//! | `start` | 백그라운드 spawn + `.genasis/listen.pid` 작성 |
//! | `stop` | PID 파일 가리키는 daemon 에 SIGTERM (3 초 후 SIGKILL) |
//! | `status` | PID + 최근 로그 3 줄 |
//! | `restart` | stop → start |
//! | `logs` | `.genasis/listen.log` 출력. `-f` 면 tail follow |
//!
//! flavor 라우팅 (`genasis.toml`):
//! - `[trial].enabled = true` 또는 `--trial` 명시 → `TrialAppSseStream`
//!   + `TrialAppSink`.
//! - 그 외 → `MattermostWsStream` + `MattermostSink`.

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

use genasis_core::config::{slugify, Config, CONFIG_FILE_NAME};

use crate::listen::lifecycle::{
    log_path, pid_path, read_pid, remove_pid_file, start_precheck, status, stop_daemon, write_pid,
    StartPrecheck,
};
use crate::listen::mattermost_ws::MattermostWsStream;
use crate::listen::session;
use crate::listen::trial_sse::TrialAppSseStream;
use crate::listen::{run_listen_loop_multi, EventStream, LoopConfig, SessionFactory};

#[derive(Parser, Debug)]
pub struct Args {
    /// Project root. Defaults to current working directory.
    #[arg(long, value_name = "DIR", global = true)]
    pub project: Option<PathBuf>,

    /// Force trial flavor even if `genasis.toml [trial].enabled = false`.
    #[arg(long, global = true)]
    pub trial: bool,

    /// Echo-only mode — `claude --print` 호출 안 함. CI / claude CLI
    /// 미설치 환경에서 reactive 파이프라인만 검증.
    #[arg(long, global = true)]
    pub echo_only: bool,

    /// 응답 게시할 default actor (페르소나). trial flavor 의 sim_posts
    /// `actor` 필드, real Mattermost 의 메시지 prefix 둘 다 이 값 사용.
    #[arg(long, default_value = "pm", global = true)]
    pub default_actor: String,

    /// `claude --print` per-call timeout (초).
    ///
    /// D-093: default 600s — cold `npm install` (2~4 분) 이 들어가는 trial
    /// flavor 의 frontend → devops chain 이 120s 안에 못 끝나서 turn 이
    /// silently killed 되는 케이스가 빈번. devops 의 `announce_dev_server_url`
    /// 호출 전 끊기면 사용자 iframe 이 "에이전트가 빌드 중…" 무한 대기.
    /// CI / smoke-test 가 더 짧은 값을 원하면 `--claude-timeout-secs 120`
    /// 명시로 override.
    #[arg(long, default_value_t = 600u32, global = true)]
    pub claude_timeout_secs: u32,

    /// 처리할 이벤트 최대 개수. 0 = 무한 (default). 자가테스트에서 명시
    /// 적으로 1·2 등 작은 값 줘서 deterministic 검증.
    #[arg(long, default_value_t = 0u32, global = true)]
    pub max_events: u32,

    /// D-028: 각 agent 의 "착수 → 완료" 사이 의도적 작업 시간 (초). echo-only
    /// 모드에서도 사람이 칸반의 In Progress 카드 이동을 인지할 수 있게 한다.
    /// 0 으로 두면 즉시 done — 시뮬레이션 티 나는 옛 동작.
    #[arg(long, default_value_t = 6u32, global = true)]
    pub agent_work_secs: u32,

    /// D-028: 한 agent 완료 후 다음 agent 착수까지 대기 (초). 동시 처리가
    /// 아니라 순차 협업으로 보이게 한다.
    #[arg(long, default_value_t = 3u32, global = true)]
    pub agent_gap_secs: u32,

    #[command(subcommand)]
    pub sub: Option<ListenCmd>,
}

#[derive(Subcommand, Debug)]
pub enum ListenCmd {
    /// 백그라운드 daemon 으로 띄움 — PID 파일 `.genasis/listen.pid`,
    /// 로그 `.genasis/listen.log`.
    Start,
    /// 백그라운드 daemon 종료. SIGTERM → 3 초 대기 → SIGKILL.
    Stop,
    /// 현재 PID + 최근 로그 3 줄.
    Status,
    /// `stop` + `start`.
    Restart,
    /// `.genasis/listen.log` 출력. `-f` 면 follow.
    Logs {
        #[arg(short = 'f', long)]
        follow: bool,
    },
}

pub async fn run(args: Args) -> Result<()> {
    let project_root = resolve_project_root(args.project.as_deref())?;
    match args.sub {
        None => run_foreground(&project_root, &args).await,
        Some(ListenCmd::Start) => cmd_start(&project_root, &args),
        Some(ListenCmd::Stop) => stop_daemon(&project_root),
        Some(ListenCmd::Status) => status(&project_root),
        Some(ListenCmd::Restart) => {
            stop_daemon(&project_root).ok();
            cmd_start(&project_root, &args)
        }
        Some(ListenCmd::Logs { follow }) => cmd_logs(&project_root, follow),
    }
}

async fn run_foreground(project_root: &Path, args: &Args) -> Result<()> {
    let (cfg, stream) = build_loop_components(project_root, args).await?;
    println!(
        "→ genasis listen (foreground) — project={} max_events={}",
        cfg.project_slug, args.max_events
    );
    println!("  Ctrl-C 로 종료. 백그라운드 daemon 으로 띄우려면 `genasis listen start`.");

    // v0.6.0 alpha.6+: long-running claude session + MCP tool 모드 만.
    // v0.5.x marker fallback / echo-only / claude --print 경로 모두 폐기.
    // M-v6.0.4: factory closure 를 run_listen_loop_multi 에 넘김 — 들어오는
    // team_token 별로 lazy-spawn. 단일 team 케이스도 같은 코드 경로.
    let cfg_data = genasis_core::config::Config::load(&project_root.join(CONFIG_FILE_NAME))?;
    let flavor = if args.trial || cfg_data.trial.as_ref().map(|t| t.enabled).unwrap_or(false) {
        "trial"
    } else {
        "real"
    };
    let trial_url = cfg_data
        .trial
        .as_ref()
        .map(|t| t.url.clone())
        .unwrap_or_default();
    // D-057: bundled MCP server + SDK runtime unpack. release 바이너리는 CI
    // 머신 경로 (`/home/runner/...`) 가 박힌 채 ship 되고, 사용자 머신엔
    // `@modelcontextprotocol/sdk` 도 보통 글로벌 설치 안 되어 있다 → 두
    // 문제를 mcp_bundle 이 한 번에 해결. 첫 호출에서 `~/.cache/genasis/`
    // 에 .mjs 임베드 본문을 풀고 `npm install` 로 SDK 도 받아둔다.
    let mcp_bundle = crate::mcp_bundle::ensure_mcp_servers()
        .context("preparing bundled MCP server scripts + SDK runtime")?;

    let project_root_owned = project_root.to_path_buf();
    let project_slug = cfg.project_slug.clone();
    let project_name = cfg.project_name.clone();
    let flavor_owned = flavor.to_string();
    let trial_url_owned = trial_url;
    let mcp_dir_owned = mcp_bundle.server_dir;
    let node_modules_owned = mcp_bundle.node_modules;

    let factory: SessionFactory = Box::new(move |team_token: &str| {
        let project_root = project_root_owned.clone();
        let project_slug = project_slug.clone();
        let project_name = project_name.clone();
        let flavor = flavor_owned.clone();
        let trial_url = trial_url_owned.clone();
        let mcp_dir = mcp_dir_owned.clone();
        let node_modules = node_modules_owned.clone();
        let team_token = team_token.to_string();
        Box::pin(async move {
            let mcp_config = session::build_mcp_config(
                &flavor,
                &trial_url,
                &team_token,
                &project_slug,
                &project_name,
                &mcp_dir,
                &node_modules,
            );
            let append = session::build_append_system_prompt(
                &flavor,
                &team_token,
                &project_slug,
                &project_name,
            );
            let mcp_config_str = serde_json::to_string(&mcp_config)?;
            // D-098: pass team identity through env so any Bash subprocess
            // the agent spawns (e.g. `vite --base /dev/$GENASIS_TEAM_TOKEN/`)
            // expands correctly. Without these the orchestrator claude
            // inherits the daemon's env which only had GENASIS_TEAM_TOKEN
            // defined for the MCP subprocess, not the parent.
            let env_vars = vec![
                ("GENASIS_TEAM_TOKEN".to_string(), team_token.clone()),
                ("GENASIS_PROJECT_SLUG".to_string(), project_slug.clone()),
                ("GENASIS_PROJECT_NAME".to_string(), project_name.clone()),
                ("GENASIS_FLAVOR".to_string(), flavor.clone()),
                ("GENASIS_TRIAL_URL".to_string(), trial_url.clone()),
            ];
            session::ClaudeTeamSession::spawn(
                &project_root,
                &mcp_config_str,
                &append,
                &env_vars,
            )
            .await
        })
    });

    run_listen_loop_multi(stream, cfg, factory).await
}

fn cmd_start(project_root: &Path, args: &Args) -> Result<()> {
    match start_precheck(project_root)? {
        StartPrecheck::AlreadyRunning(pid) => {
            anyhow::bail!(
                "listen daemon 이미 실행 중 (PID {pid}). `genasis listen stop` 또는 `restart` 로 종료 후 재시작."
            );
        }
        StartPrecheck::StalePid => {
            println!("listen: stale PID 파일 발견 — 정리 후 진행");
            remove_pid_file(project_root);
        }
        StartPrecheck::Clean => {}
    }

    // 동일 인자로 자기 자신을 background spawn. `--project` 명시 +
    // subcommand 없는 foreground 모드 + nohup 패턴.
    let exe = std::env::current_exe().context("current_exe")?;
    let mut cmd = std::process::Command::new(&exe);
    cmd.arg("listen");
    cmd.arg("--project").arg(project_root);
    if args.trial {
        cmd.arg("--trial");
    }
    if args.echo_only {
        cmd.arg("--echo-only");
    }
    cmd.arg("--default-actor").arg(&args.default_actor);
    cmd.arg("--claude-timeout-secs")
        .arg(args.claude_timeout_secs.to_string());
    cmd.arg("--max-events").arg(args.max_events.to_string());
    cmd.arg("--agent-work-secs")
        .arg(args.agent_work_secs.to_string());
    cmd.arg("--agent-gap-secs")
        .arg(args.agent_gap_secs.to_string());

    let log = log_path(project_root);
    crate::listen::lifecycle::ensure_genasis_dir(project_root)?;
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log)
        .with_context(|| format!("open log {}", log.display()))?;
    let stderr_file = log_file.try_clone().context("clone log fd for stderr")?;
    cmd.stdout(log_file)
        .stderr(stderr_file)
        .stdin(std::process::Stdio::null());

    // detach: setsid (POSIX) 로 새 세션 → SIGHUP 부모 종료 영향 안 받음.
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            cmd.pre_exec(|| {
                libc_setsid();
                Ok(())
            });
        }
    }

    let child = cmd.spawn().context("spawn background listen")?;
    let pid = child.id();
    write_pid(project_root, pid)?;
    println!("listen: ✅ 시작 (PID {pid}). 로그: {}", log.display());
    println!("  상태: `genasis listen status`,  종료: `genasis listen stop`");
    Ok(())
}

#[cfg(unix)]
unsafe extern "C" {
    fn setsid() -> i32;
}

#[cfg(unix)]
unsafe fn libc_setsid() {
    unsafe {
        setsid();
    }
}

#[cfg(not(unix))]
unsafe fn libc_setsid() {}

fn cmd_logs(project_root: &Path, follow: bool) -> Result<()> {
    let log = log_path(project_root);
    if !log.is_file() {
        println!(
            "listen: 로그 파일 없음 ({}). 아직 한 번도 안 띄움?",
            log.display()
        );
        return Ok(());
    }
    if !follow {
        let s = std::fs::read_to_string(&log)?;
        print!("{s}");
        return Ok(());
    }
    // 간단한 follow — tail -f 형태.
    use std::io::{Read, Seek, SeekFrom};
    let mut f = std::fs::File::open(&log)?;
    f.seek(SeekFrom::End(0))?;
    loop {
        let mut buf = [0u8; 4096];
        let n = f.read(&mut buf)?;
        if n == 0 {
            std::thread::sleep(std::time::Duration::from_millis(200));
            continue;
        }
        print!("{}", String::from_utf8_lossy(&buf[..n]));
    }
}

async fn build_loop_components(
    project_root: &Path,
    args: &Args,
) -> Result<(LoopConfig, Box<dyn EventStream>)> {
    let cfg_path = project_root.join(CONFIG_FILE_NAME);
    if !cfg_path.is_file() {
        return Err(anyhow!(
            "no {} at {} — `genasis init --trial` 먼저",
            CONFIG_FILE_NAME,
            cfg_path.display()
        ));
    }
    let cfg = Config::load(&cfg_path)?;
    let project_name = cfg
        .plane
        .as_ref()
        .and_then(|p| p.project_name.clone())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| cfg.project.name.clone());
    let project_slug = slugify(&project_name);
    let trial_enabled = args.trial || cfg.trial.as_ref().map(|t| t.enabled).unwrap_or(false);
    let loop_cfg = LoopConfig {
        max_events: args.max_events,
        project_name: project_name.clone(),
        project_slug: project_slug.clone(),
        // v0.6.0: project_root 는 사용자 sandbox cwd. agent SDK 의 cwd 가 됨.
        project_root: project_root.to_path_buf(),
    };
    if trial_enabled {
        let trial = cfg
            .trial
            .as_ref()
            .ok_or_else(|| anyhow!("[trial] section 누락"))?;
        let team_token = trial
            .team_token
            .clone()
            .ok_or_else(|| anyhow!("[trial].team_token 없음 — `genasis init --trial` 다시"))?;
        info!(
            target: "listen",
            flavor = "trial",
            base = %trial.url,
            team_token_short = %team_token.chars().take(8).collect::<String>(),
            "listen daemon → trial flavor"
        );
        let stream: Box<dyn EventStream> =
            Box::new(TrialAppSseStream::new(&trial.url, &team_token)?);
        Ok((loop_cfg, stream))
    } else {
        let mm = cfg
            .mattermost
            .as_ref()
            .ok_or_else(|| anyhow!("[mattermost] section 누락"))?;
        let token = std::env::var("MM_ADMIN_TOKEN").map_err(|_| {
            anyhow!(
                "real Mattermost listen 은 MM_ADMIN_TOKEN env 필수 (또는 \
                 `genasis listen --trial` 로 trial flavor 강제)"
            )
        })?;
        info!(
            target: "listen",
            flavor = "real",
            mm_url = %mm.url,
            "listen daemon → real Mattermost"
        );
        // M-v6.0.4: real flavor 도 multi-team 가능. team key 는 MM_TEAM_ID
        // (없으면 mattermost URL host) — 운영자가 N 개의 MM 인스턴스 동시
        // 호스팅 하려면 데몬도 N 개 띄우면 되고, 같은 MM 에서 채널만 분리
        // 한다면 단일 데몬 단일 key 로 충분.
        let team_key = std::env::var("MM_TEAM_ID").unwrap_or_else(|_| mm.url.clone());
        let stream: Box<dyn EventStream> =
            Box::new(MattermostWsStream::connect(&mm.url, &token, vec![], team_key).await?);
        Ok((loop_cfg, stream))
    }
}

fn resolve_project_root(arg: Option<&Path>) -> Result<PathBuf> {
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

// 디버그: 사용 안되는 import 경고 회피용 stub
#[allow(dead_code)]
fn _stub(_pid_path: &Path) {
    let _ = pid_path;
    let _ = read_pid;
}
