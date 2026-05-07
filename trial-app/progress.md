# Trial App — Progress

> Ralph 자율 이터레이션의 사람이 읽을 수 있는 요약본.
> 상세 빌드 로그는 `ralph/progress.txt`, 스토리 정의는 `ralph/prd.json`.
> Branch: `ralph/trial-webapp` · 작업 디렉터리: `/work/genasis/trial-app/`.

## 한눈에 보기

| # | Story | 상태 | 완료일 |
|---|---|---|---|
| US-001 | Bootstrap Next.js 15 + Tailwind | ✅ Done | 2026-05-06 |
| US-002 | SQLite persistence layer | ✅ Done | 2026-05-06 |
| US-003 | App bar + 체험/신청 탭 | ✅ Done | 2026-05-06 |
| US-004 | Static kanban board UI | ✅ Done | 2026-05-07 |
| US-005 | Static chat thread UI | ✅ Done | 2026-05-07 |
| US-006 | Scripted demo sprint state machine | ⬜ TODO | — |
| US-007 | Signup form UI | ⬜ TODO | — |
| US-008 | `/api/submit` + Mattermost POST | ⬜ TODO | — |
| US-009 | `/status/[token]` (pending state) | ⬜ TODO | — |
| US-010 | `/api/webhook` for credentials | ⬜ TODO | — |
| US-011 | Status page credentials + `genasis.toml` snippet | ⬜ TODO | — |
| US-012 | Dockerfile + deploy config | ⬜ TODO | — |
| US-013 | `genasis init --trial` flag (Rust CLI) | ⬜ TODO | — |
| US-014 | `genasis example` subcommand (Rust CLI) | ⬜ TODO | — |
| **US-015** | **Plane/MM `Trial` flavor + `[trial]` config wiring** | ⬜ TODO | — |
| **US-016** | **`TrialPlaneProvider` + `TrialMattermostProvider` HTTP forwarders** | ⬜ TODO | — |
| **US-017** | **Sim Plane/MM SQLite schema + helpers in trial-app** | ⬜ TODO | — |
| **US-018** | **`/api/plane/*` bridge endpoints** | ⬜ TODO | — |
| **US-019** | **`/api/mattermost/*` bridge endpoints** | ⬜ TODO | — |
| **US-020** | **`/api/events/stream` SSE broadcaster** | ⬜ TODO | — |
| **US-021** | **Live KanbanBoard + ChatThread (SSE-driven)** | ⬜ TODO | — |
| **US-022** | **Human co-work UI (drag-drop + chat composer, bidirectional)** | ⬜ TODO | — |

## 지금까지 들어간 것

### US-001 — Next.js 15 스켈레톤
- `next 15.5.15` (CVE-2025-66478 패치 버전), `react 19`, `tailwind 3.4.17`, `typescript 5.7.2`.
- `app/layout.tsx`(lang="ko", title "Genasis Trial"), `app/page.tsx`, `app/globals.css`.
- `tsconfig.json` strict + `paths: { @/* }`, `ralph/` 제외.
- `npm run dev` → localhost:3000 200 OK.

### US-002 — SQLite persistence
- `better-sqlite3@12.9.0` 네이티브 모듈, `db/index.ts`에 `getDb()` / `closeDb()` 싱글턴.
- `submissions` 테이블: `id, token (unique), name, email, phone, project_name, team_size, tech_stack(json), message, status (pending|provisioned|revoked, CHECK), credentials_json, created_at, updated_at` + token / status 인덱스.
- `lib/token.ts`의 `generateToken()` — 24바이트 → 32자 base64url.
- `DATABASE_PATH` env var, 기본 `./data/trial.db`. `data/`는 gitignore.

### US-003 — 앱 바 & 탭 라우팅
- `app/components/AppBar.tsx` 서버 컴포넌트. 브랜드 + `<Link>` 두 개 (`?tab=demo|signup`). 활성 탭은 `aria-current="page"`.
- `app/page.tsx`가 `searchParams`(Promise)를 await → `DemoSection` / `SignupSection` 렌더.

### US-004 — Static KanbanBoard
- `app/components/KanbanBoard.tsx` 서버 컴포넌트. props `cards: { id, title, column }[]`.
- 3 컬럼 `<section data-column="todo|inprogress|done">` (gray/blue/green 헤더), 카드 `#<id> <title>` rounded + shadow + `data-card-id`.
- 고정 높이 `h-[420px]`, 컬럼별 `overflow-y-auto`.
- `app/page.tsx` `DemoSection`에 placeholder `DEMO_INITIAL_CARDS` (3개) 마운트.
- 검증: `npm run typecheck` ✓, `curl /?tab=demo` → board / 3 columns / 3 cards 모두 렌더 ✓.

### US-005 — Static ChatThread
- `app/components/ChatThread.tsx` `"use client"` 컴포넌트. props `messages: { time, actor, text }[]`, optional `typing`, `channel`.
- `<ol aria-live="polite">` + `useRef` + `useEffect`로 새 메시지/타이핑 상태 변화 시 부드럽게 bottom 스크롤.
- 액터별 배지 색 매핑(`ACTOR_BADGE` — pm/frontend/backend/code-reviewer/qa/designer/architect/devops/ux/human + fallback). 미지정 액터는 중립색.
- `typing=true`일 때 staggered animate-bounce 3-dot 인디케이터 렌더 (`data-testid="chat-typing-indicator"`).
- `app/page.tsx` `DemoSection`을 `grid-cols-1 lg:grid-cols-[1fr_minmax(280px,360px)]` 2단 레이아웃으로 변경, 칸반 옆에 마운트. placeholder 메시지 3개(`DEMO_INITIAL_MESSAGES`).
- 검증: `npm run typecheck` ✓, `curl /?tab=demo` → chat-thread / message-list / 3개 message-index / 액터 배지 / 14:0X 타임스탬프 / `#scrum-demo` 헤더 / typing indicator 모두 렌더, 칸반도 그대로 ✓.

## 핵심 코드베이스 패턴 (재진입자용)

- 프로젝트 루트는 `/work/genasis/trial-app/` (PRD 문구의 `apps/trial-app/`은 무시).
- 공유 컴포넌트는 `app/components/<Name>.tsx`, `@/app/components/<Name>`로 import.
- 페이지 서버 컴포넌트의 `searchParams`/`params`는 Next 15에서 `Promise` — 반드시 `await`.
- `better-sqlite3`는 Node 런타임 전용. DB 호출 코드는 `"use client"` 파일에서 import 금지. API 라우트는 `export const runtime = "nodejs"` 명시.
- `tech_stack`/`credentials_json`은 TEXT(JSON 문자열)로 저장. 호출부에서 `JSON.stringify`/`parse`.
- 사용자 노출 문자열은 한국어, 코드 식별자/주석은 영어.
- 활성 상태 스타일은 `aria-current="page"` 기반으로 — curl/grep으로 검증 가능.
- DOM 검증을 위해 `data-*` 어트리뷰트(`data-column`, `data-card-id`, `data-testid`, `data-message-index`, `data-actor`) 사용. Tailwind 클래스 문자열보다 안정적.
- 컬럼별 시각적 인코딩(gray=Todo, blue=InProgress, green=Done)은 `KanbanBoard.tsx`의 `COLUMNS` 상수에서 관리 — 호출부에서 색을 다시 정하지 말 것. 액터 배지 색은 `ChatThread.tsx`의 `ACTOR_BADGE`.
- React 19는 인접 텍스트 노드(`#{var}` 같은 식) 사이에 `<!-- -->` HTML 코멘트를 삽입함. curl/grep 검증 시 텍스트만 grep하지 합쳐 grep하지 말 것.
- 칸반·채팅 placeholder 데이터(`DEMO_INITIAL_CARDS`, `DEMO_INITIAL_MESSAGES`)는 현재 `app/page.tsx`에 인라인. US-006의 스크립트 sprint가 들어오면 모두 `lib/demo-script.ts`로 이전.

## 아키텍처 전환 — Trial Bridge (US-015~US-022)

### 배경

처음 PRD는 "스크립트된 데모 시뮬레이션"(US-006)을 핵심으로 잡았지만, 실제 가치는 **genasis 에이전트 팀이 진짜 Plane/MM 없이도 동작하는 환경을 trial-app이 대신 제공**하는 데 있음. 즉, trial-app은 Plane + Mattermost 의 lightweight 시뮬레이터로 작동하고, 사용자는 **agent 호출이 만든 변화를 한 화면에서 라이브로 관찰·코워크** 한다.

### 구조

```
[ genasis-cli / runtime ]
       │
       ▼  (calls trait methods: ensure_channel, post_root, transition issue, …)
[ genasis-providers ]
   ├─ PlaneProvider trait        ←── 기존: Upstream / AgentAware
   │     └─ TrialPlaneProvider   ←── 신규 (US-015 / US-016)
   └─ MattermostProvider trait
         └─ TrialMattermostProvider
             │
             ▼  (HTTPS POST/GET, X-Genasis-Trial-Secret header)
[ trial-app /api/plane/*, /api/mattermost/* ]   ← US-018 / US-019
       │  (CRUD on sim_issues / sim_posts / sim_channels)
       ▼
[ db/sim.ts → SQLite ]                          ← US-017
       │  emit(event)
       ▼
[ /api/events/stream  (SSE) ]                   ← US-020
       │
       ▼
[ LiveKanbanBoard / LiveChatThread ]            ← US-021
       │  (drag-drop / 채팅 입력 → 같은 /api/* 엔드포인트)
       ▼  bidirectional
[ Human co-work loop ]                          ← US-022
```

### 핵심 결정

1. **공통 라이브러리는 기존 `crates/genasis-providers`를 그대로 활용**. `FlavorChoice`에 `Trial` 변형을 추가하고 별도 `trial.rs` 모듈에 trait 구현체를 넣는다. 새 크레이트를 만들지 않는다 — 이미 trait 추상이 있다.
2. **Config 진입점은 `[trial] enabled = true, url = …, shared_secret = …`** 한 블록. 기존 `[plane] flavor`/`[mattermost] flavor`가 명시되면 그걸 우선하고, 없으면 `[trial].enabled` 가 trial 플레이버를 끌어온다.
3. **Trial-app은 Plane/MM 의 *최소* 도메인만 시뮬**한다. issues, projects, channels, posts. labels/cycles/permissions 같은 고급 개념은 stub return 으로 OK.
4. **인증은 `X-Genasis-Trial-Secret` 헤더 한 줄**. 실제 prod 보안이 아니라 같은 머신/네트워크에서 우연히 호출되는 걸 막는 정도.
5. **양방향**: 에이전트가 만든 카드/메시지가 UI에 라이브로 흘러들어가야 하고, 사람이 UI에서 카드 옮기거나 메시지 쓰면 같은 `/api/*` 로 흐른다 → 에이전트가 다음 폴링/콜에서 그 변화를 본다.
6. **기존 US-006(스크립트 데모)는 폐기하지 않는다**. 라이브 모드에 sim 상태가 비어 있을 때 보여주는 "intro 애니메이션" 으로 남는다.

### 변경 영향

- US-005 (ChatThread)는 그대로 진행. US-021 의 `LiveChatThread` 가 이걸 wrap 한다.
- US-006 (scripted state machine) 도 그대로 진행. 단 lifecycle 은 "no agent connected" 모드.
- US-007 ~ US-014 (signup / status / CLI 통합)는 트라이얼 브리지와 독립. 기존 우선순위 유지.
- 신규 US-015 ~ US-022 는 우선순위 15+ 로 추가. 단 **사람이 코워크 가능한 라이브 데모는 본 PRD 가 노리는 핵심 가치**이므로, 다음 마일스톤 끝에는 대부분 닫혀 있어야 한다.

## 다음 이터레이션 계획

다음 우선순위(가장 빠른 `passes: false`)는 **US-006 — Scripted demo sprint state machine**.
구현 명세:
- `lib/demo-script.ts`에 8단계 스프린트 스크립트(PM → Frontend → Code-reviewer → QA, 0/2/3/6/7/9/10/12s 오프셋) 인코딩. 칸반/채팅 placeholder 상수도 여기로 이전.
- `useDemoSprint` 클라이언트 훅: 타임라인을 따라 카드 전환과 메시지 append를 발화. 600ms typing indicator를 메시지 직전에 보여줌.
- `[▶ Run Demo Sprint]` / `[Reset]` 컨트롤 버튼.
- 검증: 브라우저에서 한 사이클 재생, 카드가 Todo→InProgress→InReview→Done 순서로 이동, 채팅 메시지 8건 누적.

이후 순서: US-007 (signup form) → US-008..014 → US-015~022 (trial bridge).

## 참고

- 이 문서는 사람이 한 번에 상태를 파악하기 위한 요약본입니다. 자동화 루프(Ralph)는 `ralph/progress.txt`와 `ralph/prd.json`만 읽고 씁니다.
- 이터레이션이 끝날 때마다 표 상태와 "지금까지 들어간 것" 섹션을 갱신합니다.
