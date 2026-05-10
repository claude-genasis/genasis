# 릴리즈 노트 — v0.5.0

> English: [`../RELEASE-NOTES-v0.5.0.md`](../RELEASE-NOTES-v0.5.0.md)

**릴리즈일**: 2026-05-10

Genasis 의 첫 공개 바이너리 릴리즈. v0.1.0 ~ v0.4.x 를 건너뛰는 이유는
코드베이스가 v0.1.0 으로 계획되었던 M0–M11 범위를 한참 넘어선 상태이기
때문이며, v0.5.0 은 M0–M21 마일스톤과 ADR 1~15 까지 실제로 구현된
기능 집합을 반영합니다.

---

## Genasis 가 무엇인가

Genasis 는 AI 에이전트를 사람과 동일한 **일급 팀원**으로 만들어, 팀이
이미 사용 중인 협업 도구 (이슈는 Plane, 채팅은 Mattermost) 안에
배치합니다. 한 명령으로 10명 역할의 에이전트 팀 전체를 프로비저닝하고,
Plane 워크스페이스와 Mattermost 팀에 연결한 뒤, 사람 팀원과 함께
돌리던 그 동일한 스프린트 워크플로를 그대로 넘겨줍니다 — 이제는
에이전트가 티켓을 집어들고, TDD 를 돌리고, 스레드에 답변까지 합니다.

---

## 주요 기능

### Bolt-on 에이전트 팀 (ADR-001, ADR-010)
- `genasis init` / `genasis bootstrap` — 어떤 프로젝트(빈 프로젝트든
  기존 프로젝트든)에 대해서도 표준 10 역할 (PM, Planner, Architect,
  Frontend, Backend, QA, Designer, Security, DevOps, Code-reviewer)
  을 `.claude/agents/` 에 scaffold.
- Marker-fence 오버레이 모델: 기존 에이전트 팀(ECC, knowledge-work-plugins,
  claude-code-templates) 을 비파괴적으로 증강. fence 를 제거하면 원본이
  byte-identical 로 복원.

### 실제 팀 통합
- **Plane**: 3개 flavor (`upstream`, `agent-aware`, `auto`) 의 native
  REST 통합 — probe 시점에 자동 감지.
- **Mattermost**: 봇 계정 프로비저닝, 채널 ensure, 스레드 답변,
  post-tool sync 훅.
- **Trial 브릿지** (ADR-013): `http://localhost:3000` 에서 Plane +
  Mattermost 를 시뮬레이션하는 번들 trial-app — 실제 서버 없이 Genasis
  를 평가 가능.

### 사람 로스터 (ADR-014, 신규)
- `genasis.toml` 의 `[[humans]]` 배열로 사람 stakeholder 등록.
- `genasis humans add | edit | remove | list | sync` CRUD CLI.
- TUI wizard 7 단계 Humans 모달, `a/e/d/s/Enter` 키바인딩.
- Mattermost 관리자 즉시 생성 + 24자 임시 비밀번호 (첫 로그인 시 변경
  강제).
- 에이전트는 `GENASIS.md § 사람 로스터` 를 통해 등록된 사람을 인식하고
  그들의 메시지를 **바인딩 stakeholder 요구사항**으로 처리; 미등록
  발신자는 PM 검증을 거침.

### 다중 호스트 서버 스택 (ADR-015, 신규)
- `servers/docker-compose.yml` 이 **공유 PostgreSQL** (role + database
  격리) 로 Plane + Mattermost 부팅 — 쌍둥이-postgres 대비 ~400 MB
  RAM 절약.
- `servers/init/init-databases.sh` (최소 권한 role grant) 와
  `servers/scripts/setup-user-env.sh` (운영자별 자격증명 부트스트랩).
- `docs/MIGRATE-PG-CONSOLIDATION.md` — 쌍둥이-postgres 에서 마이그레이션
  하는 운영자 runbook.

### 슬래시 명령 + 훅
- 17 개 슬래시 명령 (`/check-inbox`, `/sprint-start`, `/issue-link`,
  `/intake-review`, `/db-migrate`, `/design-change` 등) — `GENASIS.md`
  와 역할별 오버레이 fence 로의 thin pointer.
- 6 개 훅 (post-tool MM sync, post-tool token-trim, pre-tool branch
  guard, pre-tool worktree guard, session-start, user-prompt-submit).

### 데이터베이스 채널 (ADR-004)
- 읽기 전용 `genasis db query "SELECT ..."` — DDL/DML/트랜잭션은 SQL
  guard 가 거부.
- `genasis db migrate` — Atlas / Drizzle Kit / DuckDB raw_runner 자동
  감지.

### 디자인 swap (ADR-009)
- `genasis design swap <slug>` — 외부 getdesign.md 카탈로그 또는 로컬
  파일에서 `design-system.md` 를 hot-swap.
- override log + verify + ticket emitter — 재디자인 기반 영역별 이슈
  플랜 자동 생성.

### 토큰 경제 (ADR-006)
- `[token_economics] trim_threshold_kb` (기본 32 KB) 초과 도구 결과를
  자동 trim.
- RTK 토큰 카운터가 설치되어 있으면 shell 도구 호출에 자동 wrap.

### 이중언어 (ADR-008)
- 모든 문서, 에이전트 오버레이, 슬래시 명령, 런타임 문자열이 영어 **와**
  한국어 양쪽 제공.
- `genasis lang switch en|ko` — 에이전트 컨텍스트 atomic 전환.
- Mirror-drift CI 게이트 — EN/KO 문서 분기 방지.

### Debug history 피드백 루프 (ADR-012)
- baseline manifest 와 라이브 `.claude/genasis/` 파일 간 drift 를 조용히
  추적.
- `genasis debug collect` — 익명화된 patch.json 생성; `submit` —
  genasis 본 리포의 `debug-history/` 에 PR 자동 발행.
- Data-only-PR 게이트가 contributor 범위 강제 (patches/*.patch.json
  만).

### Monitor TUI (ADR-007)
- `genasis monitor` — ratatui 대시보드 (스프린트 상태, 토큰 소비,
  에이전트 세션, 배포 상태, 네트워크, 로그).

### 베스트-오브-브리드 에이전트 카탈로그 (ADR-011)
- `genasis agents browse | install | list | remove` — 커뮤니티
  에이전트 (ECC, wshobson, VoltAgent, dl-ezo) 의 버전 관리된 카탈로그
  지원.
- `agents-pool` 비공개 리포 — 검증된 카탈로그 항목 큐레이팅 + 재배포.

---

## 설치

```bash
# 사전 빌드된 바이너리 (권장)
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/claude-genasis/genasis/main/install.sh | sh

# 소스에서 빌드
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/claude-genasis/genasis/main/build.sh | sh
```

지원 플랫폼: **Linux x86_64**, **Linux aarch64**, **macOS x86_64
(Intel)**, **macOS aarch64 (Apple Silicon)**. Windows 사용자: WSL2
+ Linux x86_64 바이너리.

설치 후:

```bash
genasis doctor          # 환경 검증
genasis init --trial    # 실제 Plane/MM 스택 없이 Genasis 체험
genasis init            # 실제 Plane + MM 에 에이전트 팀 셋업
```

---

## Breaking changes

없음 — 첫 공개 릴리즈.

---

## 알려진 제한

- `genasis migrate-from-genesis` 는 문서 전용; 실제 마이그레이션 도구는
  실 Genesis bash 팀 운영 데이터를 받은 v0.5 이후 도입.
- `genasis agents status` 가 `#[tokio::main]` 안에서
  `reqwest::blocking` 을 호출해 런타임 종료 시 패닉. README 에 노출되지
  않음; v0.6 후속.
- `genasis debug submit` 의 실제 `gh pr create` 호출은 여전히
  `--dry-run` 게이트; canonical 계약은 E2E 에서 assert.
- Plane Playwright 사용자 프로비저닝 (`provision-plane-users.mjs`) 은
  stub — 실제 UI 자동화는 v0.6+ 에서. 그때까지는 에이전트 사용자 계정을
  수동 생성하거나 `--trial` flavor 사용.
- `nightly-e2e` 실서버 스모크는 **로컬 전용 pre-push 게이트**
  (`scripts/nightly-e2e.sh`) — GitHub Actions 워크플로 아님.
  [`docs/ko/TESTING.md` § L9](TESTING.md#l9--실서버-스모크-scriptsnightly-e2esh)
  참조.
- TUI wizard 테스트 커버리지는 현재 0% (1,200 줄 미커버). headless
  ratatui 하네스는 v0.6 계획.

---

## 커버리지 및 테스트

- 10개 workspace crate 에 걸쳐 **245개 테스트**, 모두 green
- 라인 커버리지 **53.97%** (CI 의 `coverage` job 이 Codecov 에 업로드)
- 미커버 대부분은 TUI wizard (의도적 — [`docs/ko/TESTING.md`](TESTING.md)
  참조)

---

## 업그레이드 경로

이것이 v0.5.0 — 업그레이드할 이전 릴리즈가 없습니다. v0.5 → v0.6 은
marker fence v1.0 계약을 유지; 사용자는 언제든 `genasis upgrade
--fence-version 1.0` 을 실행해 no-op 결과를 기대할 수 있습니다.

---

## 감사

전체 upstream 프로젝트 + contributor agent 목록은
[`docs/ko/CREDITS.md`](CREDITS.md) 참조. ECC, knowledge-work-plugins,
claude-code-templates, awesome-design-md, 그리고 카탈로그를 시드한
wshobson / VoltAgent / dl-ezo 에이전트 모음의 메인테이너에게 특별한
감사를 표합니다.

trial-app 의 초기 22개 user story 는 [Ralph 자율-루프 패턴](https://github.com/snarktank/ralph)
([`docs/famous-agents.md` §11](../famous-agents.md)) 의 이터레이션으로
작성됐으며, 이후 Ralph 상태 디렉터리는 일반 feature-PR 흐름으로 대체되며
retire 됐습니다.

---

## 전체 commit 목록 (프로젝트 시작 이래)

전체 마일스톤 히스토리 (M0 ~ M21) 는 [`progress.ko.md`](../../progress.ko.md)
(한국어 SSOT) / [`progress.md`](../../progress.md) (영문) 에 보존.
v0.5.0 에 81개 커밋이 포함됩니다.
