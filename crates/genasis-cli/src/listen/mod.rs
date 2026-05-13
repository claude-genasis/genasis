//! Reactive bridge — `genasis listen` 의 내부 구조 (v0.6.0).
//!
//! 흐름은 단순:
//! 1. EventStream (trial-app SSE 또는 Mattermost WS) 에서 사람 메시지 수신
//! 2. team 별 ClaudeTeamSession 의 stdin 에 NDJSON push
//! 3. session 안의 PM / sub-agent 가 MCP tool 로 외부 시스템 직접 조작
//! 4. 데몬은 broker — marker 파싱 / sim DB 호출 / cleanup 모두 폐기 (v0.5.x)

use anyhow::Result;
use async_trait::async_trait;
use std::time::Duration;
use tracing::{info, warn};

pub mod lifecycle;
pub mod mattermost_ws;
pub mod sdk;
pub mod session;
pub mod trial_sse;

/// 사람이 채팅 채널에 올린 메시지의 정규화된 형태.
#[derive(Debug, Clone)]
pub enum InboundEvent {
    PostCreated {
        post_id: String,
        channel_id: String,
        channel_name: Option<String>,
        actor: String,
        message: String,
        thread_root_id: Option<String>,
        is_human: bool,
    },
}

impl InboundEvent {
    pub fn is_human(&self) -> bool {
        match self {
            InboundEvent::PostCreated { is_human, .. } => *is_human,
        }
    }

    pub fn message_preview(&self) -> String {
        match self {
            InboundEvent::PostCreated { message, .. } => {
                message.chars().take(120).collect()
            }
        }
    }
}

/// flavor 의 inbound 채널. cancel-safe.
#[async_trait]
pub trait EventStream: Send {
    async fn next_event(&mut self) -> Result<InboundEvent>;
}

#[derive(Debug, Clone)]
pub struct LoopConfig {
    pub project_name: String,
    pub project_slug: String,
    /// agent SDK 의 cwd. agent 의 Read/Edit/Write/Bash 가 이 디렉토리 기준.
    pub project_root: std::path::PathBuf,
    /// 처리할 이벤트 최대 개수. 0 = 무한. 자가테스트용.
    pub max_events: u32,
}

/// v0.6.0 main loop: 사람 메시지 → session.send_user_message.
/// session 의 stdout event 는 별도 background task 가 drain + 로그 + 사용자
/// status announce (D-050).
pub async fn run_listen_loop_session(
    mut stream: Box<dyn EventStream>,
    mut session: session::ClaudeTeamSession,
    mut events: tokio::sync::mpsc::Receiver<session::SessionEvent>,
    cfg: LoopConfig,
) -> Result<()> {
    // background drain — D-048: tool_use 와 assistant text 둘 다 로그.
    tokio::spawn(async move {
        while let Some(ev) = events.recv().await {
            match ev {
                session::SessionEvent::Init {
                    session_id,
                    mcp_servers,
                } => {
                    info!(
                        target: "listen",
                        session_id = %session_id,
                        mcp_servers = ?mcp_servers,
                        "claude team session init (lazy)"
                    );
                }
                session::SessionEvent::AssistantText { text, .. } => {
                    info!(
                        target: "listen",
                        text_preview = %text.chars().take(120).collect::<String>(),
                        "session assistant text"
                    );
                }
                session::SessionEvent::ToolUse { tool_name, input } => {
                    info!(
                        target: "listen",
                        tool = %tool_name,
                        input_preview = %input.to_string().chars().take(160).collect::<String>(),
                        "session tool_use"
                    );
                }
                session::SessionEvent::Result {
                    success,
                    duration_ms,
                    ..
                } => {
                    info!(target: "listen", success, duration_ms, "session turn complete");
                }
                session::SessionEvent::Other(_) => {}
            }
        }
    });

    let mut handled: u32 = 0;
    loop {
        let event = match stream.next_event().await {
            Ok(e) => e,
            Err(e) => {
                warn!("event stream error: {e} — sleeping 5s before retry");
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        };
        if !event.is_human() {
            continue;
        }
        let preview = event.message_preview();
        let message = match &event {
            InboundEvent::PostCreated { message, .. } => message.clone(),
        };
        info!(
            target: "listen",
            preview = %preview,
            "received human-authored post → session.send"
        );
        if let Err(e) = session.send_user_message(&message).await {
            warn!("session send failed: {e}");
        }
        handled += 1;
        if cfg.max_events > 0 && handled >= cfg.max_events {
            println!("→ reached --max-events={}, exiting", cfg.max_events);
            return Ok(());
        }
    }
}

/// 사람 actor 휴리스틱 — system actor 들 (deploy/cleanup 같은 데몬 자체
/// 게시본) 제외. session path 가 자기 응답을 SSE 로 다시 안 받게.
pub fn is_human_actor(actor: &str) -> bool {
    let a = actor.to_ascii_lowercase();
    const HINTS: &[&str] = &["human", "user", "you", "사람", "사용자"];
    if HINTS.iter().any(|h| a.contains(h)) {
        return true;
    }
    const KNOWN_BOTS: &[&str] = &[
        "pm",
        "planner",
        "architect",
        "frontend",
        "backend",
        "qa",
        "designer",
        "security",
        "devops",
        "code-reviewer",
        "genasis",
        "diagnostic",
        "system",
        "deploy",
        "cleanup",
        "agent",
    ];
    !KNOWN_BOTS.iter().any(|b| a.eq_ignore_ascii_case(b))
}
