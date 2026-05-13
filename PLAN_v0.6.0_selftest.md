# v0.6.0 자가 테스트 결과 + 문제점 + 개선 계획

라이브 검증 시점: 2026-05-14 KST 01:33~01:45 (v0.6.0-alpha.5 라이브)
검증 메시지: 사람 msg id=275 "시작 버튼을 진한 보라색으로 바꿔줘"
세션: `86d20006-ebb7-42f2-83de-98fa285d92fc`, 1 turn, 135 s

## 발견된 결함

### D-048 (Medium) — `tool_use` event 가 채널로 forward 안 됨

**원인**: `session.rs::parse_tool_use` 가 top-level `type: "tool_use"` 만 매칭. 그러나 claude stream-json 의 tool 호출은 `assistant` message 의 `content[]` 안의 `{type: "tool_use", ...}` block.

**영향**: monitor 가 어떤 MCP tool 이 호출됐는지 직접 추적 못 함 — debug 시 sim DB 변경 결과로 역추론. 안정성과는 무관.

**개선**: `parse_assistant` 가 text block 만 추출하지 말고 tool_use block 도 별도 SessionEvent 로 emit. 데몬 로그가 "tool: post_message {args}" 같은 구조화 정보 제공.

### D-049 (Low) — assistant text 에 raw markdown blocks 들어가서 채팅 가독성 떨어질 수 있음

**원인**: agent prompt 가 코드 변경 후 자연어 설명을 길게 출력 가능 — 채팅에 markdown blocks (`​```typescript ... ​````) 가 노출되면 LiveChatThread 가 plain text 로 렌더해 가독성 떨어짐.

**영향**: 사용자가 채팅에서 응답 읽기 어려움.

**개선**: LiveChatThread 가 마크다운 렌더링 지원 (react-markdown 같은 lib), 또는 overlay prompt 가 "채팅 message 본문은 plain 한국어 / 코드 블록 사용 금지" 강제.

### D-050 (Medium) — 1 turn 처리에 ~135 초 — 사람 대기 길어 진행감 끊김

**원인**: session 의 PM → 4 sub-agent Task 호출 → 각자 진짜 코드 변경 → 결과 종합. 진짜 작업이라 시간 든다.

**영향**: 사람이 "메시지 보냈는데 응답 없다" 라 느낌. v0.5.x 의 staged sleep (D-028) 은 시뮬레이션 효과였는데 session 에선 사라짐.

**개선**:
- PM 의 첫 응답은 "🟢 접수: ..." 만 즉시 (`post_message` 1초 안). 그 후 백그라운드로 sub-agent 호출.
- sub-agent 들이 자기 작업 시작/종료 시 별도 `post_message` 로 진행 노출.
- 데몬이 SessionEvent::ToolUse 받을 때마다 "🔧 frontend 가 src/components/StartScreen.tsx 수정 중..." 같은 status 메시지 사용자에게 전달 (별도 actor 또는 ping).

### D-051 (Medium) — 데몬 stop 시 session subprocess kill 보장 안 됨

**원인**: `ClaudeTeamSession` struct 의 `child: Child` 는 데몬 종료 시 drop 되며 SIGTERM 보내지만, Rust tokio Drop 이 async 가 아니라 fire-and-forget.

**영향**: 데몬 stop 후 zombie `claude --print` subprocess 남을 가능성. MCP server (`node ... trial-app/index.mjs`) 도 같이 leak 가능.

**개선**:
- `Drop` impl 대신 explicit `shutdown(&mut self)` 호출 — `child.kill().await` + `child.wait().await`.
- cmd_listen 의 signal handler (`SIGTERM`) 에서 session.shutdown() 호출.
- 또는 setsid + process group kill 로 child tree 한 번에.

### D-052 (Low) — MCP server 의 NODE_PATH hard-code

**원인**: `session.rs::build_mcp_config` 의 `NODE_PATH` default = `/home/bravo/.npm-global/lib/node_modules` — 본 host 환경에만 맞음.

**영향**: 다른 사용자 (NODE_PATH 가 다른 환경) 에서는 MCP server 의 `require('@modelcontextprotocol/sdk')` MODULE_NOT_FOUND. `GENASIS_NODE_PATH` env 로 override 가능하지만 자동 탐지 안 함.

**개선**:
- runtime 에 `npm root -g` 호출해 동적으로 결정.
- 또는 MCP server 를 ESM 으로 만들어 `node` 가 자동 resolve 하도록 (npm install 한 위치의 node_modules 가 default 검색).
- 또는 install.sh 에서 `genasis-mcp-trial-app` 같은 npm global package 로 설치 + bin path 사용.

### D-053 (High) — agents-pool 의 `verified/` 가 비어 있어 정식 publish.sh 불가

**원인**: agents-pool 의 catalog publish 흐름은 crawler → verified/ → publish.sh. verified/ 가 v1.0.0 release 후 cleanup 됐는지 안 채워져 있음. 우리가 v1.0.1 publish 한 방식은 release-assets/v1.0.0.tar.gz 풀어 base 재사용 + overlays 만 갱신 + 새 tar 만든 우회.

**영향**: 정식 publish 흐름 (crawl → verify → publish) 미작동. base agent 업데이트 시 어떤 source 도 못 가져옴.

**개선**:
- `scripts/crawl.sh` + `scripts/verify.sh` 가 실제로 sources/ 의 community repo 들에서 agent .md 가져와 verified/ 채우도록 복구.
- 또는 release-assets/ 의 마지막 tarball 의 base/ 를 verified/ 의 backup 으로 사용 (incremental publish).
- 또는 publish.sh 에 `--use-prev-base` flag 추가 — `verified/` 비어도 이전 release 의 base 그대로 + overlays/ 만 갱신.

### D-054 (Medium) — v0.5.x simulation code 미제거

**원인**: alpha.6 의 "simulation cleanup" 작업이 시간 한계로 일부만 진행. cmd_listen 의 echo-only path 만 제거, mod.rs / routing.rs / sdk.rs 의 dead code 는 그대로.

**영향**: 컴파일러 warning 50+, 코드베이스 노이즈. 다음 contributor 가 어느 path 가 활성인지 헷갈림.

**개선** (alpha.6 후속):
- `mod.rs::handle_human_post / run_agent_step / build_echo_* / run_claude_print / run_listen_loop (old)` 모두 제거.
- `EventSink trait` 의 `apply_pm_routing / cleanup_stuck_cards / maybe_transition_for_directive / reply` 제거 — session path 가 안 씀.
- `trial_sse.rs / mattermost_ws.rs` 의 sink struct 통째 제거.
- `routing.rs` 통째 제거 (PmRouting struct / parse_pm_routing / build_pm_prompt / build_agent_prompt 다 unused).
- `sdk.rs::run_claude_agent_sdk` 제거 (session 으로 통합됨).
- `LoopConfig` 단순화 — project_root + max_events 만 유지.
- `cmd_listen Args` 단순화 — --trial / --project / --max-events 만.

### D-055 (Low) — Trial-app MCP server 의 channel_id 캐시 invalidate 안 됨

**원인**: `mcp-servers/trial-app/index.mjs` 의 `resolveChannelId()` 가 첫 호출 시 sim_posts 의 첫 post 의 channel_id 캐시. 만약 사용자가 다른 채널 (예: design channel) 도 쓰고 싶으면 첫 캐시된 scrum channel 만 사용.

**영향**: trial-app 의 sim_channels 가 multi-channel 지원해도 MCP server 는 1 channel 만.

**개선**:
- `post_message` 의 input schema 에 `channel` (선택) param 추가 — `scrum` (default) / `design` / ...
- channel name → id 매핑을 runtime lookup (캐시 + invalidate).

### D-056 (Low) — devops agent 의 dev server URL 자동 announce 가 overlay 본문에만 있고 enforce 안 됨

**원인**: overlay prompt 가 "devops 가 npm run dev 후 `post_message` 로 URL 안내" 라고 명시. 하지만 agent 가 안 따라도 데몬은 모름.

**영향**: 사용자가 URL 모르고 ShowcasePanel 에 직접 입력해야 — UX 마찰.

**개선**:
- devops 의 MCP tool 에 `announce_dev_server_url(url, port)` 추가 — 호출하면 trial-app 의 sim_teams 에 저장 + ShowcasePanel 의 LocalDevServerOrFallback 가 자동 prefill.
- 또는 데몬이 SessionEvent::ToolUse 를 inspect 해서 `Bash` tool 의 cmd 안에 `npm run dev` 보이면 자동으로 listen port scan + announce.

## 우선순위 + 다음 사이클 권장

| 우선 | 결함 | 사이클 |
|---|---|---|
| 1 | D-051 (process leak) | alpha.6 |
| 1 | D-053 (publish workflow) | alpha.6 |
| 2 | D-054 (simulation cleanup) | alpha.6 |
| 3 | D-052 (NODE_PATH 자동 탐지) | alpha.7 |
| 3 | D-050 (진행감 끊김 — 중간 status 안내) | alpha.7 |
| 4 | D-048 (tool_use event forward) | alpha.7 |
| 5 | D-056 (devops URL announce) | alpha.7 |
| 5 | D-049 (markdown 렌더) | alpha.7 |
| 6 | D-055 (multi-channel MCP) | beta |

## 다음 본 사이클 (alpha.6) — 결정적 단계

1. **simulation code 일괄 제거** (D-054) — 코드베이스 정화
2. **process kill 안전망** (D-051) — listen stop 시 session + MCP server 모두 종료
3. **agents-pool publish workflow 복구** (D-053) — verified/ 또는 우회 publish 자동화
4. **자가 테스트 흐름 표준화** — 이 PLAN 의 결함 list 가 매 사이클 갱신되도록 routine

다음 사이클은 cleanup + 안정성 위주. M-v6.0.4 (multi-team sandbox) 와 beta (real MCP) 는 그 위에 자연스럽게.
