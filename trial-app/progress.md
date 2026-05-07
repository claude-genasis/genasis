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
| US-004 | Static kanban board UI | ⏳ In progress | — |
| US-005 | Static chat thread UI | ⬜ TODO | — |
| US-006 | Scripted demo sprint state machine | ⬜ TODO | — |
| US-007 | Signup form UI | ⬜ TODO | — |
| US-008 | `/api/submit` + Mattermost POST | ⬜ TODO | — |
| US-009 | `/status/[token]` (pending state) | ⬜ TODO | — |
| US-010 | `/api/webhook` for credentials | ⬜ TODO | — |
| US-011 | Status page credentials + `genasis.toml` snippet | ⬜ TODO | — |
| US-012 | Dockerfile + deploy config | ⬜ TODO | — |
| US-013 | `genasis init --trial` flag (Rust CLI) | ⬜ TODO | — |
| US-014 | `genasis example` subcommand (Rust CLI) | ⬜ TODO | — |

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

## 핵심 코드베이스 패턴 (재진입자용)

- 프로젝트 루트는 `/work/genasis/trial-app/` (PRD 문구의 `apps/trial-app/`은 무시).
- 공유 컴포넌트는 `app/components/<Name>.tsx`, `@/app/components/<Name>`로 import.
- 페이지 서버 컴포넌트의 `searchParams`/`params`는 Next 15에서 `Promise` — 반드시 `await`.
- `better-sqlite3`는 Node 런타임 전용. DB를 만지는 코드는 `"use client"` 파일에서 import 금지.
- `tech_stack`/`credentials_json`은 TEXT(JSON 문자열)로 저장. 호출부에서 `JSON.stringify`/`parse`.
- 사용자 노출 문자열은 한국어, 코드 식별자/주석은 영어.
- 활성 상태 스타일은 `aria-current="page"` 기반으로 — curl/grep으로 검증 가능.

## 다음 이터레이션 계획

### US-004 — Static KanbanBoard (착수 중)
- `app/components/KanbanBoard.tsx` 서버 컴포넌트.
- props: `cards: { id: number; title: string; column: "todo" | "inprogress" | "done" }[]`.
- 3 컬럼 (Todo gray / In Progress blue / Done green), Tailwind. 카드: `#<id> <title>` rounded + shadow.
- 고정 높이, 컬럼별 스크롤. 데모 섹션에 placeholder 카드와 함께 마운트.
- 검증: `npm run typecheck` + dev 서버 curl로 컬럼·카드 렌더 확인.

### US-005 이후
- ChatThread → Demo state machine (8단계 스크립트) → SignupForm → `/api/submit` (zod + MM REST) → `/status/[token]` 서버 페이지 → `/api/webhook` (HMAC shared secret) → 자격증명 + `genasis.toml` 스니펫 → Dockerfile → Rust CLI 두 건.

## 참고

- 이 문서는 사람이 한 번에 상태를 파악하기 위한 요약본입니다. 자동화 루프(Ralph)는 `ralph/progress.txt`와 `ralph/prd.json`만 읽고 씁니다.
- 이터레이션이 끝날 때마다 표 상태와 "지금까지 들어간 것" 섹션을 갱신합니다.
