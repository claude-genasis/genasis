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
pub mod sdk;
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
    ///
    /// D-037: 반환값은 (title → sequence_id) 매핑. trial flavor 는 bootstrap
    /// 응답의 `demo_issues[]` 에서 채움. real Plane flavor 는 issue create
    /// API 응답에서 받음 (현재 stub 으로 빈 map). 데몬의 fan-out 이 이 맵을
    /// 보고 agent prompt 의 `#N` placeholder 를 진짜 카드 번호로 대체.
    async fn apply_pm_routing(
        &self,
        routing: &routing::PmRouting,
    ) -> Result<std::collections::HashMap<String, u64>>;

    /// D-041: 사용자 메시지에 "정리/마무리/완료/cleanup" 키워드가 있을 때
    /// 데몬이 호출. 현재 inprogress/todo 상태로 stuck 된 sim_issues 카드들을
    /// 일괄 done 으로 옮긴다. 반환값은 정리된 카드 개수. trial flavor 는
    /// listIssues GET + 각 카드별 transition 마커 → bootstrap. real Plane
    /// flavor 는 v0.6.0 에서 issue PATCH 일괄 호출 (현재 stub 0).
    async fn cleanup_stuck_cards(&self) -> Result<usize>;
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
    // (0) D-041: 사용자 메시지에 "정리/마무리/완료/cleanup" 키워드가
    // 있으면 PM 응답 받기 전에 sim_issues 의 stuck (inprogress/todo)
    // 카드들을 일괄 done 으로 정리. 누적된 이전 사이클 잔여 카드 청소.
    let m_lower = message.to_lowercase();
    let wants_cleanup = ["정리", "마무리", "완료해", "cleanup", "tidy up", "wrap up"]
        .iter()
        .any(|k| m_lower.contains(k));
    if wants_cleanup {
        match sink.cleanup_stuck_cards().await {
            Ok(n) if n > 0 => {
                let msg = format!(
                    "🧹 잔여 카드 정리 — sim_issues 의 inprogress/todo 카드 {n} 건을 done 으로 일괄 transition. 이전 사이클 stuck 카드 청소 완료."
                );
                if let Err(e) = sink.reply(event, "cleanup", &msg).await {
                    warn!("cleanup announce failed: {e}");
                }
            }
            Ok(_) => {}
            Err(e) => warn!("cleanup_stuck_cards failed: {e}"),
        }
    }

    // (1) PM 응답 생성
    let pm_response = if cfg.echo_only {
        build_echo_pm_response(message, cfg)
    } else {
        let prompt = routing::build_pm_prompt(&cfg.project_name, &cfg.project_slug, message, None);
        // v0.6.0 M-v6.0.1: PM 은 Agent SDK 모드로 호출 — cwd 의 PRD.md 등
        // 사용자 프로젝트 컨텍스트를 Read tool 로 직접 인지. tool 권한은
        // Read + Bash 만 (Edit 은 frontend 단계에서). claude --print stateless
        // 호출은 v0.5.x 시뮬레이션 시대의 한계라 폐기.
        let sdk_result = sdk::run_claude_agent_sdk(
            &prompt,
            &cfg.project_root,
            &["Read", "Bash"],
            cfg.claude_timeout_secs as u64,
        )
        .await;
        match sdk_result {
            Ok(s) if !s.trim().is_empty() => s,
            Ok(_) => {
                warn!("PM Agent SDK returned empty — fallback to claude --print");
                run_claude_print(&prompt, cfg).await.unwrap_or_else(|e| {
                    warn!("PM claude --print fallback failed: {e}");
                    build_echo_pm_response(message, cfg)
                })
            }
            Err(e) => {
                warn!("PM Agent SDK failed: {e} — fallback to claude --print");
                run_claude_print(&prompt, cfg).await.unwrap_or_else(|e| {
                    warn!("PM claude --print fallback failed: {e}");
                    build_echo_pm_response(message, cfg)
                })
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

    // (4) sim DB 적용 (sim_teams app_kind/features + sim_issues 카드).
    // D-037: 반환된 (title → sequence_id) 맵을 fan-out 에 넘겨서 agent
    // 응답의 `#N` placeholder 를 실제 카드 번호로 대체.
    let seq_map = match sink.apply_pm_routing(&route).await {
        Ok(m) => m,
        Err(e) => {
            warn!("apply_pm_routing failed: {e}");
            std::collections::HashMap::new()
        }
    };

    // (5) D-028 + D-032: 각 agent fan-out 을 "착수 → 작업 시간 → 완료"
    // 두 단계로 펼치고, **여러 agent 를 병렬로** 실행한다 (D-032 — 사용자
    // §"병렬처리로 시간 아낄 수 있다면"). 각 agent 내부에서는 inprogress
    // 카드가 사람 눈에 보일 시간 (`agent_work_secs`) 만큼 sleep, 외부
    // 에서는 모든 agent 가 동시 시작 + `agent_gap_secs` 만큼 staggered
    // start 로 사람이 "여러 agent 가 협업 중" 임을 인지하게 한다.
    use tokio::time::{sleep, Duration};
    let agent_work_secs = cfg.agent_work_secs.max(1) as u64;
    let agent_gap_secs = cfg.agent_gap_secs as u64;

    // PM 응답 직후 첫 agent 시작 전 짧은 호흡 — 사람이 PM 메시지를 읽을 시간.
    sleep(Duration::from_millis(800)).await;

    // 각 agent 의 작업 future 를 만들어 staggered start 로 spawn.
    let mut tasks = futures_util::stream::FuturesUnordered::new();
    for (idx, assignment) in route.assignments.iter().enumerate() {
        let stagger = Duration::from_secs(idx as u64 * agent_gap_secs);
        // 이 agent role 에 분배된 카드 찾고 (assignee 매칭),
        // 그 카드의 정확한 PM seed title + sequence_id 를 fan-out 에 주입.
        let assigned_card = route
            .new_cards
            .iter()
            .find(|c| c.assignee.as_deref() == Some(assignment.role.as_str()));
        let card_title = assigned_card.map(|c| c.title.clone());
        let seq_id = card_title.as_ref().and_then(|t| seq_map.get(t).copied());
        let task = run_agent_step(
            event,
            sink,
            cfg,
            assignment,
            &route,
            card_title,
            seq_id,
            agent_work_secs,
            stagger,
        );
        tasks.push(task);
    }
    use futures_util::StreamExt;
    while tasks.next().await.is_some() {
        // 각 agent step 결과는 step 내부에서 reply 및 transition 모두
        // 적용 — 여기서는 join 만.
    }

    // (6) D-030 + D-040: [DEPLOY: ...] 마커별 후처리. 데몬이 직접
    // `[deploy]` actor 명의로 "✅ 배포 완료" announce 자동 게시 —
    // features-only 면 agent fan-out 끝난 직후 PM 스레드에 한 줄.
    // 사용자 §"배포했다고 답한 agent 가 없었다" 결함 해결.
    if let Some(mode) = &route.deploy {
        info!(target: "listen", deploy = %mode, "deploy routing");
        let announce = match mode.as_str() {
            "features-only" => Some(format!(
                "✅ 배포 완료 — sim_teams.app_features = {:?} 가 즉시 쇼케이스에 반영되었습니다. \
                 (모드: features-only / no code build needed)",
                route.app_features,
            )),
            m if m.starts_with("by-last-agent") => Some(format!(
                "✅ 배포 완료 — 마지막 작업 agent ({mode}) 가 코드 변경 + 배포 완료 announce 함."
            )),
            "devops" => Some(
                "✅ 배포 완료 — devops agent 가 빌드 + 릴리스 파이프라인 실행 완료.".to_string(),
            ),
            _ => None,
        };
        if let Some(text) = announce {
            if let Err(e) = sink.reply(event, "deploy", &text).await {
                warn!("deploy announce failed: {e}");
            }
        }
    }

    Ok(())
}

/// D-032: 한 agent 의 "착수 → sleep → 완료" 단계를 묶은 future.
/// 여러 agent 가 병렬로 실행되며, `stagger` 만큼 시작을 어긋나게 해서
/// 사람 눈에 여러 agent 가 순차적으로 일에 들어가는 모습을 보여준다.
///
/// D-036 + D-037: `card_title` 은 PM seed 의 정확한 title (의역 금지),
/// `seq_id` 는 sim_issues 의 sequence_id (Plane 호환). 둘 다 fan-out 시
/// agent 응답 마커에 그대로 보간되어 ensureIssue dedup + 사람이 보는
/// 카드 번호가 일치하도록.
#[allow(clippy::too_many_arguments)]
async fn run_agent_step(
    event: &InboundEvent,
    sink: &dyn EventSink,
    cfg: &LoopConfig,
    assignment: &routing::AgentAssignment,
    pm_route: &routing::PmRouting,
    card_title: Option<String>,
    seq_id: Option<u64>,
    work_secs: u64,
    stagger: tokio::time::Duration,
) {
    use tokio::time::{sleep, Duration};
    if !stagger.is_zero() {
        sleep(stagger).await;
    }

    // (a) "착수" — inprogress transition 만 포함. echo stub 사용 (LLM
    // 호출 절약 — 진짜 응답은 done 단계에 한 번만).
    let start_reply = build_echo_agent_start(assignment, card_title.as_deref(), seq_id, cfg);
    if let Err(e) = sink.reply(event, &assignment.role, &start_reply).await {
        warn!("agent {} start reply failed: {e}", assignment.role);
    }
    let start_route = routing::parse_pm_routing(&start_reply);
    if !start_route.transitions.is_empty() {
        let _ = sink.apply_pm_routing(&start_route).await;
    }

    info!(
        target: "listen",
        role = %assignment.role,
        secs = work_secs,
        "agent working — visible in-progress window"
    );
    sleep(Duration::from_secs(work_secs)).await;

    // (b) "완료" — done transition. echo-only 면 stub, 아니면 v0.6.0
    // M-v6.0.2 의 Agent SDK 모드로 호출 (role-별 tool 권한 + cwd=프로젝트).
    // agent 가 텍스트 응답 + 진짜 file Write/Edit/Bash 둘 다 수행하고
    // 마지막에 [CARD: → done] 마커를 응답에 포함.
    let _ = pm_route; // 미래 확장 — done 시점 routing 컨텍스트
    let done_reply = if cfg.echo_only {
        build_echo_agent_done(assignment, card_title.as_deref(), seq_id, cfg)
    } else {
        let prompt = routing::build_agent_prompt(
            &assignment.role,
            &assignment.task,
            &cfg.project_name,
            &cfg.project_slug,
            card_title.as_deref(),
            seq_id,
        );
        let tools = agent_tools_for(&assignment.role);
        // Agent SDK 호출 timeout 은 코드 작성/빌드 시간을 고려해 PM 보다
        // 너그럽게. claude_timeout_secs 의 2배 (단, 최소 180s).
        let sdk_timeout = std::cmp::max(180, cfg.claude_timeout_secs as u64 * 2);
        let sdk_result =
            sdk::run_claude_agent_sdk(&prompt, &cfg.project_root, tools, sdk_timeout).await;
        match sdk_result {
            Ok(s) if !s.trim().is_empty() => s,
            Ok(_) => {
                warn!(
                    "agent {} Agent SDK empty — fallback claude --print",
                    assignment.role
                );
                run_claude_print(&prompt, cfg).await.unwrap_or_else(|e| {
                    warn!(
                        "agent {} claude --print fallback failed: {e}",
                        assignment.role
                    );
                    build_echo_agent_done(assignment, card_title.as_deref(), seq_id, cfg)
                })
            }
            Err(e) => {
                warn!(
                    "agent {} Agent SDK failed: {e} — fallback claude --print",
                    assignment.role
                );
                run_claude_print(&prompt, cfg).await.unwrap_or_else(|e| {
                    warn!(
                        "agent {} claude --print fallback failed: {e}",
                        assignment.role
                    );
                    build_echo_agent_done(assignment, card_title.as_deref(), seq_id, cfg)
                })
            }
        }
    };
    if let Err(e) = sink.reply(event, &assignment.role, &done_reply).await {
        warn!("agent {} done reply failed: {e}", assignment.role);
    }
    let done_route = routing::parse_pm_routing(&done_reply);
    if !done_route.transitions.is_empty() {
        let _ = sink.apply_pm_routing(&done_route).await;
    }
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
    if m.contains("청록") || m.contains("teal") || m.contains("민트") || m.contains("cyan") {
        features.push("accent-teal");
    }
    if m.contains("노란") || m.contains("노랑") || m.contains("yellow") {
        features.push("accent-yellow");
    }
    if m.contains("주황") || m.contains("orange") || m.contains("오렌지") {
        features.push("accent-orange");
    }
    if m.contains("분홍") || m.contains("pink") || m.contains("핑크") {
        features.push("accent-pink");
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

/// D-028 + D-036 + D-037: agent 의 "착수" 메시지. PM seed 의 정확한
/// 카드 title 과 sequence_id 를 받아서 의역 없이 그대로 마커에 보간.
/// `card_title` 이 None 이면 (이상 경로) assignment.task 의 앞부분 사용.
fn build_echo_agent_start(
    assignment: &routing::AgentAssignment,
    card_title: Option<&str>,
    seq_id: Option<u64>,
    cfg: &LoopConfig,
) -> String {
    let _ = cfg;
    let title = card_title
        .map(str::to_string)
        .unwrap_or_else(|| first_three_words(&assignment.task));
    let card_ref = seq_id
        .map(|n| format!("#{n}"))
        .unwrap_or_else(|| "#(no-id)".to_string());
    format!(
        r#"✋ @human {card_ref} {role} 착수 — {task}

[CARD: {title} → inprogress]"#,
        card_ref = card_ref,
        role = assignment.role,
        task = assignment.task,
        title = title,
    )
}

fn build_echo_agent_done(
    assignment: &routing::AgentAssignment,
    card_title: Option<&str>,
    seq_id: Option<u64>,
    cfg: &LoopConfig,
) -> String {
    let _ = cfg;
    let title = card_title
        .map(str::to_string)
        .unwrap_or_else(|| first_three_words(&assignment.task));
    let card_ref = seq_id
        .map(|n| format!("#{n}"))
        .unwrap_or_else(|| "#(no-id)".to_string());
    format!(
        r#"✅ @human {card_ref} {role} 완료 — 시뮬레이션 모드에서 작업 마침

[CARD: {title} → done]"#,
        card_ref = card_ref,
        role = assignment.role,
        title = title,
    )
}

fn first_three_words(s: &str) -> String {
    s.split_whitespace().take(4).collect::<Vec<_>>().join(" ")
}

/// v0.6.0 M-v6.0.2: role 별 Agent SDK tool 권한 매핑. 각 role 의 책임
/// 영역에 맞춰 좁게 부여 — 사용자 sandbox 안전성 + agent 의도 명확화.
///
/// - frontend / backend: 풀 코드 권한 (`Read`/`Edit`/`Write`/`Bash`)
/// - designer: 디자인 토큰/CSS 작성 — `Bash` 없이 file 작업만
/// - qa: 테스트 작성 + 실행 — `Write` + `Bash`
/// - devops: 빌드/배포/서버 관리 — `Bash` 중심 (코드 직접 수정은 frontend)
/// - pm / planner / architect / code-reviewer: 코드 인지 + 진단만 (`Read`/`Bash`)
/// - security: 코드 인지 + 진단 + 보안 패치
fn agent_tools_for(role: &str) -> &'static [&'static str] {
    match role {
        "frontend" | "backend" => &["Read", "Edit", "Write", "Bash"],
        "designer" => &["Read", "Edit", "Write"],
        "qa" => &["Read", "Write", "Bash"],
        "devops" => &["Read", "Bash"],
        "security" => &["Read", "Edit", "Bash"],
        "pm" | "planner" | "architect" | "code-reviewer" => &["Read", "Bash"],
        _ => &["Read", "Bash"],
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
    /// v0.6.0 M-v6.0.1: agent SDK 의 cwd. 이 디렉토리가 agent 의 Read/
    /// Edit/Bash tool 의 작업 영역이 된다. `genasis init --trial` 디렉토리
    /// 가 기본. 사용자 sandbox = 사용자 자기 프로젝트 디렉토리.
    pub project_root: std::path::PathBuf,
    /// D-028: 각 agent 의 "착수 → 완료" 사이 작업 시간 (초). 칸반 In
    /// Progress 카드를 사람이 인지할 시간을 준다. echo-only 모드에서도
    /// 실제로 일하는 것처럼 보이는 진행감을 만든다.
    pub agent_work_secs: u32,
    /// D-028: 한 agent 작업 종료 후 다음 agent 시작까지 대기 (초).
    /// 순차적 협업 인지를 위해 0보다 큰 값 권장.
    pub agent_gap_secs: u32,
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
        // D-042: 데몬 자체가 게시하는 system actor 들. 자기 메시지를
        // SSE 로 다시 받았을 때 PM 응답 트리거 (무한 루프) 방지.
        "deploy",
        "cleanup",
    ];
    !KNOWN_BOTS.iter().any(|b| a.eq_ignore_ascii_case(b))
}

/// "X 완료", "전체 done", "정리해줘" 류 휴리스틱.
pub fn message_requests_done(message: &str) -> bool {
    let m = message.to_ascii_lowercase();
    const KEYWORDS: &[&str] = &["done", "완료", "끝났", "정리", "마무리"];
    KEYWORDS.iter().any(|k| m.contains(k))
}
