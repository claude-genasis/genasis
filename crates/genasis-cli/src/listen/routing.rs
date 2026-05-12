//! PM agent multi-agent fan-out — secusy `/work/secusy/.claude/commands/sprint-start.md`
//! + strategy.md §9 (Mattermost 소통 프로토콜) + §26 (Ownership-based
//! atomic transaction) 의 trial flavor 등가물.
//!
//! 흐름:
//!   1. 사람 메시지 도착 → `pm` 페르소나로 `claude --print` 호출
//!   2. PM 응답에서 다음 5 가지 마커 파싱:
//!      - `[APP: <kind>]` — sim_teams.app_kind 갱신
//!      - `[FEATURES: a, b, c]` — sim_teams.app_features 누적 추가
//!      - `## 작업 분배` 블록 — `- @<role>: <task>` 라인들
//!      - `## 새 카드` 블록 — `- "<title>" [@<assignee>] [state=<s>]`
//!      - `[CARD: <title> → <state>]` — 기존 카드 transition
//!   3. PM 응답 본문을 사람 메시지 스레드 (`root_id = human post id`) 에 reply
//!   4. `## 작업 분배` 의 각 멘션된 role 에 대해 follow-up `claude --print`
//!      (그 role 페르소나) — 각 응답도 같은 thread 에 reply
//!   5. `## 새 카드` 의 카드들을 sim_issues 에 INSERT (state + assignee)
//!   6. `[CARD: ...]` 마커로 기존 카드 state 동기화

use anyhow::Result;
use serde::Serialize;

/// PM 응답에서 추출한 routing 명세.
#[derive(Debug, Default, Clone, Serialize)]
pub struct PmRouting {
    pub app_kind: Option<String>,
    pub app_features: Vec<String>,
    pub assignments: Vec<AgentAssignment>,
    pub new_cards: Vec<NewCard>,
    pub transitions: Vec<CardTransition>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentAssignment {
    pub role: String,
    pub task: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NewCard {
    pub title: String,
    pub assignee: Option<String>,
    pub state: String, // "todo" | "inprogress" | "inreview" | "done"
}

#[derive(Debug, Clone, Serialize)]
pub struct CardTransition {
    pub title_substring: String,
    pub to_state: String,
}

/// PM agent 가 사용할 system prompt + 응답 형식 명세. 사람 메시지 +
/// PRD 컨텍스트를 받아서 multi-agent 분배 결정을 강제한다.
///
/// ADR-018 §"기존 앱 수정 시나리오": 트라이얼의 쇼케이스 앱은 이미
/// example PRD 결과물 (Claude Code 전문가 진단 퀴즈) 으로 정해져 있다.
/// 사람의 채팅 요청은 그 앱에 대한 **수정 / 커스터마이즈 요구** 로
/// 해석한다 (전체 앱 교체 X). 따라서 `[APP: ...]` 는 일반적으로 quiz
/// 유지 + `[FEATURES: ...]` 에 색상·버튼·테마 등 시각 변경 누적.
pub fn build_pm_prompt(
    project_name: &str,
    project_slug: &str,
    human_message: &str,
    prd_summary: Option<&str>,
) -> String {
    let prd_block = prd_summary
        .map(|s| format!("\n\n프로젝트 PRD 요약:\n{}\n", s))
        .unwrap_or_default();
    format!(
        r#"당신은 Genasis agentic 팀의 PM 입니다. 프로젝트는 "{project_name}" (slug: {project_slug}).

본 프로젝트의 쇼케이스 앱은 이미 `genasis example prd` 가 만든 **Claude Code
전문가 진단 퀴즈** (app_kind=`quiz`) 로 배포되어 있습니다. 사람이 채팅에 보낸
새 요청은 그 기존 앱에 대한 **수정/커스터마이즈 요구** 로 받아들이세요. 새 앱을
처음부터 만들지 마세요.{prd}

사람이 채팅 채널 #scrum-{project_slug} 에 다음 요구를 게시했습니다.

```
{human_message}
```

당신의 임무 (반드시 아래 형식 정확히 준수, 마크다운 사용):

📥 요구사항 정리: <2-3 줄로 사람 요구의 핵심 정리>

[APP: quiz]
[FEATURES: <feature1>, <feature2>, ...]

`[APP: ...]` 는 quiz 그대로 둡니다 (사용자가 명시적으로 다른 앱 종류로 교체
요구하지 않는 한). `[FEATURES: ...]` 는 사람 요구를 다음 flag 로 매핑:

  - `accent-red` — 강조 버튼/색을 빨간색
  - `accent-blue` — 강조색을 파란색
  - `accent-green` — 강조색을 초록색
  - `share-button` — 결과 화면에 공유 버튼 추가
  - `dark-mode` — 다크 테마
  - `i18n` — 영어/한국어 전환
  - `larger-text` — 글자 크기 증가

매핑 안 되는 요구는 가장 비슷한 flag 로 근사. 여러 개 동시 활성 OK (set
union 누적).

## 작업 분배
- @<role>: <한 줄 작업 지시>
- @<role>: <한 줄 작업 지시>

분배 가능한 role: pm, planner, architect, frontend, backend, qa, designer,
security, devops, code-reviewer. 사람 요구에 실제 필요한 role 만 (보통 2-3
개 — 시각 변경이면 designer + frontend + qa). 각 줄은 `- @role: 작업` 형식
정확히 지킬 것.

## 새 카드
- "<카드 제목>" [@<assignee>] [state=todo]
- "<카드 제목>" [@<assignee>] [state=todo]

각 작업 분배에 대응하는 새 칸반 카드. assignee 는 위 작업 분배의 role 과 일치.
state 는 기본 `todo` (해당 agent 가 착수 시 자기 atomic transaction 으로 `inprogress`
로 옮김).

응답 마지막에는 한 줄로 사용자에게 진행 안내:
> @human 작업 분배 완료. 각 agent 의 응답을 같은 스레드에서 확인하세요.
"#,
        project_name = project_name,
        project_slug = project_slug,
        human_message = human_message,
        prd = prd_block,
    )
}

/// role agent (frontend / qa / designer 등) 의 follow-up prompt — PM
/// 의 작업 지시 받고 자기 카드 만들고 In Progress 로 옮기는 atomic
/// transaction 의 reply 본문 작성.
pub fn build_agent_prompt(
    role: &str,
    task: &str,
    project_name: &str,
    project_slug: &str,
    card_title: Option<&str>,
) -> String {
    let card_ref = card_title.map(|t| format!("(카드: \"{}\")", t)).unwrap_or_default();
    format!(
        r#"당신은 Genasis 팀의 {role} 역할 agent 입니다. 프로젝트 "{project_name}".
PM 이 다음 작업을 위임했습니다 {card_ref}:

```
{task}
```

작업에 착수합니다. 다음 형식으로 응답하세요 (각 줄 정확히 준수):

✋ @human 카드 #N {role} 착수 — <한 줄 진행 계획>

[CARD: <위 카드 제목> → inprogress]

이후 작업을 시뮬레이션한다고 가정하고 (실제 코드 작성은 본 데모 범위 밖),
즉시 완료 보고 한 줄 추가:

✅ @human 카드 #N {role} 완료 — <한 줄 결과 요약>

[CARD: <위 카드 제목> → done]

만약 작업 도중 다른 role 또는 사람의 결정이 필요한 이슈가 있다면 done 대신:

⏳ @human 카드 #N {role} 보류 — <문제 요약> · 결정 필요

[CARD: <위 카드 제목> → inreview]

3-5 문장 이내로 한국어로 응답. 마크다운 헤더 사용 금지.
"#,
        role = role,
        task = task,
        project_name = project_name,
        card_ref = card_ref,
    )
}

/// PM 응답을 파싱해서 routing 추출. 정규식 5종.
pub fn parse_pm_routing(pm_response: &str) -> PmRouting {
    let mut out = PmRouting::default();

    // [APP: <kind>] - 첫 번째 매칭만 사용
    if let Some(cap) = regex_capture(pm_response, r"\[APP:\s*([a-z-]+)\s*\]") {
        out.app_kind = Some(cap.to_string());
    }

    // [FEATURES: a, b, c]
    if let Some(cap) = regex_capture(pm_response, r"\[FEATURES:\s*([^\]]+)\]") {
        out.app_features = cap
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }

    // ## 작업 분배 블록 안의 `- @<role>: <task>` 라인
    if let Some(block) = extract_section(pm_response, "## 작업 분배") {
        for line in block.lines() {
            let line = line.trim();
            if !line.starts_with("- @") {
                continue;
            }
            // 형식: `- @role: task` (role 은 영문, : 다음 task)
            if let Some(rest) = line.strip_prefix("- @") {
                if let Some((role, task)) = rest.split_once(':') {
                    let role = role.trim().trim_end_matches('.').to_string();
                    let task = task.trim().to_string();
                    if !role.is_empty() && !task.is_empty() {
                        out.assignments.push(AgentAssignment { role, task });
                    }
                }
            }
        }
    }

    // ## 새 카드 블록 안의 `- "<title>" [@<assignee>] [state=<s>]` 라인
    if let Some(block) = extract_section(pm_response, "## 새 카드") {
        for line in block.lines() {
            let line = line.trim();
            if !line.starts_with("- \"") {
                continue;
            }
            // 1) title (between first pair of double-quotes)
            let after_open = match line.strip_prefix("- \"") {
                Some(s) => s,
                None => continue,
            };
            let (title, rest) = match after_open.split_once('"') {
                Some(p) => p,
                None => continue,
            };
            let title = title.trim().to_string();
            if title.is_empty() {
                continue;
            }
            // 2) assignee — `[@<role>]` 패턴
            let assignee = regex_capture(rest, r"\[@([a-z-]+)\]").map(String::from);
            // 3) state — `[state=<value>]`
            let state = regex_capture(rest, r"\[state=([a-z]+)\]")
                .unwrap_or("todo")
                .to_string();
            out.new_cards.push(NewCard {
                title,
                assignee,
                state,
            });
        }
    }

    // [CARD: <title> → <state>]  (또는 -> ASCII)
    for cap in regex_all_captures(pm_response, r"\[CARD:\s*([^→\->\]]+?)\s*(?:→|->)\s*([a-z]+)\s*\]")
    {
        let title = cap.0.trim_matches('"').trim().to_string();
        let to_state = cap.1.trim().to_string();
        if !title.is_empty() {
            out.transitions.push(CardTransition {
                title_substring: title,
                to_state,
            });
        }
    }

    out
}

/// 간단 정규식 wrapper — 첫 번째 capture group 만 반환. 정규식 컴파일
/// 실패시 None (panic 안 함).
fn regex_capture<'a>(haystack: &'a str, pattern: &str) -> Option<&'a str> {
    let re = match regex::Regex::new(pattern) {
        Ok(r) => r,
        Err(_) => return None,
    };
    let cap = re.captures(haystack)?;
    cap.get(1).map(|m| m.as_str())
}

/// 모든 매칭의 (cap1, cap2) 반환.
fn regex_all_captures(haystack: &str, pattern: &str) -> Vec<(String, String)> {
    let re = match regex::Regex::new(pattern) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    re.captures_iter(haystack)
        .filter_map(|c| {
            let g1 = c.get(1)?.as_str().to_string();
            let g2 = c.get(2)?.as_str().to_string();
            Some((g1, g2))
        })
        .collect()
}

/// `## <title>` 헤더와 다음 `## ` 헤더 (또는 EOF) 사이의 본문 추출.
fn extract_section<'a>(haystack: &'a str, header: &str) -> Option<&'a str> {
    let start = haystack.find(header)?;
    let after_header = &haystack[start + header.len()..];
    let body_start = after_header.find('\n')? + 1;
    let body = &after_header[body_start..];
    let end = body.find("\n## ").unwrap_or(body.len());
    Some(&body[..end])
}

pub fn render_routing_summary(r: &PmRouting) -> String {
    let mut lines = Vec::new();
    if let Some(k) = &r.app_kind {
        lines.push(format!("app_kind={k}"));
    }
    if !r.app_features.is_empty() {
        lines.push(format!("features={:?}", r.app_features));
    }
    lines.push(format!("assignments={}", r.assignments.len()));
    lines.push(format!("new_cards={}", r.new_cards.len()));
    lines.push(format!("transitions={}", r.transitions.len()));
    lines.join(" | ")
}

/// Result 호환 — 외부에서 ?-propagation 가능.
pub fn parse_or_default(pm_response: &str) -> Result<PmRouting> {
    Ok(parse_pm_routing(pm_response))
}
