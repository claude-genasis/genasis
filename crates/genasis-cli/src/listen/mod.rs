//! Reactive bridge — `genasis listen` 의 내부 구조.
//!
//! genesis §0 (사람↔팀 소통은 Mattermost+Plane 만) + §28 (Mattermost
//! Bridge) 의 trial-app 등가물. flavor 에 따라 두 갈래:
//!
//! - **trial** → `TrialAppSseStream` (trial-app `/api/events/stream` SSE)
//!   + `TrialAppSink` (`/api/mattermost/posts` POST + bootstrap idempotent
//!   transition).
//! - **auto/real** → `MattermostWsStream` (`/api/v4/websocket`,
//!   `authentication_challenge` 후 `event="posted"` 필터) +
//!   `MattermostSink` (`/api/v4/posts` POST + Plane transition).
//!
//! 두 갈래 모두 같은 `run_listen_loop` 가 소비한다. flavor 가 어디든
//! "사람-같은 actor 의 새 메시지가 도착하면 `claude --print` 띄워 응답을
//! 같은 채널에 reply 한다" 라는 단일 의미만 유지.

use anyhow::Result;
use async_trait::async_trait;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{info, warn};

pub mod lifecycle;
pub mod mattermost_ws;
pub mod trial_sse;

/// 사람이 채팅 채널에 올린 메시지 → `claude --print` 입력으로 변환할
/// 정도까지 정규화한 inbound 이벤트. Plane 이슈 변경 이벤트는 본
/// reactive loop 범위 밖이라 enum 에 안 둠 (필요 시 variant 추가).
#[derive(Debug, Clone)]
pub enum InboundEvent {
    PostCreated {
        /// trial flavor: `sim_posts.id`. real Mattermost: 빈 문자열 또는
        /// post UUID.
        post_id: String,
        channel_id: String,
        channel_name: Option<String>,
        actor: String,
        message: String,
        thread_root_id: Option<String>,
        /// 사람 패턴 휴리스틱 사전평가 — stream 단계에서 판단하는 게 다른
        /// flavor 의 actor 컨벤션 (trial: "human", mm: bot 봇 user_id !=
        /// our agent set) 을 한 곳에서 흡수하기 쉬워서.
        is_human: bool,
    },
}

/// 각 flavor 의 inbound 채널을 hide. `next_event` 는 cancel-safe 보장.
#[async_trait]
pub trait EventStream: Send {
    /// 다음 이벤트까지 await. 연결 끊기면 자체 재연결 후 다음 이벤트
    /// 반환. 영구 실패만 `Err`.
    async fn next_event(&mut self) -> Result<InboundEvent>;
}

/// 각 flavor 의 outbound 채널 — reply 작성 + 카드 transition. trial 일
/// 때는 모든 호출이 trial-app 으로, real 일 때는 진짜 Mattermost +
/// Plane 으로 라우팅된다 (genesis §0 대전제 격리).
#[async_trait]
pub trait EventSink: Send + Sync {
    /// 같은 채널 (또는 thread) 에 `actor` 명의로 `text` 글을 올린다.
    async fn reply(
        &self,
        triggered_by: &InboundEvent,
        actor: &str,
        text: &str,
    ) -> Result<()>;

    /// 직전 사람 메시지가 "X 완료" 류 의도를 담고 있을 때 호출 — flavor
    /// 별로 카드 transition (trial: bootstrap 재요청 / mm+plane: Plane
    /// REST PATCH). 인자 message 가 휴리스틱 입력.
    async fn maybe_transition_for_directive(&self, message: &str) -> Result<()>;
}

/// 모든 flavor 가 공유하는 메인 loop. EventStream / EventSink 가 어디로
/// 라우팅되든 본 함수 책임 한 곳에서:
///   1. event 받기 (await)
///   2. `is_human == false` 면 skip
///   3. `claude --print` 또는 `--echo-only` 모드에 따른 응답 생성
///   4. sink.reply
///   5. sink.maybe_transition_for_directive (idempotent)
///
/// `max_events == 0` → 무한 loop (default). 양수면 그만큼 처리 후 종료
/// — 자가테스트 / 디버그 용.
pub async fn run_listen_loop(
    mut stream: Box<dyn EventStream>,
    sink: &dyn EventSink,
    cfg: LoopConfig,
) -> Result<()> {
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
        match &event {
            InboundEvent::PostCreated {
                actor,
                message,
                is_human,
                ..
            } => {
                if !is_human {
                    continue;
                }
                info!(
                    target: "listen",
                    actor = %actor,
                    message_preview = %message.chars().take(80).collect::<String>(),
                    "received human-authored post"
                );
                let response_text = generate_response(message, &cfg).await;
                if let Err(e) = sink.reply(&event, &cfg.default_actor, &response_text).await {
                    warn!("sink.reply failed: {e}");
                }
                if let Err(e) = sink.maybe_transition_for_directive(message).await {
                    warn!("sink.transition failed: {e}");
                }
            }
        }
        handled += 1;
        if cfg.max_events > 0 && handled >= cfg.max_events {
            println!("→ reached --max-events={}, exiting", cfg.max_events);
            return Ok(());
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoopConfig {
    pub default_actor: String,
    pub claude_timeout_secs: u32,
    pub echo_only: bool,
    pub max_events: u32,
    /// claude --print 의 system context 보강용 (PRD 제목 + 프로젝트 명).
    pub project_name: String,
    pub project_slug: String,
}

async fn generate_response(message: &str, cfg: &LoopConfig) -> String {
    if cfg.echo_only {
        return format!(
            "[{}] (echo-only) 받음 → \"{}\". 실제 응답은 `genasis listen` 을 \
             --echo-only 없이 띄울 때 활성화됩니다.",
            cfg.default_actor,
            message.chars().take(60).collect::<String>()
        );
    }
    match run_claude_print(message, cfg).await {
        Ok(reply) => reply,
        Err(e) => {
            warn!("claude --print failed: {e} — falling back to canned");
            format!(
                "[{}] 죄송합니다. 응답 생성에 실패했습니다. ({e}) \
                 관련 카드를 점검 중입니다.",
                cfg.default_actor
            )
        }
    }
}

async fn run_claude_print(message: &str, cfg: &LoopConfig) -> Result<String> {
    use tokio::process::Command;
    let prompt = format!(
        "당신은 Genasis 에이전트 팀의 {actor} 역할입니다. \
         프로젝트 \"{project_name}\" (slug: {project_slug}) 의 \
         #scrum-{project_slug} 채널에서 사람이 다음 메시지를 보냈습니다.\n\n\
         사람 메시지: \"{message}\"\n\n\
         3 문장 이내로 답하세요. 필요시 관련 칸반 카드의 상태가 정합되도록 \
         후속 행동(예: '카드 X 를 Done 으로 이동')을 한 줄 명시. 마크다운 사용 금지.",
        actor = cfg.default_actor,
        project_name = cfg.project_name,
        project_slug = cfg.project_slug,
        message = message
    );
    let mut cmd = Command::new("claude");
    cmd.arg("--print")
        .arg(&prompt)
        .arg("--permission-mode")
        .arg("bypassPermissions")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let child = cmd.spawn().map_err(|e| {
        anyhow::anyhow!("spawn `claude` (is it on $PATH?): {e}")
    })?;
    let out = timeout(
        Duration::from_secs(cfg.claude_timeout_secs as u64),
        child.wait_with_output(),
    )
    .await
    .map_err(|_| anyhow::anyhow!("claude --print timed out"))??;
    if !out.status.success() {
        anyhow::bail!(
            "claude --print exited {}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if stdout.is_empty() {
        anyhow::bail!("claude --print produced empty output");
    }
    Ok(stdout)
}

/// 사람 actor 휴리스틱 — trial flavor 의 sim_posts.actor 가 자유 문자열
/// 이라 정확한 화이트리스트 못 만들고 "known agent bot 이 아닌 것" 룰
/// 사용. real Mattermost 일 때는 bot user_id 비교가 더 정확.
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
    ];
    !KNOWN_BOTS.iter().any(|b| a.eq_ignore_ascii_case(b))
}

/// "X 완료", "전체 done", "정리해줘" 류 휴리스틱.
pub fn message_requests_done(message: &str) -> bool {
    let m = message.to_ascii_lowercase();
    const KEYWORDS: &[&str] = &["done", "완료", "끝났", "정리", "마무리"];
    KEYWORDS.iter().any(|k| m.contains(k))
}
