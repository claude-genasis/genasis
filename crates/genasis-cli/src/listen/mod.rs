//! Reactive bridge — `genasis listen` 의 내부 구조 (v0.6.0).
//!
//! 흐름은 단순:
//! 1. EventStream (trial-app SSE 또는 Mattermost WS) 에서 사람 메시지 수신
//! 2. 메시지의 team_token 기준 ClaudeTeamSession 을 HashMap 에서 lookup
//!    → 없으면 그 자리에서 spawn (M-v6.0.4 lazy multi-team)
//! 3. session 안의 PM / sub-agent 가 MCP tool 로 외부 시스템 직접 조작
//! 4. 데몬은 broker — marker 파싱 / sim DB 호출 / cleanup 모두 폐기 (v0.5.x)

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{info, warn};

pub mod lifecycle;
pub mod mattermost_ws;
pub mod session;
pub mod trial_sse;

/// 사람이 채팅 채널에 올린 메시지의 정규화된 형태.
#[derive(Debug, Clone)]
pub enum InboundEvent {
    PostCreated {
        /// M-v6.0.4: 어떤 team 의 session 으로 라우팅할지 결정하는 키.
        /// trial flavor 에서는 SSE 구독 시점의 team_token, real flavor 에서는
        /// 데몬이 binding 된 team_id 또는 추출된 team_token.
        team_token: String,
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
            InboundEvent::PostCreated { message, .. } => message.chars().take(120).collect(),
        }
    }

    pub fn team_token(&self) -> &str {
        match self {
            InboundEvent::PostCreated { team_token, .. } => team_token.as_str(),
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

/// M-v6.0.4: 새 team_token 이 처음 등장했을 때 그 team 의 session 을
/// 만들어 주는 factory. cmd_listen 이 mcp_config / append_system_prompt /
/// project_root 등을 capture 하는 closure 를 전달한다.
pub type SessionFactory = Box<
    dyn for<'a> Fn(
            &'a str,
        ) -> Pin<
            Box<
                dyn Future<
                        Output = Result<(
                            session::ClaudeTeamSession,
                            mpsc::Receiver<session::SessionEvent>,
                        )>,
                    > + Send
                    + 'a,
            >,
        > + Send
        + Sync,
>;

/// v0.6.0 main loop (M-v6.0.4 multi-team).
///
/// inbound 이벤트의 `team_token` 으로 HashMap 을 조회 — 없으면 factory 호출
/// 로 lazy spawn. 사용자가 단일 team 만 운영하는 일반 케이스도 같은 코드
/// 경로를 타지만, 한 데몬이 여러 team_token 을 수신 (예: 운영자가 여러
/// trial 팀을 동시에 호스팅) 하면 자동으로 분리된 session 으로 라우팅된다.
pub async fn run_listen_loop_multi(
    mut stream: Box<dyn EventStream>,
    cfg: LoopConfig,
    spawn_session: SessionFactory,
) -> Result<()> {
    let mut sessions: HashMap<String, session::ClaudeTeamSession> = HashMap::new();
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
        let team_token = event.team_token().to_string();
        if team_token.is_empty() {
            warn!(target: "listen", "inbound event missing team_token — dropping");
            continue;
        }
        let message = match &event {
            InboundEvent::PostCreated { message, .. } => message.clone(),
        };

        // get-or-spawn
        if !sessions.contains_key(&team_token) {
            let team_label = team_token.clone();
            info!(
                target: "listen",
                team_token_short = %team_label.chars().take(8).collect::<String>(),
                "new team_token — spawning ClaudeTeamSession (lazy)"
            );
            match spawn_session(&team_token).await {
                Ok((sess, rx)) => {
                    spawn_drain(rx, team_label.clone());
                    sessions.insert(team_token.clone(), sess);
                }
                Err(e) => {
                    warn!(
                        target: "listen",
                        team_token_short = %team_label.chars().take(8).collect::<String>(),
                        "session spawn failed: {e} — dropping event"
                    );
                    continue;
                }
            }
        }

        let session = sessions
            .get_mut(&team_token)
            .expect("just inserted or pre-existing");
        info!(
            target: "listen",
            team_token_short = %team_token.chars().take(8).collect::<String>(),
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

/// 한 team session 의 stdout 이벤트를 백그라운드에서 drain.
/// D-048: tool_use 와 assistant text 둘 다 로그.
fn spawn_drain(mut events: mpsc::Receiver<session::SessionEvent>, team_label: String) {
    let label_short: String = team_label.chars().take(8).collect();
    tokio::spawn(async move {
        while let Some(ev) = events.recv().await {
            match ev {
                session::SessionEvent::Init {
                    session_id,
                    mcp_servers,
                } => {
                    info!(
                        target: "listen",
                        team = %label_short,
                        session_id = %session_id,
                        mcp_servers = ?mcp_servers,
                        "claude team session init (lazy)"
                    );
                }
                session::SessionEvent::AssistantText { text, .. } => {
                    info!(
                        target: "listen",
                        team = %label_short,
                        text_preview = %text.chars().take(120).collect::<String>(),
                        "session assistant text"
                    );
                }
                session::SessionEvent::ToolUse { tool_name, input } => {
                    info!(
                        target: "listen",
                        team = %label_short,
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
                    info!(
                        target: "listen",
                        team = %label_short,
                        success,
                        duration_ms,
                        "session turn complete"
                    );
                }
                session::SessionEvent::Other(_) => {}
            }
        }
    });
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
