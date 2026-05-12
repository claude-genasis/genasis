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
pub mod routing;
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
    async fn reply(&self, triggered_by: &InboundEvent, actor: &str, text: &str) -> Result<()>;

    /// 직전 사람 메시지가 "X 완료" 류 의도를 담고 있을 때 호출 — flavor
    /// 별로 카드 transition (trial: bootstrap 재요청 / mm+plane: Plane
    /// REST PATCH). 인자 message 가 휴리스틱 입력.
    async fn maybe_transition_for_directive(&self, message: &str) -> Result<()>;

    /// ADR-018: PM agent 응답에서 추출한 routing 결과를 sink 에 적용.
    /// - app_kind/features → sim_teams 갱신 (trial) 또는 Plane meta 라벨
    ///   (real, 추후 확장)
    /// - new_cards → sim_issues 또는 Plane issue create
    /// - transitions → sim_issues UPDATE 또는 Plane issue PATCH
    async fn apply_pm_routing(&self, routing: &routing::PmRouting) -> Result<()>;
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
                if let Err(e) = handle_human_post(&event, message, sink, &cfg).await {
                    warn!("handle_human_post failed: {e}");
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

/// 단일 사람 메시지를 다음 흐름으로 처리:
///   1. PM 프롬프트로 `claude --print` → PM 응답 (echo-only 면 stub)
///   2. PM 응답을 사람 메시지 스레드에 reply
///   3. `routing::parse_pm_routing` 으로 app/카드/멘션 추출
///   4. sink.apply_pm_routing(routing) — sim DB 업데이트
///   5. 각 멘션된 agent 에 대해 follow-up `claude --print` → 같은 스레드 reply
async fn handle_human_post(
    event: &InboundEvent,
    message: &str,
    sink: &dyn EventSink,
    cfg: &LoopConfig,
) -> Result<()> {
    // (1) PM 응답 생성
    let pm_response = if cfg.echo_only {
        build_echo_pm_response(message, cfg)
    } else {
        let prompt = routing::build_pm_prompt(&cfg.project_name, &cfg.project_slug, message, None);
        match run_claude_print(&prompt, cfg).await {
            Ok(s) => s,
            Err(e) => {
                warn!("PM claude --print failed: {e} — fallback to echo");
                build_echo_pm_response(message, cfg)
            }
        }
    };

    // (2) PM 응답을 사람 메시지 스레드에 reply
    if let Err(e) = sink.reply(event, &cfg.default_actor, &pm_response).await {
        warn!("PM reply failed: {e}");
    }

    // (3) routing 추출
    let route = routing::parse_pm_routing(&pm_response);
    info!(target: "listen", routing = %routing::render_routing_summary(&route), "PM routing parsed");

    // (4) sim DB 적용 (sim_teams app_kind/features + sim_issues 카드)
    if let Err(e) = sink.apply_pm_routing(&route).await {
        warn!("apply_pm_routing failed: {e}");
    }

    // (5) 각 멘션된 agent 에 대해 follow-up
    for assignment in &route.assignments {
        let agent_reply = if cfg.echo_only {
            build_echo_agent_response(assignment, cfg)
        } else {
            // 카드 제목은 new_cards 의 같은 assignee 중 첫 번째와 매칭.
            let card_title = route
                .new_cards
                .iter()
                .find(|c| c.assignee.as_deref() == Some(assignment.role.as_str()))
                .map(|c| c.title.clone());
            let prompt = routing::build_agent_prompt(
                &assignment.role,
                &assignment.task,
                &cfg.project_name,
                &cfg.project_slug,
                card_title.as_deref(),
            );
            match run_claude_print(&prompt, cfg).await {
                Ok(s) => s,
                Err(e) => {
                    warn!(
                        "agent {} claude --print failed: {e} — fallback to echo",
                        assignment.role
                    );
                    build_echo_agent_response(assignment, cfg)
                }
            }
        };
        if let Err(e) = sink.reply(event, &assignment.role, &agent_reply).await {
            warn!("agent {} reply failed: {e}", assignment.role);
        }
        // agent 응답에서 추가 [CARD: ... → state] 마커가 있을 수 있음 — 한 번 더 parse.
        let agent_route = routing::parse_pm_routing(&agent_reply);
        if !agent_route.transitions.is_empty() {
            let _ = sink.apply_pm_routing(&agent_route).await;
        }
    }
    Ok(())
}

fn build_echo_pm_response(message: &str, cfg: &LoopConfig) -> String {
    // ADR-018 §"기존 앱 수정 시나리오": echo-only 모드 stub 도 본
    // 시나리오에 맞춰 quiz 유지 기본. 사람 요구의 키워드를 features 로
    // 매핑한 결과만 누적. PM prompt 가 LLM 모드와 동일한 의도를 유지.
    let _ = cfg;
    let m = message.to_ascii_lowercase();

    // app_kind: 명시적 교체 요구가 있을 때만 변경 (예: "todo 앱으로 바꿔줘")
    let app_kind = if m.contains("todo 앱으로 바꿔") || m.contains("change to todo") {
        "todo"
    } else {
        // 기본: 기존 quiz 유지 (사람 요청은 그 quiz 의 수정/커스터마이즈)
        "quiz"
    };

    // features: 시각 변경 키워드 매핑
    let mut features = Vec::new();
    if m.contains("빨간") || m.contains("red") || m.contains("레드") {
        features.push("accent-red");
    }
    if m.contains("파란") || m.contains("blue") || m.contains("블루") {
        features.push("accent-blue");
    }
    if m.contains("초록") || m.contains("green") || m.contains("그린") {
        features.push("accent-green");
    }
    if m.contains("보라") || m.contains("purple") || m.contains("퍼플") {
        features.push("accent-purple");
    }
    if m.contains("다크") || m.contains("dark") {
        features.push("dark-mode");
    }
    if m.contains("공유") || m.contains("share") {
        features.push("share-button");
    }
    if m.contains("큰 글자") || m.contains("larger") || m.contains("text larger") {
        features.push("larger-text");
    }
    if m.contains("한국어") || m.contains("영어") || m.contains("i18n") || m.contains("다국어")
    {
        features.push("i18n");
    }

    let features_str = if features.is_empty() {
        String::new()
    } else {
        features.join(", ")
    };
    let features_note = if features.is_empty() {
        "스타일 변경 없음".to_string()
    } else {
        features.join(", ")
    };

    format!(
        r#"📥 요구사항 정리: {preview}

기존 쇼케이스 앱 (Claude Code 전문가 진단 퀴즈) 에 대한 수정 요구로 해석.

[APP: {app_kind}]
[FEATURES: {features_str}]

## 작업 분배
- @designer: 시각 변경 검토 ({features_note})
- @frontend: UI 수정 반영 (QuizApp 의 features prop)
- @qa: 변경 후 회귀 (시작 → 질문 → 결과 흐름 정상)

## 새 카드
- "시각 변경 디자인 검토" [@designer] [state=todo]
- "QuizApp features 반영" [@frontend] [state=todo]
- "회귀 시나리오 검증" [@qa] [state=todo]

> @human 작업 분배 완료 (echo-only). 각 agent 의 응답을 같은 스레드에서 확인하세요."#,
        preview = message.chars().take(80).collect::<String>(),
        app_kind = app_kind,
        features_str = features_str,
        features_note = features_note,
    )
}

fn build_echo_agent_response(assignment: &routing::AgentAssignment, cfg: &LoopConfig) -> String {
    let _ = cfg;
    format!(
        r#"✋ @human {role} 착수 — {task}

[CARD: {task_title} → inprogress]

✅ @human {role} 완료 — 시뮬레이션 모드에서 작업 마침

[CARD: {task_title} → done]"#,
        role = assignment.role,
        task = assignment.task,
        task_title = first_three_words(&assignment.task),
    )
}

fn first_three_words(s: &str) -> String {
    s.split_whitespace().take(4).collect::<Vec<_>>().join(" ")
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
    let child = cmd
        .spawn()
        .map_err(|e| anyhow::anyhow!("spawn `claude` (is it on $PATH?): {e}"))?;
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
