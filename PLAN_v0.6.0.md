# PLAN — v0.6.0 본질 회복: per-team sandbox + agent code access

> 한국어: 이 PLAN 은 사용자 §"trial-app 이 시뮬레이션이 아닌 실제 동작" 요구의
> 본질을 v0.5.x 사이클 내내 시뮬레이션 외형 다듬기로 회피해 온 것을 인정하고,
> v0.6.0 부터 진짜 agent code-access + per-team sandbox 모델로 재구축한다.

## 사용자 의도 (정확히 재정리)

genasis 의 본질:
- **Claude 의 agentic team 기능을 이용해 multi-agent team 을 생성·관리하는 도구**
- 사람은 **칸반 + 채팅** 인터페이스로 그 팀과 소통
- 두 가지 사용자 층:
  - **인프라 가능자**: real Plane + Mattermost 연동
  - **체험 사용자**: trial-app 이 minimal Plane/Mattermost 대용 (per-team)

trial-app 의 진짜 역할 (v0.5.x 에서 회피한 것):
- **per-team 격리** — 각 team 이 자기 작업 공간
- **사용자 채팅 명령이 실제 agentic team 에 전달**
- **팀이 진짜 코드 작업 수행** (text 응답이 아닌 file Edit + build + deploy)
- **사용자가 결과 시각 확인** (브라우저 dev server)

**핵심 원칙**: `genasis` 자체는 **인프라/오케스트레이션만 담당**. 코드 생성·변경·빌드·배포는 **agent 의 일** 이지 genasis CLI 의 일이 아니다. 예: `genasis example prd` 는 **PRD.md 만 생성**, React scaffold 까지 만들지 않음 — scaffold 는 frontend agent 가 PRD 보고 처음부터 짜는 게 진짜 흐름.

## v0.5.x 회피 회고

| 결함 ID | 회피의 본질 |
|---|---|
| D-029b (색상 사전 확장) | 진짜 코드 변경 회피 → feature flag string 사전 hard-code |
| D-039 (타이틀 accent 적용) | 진짜 코드 변경 회피 → QuizApp Tailwind 분기 hard-code |
| D-040 (deploy announce) | 진짜 빌드/배포 회피 → 채팅 announce 문구 hard-code |
| D-041 (cleanup_stuck) | 진짜 카드 상태 추적 회피 → "정리해줘" 키워드 보면 일괄 done |
| 모든 v0.5.x | **agent 가 코드 못 봄, 못 수정** — `claude --print` stateless |

frontend agent 가 채팅에 "Button 컴포넌트에 shimmer-border 클래스 적용해 전 버튼에 자동 반영, 스토리북 스냅샷 통과" 라고 한 응답은 **모두 가짜** — 실제 코드 변경 없었음.

## v0.6.0 아키텍처

```
사용자 (브라우저)
  │
  ├─ 채팅 패널 (호스팅 trial-app)
  │   └─ 호스팅 trial-app = 칸반/채팅 UI 만 (시각 결과 X)
  │
  └─ 시각 결과 (사용자 자기 localhost:<port>)
      └─ 사용자 로컬 dev server (per-team sandbox)

사용자 채팅 입력
  │
  ▼ SSE
사용자 로컬 데몬 (genasis listen, cwd=<sandbox>)
  │
  ▼
PM agent 호출 (Claude Agent SDK Node subprocess)
  - cwd: <sandbox>
  - allowed_tools: Read, Bash (코드 컨텍스트 파악)
  - permission_mode: acceptEdits
  │
  ▼ PM 응답 + routing 마커
각 role agent 병렬 호출 (Agent SDK)
  - frontend: cwd=<sandbox> + Read/Edit/Bash (실제 React 코드 수정)
  - designer: Tailwind 토큰 / globals.css 수정
  - qa: Playwright 테스트 작성 + 실행
  - devops: npm build + dev server restart
  │
  ▼ agent 가 진짜 파일 Edit
  ▼ devops 가 hot-reload 또는 build
  ▼
사용자 브라우저 자동 새로고침 → 변경 시각 보임
```

## 마일스톤

### M-v6.0.1 — Agent SDK 통합 (첫 진전)

- 데몬에 `run_claude_agent_sdk(prompt, cwd, tools) -> Result<String>` 함수 추가
- Node subprocess + `NODE_PATH=/home/bravo/.npm-global/lib/node_modules` + `@anthropic-ai/claude-agent-sdk`
- PM 호출 먼저 SDK 모드로 전환 (PRD.md 읽어 컨텍스트화)
- agent fan-out 은 일단 그대로 (Phase 2 에서 전환)
- **검증**: PM 응답이 PRD 내용 인지하고 정확한 작업 분배

### v0.6.0-alpha.3 ~ beta — Long-running session + MCP 전환 (사용자 §"방식 B 본질")

**문제 인식 (alpha.2 사후 회고)**:
- 현재 sdk.rs 는 매 호출마다 fresh Node + claude subprocess 띄움 → **disposable workers**, "team" 정신 없음, session warm-up 반복
- routing.rs 의 marker 파싱 (D-029~D-042) 은 LLM 자유 응답을 강제로 정형화하는 brittle 패턴

**전환 방향 (사용자 결정)**: alpha.4 + beta 통합 진행 — 처음부터 MCP 모델.

#### Phase 1 (alpha.3) — ClaudeTeamSession (Rust 데몬)

- `crates/genasis-cli/src/listen/session.rs` 신규
- team_token 별 1 long-running `claude -p --input-format stream-json --output-format stream-json --mcp-config <path>` subprocess
- stdin tx: NDJSON `{"type":"user","message":...}` push
- stdout rx: NDJSON event stream 파싱 → assistant/tool_use/result event
- crash recovery (subprocess 죽으면 자동 재기동, 마지막 사람 메시지부터 resume)
- session lazy spawn (첫 사람 메시지 도착 시)

#### Phase 2 (alpha.3) — trial-app MCP server

- `mcp-servers/trial-app/` 신규 (Node, `@modelcontextprotocol/sdk`)
- tools (trial-app REST API thin wrapper):
  - `post_message(channel_name, actor, message, root_id?)` — sim_posts 게시
  - `list_posts(channel_name)` — 채팅 history
  - `create_issue(project_slug, title, assignee, state?)` — sim_issues INSERT
  - `transition_issue(id, state)` — 카드 state 변경
  - `list_issues(project_slug)` — 칸반 현재 상태
  - `set_app_features(features)` — sim_teams.app_features (LRU 방식 유지)
  - `set_app_kind(kind)` — sim_teams.app_kind
- 데몬이 session spawn 시 mcp-config 로 등록 → agent 가 자연스러운 tool call

#### Phase 3 (alpha.3) — agent.md overlay 변경

- `.claude/agents/pm.md` overlay: `tools: [Read, Bash, Task]` + `mcpServers: [trial-app]` (frontmatter)
- 각 role agent: 자기 role 에 맞는 tools + mcpServers
- overlay protocol 본문: marker 출력 → MCP tool call 안내
  - 예: "카드 이동은 `mcp_trial-app.transition_issue(id, state)` 호출"
  - 예: "채팅 게시는 `mcp_trial-app.post_message(...)` 호출"
- PM 의 Task tool sub-agent 호출 패턴 명시 (`.claude/agents/<role>.md` 가 정의됐다고 가정)

#### Phase 4 (alpha.4) — marker 파싱 폐기

- `routing.rs::parse_pm_routing` deprecate → 제거
- `mod.rs::handle_human_post` 폐기 → `session.send_user_message(text)` 한 줄
- `EventSink::apply_pm_routing` / `cleanup_stuck_cards` 폐기 (MCP tool 이 직접)
- D-029 (parser robust) / D-035 (title literal) / D-036 (echo title) / D-037 (sequence_id) / D-038 (regex hyphen) / D-041 (cleanup heuristic) 모두 **자연 해소** — MCP tool call 은 구조화 데이터라 parsing 없음

#### Phase 5 (beta) — real Mattermost / Plane MCP

- `mcp-servers/mattermost/` — real Mattermost API wrapper (admin token)
- `mcp-servers/plane/` — real Plane REST wrapper (PLANE_API_KEY)
- agent.md frontmatter 의 `mcpServers:` 만 trial → mattermost/plane 으로 swap
- agent 입장에선 같은 `post_message` / `transition_issue` 인터페이스 — flavor 차이는 데몬의 MCP server 선택만

#### 폐기 / 마이그레이션 표

| 폐기 대상 (v0.5.x) | 대체 |
|---|---|
| `sdk.rs::run_claude_agent_sdk` (매번 spawn) | `session.rs::ClaudeTeamSession` (long-running) |
| `routing.rs::parse_pm_routing` + marker | MCP tool call structured data |
| `EventSink::apply_pm_routing` | MCP server 가 trial-app API 직접 |
| `EventSink::cleanup_stuck_cards` | agent 가 `transition_issue` 일괄 호출 |
| `build_echo_*` stub | unused (session 안에서 agent 가 진짜 작업) |
| D-029a/b, D-035, D-036, D-037, D-038, D-041 fix 들 | 자연 해소 (parsing 자체 없음) |

### M-v6.0.2 — 모든 role agent SDK + Edit/Write/Bash tool 부여

- `genasis example prd` 는 **PRD.md 만 생성** (요구사항 텍스트). 코드 scaffold 생성 안 함 — 그건 agent 의 일.
- frontend / backend / designer / qa / devops 호출이 모두 Agent SDK 로
- 각 role 마다 적합한 tool 권한:
  - frontend / backend: Read, **Edit, Write**, Bash
  - designer: Read, **Edit, Write** (Tailwind config, globals.css, design tokens)
  - qa: Read, Write (테스트 파일), **Bash** (Playwright/jest 실행)
  - devops: Read, **Bash** (npm install/build/dev, port 관리)
  - pm / planner / architect: Read, Bash (코드 인지 + git log 같은 진단)
- agent prompt 변경: "텍스트 응답이 아닌 **진짜 file Write/Edit** 수행. 코드는 cwd 에 만들고, 진행은 [CARD: ... → state] 마커로 보고"

### M-v6.0.3 — agent 가 PRD 보고 처음부터 앱 구현

이게 본 사이클의 진짜 본질 검증.

- 빈 프로젝트 + PRD.md 만 있는 sandbox 에서 사용자가 "이 PRD 대로 앱 만들어줘" 요청
- PM 이 PRD 읽고 작업 분배: frontend (React scaffold + 컴포넌트), designer (디자인 토큰), qa (테스트), devops (dev server)
- **frontend agent 가 `npm init` 또는 vite/next create + React 컴포넌트 진짜 Write**
- designer 가 Tailwind config + 디자인 토큰 진짜 Write
- qa 가 Playwright 테스트 진짜 작성
- **devops agent 가 `npm install` + `npm run dev` Bash 호출 → 자기 dev server spawn (예: localhost:30000+team_index)**
- 사용자가 자기 브라우저로 그 URL 접속 → **agentic team 이 처음부터 만든 진짜 앱** 봄
- 후속 요청 ("버튼 빨강으로", "shimmer 효과") 도 같은 agent 들이 진짜 코드 Edit + hot-reload

### M-v6.0.4 — per-team 격리

- 각 team_token 별 sandbox 디렉토리 자동 생성 + dev server port 할당
- 다중 team 동시 작업 가능 (서로 격리)
- 사용자가 `genasis init --trial` → sandbox 만들어짐 → 자기 localhost:<port> 접속

### M-v6.0.5 — 호스팅 trial-app 역할 재정의

- 호스팅 (`mmplane-trial.realstory.blog`) = 칸반/채팅 UI + 시뮬레이션 demo
- "에이전트가 만든 앱 보기" 모달:
  - 옵션 A: 사용자 로컬 dev server URL 사용자 입력 + iframe (CORS 처리)
  - 옵션 B: 호스팅 측은 demo placeholder + "실제 동작은 자기 컴퓨터 localhost:<port> 참고"
- PM prompt 가 "시뮬레이션 demo 한정" 명시 (호스팅 인스턴스에서)
- 사용자 로컬에서는 "real code edit" 모드 (sandbox cwd 가 agent 에게 노출됨)

## v0.6.0-alpha.1 (이 사이클 시작점)

**범위**: M-v6.0.1 만. PM 호출의 Agent SDK 전환 + cwd 의 PRD.md 인지.

**작업 단위**:
1. 신규 함수 `crates/genasis-cli/src/listen/sdk.rs::run_claude_agent_sdk(prompt, cwd, tools)`
   - Node subprocess `NODE_PATH=... node -e "<inline script>"`
   - stream-json 출력 파싱 → 최종 assistant message 추출
   - tool 권한 + cwd + permission_mode
2. `mod.rs::handle_human_post` 의 PM 호출 분기:
   - `claude --print` 대신 `run_claude_agent_sdk` 호출
   - cwd 는 `cfg.project_root` (데몬 시작 시 결정된 sandbox dir)
   - tools=Read, Bash (Edit 는 frontend 단계에서)
3. `LoopConfig` 에 `project_root: PathBuf` 추가, cmd_listen 에서 채움
4. echo_only 모드는 그대로 유지 (CI 검증용)
5. fallback: SDK 호출 실패 시 echo PM response

**검증**:
- 사용자 sandbox `/work/agenteams/team-ex/v516-final/` 에 PRD.md 가 있음
- 데몬 진짜 LLM 모드로 띄움
- 채팅 메시지 보내면 PM 이 PRD.md 의 컨텍스트 (Claude Code 진단 퀴즈) 인지하고 응답
- 데몬 로그에 SDK 호출 + cwd + tool 사용 흔적 노출

## 시뮬레이션 시대 (v0.5.x) 의 종료 선언

본 PLAN 채택 후 다음을 폐기 또는 점진 폐기:
- `build_echo_pm_response` 의 키워드 사전 (D-029b 등) — 진짜 LLM 이 처리
- QuizApp 의 hard-coded `accentClass` / `titleAccentClass` (D-039) — 진짜 코드 변경
- 데몬의 deploy announce (D-040) — devops agent 가 진짜 build 후 자동 announce
- `cleanup_stuck_cards` 휴리스틱 (D-041) — agent 가 자기 작업 마무리 시 진짜 transition

지금 즉시 제거하지는 않음 — 점진 폐기 (alpha 사이클 진행하면서 의존 풀린 것부터).
