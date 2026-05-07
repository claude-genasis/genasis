# Trial App — Progress

> Ralph 자율 이터레이션의 사람이 읽을 수 있는 요약본.
> 상세 빌드 로그는 `ralph/progress.txt`, 스토리 정의는 `ralph/prd.json`.
> Branch: `ralph/trial-webapp` · 작업 디렉터리: `/work/genasis/trial-app/`.

## 한눈에 보기 — 22 / 22 ✅

| # | Story | 상태 | 완료일 |
|---|---|---|---|
| US-001 | Bootstrap Next.js 15 + Tailwind | ✅ | 2026-05-06 |
| US-002 | SQLite persistence layer | ✅ | 2026-05-06 |
| US-003 | App bar + 체험/신청 탭 | ✅ | 2026-05-06 |
| US-004 | Static kanban board UI | ✅ | 2026-05-07 |
| US-005 | Static chat thread UI | ✅ | 2026-05-07 |
| US-006 | Scripted demo sprint state machine | ✅ | 2026-05-07 |
| US-007 | Signup form UI | ✅ | 2026-05-07 |
| US-008 | `/api/submit` + Mattermost POST | ✅ | 2026-05-07 |
| US-009 | `/status/[token]` (pending state) | ✅ | 2026-05-07 |
| US-010 | `/api/webhook` for credentials | ✅ | 2026-05-07 |
| US-011 | Status page credentials + `genasis.toml` snippet | ✅ | 2026-05-07 |
| US-012 | Dockerfile + deploy config | ✅ | 2026-05-07 |
| US-013 | `genasis init --trial` (Rust CLI) | ✅ | 2026-05-07 |
| US-014 | `genasis example` subcommand (Rust CLI) | ✅ | 2026-05-07 |
| US-015 | Plane/MM `Trial` flavor + `[trial]` config | ✅ | 2026-05-07 |
| US-016 | `TrialPlaneProvider` + `TrialMattermostProvider` | ✅ | 2026-05-07 |
| US-017 | Sim Plane/MM SQLite schema + helpers | ✅ | 2026-05-07 |
| US-018 | `/api/plane/*` bridge endpoints | ✅ | 2026-05-07 |
| US-019 | `/api/mattermost/*` bridge endpoints | ✅ | 2026-05-07 |
| US-020 | `/api/events/stream` SSE broadcaster | ✅ | 2026-05-07 |
| US-021 | Live KanbanBoard + ChatThread (SSE-driven) | ✅ | 2026-05-07 |
| US-022 | Human co-work UI (drag-drop + chat composer) | ✅ | 2026-05-07 |

## 최종 아키텍처

```
┌─ genasis CLI (Rust) ──────────────────────────────────────────┐
│                                                                │
│  genasis init --trial → writes [trial]+[plane]+[mattermost]   │
│                          all flavor="trial"                    │
│  genasis example prd|design|prd2  → drops sample doc          │
│                                                                │
│  (any subcommand that builds a PlaneProvider/MattermostProvider│
│   from config — `init`, `plane`, `mm`, runtime/agents, …)     │
│                                                                │
│  crates/genasis-providers                                      │
│   ├─ plane::FlavorChoice::Trial → TrialPlane (HTTP forwarder) │
│   └─ mattermost::FlavorChoice::Trial → TrialMattermost        │
│                                                                │
└──────────────┬─────────────────────────────────────────────────┘
               │  reqwest POST / PATCH / GET, X-Genasis-Trial-Secret
               ▼
┌─ trial-app (Next.js 15) at trial.realstory.blog / localhost:3000┐
│                                                                  │
│ Tabs:                                                            │
│  체험하기      → /          → DemoBoard (scripted sprint)       │
│  라이브 트라이얼 → /?tab=live → LiveBoard (kanban + chat live)  │
│  신청하기      → /?tab=signup → SignupForm                       │
│                                                                  │
│ Bridge endpoints (auth: secret OR Sec-Fetch-Site=same-origin)   │
│  POST /api/plane/projects                                        │
│  POST /api/plane/issues                                          │
│  PATCH /api/plane/issues/[id]   GET /api/plane/issues            │
│  POST /api/mattermost/channels                                   │
│  POST /api/mattermost/posts     GET /api/mattermost/posts        │
│  GET  /api/events/stream  (SSE — connected/issue.created/        │
│                            issue.updated/post.created/ping)     │
│                                                                  │
│ Public app endpoints                                             │
│  POST /api/submit  (zod → SQLite + Mattermost notify)           │
│  POST /api/webhook (X-Genasis-Webhook-Secret → credentials)     │
│  /status/[token]  (pending / provisioned / revoked branches)    │
│                                                                  │
│ db/                                                              │
│  index.ts  — submissions table + helpers                        │
│  sim.ts    — sim_projects/issues/channels/posts + helpers       │
│              every mutation calls lib/events.ts emit()          │
│                                                                  │
│ lib/                                                             │
│  events.ts        — in-process pub/sub (HMR-safe globalThis)    │
│  trial-auth.ts    — bridge auth (secret OR same-origin)         │
│  genasis-toml.ts  — credential → toml snippet                   │
│  demo-script.ts   — scripted demo timeline                      │
│  use-demo-sprint.ts — hook driving the scripted demo            │
│  token.ts         — 24-byte URL-safe token gen                  │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

## 핵심 결정 / 학습

### 라이브 vs 스크립트
- **체험하기 (US-006)** — placeholder/intro 애니메이션. 8단계 사전 스크립트.
- **라이브 트라이얼 (US-021/022)** — 실제 sim 상태에 SSE로 연결된 양방향 보드. genasis 에이전트가 trial flavor로 호출하면 카드/메시지가 흘러들어오고, 사람이 드래그/입력하면 같은 엔드포인트로 흘러나가 에이전트가 다음 폴링에서 본다. 이게 본질적 가치.

### 트라이얼 브리지 인증
- `/api/plane/*` 와 `/api/mattermost/*` 는 두 가지 방법 중 하나로 통과:
  1. `X-Genasis-Trial-Secret: $TRIAL_SHARED_SECRET` (Rust providers, 서버-서버)
  2. `Sec-Fetch-Site: same-origin` (브라우저가 자동 부착, 트라이얼 앱 자체 UI)
- 이 이중 패턴 덕분에 시크릿이 브라우저로 새지 않고도 라이브 UI가 같은 브리지 라우트를 호출 가능.

### Plane/MM 트레잇 ↔ Sim 임피던스
- `PlaneProvider::ensure_project`가 반환하는 "id" 가 후속 호출에서 `project_id` 파라미터로 사용. Sim 의 `/api/plane/issues` 는 `project_slug` 를 받으므로, `TrialPlane.ensure_project` 는 응답의 `slug` 필드를 반환해 트레잇 contract 와 sim API 양쪽을 잇는다.
- `MattermostProvider` 트레잇은 actor 를 노출하지 않음 (실제 MM에선 bot 토큰 identity 가 actor). Sim 은 명시적 actor 필요. `TrialMattermost` 는 기본 `"agent"` 사용, `GENASIS_TRIAL_ACTOR` env 로 오버라이드.

### Tailwind / Next.js
- `output: "standalone"` 로 Docker 이미지를 작게.
- `runtime = "nodejs"` + `dynamic = "force-dynamic"` 는 better-sqlite3 + POST 라우트 모두에 필수.
- React 19 는 인접 텍스트 노드 사이에 `<!-- -->` 주입. curl/grep 검증 시 합쳐서 grep 하지 말 것.
- Next 15 의 `searchParams` / `params` 는 `Promise` — 서버 컴포넌트에서 await.
- `notFound()` 응답이 RSC wire format 으로 직렬화될 수 있음. 404 + 컴포넌트 정체성은 보존.

### 클라이언트 패턴
- 카드 컬럼 이동 애니메이션은 React reconciliation (parent 변경 → unmount/remount) + 단일 `animate-card-enter` keyframe 으로 충분. Framer Motion 안 씀.
- 타이머 클라이언트 훅은 `useRef<setTimeout[]>` + `clearTimers()` 패턴으로 run/reset/unmount 일괄.
- 폼 검증: `errors = validate(form)` 매 렌더 재계산. state 는 `form` + `touched` 둘만.
- `aria-invalid={Boolean || undefined}` — false 면 어트리뷰트 자체 제거.
- HMR-safe SSE 버스: `globalThis.__genasisSimSubscribers` 에 Set 보관.
- 외부 API 통합은 tagged union `Result = sent|skipped|failed` 패턴.
- 외부 부수효과 호출은 **DB 영속화 이후** (알림 실패해도 행 살아있게).
- env-var 정책: 모두 unset = 스킵, 일부라도 set + 실패 = 500.

## 검증 레시피 (사용자가 실행)

전체 시스템을 실제로 돌려보려면:

```bash
# 1) trial-app 띄우기 (격리 DB 권장)
cd /work/genasis/trial-app
DATABASE_PATH=/tmp/trial-dev.db \
WEBHOOK_SHARED_SECRET=hooksecret \
TRIAL_SHARED_SECRET=trialsecret \
npm run dev
# →  http://localhost:3000

# 2) 브라우저에서 세 탭을 각각 확인
#  http://localhost:3000/?tab=demo    →  Run Demo Sprint 클릭, 카드와 채팅 진행
#  http://localhost:3000/?tab=live    →  처음엔 빈 보드. 다른 터미널에서 step 3 실행하면 카드/메시지가 라이브로 들어옴
#  http://localhost:3000/?tab=signup  →  폼 채우고 Submit → /status/<token> 으로 redirect

# 3) (다른 터미널) Rust trial provider 가 라이브 보드에 칩스 추가
cd /work/genasis
TRIAL_BASE=http://localhost:3000 TRIAL_SECRET=trialsecret \
  cargo test -p genasis-providers --lib trial_e2e -- --ignored --nocapture
TRIAL_BASE=http://localhost:3000 TRIAL_SECRET=trialsecret \
  cargo test -p genasis-providers --lib mm_trial_e2e -- --ignored --nocapture
# 라이브 탭에서 카드와 채팅이 추가되는 게 보일 것

# 4) 사람-에이전트 코워크 시연
#  라이브 탭에서 "Done" 컬럼으로 카드를 드래그 → PATCH /api/plane/issues/:id 가 가서 다른 클라이언트에 SSE 로 동기화됨
#  채팅 입력창에 메시지 → POST /api/mattermost/posts → 같은 채널에 연결된 다른 브라우저에도 즉시 표시

# 5) 신청 → 자격증명 발급 시뮬
TOKEN=$(curl -sS -X POST http://localhost:3000/api/submit \
  -H 'content-type: application/json' \
  -d '{"name":"You","email":"you@example.com","projectName":"my-app","teamSize":"solo"}' \
  | python3 -c "import sys,json;print(json.load(sys.stdin)['token'])")
echo "TOKEN=$TOKEN"
# 브라우저에서 http://localhost:3000/status/$TOKEN  →  pending 상태

curl -sS -X POST http://localhost:3000/api/webhook \
  -H 'content-type: application/json' \
  -H "x-genasis-webhook-secret: hooksecret" \
  -d "{\"token\":\"$TOKEN\",\"plane\":{\"url\":\"https://plane.realstory.blog\",\"login\":\"you\",\"password\":\"hunter2\",\"api_key\":\"plane-api-AAA\",\"workspace_slug\":\"you-ws\"},\"mattermost\":{\"url\":\"https://mm.realstory.blog\",\"login\":\"you\",\"password\":\"hunter3\",\"bot_tokens\":{\"pm\":\"bot-pm\",\"frontend\":\"bot-fe\",\"qa\":\"bot-qa\"}}}"
# 브라우저 status 페이지 새로고침 → provisioned + 자격증명 표시 + genasis.toml Copy 가능

# 6) Rust CLI 자체
/work/genasis/target/debug/genasis example --help
/work/genasis/target/debug/genasis example prd --project /tmp/example-project
/work/genasis/target/debug/genasis init --trial --probe-only --project /tmp/trial-init-test

# 7) 모든 자동화 검증
cd /work/genasis/trial-app && npm run typecheck && npm run build
cd /work/genasis && cargo test -p genasis-providers --lib
cd /work/genasis && cargo test -p genasis-cli --bin genasis
```

## 참고

- 이 문서는 사람이 한 번에 상태를 파악하기 위한 요약본입니다. 자동화 루프(Ralph)는 `ralph/progress.txt` 와 `ralph/prd.json` 만 읽고 씁니다.
- 모든 인터랙티브 UX (드래그-드롭 / 키보드 입력 / 클립보드 복사 / 타이머 애니메이션)는 SSR + 클라이언트 정적 검증이 끝났지만, 최종 시각/동작 확인은 브라우저에서 수동으로 한 번 해주세요.
