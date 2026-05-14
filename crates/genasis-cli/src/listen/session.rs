//! v0.6.0 alpha.3 — Long-running team session (사용자 §"진짜 agentic team").
//!
//! `claude -p --input-format stream-json --output-format stream-json` 을
//! team_token 별로 1개씩 spawn 한다. 외부 (trial-app SSE / real Mattermost WS)
//! 에서 사람 메시지가 도착하면 같은 session 의 stdin 에 NDJSON push —
//! main agent (PM) 가 받아 Task tool 로 sub-agent 호출, 같은 컨텍스트
//! 안에서 협업.
//!
//! v0.5.x 의 marker 파싱 (parse_pm_routing + apply_pm_routing) 은 폐기
//! 대상. agent 가 외부 시스템 (trial-app / Plane / Mattermost) 을 MCP
//! tool 로 직접 호출하므로 데몬은 router 가 아니라 단순 message broker.
//!
//! 인증: claude CLI subprocess — 사용자 Pro 구독. 별도 API 비용 없음
//! (memory: `feedback_no_claude_api`).

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::Path;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// 한 team 의 long-running claude session. 사람 메시지 N건이 같은
/// 인스턴스로 흘러들어가 main agent 가 컨텍스트 유지하며 처리.
///
/// 데몬은 보통 `send_user_message` 만 호출. session 안의 agent 들이
/// MCP tool (post_message / transition_issue 등) 로 직접 외부 시스템
/// 조작하므로 데몬은 응답 stream 을 별도 처리할 필요 없음 — 그저 로그.
/// `events_rx` 는 spawn 시 외부로 노출되어 background task 가 drain.
pub struct ClaudeTeamSession {
    #[allow(dead_code)]
    child: Child,
    stdin: ChildStdin,
    /// session_id (claude 가 init 이벤트로 emit) — debug 용
    session_id: Option<String>,
}

/// stdout NDJSON 이벤트의 정규화 enum. 데몬은 보통 `AssistantText`
/// (사용자에게 보여줄 응답) 와 `Result` (한 turn 완료) 만 신경 쓰면 됨.
/// `ToolUse` 는 디버깅용 stream — agent 가 어떤 MCP tool 호출했는지 추적.
#[derive(Debug, Clone)]
pub enum SessionEvent {
    /// 초기화 — session_id, 사용 가능한 tool/MCP server 목록
    Init {
        session_id: String,
        mcp_servers: Vec<String>,
    },
    /// 자연어 응답 텍스트 (사람에게 보여줄 본문 + 마커 추출 — 임시 P4 전까지)
    AssistantText { text: String, session_id: String },
    /// 도구 호출 — MCP tool 또는 built-in (Read/Edit/Write/Bash/Task)
    ToolUse { tool_name: String, input: Value },
    /// 한 turn (사용자 메시지 1건) 처리 종료. duration / cost / 최종 텍스트
    Result {
        success: bool,
        duration_ms: u64,
        final_text: String,
    },
    /// 그 외 (rate_limit_event, hook_event 등) — 보통 로그만 하고 패스
    Other(Value),
}

impl ClaudeTeamSession {
    /// 새 session spawn. `cwd` = team sandbox 디렉토리, `mcp_config` =
    /// MCP server 들이 정의된 in-memory JSON config 또는 file path.
    /// `append_system_prompt` = 데몬이 PM 에게 알려줄 컨텍스트 (channel
    /// 이름, team_token, role 분배 가능 목록 등).
    pub async fn spawn(
        cwd: &Path,
        mcp_config_json: &str,
        append_system_prompt: &str,
    ) -> Result<(Self, mpsc::Receiver<SessionEvent>)> {
        let claude_path = which::which("claude").context("claude CLI 가 PATH 에 없음")?;
        let mut cmd = Command::new(claude_path);
        cmd.arg("-p")
            .arg("--input-format")
            .arg("stream-json")
            .arg("--output-format")
            .arg("stream-json")
            .arg("--verbose")
            .arg("--permission-mode")
            .arg("acceptEdits")
            .arg("--mcp-config")
            .arg(mcp_config_json);
        if !append_system_prompt.trim().is_empty() {
            cmd.arg("--append-system-prompt").arg(append_system_prompt);
        }
        cmd.current_dir(cwd)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            // D-051: 데몬 stop 시 session subprocess (그 자식 MCP server
            // 들 포함) leak 안 되도록. Drop 시 SIGKILL.
            .kill_on_drop(true);

        let mut child = cmd.spawn().context("spawn claude session subprocess")?;
        let stdin = child.stdin.take().ok_or_else(|| anyhow!("no stdin"))?;
        let stdout = child.stdout.take().ok_or_else(|| anyhow!("no stdout"))?;

        let (tx, rx) = mpsc::channel::<SessionEvent>(256);
        let session_id: Option<String> = None;

        // claude stream-json 모드는 stdin 으로 첫 NDJSON 메시지가 도착할
        // 때까지 init 도 발행 안 함. spawn 시점에서 init 동기 wait 하면
        // 영원히 timeout — lazy init 으로 변경. background task 가 stdout
        // 을 drain 하면서 init/assistant/result 이벤트 발견하면 channel
        // 로 forward. session_id 는 첫 init 이벤트 도착 시점에 채워짐 (외부
        // 에서 SessionEvent::Init 받는 쪽에서 인지).
        let reader = BufReader::new(stdout).lines();
        info!(
            target: "listen",
            "claude team session subprocess spawned — waiting for first user message before init"
        );

        // background task: stdout 을 SessionEvent 로 변환 후 tx 로 push.
        // init / assistant / result 모두 같은 stream 에 섞여 옴.
        tokio::spawn(async move {
            let mut reader = reader;
            while let Ok(Some(line)) = reader.next_line().await {
                if line.trim().is_empty() {
                    continue;
                }
                let v: Value = match serde_json::from_str(&line) {
                    Ok(v) => v,
                    Err(e) => {
                        warn!(target: "listen", err=%e, raw=%line, "session non-JSON line");
                        continue;
                    }
                };
                let ty = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
                let subtype = v.get("subtype").and_then(|t| t.as_str()).unwrap_or("");
                // D-048: assistant 는 content block 안에 text + tool_use
                // 섞임 → 여러 SessionEvent 로 emit.
                let events: Vec<SessionEvent> = match (ty, subtype) {
                    ("system", "init") => {
                        let sid = v
                            .get("session_id")
                            .and_then(|s| s.as_str())
                            .unwrap_or("")
                            .to_string();
                        let mcp_list = v
                            .get("mcp_servers")
                            .and_then(|m| m.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|s| {
                                        s.get("name").and_then(|n| n.as_str()).map(String::from)
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();
                        info!(
                            target: "listen",
                            session_id = %sid,
                            mcp_servers = ?mcp_list,
                            "claude team session init (lazy)"
                        );
                        vec![SessionEvent::Init {
                            session_id: sid,
                            mcp_servers: mcp_list,
                        }]
                    }
                    ("assistant", _) => parse_assistant(&v),
                    ("tool_use", _) => parse_tool_use(&v).into_iter().collect(),
                    ("result", _) => parse_result(&v).into_iter().collect(),
                    _ => vec![SessionEvent::Other(v)],
                };
                for ev in events {
                    if tx.send(ev).await.is_err() {
                        debug!(target: "listen", "session event receiver dropped");
                        return;
                    }
                }
            }
            debug!(target: "listen", "claude session stdout reader ended");
        });

        Ok((
            Self {
                child,
                stdin,
                session_id,
            },
            rx,
        ))
    }

    /// 사람 메시지 1건을 session 에 push. main agent 가 받아 처리.
    pub async fn send_user_message(&mut self, text: &str) -> Result<()> {
        let msg = json!({
            "type": "user",
            "message": {
                "role": "user",
                "content": text,
            }
        });
        let line = format!("{}\n", serde_json::to_string(&msg)?);
        self.stdin
            .write_all(line.as_bytes())
            .await
            .context("write to claude session stdin")?;
        self.stdin.flush().await.context("flush stdin")?;
        Ok(())
    }

    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }
}

/// MCP config inline JSON 생성 — trial flavor 면 trial-app server 등록,
/// real flavor 면 mattermost + plane server 등록. 환경변수로 team_token /
/// project_slug 등 주입 (MCP server 의 env 로).
pub fn build_mcp_config(
    flavor: &str,
    trial_url: &str,
    team_token: &str,
    project_slug: &str,
    project_name: &str,
    mcp_server_dir: &Path,
    node_modules: &Path,
) -> serde_json::Value {
    let channel_name = format!("scrum-{}", project_slug);
    let trial_index = mcp_server_dir
        .join("trial-app")
        .join("index.mjs")
        .display()
        .to_string();
    let mut servers = serde_json::Map::new();
    // D-057: NODE_PATH 는 mcp_bundle 이 만든 cache 의 node_modules 를 사용.
    // GENASIS_NODE_PATH env override 는 유지 (디버깅 / 자체 npm 설치 환경).
    // 이전엔 `npm root -g` 였지만 사용자가 `@modelcontextprotocol/sdk` 를
    // 글로벌 설치하지 않은 경우 SDK 가 안 잡혔다 — D-057 가 cache 디렉터리
    // 안에 lazy npm install 로 SDK 를 받아두므로 그 경로를 우선 사용.
    let node_path =
        std::env::var("GENASIS_NODE_PATH").unwrap_or_else(|_| node_modules.display().to_string());
    if flavor == "trial" || flavor == "auto" {
        servers.insert(
            "trial-app".to_string(),
            serde_json::json!({
                "command": "node",
                "args": [trial_index],
                "env": {
                    "NODE_PATH": node_path,
                    "GENASIS_TRIAL_URL": trial_url,
                    "GENASIS_TEAM_TOKEN": team_token,
                    "GENASIS_PROJECT_SLUG": project_slug,
                    "GENASIS_PROJECT_NAME": project_name,
                    "GENASIS_CHANNEL_NAME": channel_name,
                },
            }),
        );
    }
    // beta: real flavor 면 mattermost + plane MCP server 등록.
    // 환경변수는 daemon process 환경에서 그대로 전달 — 운영자가
    // `genasis listen` 실행 전 export.
    if flavor == "real" || flavor == "auto" {
        let mattermost_index = mcp_server_dir
            .join("mattermost")
            .join("index.mjs")
            .display()
            .to_string();
        let plane_index = mcp_server_dir
            .join("plane")
            .join("index.mjs")
            .display()
            .to_string();
        if std::env::var("MM_URL").is_ok() && std::env::var("MM_ADMIN_TOKEN").is_ok() {
            servers.insert(
                "mattermost".to_string(),
                serde_json::json!({
                    "command": "node",
                    "args": [mattermost_index],
                    "env": {
                        "NODE_PATH": node_path,
                        "MM_URL": std::env::var("MM_URL").unwrap_or_default(),
                        "MM_ADMIN_TOKEN": std::env::var("MM_ADMIN_TOKEN").unwrap_or_default(),
                        "MM_TEAM_ID": std::env::var("MM_TEAM_ID").unwrap_or_default(),
                        "MM_DEFAULT_CHANNEL_ID": std::env::var("MM_DEFAULT_CHANNEL_ID").unwrap_or_default(),
                    },
                }),
            );
        }
        if std::env::var("PLANE_URL").is_ok() && std::env::var("PLANE_API_KEY").is_ok() {
            let mut plane_env = serde_json::Map::new();
            plane_env.insert(
                "NODE_PATH".to_string(),
                serde_json::Value::String(node_path.clone()),
            );
            plane_env.insert(
                "PLANE_URL".to_string(),
                serde_json::Value::String(std::env::var("PLANE_URL").unwrap_or_default()),
            );
            plane_env.insert(
                "PLANE_API_KEY".to_string(),
                serde_json::Value::String(std::env::var("PLANE_API_KEY").unwrap_or_default()),
            );
            plane_env.insert(
                "PLANE_WORKSPACE_SLUG".to_string(),
                serde_json::Value::String(
                    std::env::var("PLANE_WORKSPACE_SLUG").unwrap_or_default(),
                ),
            );
            plane_env.insert(
                "PLANE_PROJECT_ID".to_string(),
                serde_json::Value::String(std::env::var("PLANE_PROJECT_ID").unwrap_or_default()),
            );
            // PLANE_USER_ID_<ROLE> — pass through any matching env.
            for (k, v) in std::env::vars() {
                if k.starts_with("PLANE_USER_ID_") {
                    plane_env.insert(k, serde_json::Value::String(v));
                }
            }
            servers.insert(
                "plane".to_string(),
                serde_json::json!({
                    "command": "node",
                    "args": [plane_index],
                    "env": serde_json::Value::Object(plane_env),
                }),
            );
        }
    }
    serde_json::json!({ "mcpServers": serde_json::Value::Object(servers) })
}

/// PM 에게 주입할 system context — channel / team / role 목록 등.
/// overlay 의 Tera 변수와 별개로 runtime 에 결정되는 값들.
pub fn build_append_system_prompt(
    flavor: &str,
    team_token: &str,
    project_slug: &str,
    project_name: &str,
) -> String {
    format!(
        r#"You are the orchestrator of a Genasis agentic team.

Runtime context:
- Project: {project_name} (slug: {project_slug})
- Flavor: {flavor}
- Team token: {token_short}…
- Scrum channel: scrum-{project_slug}
- MCP servers available: trial-app (if trial) | mattermost+plane (if real)

When a human posts a message, follow the protocol in .claude/agents/pm.md
(MCP tool calls, not marker text). Use the Task tool to invoke sub-agents
defined in .claude/agents/<role>.md. Sub-agents inherit MCP servers and
should call mcp__<server>__<tool> directly — do not emit [CARD: ...] or
similar v0.5.x markers in chat text."#,
        flavor = flavor,
        project_name = project_name,
        project_slug = project_slug,
        token_short = &team_token[..team_token.len().min(8)],
    )
}

/// D-048: assistant message 의 content block 들 안에 text + tool_use 가
/// 섞여 있음. 둘 다 별도 SessionEvent 로 emit 해야 데몬이 어느 MCP tool
/// 호출됐는지 추적 가능. 반환값을 Option<Vec<SessionEvent>> 로 변경.
fn parse_assistant(v: &Value) -> Vec<SessionEvent> {
    let session_id = v
        .get("session_id")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let blocks = v
        .pointer("/message/content")
        .and_then(|c| c.as_array())
        .cloned()
        .unwrap_or_default();
    let mut out = Vec::new();
    for b in &blocks {
        match b.get("type").and_then(|t| t.as_str()) {
            Some("text") => {
                if let Some(t) = b.get("text").and_then(|t| t.as_str()) {
                    if !t.trim().is_empty() {
                        out.push(SessionEvent::AssistantText {
                            text: t.to_string(),
                            session_id: session_id.clone(),
                        });
                    }
                }
            }
            Some("tool_use") => {
                let tool_name = b
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string();
                let input = b.get("input").cloned().unwrap_or(json!({}));
                if !tool_name.is_empty() {
                    out.push(SessionEvent::ToolUse { tool_name, input });
                }
            }
            _ => {}
        }
    }
    out
}

fn parse_tool_use(v: &Value) -> Option<SessionEvent> {
    // claude stream-json 의 tool_use 는 보통 assistant message 안의
    // content block 으로 들어옴. 별도 top-level "tool_use" type 은 드물지만
    // P3 에서 MCP tool 호출 인지하려면 여기서 처리.
    let tool_name = v
        .pointer("/tool_use/name")
        .or_else(|| v.get("name"))
        .and_then(|s| s.as_str())?
        .to_string();
    let input = v
        .pointer("/tool_use/input")
        .or_else(|| v.get("input"))
        .cloned()
        .unwrap_or(json!({}));
    Some(SessionEvent::ToolUse { tool_name, input })
}

fn parse_result(v: &Value) -> Option<SessionEvent> {
    #[derive(Deserialize)]
    struct R {
        is_error: Option<bool>,
        duration_ms: Option<u64>,
        result: Option<String>,
    }
    let r: R = serde_json::from_value(v.clone()).ok()?;
    Some(SessionEvent::Result {
        success: !r.is_error.unwrap_or(false),
        duration_ms: r.duration_ms.unwrap_or(0),
        final_text: r.result.unwrap_or_default(),
    })
}
