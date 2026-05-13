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
pub struct ClaudeTeamSession {
    #[allow(dead_code)]
    child: Child,
    stdin: ChildStdin,
    /// stdout NDJSON event stream — assistant / tool_use / result 등
    events: mpsc::Receiver<SessionEvent>,
    /// session_id (claude 가 init 이벤트로 emit) — debug + crash 후 resume
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
    ) -> Result<Self> {
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
            .stderr(std::process::Stdio::piped());

        let mut child = cmd.spawn().context("spawn claude session subprocess")?;
        let stdin = child.stdin.take().ok_or_else(|| anyhow!("no stdin"))?;
        let stdout = child.stdout.take().ok_or_else(|| anyhow!("no stdout"))?;

        let (tx, rx) = mpsc::channel::<SessionEvent>(256);
        let mut session_id = None;

        // 첫 init event 를 받아서 session_id 채움 + 나머지는 background task
        let mut reader = BufReader::new(stdout).lines();

        // 동기 phase: init 라인 1개 + 옵션 추가 metadata 라인까지 받기 위해
        // 짧은 loop. 5초 안에 init 안 오면 에러.
        let init_deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(15);
        loop {
            let read_result = tokio::time::timeout(
                init_deadline.saturating_duration_since(tokio::time::Instant::now()),
                reader.next_line(),
            )
            .await;
            match read_result {
                Ok(Ok(Some(line))) => {
                    if let Ok(v) = serde_json::from_str::<Value>(&line) {
                        if v.get("type").and_then(|t| t.as_str()) == Some("system")
                            && v.get("subtype").and_then(|s| s.as_str()) == Some("init")
                        {
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
                            session_id = Some(sid.clone());
                            info!(
                                target: "listen",
                                session_id = %sid,
                                mcp_servers = ?mcp_list,
                                "claude team session init"
                            );
                            let _ = tx
                                .send(SessionEvent::Init {
                                    session_id: sid,
                                    mcp_servers: mcp_list,
                                })
                                .await;
                            break;
                        }
                    }
                }
                Ok(Ok(None)) => return Err(anyhow!("claude session stdout closed before init")),
                Ok(Err(e)) => return Err(anyhow!("claude session stdout read: {e}")),
                Err(_) => return Err(anyhow!("claude session init timeout (15s)")),
            }
        }

        // background task: 나머지 stdout 을 SessionEvent 로 변환 후 tx 로 push
        tokio::spawn(async move {
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
                let event = match ty {
                    "assistant" => parse_assistant(&v),
                    "tool_use" => parse_tool_use(&v),
                    "result" => parse_result(&v),
                    _ => Some(SessionEvent::Other(v)),
                };
                if let Some(ev) = event {
                    if tx.send(ev).await.is_err() {
                        debug!(target: "listen", "session event receiver dropped");
                        break;
                    }
                }
            }
            debug!(target: "listen", "claude session stdout reader ended");
        });

        Ok(Self {
            child,
            stdin,
            events: rx,
            session_id,
        })
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

    /// 다음 이벤트까지 await. None 반환 = stdout 닫힘 (subprocess 종료).
    pub async fn next_event(&mut self) -> Option<SessionEvent> {
        self.events.recv().await
    }

    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }
}

fn parse_assistant(v: &Value) -> Option<SessionEvent> {
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
    let mut text = String::new();
    for b in &blocks {
        if b.get("type").and_then(|t| t.as_str()) == Some("text") {
            if let Some(t) = b.get("text").and_then(|t| t.as_str()) {
                if !text.is_empty() {
                    text.push('\n');
                }
                text.push_str(t);
            }
        }
    }
    if text.is_empty() {
        return None;
    }
    Some(SessionEvent::AssistantText { text, session_id })
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
