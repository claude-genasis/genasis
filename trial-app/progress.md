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
| US-006 | Scripted demo sprint state machine | ✅ Done | 2026-05-07 |
| US-007 | Signup form UI | ✅ Done | 2026-05-07 |
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
- `app/components/ChatThread.tsx` `"use client"` 컴포넌트. props `messages: { time, actor, text }[]`, optional `typing`(US-006에서 `typingActor: string | null`로 교체), `channel`.
- `<ol aria-live="polite">` + `useRef` + `useEffect`로 새 메시지/타이핑 상태 변화 시 부드럽게 bottom 스크롤.
- 액터별 배지 색 매핑(`ACTOR_BADGE` — pm/frontend/backend/code-reviewer/qa/designer/architect/devops/ux/human + fallback). 미지정 액터는 중립색.
- `typing=true`(US-006 이후 `typingActor`)일 때 staggered animate-bounce 3-dot 인디케이터 렌더 (`data-testid="chat-typing-indicator"`).
- `app/page.tsx` `DemoSection`을 `grid-cols-1 lg:grid-cols-[1fr_minmax(280px,360px)]` 2단 레이아웃으로 변경, 칸반 옆에 마운트. placeholder 메시지 3개(`DEMO_INITIAL_MESSAGES`)는 US-006에서 `lib/demo-script.ts` 로 이전.
- 검증: `npm run typecheck` ✓, `curl /?tab=demo` → chat-thread / message-list / 3개 message-index / 액터 배지 / 14:0X 타임스탬프 / `#scrum-demo` 헤더 / typing indicator 모두 렌더, 칸반도 그대로 ✓.

### US-007 — Trial signup form UI
- `app/components/SignupForm.tsx` (`"use client"`). FormState — `name`/`email`/`phone`/`projectName`/`teamSize: "solo"|"small"|"medium"|""`/`techStack: string[]`/`message`. `validate()`이 한국어 에러 메시지를 키별로 반환하고, `Touched` Set 으로 blur 한 필드만 인라인 에러 노출.
- `noValidate` 폼 + `aria-invalid={showError(key) || undefined}` 로 브라우저 툴팁 무력화하고 자체 인라인 에러를 단일 소스로.
- 필수 필드 모두 채울 때까지 Submit 버튼 disabled. 클릭 시(invalid라면) required 4 필드 일괄 touched 마킹 → 인라인 에러 일제히 노출. 실제 POST 는 US-008 에서.
- 기술 스택은 8개 체크박스(`techstack-react/nextjs/vue/node/python/rust/go/mobile`) 칩 토글 — 선택 시 다크 반전.
- 안내 배너 "ℹ trial.realstory.blog … 관리자 협의 후 기간 제한 없이 이용 가능합니다." 가 Submit 버튼 아래 노출(`data-testid="signup-info-banner"`).
- `app/page.tsx` `SignupSection` 이 `<SignupForm />` 마운트, max-w-3xl 로 폭 조정.
- 검증: typecheck ✓; curl `/?tab=signup` → `signup-form` / 7 `field-*` / 8 `techstack-*` / team_size 옵션(solo/small/medium) / `required`+`aria-required="true"` (필수 4필드) / 초기엔 `error-*` 0 개·`aria-invalid` 미부착 / submit 버튼 bare `disabled` / 한국어 안내 배너 / 활성 탭 "신청하기" 모두 정상 ✓. 인터랙티브(blur→에러, 입력→에러 해제, valid→Submit 활성)는 브라우저 MCP 부재로 manual verification.

### US-006 — Scripted demo sprint state machine
- `lib/demo-script.ts`(데이터, 서버/클라이언트 어디서나 import 가능) — `KanbanOp`/`DemoStep` 타입, `INITIAL_CARDS=[]`, `INITIAL_MESSAGES=[]`, `TYPING_LEAD_MS=600`, 8 step 배열(0/2/3/6/7/9/10/12 s 오프셋, PM → frontend → code-reviewer → frontend → qa).
- `lib/use-demo-sprint.ts`(`"use client"`) — `useDemoSprint()` 훅. `useRef`에 timer handle 배열을 두고 `run()`/`reset()`/cleanup에서 `clearTimers()`. 함수형 setState로 stale closure 방지. 각 step에 대해 `offsetMs - 600 ms` 시점에 `typingActor` 설정, `offsetMs` 시점에 카드/메시지 갱신 + `typingActor=null`.
- `app/components/DemoBoard.tsx` (`"use client"`) — Run / Reset 버튼 + 한국어 상태 라인(`data-testid="demo-status"`) + 기존 KanbanBoard·ChatThread 렌더. `data-status` 속성에 hook 의 `status`를 그대로 노출.
- `app/components/KanbanBoard.tsx` 카드 `<li>`에 `animate-card-enter` 추가 — 컬럼 이동 시 React가 unmount/remount → 새 컬럼에서 300 ms `cardEnter` keyframe(opacity 0→1, translateY 8px→0, scale 0.96→1) 재생.
- `app/components/ChatThread.tsx` API 교체: `typing: boolean` → `typingActor: string | null`. 인디케이터에 액터 배지 + 점 3개 + "입력 중…" 텍스트.
- `tailwind.config.ts` `theme.extend.keyframes.cardEnter` + `theme.extend.animation.card-enter` 추가.
- `app/page.tsx` 인라인 `DEMO_INITIAL_*` 제거, `<DemoBoard />`로 단일화.
- 검증: `npm run typecheck` ✓, `curl /?tab=demo` → idle 초기 상태(0 카드 / 0 메시지 / typing 인디케이터 없음 / Run 활성·Reset 비활성 / 상태 라인 "대기 중 …") 모두 정상 ✓; 스크립트 데이터(`Add login page` 등)가 `.next/static/webpack/*.js` 클라이언트 번들에 포함됨 ✓; Node로 타이머 스케줄을 dry-run하여 8 step 후 cards=[done], messages.length=8, typingActor=null 확인 ✓. 인터랙티브 재생(클릭 후 카드/메시지 진행)은 브라우저 MCP 가 없어 수동 확인 필요.

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
- 데모 시나리오 데이터는 `lib/demo-script.ts`에 단일화. 새 시나리오/스텝/액터 추가 시 이 파일만 수정. 훅 로직(`lib/use-demo-sprint.ts`)은 데이터에 의존하지 않게 forEach + setTimeout 패턴 유지.
- 컬럼 이동 애니메이션은 React reconciliation(서로 다른 부모 → unmount/remount) + Tailwind `animate-card-enter`로 충분. Framer Motion 같은 무거운 deps 추가 금지.
- 타이머 기반 클라이언트 훅은 `useRef<ReturnType<typeof setTimeout>[]>`에 모든 핸들 보관 → `run()`/`reset()`/unmount cleanup 에서 일괄 `clearTimeout`. 함수형 setState 로 stale closure 회피.
- 폼 검증 패턴(US-007): `errors = validate(form)`은 매 렌더 재계산. state 는 `form` + `touched` 둘만 보관. `aria-invalid`는 `showError(key) || undefined` — false 가 아닌 undefined 로 두어 DOM 에서 어트리뷰트 자체를 제거(스크린리더가 "explicitly valid"로 잘못 해석하지 않게).
- `<form noValidate>` + 자체 인라인 에러로 단일 소스. 브라우저 기본 툴팁은 끔.

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

다음 우선순위(가장 빠른 `passes: false`)는 **US-008 — `/api/submit` route with Mattermost notification**.
구현 명세:
- `POST /api/submit` (`app/api/submit/route.ts`) — zod 로 페이로드 검증, 토큰 생성, `submissions` 테이블에 INSERT (status='pending').
- Mattermost REST API POST: `MM_BOT_TOKEN` + `MM_TRIAL_CHANNEL_ID` env var 사용, PRD §4.3 의 Markdown 포맷.
- 응답 200 `{ token, statusUrl: "/status/<token>" }`; 400 (zod 실패); 500 (MM 실패하지만 row 는 저장).
- `SignupForm` 의 `handleSubmit` 을 `fetch('/api/submit')` + 성공 시 `router.push(statusUrl)` 으로 연결.
- 검증: typecheck, vitest 로 route handler 의 zod·DB·MM 모킹 테스트 (set up vitest if needed).

이후 순서: US-009/010/011 (status + webhook) → US-012/013/014 (deploy + CLI) → US-015~022 (trial bridge).

## 참고

- 이 문서는 사람이 한 번에 상태를 파악하기 위한 요약본입니다. 자동화 루프(Ralph)는 `ralph/progress.txt`와 `ralph/prd.json`만 읽고 씁니다.
- 이터레이션이 끝날 때마다 표 상태와 "지금까지 들어간 것" 섹션을 갱신합니다.
