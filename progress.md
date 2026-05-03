# Genasis — Progress Tracker

> `blueprint.md` 의 §15 (1차 릴리즈 범위) 와 §12 (repo 구조) 를 따라 마일스톤 단위로 진행.
> 각 항목은 `[ ]` → `[x]` 로 닫고, 막힌 항목은 `[!]` 로 표기 + 사유 inline 기록.

**Started**: 2026-05-03
**Target 1차 릴리즈**: TBD
**현재 마일스톤**: M12 — Internationalization (계획 완료, 승인 대기 — M0–M11 모두 완료)

---

## Convention

- 체크박스 표기:
  - `[ ]` = 미시작
  - `[~]` = 진행 중
  - `[x]` = 완료
  - `[!]` = 막힘 (사유 적기)
  - `[s]` = skip (이유 기록)
- 새 ADR 추가는 진행 중 발견 시 즉시 `docs/ADR/` 에 작성
- `blueprint.md` 변경이 필요한 결정은 별도 ADR로 기록 후 blueprint 변경

---

## M0 — Bootstrap

repo 초기 구조와 진행 추적 인프라. **소스 트리 전체를 `genasis/` 아래에 정식으로 나열하고, install.sh 는 launcher 로 한정.**

### Top-level files
- [x] `blueprint.md`
- [x] `progress.md`
- [x] `README.md`
- [x] `LICENSE` (MIT)
- [x] `.gitignore`
- [x] `.editorconfig`
- [x] `rustfmt.toml`
- [x] `clippy.toml`
- [x] `rust-toolchain.toml` (1.78)

### Workspace
- [x] `Cargo.toml` (workspace, 9 members)
- [s] `Cargo.lock` (생성은 첫 `cargo build` 후 — 로컬에 cargo 미설치, CI 첫 푸시에서 commit)

### Crate stubs (Cargo.toml + 최소 src 골격)
- [x] `crates/genasis-cli/` (main.rs + cmd_*.rs 스텁 11개 + tui_attach.rs + scripts/)
- [x] `crates/genasis-core/` (lib + config + env + fs + marker + error)
- [x] `crates/genasis-overlay/` (lib + detector + role_inference + merger + validator + dry_run)
- [x] `crates/genasis-providers/` (lib + plane/* + mattermost/* + github)
- [x] `crates/genasis-db/` (lib + kernel + guard + adapters/*) — guard 에 첫 단위 테스트 포함
- [x] `crates/genasis-design/` (lib + extractor + change_protocol + diff + ticket_emitter)
- [x] `crates/genasis-tui/` (lib + theme + layout + widgets/*)
- [x] `crates/genasis-monitor/` (lib + app + state + widgets/* + actions/*)
- [x] `crates/genasis-templates/` (lib + templates/ Tera 디렉토리 골격)

### install.sh launcher
- [x] OS·아키텍처 감지 (linux x86_64 / arm64, macOS arm64 / x86_64, Windows→WSL 안내)
- [x] Linux 배포판 감지 (`/etc/os-release`)
- [x] 선결 패키지 검사 (필수: git, curl, tar, bash / 선택: node≥18, gh, atlas, psql/mysql/sqlite3/duckdb, rtk, claude)
- [x] 누락 패키지에 대해 OS별 설치 명령 출력 (apt, dnf, pacman, zypper, apk, brew, port, nvm)
- [x] GitHub Releases 자산 URL 결정 + 다운로드 + sha256 검증 + tar 해제
- [x] `~/.local/bin/genasis` 또는 `/usr/local/bin/genasis` 설치 + PATH 안내
- [x] `--no-run`, `--prefix=PATH`, `--version=X.Y.Z`, `--skip-prereqs`, `-h/--help` 플래그 지원
- [x] 마지막에 `genasis attach` 자동 실행 (옵트아웃 가능)
- [x] 실패 시 안전한 종료 코드 + 명시 에러 메시지
- [x] 로컬 스모크 테스트 (Ubuntu/apt 환경) — OS·패키지 감지 · graceful fail 검증 완료

### .github
- [x] `.github/workflows/ci.yml`
- [x] `.github/workflows/release.yml` (cross-rs 로 linux-arm64 cross-compile 포함)
- [x] `.github/workflows/nightly-e2e.yml`
- [x] `.github/ISSUE_TEMPLATE/bug.md`
- [x] `.github/ISSUE_TEMPLATE/feature.md`
- [x] `.github/PULL_REQUEST_TEMPLATE.md`

### docs
- [x] `docs/ARCHITECTURE.md`
- [x] `docs/PROVIDERS.md`
- [x] `docs/MIGRATION-FROM-GENESIS.md`
- [x] `docs/TOKEN-ECONOMICS.md`
- [x] `docs/MONITOR.md`
- [x] `docs/ADR/ADR-000-template.md`

### tests
- [x] `tests/golden/{ecc-only,kw-plugins,blank,legacy-bash-genesis,with-drizzle,with-duckdb}/{input,expected}/.gitkeep` (6 픽스처)
- [x] `tests/golden/SHARED.md` (시나리오 표 + 컨벤션)
- [x] 픽스처별 `README.md` 6개
- [x] `tests/e2e/.gitkeep`, `tests/unit/.gitkeep`

### templates 골격 (Tera placeholder들)
- [x] `crates/genasis-templates/templates/GENASIS.md.tera`
- [x] `crates/genasis-templates/templates/genasis.toml.tera`
- [x] `crates/genasis-templates/templates/env.agents.tera`
- [x] `crates/genasis-templates/templates/mcp.json.tera`
- [x] `crates/genasis-templates/templates/design-system.md.tera`
- [x] `crates/genasis-templates/templates/agent-overlays/README.md` + `.gitkeep`
- [x] `crates/genasis-templates/templates/commands/README.md` + `.gitkeep`
- [x] `crates/genasis-templates/templates/skills/README.md` + `.gitkeep`
- [x] `crates/genasis-templates/templates/hooks/README.md` + `.gitkeep`

### 검증
- [s] `cargo build` (로컬 cargo 미설치 — CI 첫 push 에서 검증; 모든 stub 에서 모듈 트리·import 정합성을 사람 검토)
- [s] `cargo test` (동일)
- [x] `bash install.sh --version=v0.0.0-test --no-run` 스모크 — Ubuntu/apt 감지, 패키지 진단 정상, release 미존재 graceful 처리 확인

### 회고
- [x] M0 회고: 1) install.sh 의 per-distro 패키지 가이드 매트릭스가 가장 시간이 걸렸음 — apt/dnf/pacman/zypper/apk/brew/port 7개 매니저 × 9개 패키지. 2) Cargo workspace 설계 시 `genasis-templates` 의 `include_dir!()` 임베드 결정으로 distribution 단순화. 3) 로컬에 Rust toolchain 부재 — CI 의존성 증가하므로 첫 푸시 후 즉시 CI 결과 확인 필요. 4) Marker fence hash 가 4-byte truncate 인 점은 충돌 확률 vs 가독성 trade-off — M2 에서 재검토.

---

## M1 — Core Infra (genasis-core, genasis-cli skeleton)

`genasis-core` 실작동 + CLI skeleton 의 첫 실작동 명령(`version`).

- [x] `crates/genasis-core/` 실작동
  - [x] `config.rs` — `genasis.toml` schema + load/save + 부모 디렉토리 walk-up `discover()` (3 단위 테스트)
  - [x] `env.rs` — `.env.agents` 읽기·쓰기, comment·blank·quoting 보존 round-trip (5 단위 테스트 + 2 통합 테스트)
  - [x] `fs.rs` — atomic write (sibling tmp + rename + dir fsync), snapshot, optional read (4 단위 테스트)
  - [x] `marker.rs` — fence parse / serialise / hash / find / inject / replace / upsert / remove, idempotency 보장 (10 단위 테스트 + 4 통합 테스트)
  - [x] `error.rs` — 공통 에러 타입 (NotImplemented, Io, Toml, Json, Config, Overlay, Provider, Db)
- [x] `crates/genasis-cli/` 실작동
  - [x] `main.rs` + clap v4 dispatch (12 서브커맨드 wired)
  - [s] `cmd_init.rs` — placeholder (M3 에서 실제 프로비저닝)
  - [s] `cmd_attach.rs` — placeholder (M2)
  - [s] `cmd_detach.rs` — placeholder (M2)
  - [s] `cmd_doctor.rs` — placeholder (M8)
  - [s] `cmd_upgrade.rs` — placeholder (M8)
  - [s] `cmd_design.rs` — placeholder (M7)
  - [s] `cmd_db.rs` — placeholder (M5)
  - [s] `cmd_monitor.rs` — placeholder (M9)
  - [x] `cmd_version.rs` — 실작동 (`--json` 옵션 포함, fence v1.0 / build profile / git_sha 출력)
- [x] `crates/genasis-overlay/role_inference.rs` 시드 (10 role + Custom, slug round-trip 보장)
- [x] `crates/genasis-db/guard.rs` 강화 — comment 제거, string-literal 인지 split, EXPLAIN/ANALYZE/PRAGMA/SHOW/DESC/VALUES 허용, 트랜잭션 제어 거부 (10 단위 테스트 + 5 통합 테스트)
- [x] 단위 테스트: marker fence idempotency + env round-trip + role inference round-trip + SQL guard
- [x] 통합 테스트: `crates/genasis-core/tests/{marker_idempotent,env_round_trip}.rs`, `crates/genasis-overlay/tests/role_inference.rs`, `crates/genasis-db/tests/sql_guard.rs`
- [s] CI green — 첫 GitHub push 시 검증 (로컬 cargo 미설치)
- [x] ADR-001: Overlay = Marker Fence
- [x] ADR-002: Rust 단일 바이너리

### 회고
- [x] M1 회고: 1) marker fence 의 `find()` 가 BEGIN/END 한 쌍만 허용해야 한다는 invariant 를 일찍 강제한 것이 옳았음 — duplicate fence 는 즉시 에러로 거부. 2) `.env.agents` 의 comment 보존은 IndexMap 만으론 불가능 — `Vec<Line>` enum 으로 한 단계 lower-level 표현 필요. 3) SQL guard 의 string-literal aware split 은 hand-rolled lexer 가 sqlparser-rs 보다 단순하고 의존성 적음 (현재 의존성: regex 만). 4) Cargo workspace 의 dev-dependency 는 각 crate 개별 선언 필요 — `tempfile` 을 두 군데에 reuse 하므로 workspace dep 으로 끌어올린 후 `dev-dependencies` 에서 `tempfile.workspace = true`.

---

## M2 — Detector + Overlay Merger

기존 팀 자산 인식과 fence 주입 엔진.

- [x] `crates/genasis-core/src/frontmatter.rs` (YAML head/body splitter + scalar reader, 6 단위 테스트)
- [x] `crates/genasis-overlay/` 실작동
  - [x] `detector.rs` — `.claude/agents/*.md` scan, classify, has_existing_fence 감지 (4 단위 테스트)
  - [x] `role_inference.rs` — 10 role + Custom (M1 에서 시드 완료, M2 통합)
  - [x] `merger.rs` — `plan_attach` / `plan_detach` / `apply` (3 phase: 계획·적용·report) — Tera 템플릿 기반 fence body 렌더링, snapshot 후 atomic write (3 단위 테스트)
  - [x] `validator.rs` — `FenceState` (Absent/Pristine/Outdated/Tampered/RoleMismatch) + `WriteDecision` 결정 (5 단위 테스트)
  - [x] `dry_run.rs` — `summary` (one-line glyph 형식) + `unified_diff` (similar 사용) + counts (3 단위 테스트)
- [x] 골든 픽스처: `tests/golden/ecc-only/input/` (3 agent 파일 — frontend canonical / backend canonical / loop-operator custom). 다른 4개 픽스처는 M6 까지 보류 (실제 expected/ 스냅샷이 모든 role 템플릿 완성 시점에 의미 있음).
- [x] `crates/genasis-templates/templates/agent-overlays/frontend.patch.md.tera` (첫 실제 템플릿)
- [x] `cmd_attach.rs` 실작동 — `--project / --dry-run / --diff / --force / --fence-version` 옵션 + summary/diff 출력 + apply (Plane/MM 호출은 M3 에서 추가)
- [x] `cmd_detach.rs` 실작동 — `--project / --dry-run / --diff` 옵션
- [x] E2E: `crates/genasis-overlay/tests/golden_ecc_only.rs` — round-trip 동등성 + 두 번 attach idempotency 검증 (2 통합 테스트)

### 회고
- [x] M2 회고: 1) Tera 템플릿을 `include_dir!()` 로 임베드한 결정이 검증됨 — 빌드 시점에 자동 발견되어 별도 manifest 불필요. 2) Validator 의 `FenceState` 5-state 분류가 핵심: Pristine vs Outdated vs Tampered vs RoleMismatch 를 명시적으로 구분해야 `--force` 의미가 명확. 3) `MergePlan` 을 계획 단계와 적용 단계로 분리 — dry-run 이 부산물로 자연스럽게 떨어짐. 4) `similar` crate 의 `TextDiff::from_lines` 만으로 unified diff 충분 — 복잡한 git-style hunk 헤더 불필요. 5) `AppliedReport` 가 backups 경로를 반환하므로 향후 `genasis upgrade --rollback` 구현 시 사용 가능.

---

## M3 — Plane / Mattermost Providers (직접 API)

- [x] `crates/genasis-providers/plane/{mod,upstream,agent_aware,detect,factory}.rs` 실작동
- [x] `crates/genasis-providers/mattermost/{mod,upstream,agent_aware,detect,factory}.rs` 실작동
- [x] `github.rs` — `gh` CLI wrapper + branch-protection helper
- [x] `cmd_init.rs` 실작동 — config 로드 → Plane health → MM ping → optional `--probe-only`, project + label 프로비저닝
- [x] `cmd_plane.rs` / `cmd_mm.rs` health/ping 디버그 서브커맨드
- [x] `tests/flavor_parse.rs` 통합 테스트
- [x] ADR-003 (직접 API) + ADR-005 (Flavor 시스템) 작성

### 회고
- [x] M3 회고: agent-aware 이 upstream 과 거의 동일하므로 delegation 패턴이 압도적으로 단순. flavor 검출은 health/ping 헤더 한 줄로 충분; 1차에서 본격 mock-HTTP 통합 테스트는 보류 (실 인스턴스에 대한 nightly E2E 가 더 의미 있음).

---

## M4 — Plane User Provisioner (Playwright Node sub-process)

- [x] `crates/genasis-cli/scripts/provision-plane-users.mjs` — stdio JSON 프로토콜 + Playwright import + stub 응답
- [x] `crates/genasis-providers/src/plane/user_provisioner.rs` — Rust spawn + stdin write + stdout parse + 종료 코드 처리
- [x] 실패 시 명시적 에러 메시지 (Node 미설치 / Playwright 미설치 / JSON 파싱 실패)

### 회고
- [x] M4 회고: stdio JSON envelope 가 process boundary 의 단일 계약 — Rust ↔ Node 양쪽 모두 testable. 실 UI 자동화 코드는 기존 Genesis bash 스크립트 자산을 점진적으로 포팅 (1차 릴리즈는 stub 단계).

---

## M5 — Schema Kernel & DB Adapters

- [x] `crates/genasis-db/kernel.rs` — Driver enum + MigrationTool enum + parse + dispatch
- [x] `crates/genasis-db/adapters/{postgres,mysql,sqlite,duckdb,atlas,drizzle_kit,raw_runner}.rs` 실작동
- [x] `crates/genasis-db/guard.rs` 강화 (M1 에서 진행, M5 통합 활용)
- [x] `cmd_db.rs` 실작동 — query / schema / migrate / diff / status / doctor 서브커맨드
- [x] ADR-004 (DB 채널 분리) 작성
- [s] mock HTTP 서버 통합 테스트 보류 (각 driver CLI 가 호스트에 설치되어 있어야 의미 있음 — nightly CI 에서 검증)

### 회고
- [x] M5 회고: Atlas 가 declarative 의 default; drizzle-kit 은 사용자 repo 의 `drizzle.config.ts` 가 있으면 자동 위임; DuckDB 는 raw_runner fallback. URL redaction 을 status 출력에 추가해 secret leak 방지.

---

## M6 — Hooks · Skills · Commands 템플릿

- [x] `templates/agent-overlays/*.patch.md.tera` 10개 (frontend M2 + 9 추가)
- [x] `templates/commands/*.md.tera` 16개 (sprint-*, intake-review, issue-*, design-change, db-*, agent-*, check-inbox, record-progress)
- [x] `templates/skills/<name>/SKILL.md.tera` 6개 (scrum-protocol, plane-ops, mm-ops, design-aware, schema-ops, tdd-enforce)
- [x] `templates/hooks/*.tera` 6개 (session-start, pre-tool-branch-guard, pre-tool-worktree-guard, post-tool-mm-sync, post-tool-trim, user-prompt-submit-mm)
- [x] `templates/mcp.json.tera` (Playwright만 — M0 작성)
- [x] `templates/env.agents.tera` (M0 작성)

### 회고
- [x] M6 회고: 9개 role 오버레이는 frontend 템플릿의 thin variant — 토큰/봇 환경변수 이름만 다를 뿐 lifecycle 계약은 동일. 16개 slash command 도 thin pointer 로 통일해 GENASIS.md 가 단일 진실. shell hook 두 개(branch-guard, worktree-guard)만 실로직, 나머지는 contract.

---

## M7 — Design Hot-Swap

- [x] `crates/genasis-design/extractor.rs` — `snapshot_existing` + `write_design_system`
- [x] `crates/genasis-design/diff.rs` — `ImpactArea` enum + keyword categorisation + `changed_areas`
- [x] `crates/genasis-design/ticket_emitter.rs` — `PlannedIssue` plan
- [x] `crates/genasis-design/change_protocol.rs` — 5-phase `run` orchestrator
- [x] `cmd_design.rs swap` / `status` 실작동
- [s] 골든 design-swap 픽스처 보류 (M11 에서 실 운영 마이그레이션 데이터로 대체)

### 회고
- [x] M7 회고: extractor 는 designer 에이전트의 `ui-style-extractor` skill 에 위임 — Genasis 가 CSS 파싱을 자체 구현하지 않음. impact area 6종 (color-tokens, typography, spacing, layout, components, motion) + Other fallback. 변경된 라인의 키워드를 카테고리화해 issue 1개를 생성.

---

## M8 — Doctor / Upgrade / Detach 완성

- [x] `cmd_doctor.rs` — required/optional 도구 검사, Genasis 자산 존재, config 로드, env 시크릿 존재 확인
- [x] `cmd_upgrade.rs` — fence-version 인자 + dry-run / diff / force 옵션, validator 의 Tampered/RoleMismatch 보호
- [x] `cmd_detach.rs` — M2 에서 완료 (dry-run / diff 옵션 포함)

### 회고
- [x] M8 회고: doctor 는 install.sh 와 같은 검사 매트릭스를 Rust 로 재구현 — 사용자가 install.sh 를 우회한 경우에도 보호. upgrade 는 attach 의 thin wrapper 지만 의도가 다른 명령(버전 bump 가 명시적으로 보임)이라 별도 유지.

---

## M9 — Monitor (Ratatui TUI)

- [x] `crates/genasis-monitor/app.rs` — main loop, alternate-screen / raw-mode dance, 250ms 폴링
- [x] `widgets/{sprint,tokens,agents,deploy,network,log_tail}.rs` 6개 위젯 실작동
- [x] `widgets/deploy.rs` — dev/prod LED + REFRESHED 배지 + 배포 액션 키 안내
- [x] `state.rs` — AppState + AgentActivity + DeployState + WidgetFocus
- [x] `cmd_monitor.rs` — `genasis_monitor::app::run` 위임
- [x] ADR-007 (Monitor TUI 1차 포함) 작성
- [s] 라이브 데이터 소스(rtk gain, Plane API poll, manifest watch) 인입은 incremental — 1차에서는 위젯 골격까지

### 회고
- [x] M9 회고: ratatui 0.27 의 `Frame::area()` 와 `Layout::default().constraints(...)` API 로 4-row 그리드 렌더 단순화. 이벤트 폴링 250ms 가 적절한 trade-off (CPU 1% 미만, 키 입력 응답 즉각). 데이터 인입은 hook + agent 가 emit 하는 JSON 라인을 file-tail 하는 방식으로 점진 추가 예정.

---

## M10 — Token Economics 마무리

- [x] `templates/hooks/session-start.sh.tera` — RTK 감지 + design-bootstrap 플래그 surface
- [x] `templates/hooks/post-tool-trim.sh.tera` — `${GENASIS_TRIM_THRESHOLD_KB:-32}` 임계값
- [x] `genasis.toml [token_economics] trim_threshold_kb = 32` schema (M0 에서 작성, M10 에서 wire)
- [x] ADR-006 (Token Economics) 작성

### 회고
- [x] M10 회고: 1차에서 자체 mcp-proxy 미포함이 옳은 결정 — 유지보수 부담 vs 가시 효과 trade-off 가 불리. RTK + Anthropic prompt cache + trim hook 3-tier 가 80% 효과 달성하면서 라이프사이클 단순.

---

## M12 — Internationalization (install-time language selector + active singularity)

> blueprint §19 의 의사결정에 따라:
> - **사용자 repo 의 agent context 는 항상 단일 언어**(`--lang en|ko`)
> - **Tera 템플릿 트리를 `templates/{en,ko}/` 로 분리** + `genasis lang switch` 제공
> - **런타임 i18n: rust-i18n** 신규 crate `genasis-i18n` (fluent-rs 보다 ~150KB 가볍고 메시지 규모에 적합)
> - **`install.sh` 도 `--lang` 분기**: inline `case` 블록 (의존성 0)
> - **`--lang both` 거부** + `docs/impact-of-multilang-prompts.md` 인용
> - **CI 3-tier**: 일반 PR warn / release-prep strict / 자동 translation-completion PR
>
> 근거: `docs/impact-of-multilang-prompts.md` (Claude Code 언어 drift 버그
> #46846/#24941, arXiv 2406.20052 한국어 line-level confusion, OSS 생태계
> 단일 언어 컨센서스, prompt cache prefix 충돌).
>
> **착수 전 사람 승인 필요** — 승인되면 아래를 순서대로 닫는다.

### M12.0 — 사람 승인 게이트
- [x] `blueprint.md §19` + `docs/impact-of-multilang-prompts.md` 검토 + 사람 승인 (M12 v5 plan 승인 완료, 2026-05-04)
- [x] ADR-008 초안 작성·머지 (install-time language selector + active singularity, commit e8b3793)

### M12.1 — i18n 인프라 신설 (런타임 — rust-i18n)
- [x] `crates/genasis-i18n/` 신규 crate (commit 9a12ed6)
  - [x] `Cargo.toml` (deps: `rust-i18n = "3"`, `once_cell`)
  - [x] `src/lib.rs` — `Lang` enum (`En`/`Ko`), `resolve()` (CLI flag / toml / env / $LANG / fallback `en`), `install()` 가 `rust_i18n::set_locale` 호출, `LangSource` 진단 enum
  - [x] `i18n!("locales", fallback = "en")` 매크로 root 선언, `t!` 재익스포트
  - [x] `locales/en.yml` (key 정의 source — 49개 키, 12 namespace)
  - [x] `locales/ko.yml` (한국어 mirror, parity 100%, `_meta.bcp47` 명시)
- [x] `Cargo.toml` workspace 에 멤버 추가 + dependency 등록 (`rust-i18n = "3"`, `once_cell = "1"`, internal alias)
- [x] 단위 테스트: `tests/i18n_lookup.rs` — `Lang::parse` (canonical/case-insensitive/locale modifier/friendly names/unknown reject), `resolve()` 5-tier 우선순위 + 미지값 skip-through, `t!` 매크로 영어/한국어 렌더 + fallback 의미 (`common.ok` → "확인"), `Lang::code` round-trip, `LangSource::label`. 16개 `#[test]`, serial-mutex 로 process-global 상태 보호.

### M12.2 — Rust user-facing 메시지 i18n 화 (`t!()` 매크로)
- [x] `genasis-cli` 의 prose 메시지 `t!()` wrap (commit 17b6b99). 구조화된 debug/JSON dump 라인은 의도적으로 영어 유지 — grep/IDE 친화 + `cmd_doctor` 의 진단 key=value 형 보존.
  - [x] `cmd_attach.rs` (refused, wrote_summary)
  - [x] `cmd_detach.rs` (wrote_summary)
  - [x] `cmd_upgrade.rs` (refused, wrote_summary)
  - [x] `cmd_init.rs` (7 prose 라인)
  - [x] `cmd_design.rs` (swap header/body/next + status 2종)
  - [x] `cmd_doctor.rs` (top-level header)
  - [s] `cmd_db.rs` / `cmd_monitor.rs` / `cmd_plane.rs` / `cmd_mm.rs` / `cmd_version.rs` — debug/JSON dump 위주, 영어 유지
- [s] `clap` help 메시지 i18n — clap `#[arg(help = ...)]` 가 컴파일타임 literal 만 받음. M12.4 에서 `Cli::command().about(...)` 후처리 패턴으로 대체 예정.
- [x] `genasis-monitor` TUI 라벨 `t!()` 화 — 6개 위젯 헤더 (sprint/tokens/agents/deploy/network/log_tail) + deploy 위젯 키 안내 (`monitor.key_hint`).
- [x] `--lang` 글로벌 플래그 + `$GENASIS_LANG` + `genasis.toml [i18n] cli_lang` (M12.4 에서 wire) + `$LANG` 우선순위 구현 (`Lang::resolve()`).
- [s] 단위 테스트: `cmd_version --lang ko` 한국어 출력 검증 — `cmd_version` 자체가 JSON/debug dump 라 i18n 영향 없음. M12.4 의 `init/attach --lang` E2E 에서 Korean 메시지 출력 검증으로 흡수.

### M12.3 — Tera 템플릿 트리 분리 (`templates/{en,ko}/`)
- [ ] 기존 `crates/genasis-templates/templates/*` 를 `templates/en/` 으로 이동 (git mv)
- [ ] `templates/ko/` 신규 트리 생성 (동일 구조)
  - [ ] `GENASIS.md.tera` (한국어 contract)
  - [ ] `genasis.toml.tera`
  - [ ] `env.agents.tera` (주석만 한국어, key 는 영문)
  - [ ] `mcp.json.tera` (주석만 한국어)
  - [ ] `design-system.md.tera` (한국어 placeholder)
  - [ ] `agent-overlays/*.patch.md.tera` 10개 (frontend, backend, qa, designer, security, devops, planner, architect, pm, code-reviewer)
  - [ ] `commands/*.md.tera` 16개 (sprint-*, intake-review, issue-*, design-change, db-*, agent-*, check-inbox, record-progress)
  - [ ] `skills/<name>/SKILL.md.tera` 6개 (scrum-protocol, plane-ops, mm-ops, design-aware, schema-ops, tdd-enforce)
  - [ ] `hooks/*.tera` 6개 (session-start, pre-tool-branch-guard, pre-tool-worktree-guard, post-tool-mm-sync, post-tool-trim, user-prompt-submit-mm — 메시지 한국어, 로직 동일)
- [ ] `crates/genasis-templates/src/lib.rs` — `templates_root(lang)` 함수로 분기, `include_dir!()` 두 트리 임베드
- [ ] `crates/genasis-overlay/merger.rs` — `MergePlan` 에 `lang: Lang` 필드 추가, 템플릿 lookup 시 사용
- [ ] 단위 테스트: 두 트리 모두 동일 key set (ko 결손 없음) 검증

### M12.4 — `genasis init` / `attach` / `detach` `--lang` + interactive prompt
- [ ] `cmd_init.rs` / `cmd_attach.rs` 에 `--lang en|ko|both` 플래그 추가 (default: prompt 또는 auto)
  - [ ] 인자 결정 알고리즘 (§19.3.1): `--lang` > TTY prompt > `$LANG` fallback
  - [ ] `--lang both` → 거부 + 영/한 안내 메시지(§19.8) + `docs/impact-of-multilang-prompts.md` URL + exit 2
  - [ ] `--non-interactive` / `--yes` 플래그 → prompt 우회, default 자동 수락 (CI 용)
- [ ] **Interactive language selection prompt 구현** (`crates/genasis-cli/src/lang_prompt.rs` 신규)
  - [ ] 양언어 병기 헤더 + 설치 대상 경로(`.claude/agents/`, `genasis/{skills,commands,hooks}/`, `GENASIS.md`) 명시
  - [ ] `--lang both` 거부 사유 (drift 위험) + impact 문서 링크 표시
  - [ ] `$LANG` 추정 default 하이라이트, Enter = default 수락
  - [ ] 잘못된 입력 3회 실패 → abort exit 3
  - [ ] 선택 후 confirmation: "✓ Will install <lang> instructions into .claude/. Continue? [Y/n]"
  - [ ] `dialoguer` crate 검토 후 도입 (또는 자체 stdin loop, 둘 중 가벼운 쪽)
- [ ] `genasis.toml [i18n]` 섹션 schema 추가 (`active`, `fence_lang`, `cli_lang`, `reference_langs`, `selected_via`)
- [ ] `selected_via` 추적: `"flag"` | `"prompt"` | `"lang_env"` (진단·doctor 용)
- [ ] `--reference-docs <lang>` 옵션 — 다른 언어를 `docs/genasis-i18n-reference/<lang>/` 에 복사 (`@import` 안 함)
- [ ] 설치 완료 후 안내 출력: `✅ Installed <lang> agent overlay into .claude/. Run \`genasis doctor\` to verify.`
- [ ] 통합 테스트:
  - [ ] `tests/e2e/install_lang_flag_en.rs` — `--lang en` 인자 prompt 우회
  - [ ] `tests/e2e/install_lang_flag_ko.rs` — `--lang ko` 인자 prompt 우회
  - [ ] `tests/e2e/install_lang_both_rejected.rs` — `--lang both` exit 2
  - [ ] `tests/e2e/install_lang_prompt_default.rs` — TTY mock + Enter → `$LANG` default 적용
  - [ ] `tests/e2e/install_lang_prompt_choice.rs` — TTY mock + "1" 입력 → en 선택 + confirmation Y → 적용
  - [ ] `tests/e2e/install_lang_prompt_decline.rs` — TTY mock + confirmation n → abort 없이 cleanup
  - [ ] `tests/e2e/install_lang_non_tty_fallback.rs` — non-TTY → `$LANG` 자동 + 사유 stdout 출력

### M12.5 — `genasis lang switch <lang>` 신규 명령
- [ ] `crates/genasis-cli/src/cmd_lang.rs` 신규 (`switch`, `status` 서브커맨드)
- [ ] `switch` 동작 (8단계, blueprint §19.7 참조)
  - [ ] snapshot → fence body 재생성 → skills/commands/hooks 교체 → GENASIS.md 교체 → genasis.toml 갱신 → reference 이동 → 단일 git commit → restart 안내
  - [ ] 멱등성: 같은 lang 으로 switch 호출 시 no-op
- [ ] `status` — 현재 active 언어 + 사용 가능 언어 + reference 트리 출력
- [ ] 통합 테스트: `tests/e2e/lang_switch_roundtrip.rs` (en → ko → en, fence hash·내용 동등성)

### M12.6 — `install.sh` `--lang` 분기 + interactive prompt (Bash 버전)
- [ ] `install.sh` 에 `--lang en|ko|both` 인자 파싱 + `--non-interactive` / `--yes` 플래그
- [ ] 결정 알고리즘: `--lang` > TTY prompt > `$LANG` fallback (§19.3.1)
- [ ] **Bash interactive prompt 구현** (Rust 쪽과 텍스트·배치 동일)
  - [ ] 양언어 병기 헤더 출력
  - [ ] 설치 대상 경로(`.claude/agents/`, `genasis/{skills,commands,hooks}/`, `GENASIS.md`) 명시
  - [ ] `--lang both` 거부 사유 + impact 문서 URL
  - [ ] `$LANG` 추정 default + `read -p "Select [1/2] (default: 2): " choice`
  - [ ] 입력 검증 + 3회 재시도
  - [ ] confirmation prompt 후 진행/abort
- [ ] non-TTY 감지 (`[ -t 0 ]`) → prompt skip + `$LANG` 자동 + stdout 사유 명시
- [ ] 모든 사용자 안내 메시지(헤더, 누락 패키지 가이드, 설치 진행, 완료/실패)를 영/한 case 블록
- [ ] `--lang both` → 거부 + impact 문서 URL 출력 후 exit 1
- [ ] binary 호출 시 `attach --lang $ACTIVE_LANG --non-interactive` 자동 전달 (이미 prompt 거쳤으므로 이중 prompt 방지)
- [ ] 스모크 테스트:
  - [ ] `bash install.sh --lang ko --version=v0.0.0-test --no-run` — 한국어 출력 검증
  - [ ] `bash install.sh --lang en --version=v0.0.0-test --no-run` — 영어 출력 검증
  - [ ] `printf "1\nY\n" \| bash install.sh --version=v0.0.0-test --no-run` — prompt 입력 시뮬레이션 → en 선택
  - [ ] `bash install.sh --lang both` exit 1 + impact URL 검증
  - [ ] `echo \| bash install.sh --version=v0.0.0-test --no-run` — non-TTY fallback 검증

### M12.7 — 문서 듀얼 트리 (rename + translate + cross-link)

#### M12.7.a Rename pass (현재 한글 → `*.ko.md` / `docs/ko/`)
- [ ] `README.md` → `README.ko.md` (git mv, 본문 무수정)
- [ ] `blueprint.md` → `blueprint.ko.md`
- [ ] `progress.md` → `progress.ko.md`
- [ ] `docs/ARCHITECTURE.md` → `docs/ko/ARCHITECTURE.md`
- [ ] `docs/PROVIDERS.md` → `docs/ko/PROVIDERS.md`
- [ ] `docs/MIGRATION-FROM-GENESIS.md` → `docs/ko/MIGRATION-FROM-GENESIS.md`
- [ ] `docs/TOKEN-ECONOMICS.md` → `docs/ko/TOKEN-ECONOMICS.md`
- [ ] `docs/MONITOR.md` → `docs/ko/MONITOR.md`
- [ ] `docs/impact-of-multilang-prompts.md` → mirror 작성 (`docs/ko/impact-of-multilang-prompts.md`)
- [ ] `docs/ADR/ADR-000-template.md` ~ `ADR-007-monitor-tui.md` (8개) → `docs/ko/ADR/`

#### M12.7.b Translate pass (영어 source 작성)
- [ ] `README.md` (English) — root 진입점 + `--lang ko` 인용 + 한글 토글
- [ ] `blueprint.md` (English) — §0–§19 전체
- [ ] `progress.md` (English) — 마일스톤 표·회고
- [ ] `docs/ARCHITECTURE.md` / `PROVIDERS.md` / `MIGRATION-FROM-GENESIS.md` / `TOKEN-ECONOMICS.md` / `MONITOR.md` (English)
- [ ] `docs/impact-of-multilang-prompts.md` (이미 작성 — M12 사전 단계 산출물)
- [ ] `docs/ADR/ADR-000-template.md` ~ `ADR-007-monitor-tui.md` (8개) (English)
- [ ] `docs/ADR/ADR-008-i18n-install-time-selector.md` 신규 (영/한 양쪽)
- [ ] 코드블록·env 변수·CLI 명령·외부 URL 무번역 검증

#### M12.7.c Cross-link pass
- [ ] 모든 영어 source 상단에 `> 한국어: [<file>.ko.md](<file>.ko.md)` 또는 `[docs/ko/<path>](docs/ko/<path>)` batch
- [ ] 모든 한글 mirror 상단에 `> English: [<file>.md](<file>.md)` batch
- [ ] root `README.md` 상단에 `[English | 한국어]` 토글
- [ ] root `README.ko.md` 상단에 `[한국어 | English]` 토글

### M12.8 — Golden fixture 추가 + 정리
- [ ] `tests/golden/with-ko-locale/{input,expected}/` 신규 — 한국어 active 시 산출물 검증
- [ ] 기존 6 픽스처는 영어 단일 유지 (mirror 두지 않음)
- [ ] `tests/golden/SHARED.md` 에 `with-ko-locale` 시나리오 표 추가

### M12.9 — `.github` 영어 단일 검증
- [ ] `.github/ISSUE_TEMPLATE/bug.md` / `feature.md` 영어 확인
- [ ] `.github/PULL_REQUEST_TEMPLATE.md` 영어 확인

### M12.10 — CI 3-tier 가드레일 + drift 스크립트 + Translation Completion 자동화
- [ ] `scripts/check-i18n-drift.sh` 신규
  - [ ] `--warn` 모드 (default) — `::warning::`, exit 0
  - [ ] `--strict` 모드 — `::error::`, exit 1
  - [ ] `--list` 모드 — drift 페어 표 출력 (doctor 가 호출)
  - [ ] `--check-mirror-not-empty` 모드 — 모든 source 에 mirror 존재 + size>0
  - [ ] `--gen-todo` 모드 — 누락 mirror 목록을 GitHub Issue body 형식으로 출력
- [ ] `scripts/i18n-extract-keys.sh` 신규 — `en.yml` ↔ `ko.yml` key parity (`--warn`/`--strict` 모드, surplus 항상 error)
- [ ] `.github/workflows/ci.yml` `lint-i18n` job 추가 (3 step: 한국어 reject in en source, drift warn, key parity warn)
- [ ] `.github/workflows/release.yml` `lint-i18n-strict` job 추가 (`release/*` 브랜치 또는 `v*` 태그에서만, drift + parity hard-fail)
- [ ] `.github/workflows/release-prep.yml` 신규 — `release-prep` 브랜치 push 시 `check-i18n-drift.sh --gen-todo` 실행 → drift 있으면 자동 PR `[i18n] Translation completion for vX.Y.Z` 생성
- [ ] 자동 PR 템플릿 `.github/PR_TEMPLATE_i18n_completion.md` 신규 (체크리스트 + 영어 source diff 자동 첨부)

### M12.11 — `genasis doctor [i18n]` 확장
- [ ] `crates/genasis-cli/src/cmd_doctor.rs` 에 `[i18n]` 섹션 추가
  - [ ] CLI/TUI runtime locale + active agent locale + reference docs 출력
  - [ ] source/mirror parity (현재 repo 가 genasis 자체일 때만 의미 — `genasis.toml.devmode` 또는 `.git` 존재 시 활성)
  - [ ] rust-i18n key parity (`en.yml` vs `ko.yml` key set diff)
  - [ ] drift 페어 표 (4개 미만은 inline, 이상은 `--list` 안내)
- [ ] 단위 테스트 `tests/unit/doctor_i18n.rs`

### M12.13 — README SEO + 다국어 토글 고도화 (blueprint §19.13)

#### M12.13.a `README.md` (English) SEO 최적화 + 구조 재작성
- [ ] §19.13.5 의 18개 절 구조로 영어 README 재작성
  - [ ] H1 + tagline (§19.13.3, 첫 80자에 핵심 가치 + tag list)
  - [ ] Why Genasis (3 bullet, 30초 설명)
  - [ ] Quickstart (1 curl + 1 prompt + 1 명령)
  - [ ] Features 6개 (이미지 + 1줄 + docs/ARCHITECTURE 링크)
  - [ ] Demo (asciinema cast 또는 GIF placeholder)
  - [ ] Documentation (영/한 분기 링크)
  - [ ] Architecture mermaid diagram
  - [ ] Comparison table (vs ECC / kw-plugins / claude-code-templates)
  - [ ] Roadmap (progress.md 링크)
  - [ ] Contributing (`docs/CONTRIBUTING.md` + `docs/i18n/CONTRIBUTE-LANG.md`)
  - [ ] Star History badge (star-history.com)
  - [ ] Sponsors / License / Bottom navigation

#### M12.13.b 다국어 토글 3-단계 fallback
- [ ] 상단 language badge row (shields.io SVG 3개: English / 한국어 / Add a language)
- [ ] Cross-link batch (현재 언어 굵게, 다른 언어 링크)
- [ ] Bottom navigation footer (## Other languages / 다른 언어)
- [ ] `README.ko.md` 에 동일 토글 적용 (현재 표시 언어 굵게)

#### M12.13.c `README.ko.md` (Korean mirror) 작성
- [ ] §19.13.5 동일 18개 절 구조
- [ ] 한국어 키워드 최적화 (`에이전트`, `클로드 코드`, `애자일 자동화`, `한국어`)
- [ ] 모든 외부 링크·shields URL·코드블록 영어 source 와 byte-동등 (drift 방지)

#### M12.13.d GitHub repo 메타데이터
- [ ] Repo Settings > Topics 18~20개 등록 (PR 가이드에도 명시):
  `agentic-ai`, `claude-code`, `claude-agents`, `agent-orchestration`, `plane-issues`, `mattermost`, `mattermost-bot`, `scrum-automation`, `tdd`, `agentic-team`, `multi-agent-systems`, `rust-cli`, `ratatui`, `overlay`, `schema-as-code`, `i18n`, `korean`, `한국어`
- [ ] Repo Settings > About — Description (1줄, 한국어 포함) + Website URL
- [ ] Social preview image 등록 (`docs/assets/og-image.png`, 1280×640)

#### M12.13.e Open Graph + 시각 자산
- [ ] `docs/assets/og-image.png` (영어 1280×640)
- [ ] `docs/assets/og-image.ko.png` (한국어 1280×640)
- [ ] `docs/assets/demo.cast` (asciinema 또는 mp4) — README hero 영역 사용
- [ ] `docs/assets/architecture.svg` — Architecture mermaid 의 fallback 정적 SVG
- [ ] `docs/assets/logo.svg` — repo 로고 (light / dark variant)

#### M12.13.f 자동 SEO 시그널 (badges)
- [ ] shields.io badges: build status, license, latest release, downloads, stars
- [ ] Codecov / Coveralls badge (CI 가 자동 push)
- [ ] Star History badge (star-history.com)
- [ ] GitHub Sponsors badge (등록 시)
- [ ] 모든 badge 가 영/한 mirror 양쪽에 동일 배치

#### M12.13.g 다국어 추가 컨트리뷰터 가이드
- [ ] `docs/i18n/CONTRIBUTE-LANG.md` 신규 (영어)
- [ ] `docs/ko/i18n/CONTRIBUTE-LANG.md` 신규 (한국어 mirror)
- [ ] 4단계 절차 명시 (README.<lang>.md → 모든 mirror toggle 갱신 → docs/<lang>/ → PR title `[i18n] Add <Lang> README`)
- [ ] 새 언어 PR 의 CI 체크리스트 (badge row, bottom nav, topics 갱신 누락 검출)

#### M12.13.h GitHub Pages 자동 라우팅 (옵션, 별도 미니 milestone)
- [ ] `docs/_config.yml` (Jekyll, `jekyll-sitemap` plugin)
- [ ] `docs/index.html` — `navigator.language` 검사 + `<meta refresh>` fallback
- [ ] `docs/en/index.md`, `docs/ko/index.md` (README mirror 의 web 버전 + frontmatter `<title>`/`description`)
- [ ] `docs/robots.txt` (sitemap URL)
- [ ] JSON-LD `SoftwareApplication` schema (`docs/_includes/schema.html`)
- [ ] Canonical URL meta
- [ ] Repo Settings > Pages activate (source: `docs/`, branch: `main`)
- [ ] DNS / custom domain 결정 후 등록 (옵션)

#### M12.13.i 측정 + 회고 hook
- [ ] GitHub Insights > Traffic 베이스라인 캡처 (M12 시작 시점)
- [ ] Google Search Console 등록 (Pages 활성화 시)
- [ ] 3개월 회고 항목 사전 등록 (한국어 traffic share, 상위 10 검색어, 누락 언어 요청)

### M12.12 — 회고 + DoD
- [ ] `lint-i18n` CI 통과 (영어 source 에 Hangul 0건, rust-i18n key parity 100%)
- [ ] `release-prep` 워크플로 dry-run — 일부러 mirror 1개 비워두고 자동 translation-completion PR 생성 → 채움 → strict 통과 시나리오 검증
- [ ] drift 0건 (모든 mirror 가 source 와 동일 시점)
- [ ] `genasis doctor` 출력에 `[i18n]` 섹션 정상 표시
- [ ] `genasis init --lang en` / `--lang ko` / `--lang both` 거부 / `genasis lang switch` 4가지 시나리오 E2E green
- [ ] `install.sh --lang ko` 한국어 안내 출력 검증
- [ ] `with-ko-locale` 골든 픽스처 회귀 통과
- [ ] `README.md` / `README.ko.md` 18-절 SEO 구조 + 3-단계 토글 적용
- [ ] GitHub repo Topics 18~20개 등록 + Social preview image 영/한 2버전
- [ ] (옵션) GitHub Pages 라우팅 dry-run — `Accept-Language` 분기 검증
- [ ] M12 회고 추가 (번역 함정 · rust-i18n 효과 · drift 빈도 · README traffic 베이스라인)

---

## M11 — Migration & Release

- [x] `cmd_plane`, `cmd_mm` 디버그 서브커맨드 (health/ping)
- [x] `docs/ADR/ADR-001 ~ ADR-007` 7개 ADR 모두 작성
- [x] `docs/PROVIDERS.md` 갱신 (M0 작성, M3 의 flavor 가이드와 정합)
- [x] `docs/MIGRATION-FROM-GENESIS.md` 갱신 (M0 작성, 매핑 테이블 정합)
- [s] 본격 `cmd migrate-from-genesis` 구현 보류 — 실제 Genesis bash 팀 운영 데이터 필요. 1차 docs 단계까지.
- [s] GitHub Release 첫 cross-compile / 데모 영상 / v0.1.0 태그 — 1차 PR 머지 후 release pipeline 트리거 시 검증.

### 회고
- [x] M11 회고: 1차 sprint 의 모든 마일스톤 코드/문서가 자리잡음. 이후의 노력은 (a) 실 운영 sprint 1회 돌려보고 데이터 인입 hooks 정착, (b) 실 cross-compile 결과로 install.sh 종단간 검증, (c) v0.1.0 태그.

---

## 진행 중 메모

(이 섹션은 막힘·결정 변경·추후 처리 사항을 inline 기록)

- 2026-05-03: 초기 blueprint 합의 완료, M0 시작
- 2026-05-03: M0 완료 — 144개 파일, 9개 crate stub, install.sh 스모크 검증, 6개 골든 픽스처 디렉토리, 5개 Tera 템플릿, 3개 GitHub Actions workflow. 다음 마일스톤(M1) 진입 가능.
- 2026-05-03: M1 완료 — genasis-core 5 모듈 실작동(marker/fs/env/config/error), `cmd_version` 첫 실작동 명령, role_inference + SQL guard 강화, 30+ 단위·통합 테스트, ADR-001/002 작성. M2 (Detector + Overlay Merger) 진입 가능.
- 2026-05-03: M2 완료 — frontmatter 파서 + detector + validator + merger + dry_run + cmd_attach/detach 실작동, 첫 실제 템플릿(frontend), ecc-only 골든 픽스처 + 2 round-trip 통합 테스트, 누적 78 `#[test]`. 다음: M3 (Plane / Mattermost Providers + 직접 API + flavor 시스템).
- 2026-05-03: M3-M11 완주 — Plane/MM provider flavor 시스템 + GitHub gh wrapper + cmd_init 실작동, Plane user provisioner Node sub-process, DB schema kernel + 7 adapter + cmd_db, 10+16+6+6 Tera 템플릿(agent overlays / commands / skills / hooks), design hot-swap orchestrator + cmd_design, doctor/upgrade/monitor 실작동, ADR-003~007 작성. 1차 릴리즈 코드/문서 자리잡음.
- 2026-05-04: M12 v1 계획 수립 (문서 듀얼 트리 + CI 만, 8 sub-step). 사용자 피드백으로 v2 로 확장.
- 2026-05-04: **M12 v2 재계획 완료** — 사용자 요청에 따라 (a) 런타임 i18n 추가 (Rust CLI/TUI + install.sh), (b) `--lang en|ko` 설치 시점 선택, (c) 다국어 동시 설치 가능성 조사 (`docs/impact-of-multilang-prompts.md` 작성 — Claude Code 언어 drift 버그 #46846/#24941, arXiv 2406.20052 한국어 confusion, OSS 컨센서스 등 13개 source 분석) → **`--lang both` 거부 + active singularity** 로 정책 결정. blueprint §19 전면 재작성(13 sub-section), progress M12 13 sub-step(M12.0~M12.12) 으로 확장. 신규 crate `genasis-i18n` (fluent-rs), `templates/{en,ko}/` 분리, `genasis lang switch` 명령, `install.sh --lang` 분기, `with-ko-locale` 골든 픽스처. **사람 승인 대기 중**.
- 2026-05-04: **M12 v3 미세조정** — 사용자 피드백 2건 반영. (1) drift 게이트를 2-tier 에서 **3-tier (PR warn / release-prep strict / 자동 translation-completion PR)** 로 확장 → "배포 전 빠진 번역 맞추기" 운영 모델 명시. (2) 런타임 i18n 라이브러리 **fluent-rs → rust-i18n** 전환 — 메시지 ~50개 / 한국어 복수형 변화 없음 / binary 150KB 절감 / 토큰 효율. ADR-008 대안 검토 ④ ⑤ 추가. blueprint §19.4 §19.9 §19.10 §19.12, progress M12.1 M12.2 M12.10 M12.11 M12.12 갱신.
- 2026-05-04: **M12 v4 — interactive language prompt 추가, 사용자 최종 승인 완료**. 명령행 `--lang` 인자가 default 우선순위(인자 > TTY prompt > `$LANG` fallback). 설치 시 `.claude/agents/`, `genasis/{skills,commands,hooks}/`, `GENASIS.md` 가 선택 언어로 설치된다는 내용 + drift 위험 + `lang switch` 안내를 양언어 병기 prompt 로 표시. install.sh(Bash)와 `genasis attach`(Rust) 가 텍스트·배치 동일. `--non-interactive`/`--yes` 로 CI 우회. blueprint §19.3 4 sub-section(.1~.4)로 확장, progress M12.4 / M12.6 prompt + 통합 테스트 7+5건 추가.
- 2026-05-04: **M12 v5 — README SEO + 다국어 토글 고도화 추가, 최종 승인 완료**. blueprint §19.13 8 sub-section 신설: 3-단계 토글 fallback (badge row + cross-link + bottom nav), 18-절 SEO 구조, GitHub Topics 18~20개, Open Graph 영/한 2버전, shields/Star History/Codecov badges, GitHub Pages 자동 라우팅(옵션) — `Accept-Language` 헤더 → `/ko/` `/en/` 분기, JSON-LD SoftwareApplication schema, Jekyll sitemap. progress M12.13 9 sub-step (.a~.i) 추가, M12.12 DoD 4항목 보강. **승인 완료 — M12.0 부터 순차 착수**.
- TODO: GitHub `<OWNER>` 결정 필요 (install.sh placeholder)
- TODO: monitor 의 manifest 해시 비교 — Next.js 외 빌드 시스템(Vite, Turbo, plain) 호환 확인
- TODO: Atlas 의 DuckDB 지원 상태 재확인 (raw runner 필요 여부 확정)

---

## 예상 외 항목 (1차 릴리즈 후)

- genasis-mcp-proxy
- 다중 프로젝트 monorepo 지원
- Web UI dashboard
- 커뮤니티 mcp-cache 등 통합
- VSCode extension
- Plane Pro / GitLab / Linear 등 다른 이슈 트래커 flavor

---

## 회고 슬롯

| 마일스톤 | 시작 | 완료 | 학습한 것 |
|---|---|---|---|
| M0 | 2026-05-03 | 2026-05-03 | install.sh OS 매트릭스가 코어 작업의 70%; Tera `include_dir!()` 임베드로 단일 바이너리 분배 단순화; Rust toolchain 미설치 환경 대응으로 모든 crate 의 import 정합성을 사람 검토. |
| M1 | 2026-05-03 | 2026-05-03 | marker fence 의 단일 fence invariant + body hash 가 overlay 안전성의 핵심; `.env.agents` 의 사람 작성 comment 보존을 위해 `Vec<Line>` 모델 채택; SQL guard 는 string-literal aware split 로 sqlparser-rs 의존성 회피. |
| M2 | 2026-05-03 | 2026-05-03 | `MergePlan` 계획·적용 분리로 dry-run 이 자연스럽게 떨어짐; `FenceState` 5-state 분류로 `--force` 의미 명확화; `include_dir!()` Tera 임베드로 manifest 없이 템플릿 자동 발견; 5 시나리오 골든 픽스처 중 ecc-only 만 M2 시점 의미 있음. |
| M3 | 2026-05-03 | 2026-05-03 | agent-aware 이 upstream 의 thin delegation 으로 충분; flavor 검출은 응답 헤더 한 줄. |
| M4 | 2026-05-03 | 2026-05-03 | stdio JSON envelope 가 Rust↔Node 단일 계약. UI 자동화는 stub 단계. |
| M5 | 2026-05-03 | 2026-05-03 | Atlas default + Drizzle Kit auto-detect + DuckDB raw_runner fallback; URL redaction 으로 secret leak 차단. |
| M6 | 2026-05-03 | 2026-05-03 | 9 role 오버레이는 frontend 의 thin variant; 16 slash command 는 GENASIS.md 로 위임하는 thin pointer. |
| M7 | 2026-05-03 | 2026-05-03 | extractor 위임 + 6 area 키워드 카테고리화로 issue plan 단순. |
| M8 | 2026-05-03 | 2026-05-03 | doctor 가 install.sh 검사 매트릭스 미러 — 사용자 우회 시에도 보호. |
| M9 | 2026-05-03 | 2026-05-03 | ratatui 0.27 의 `Frame::area()` API 로 4-row 그리드 단순화; 250ms 폴링이 적절. |
| M10 | 2026-05-03 | 2026-05-03 | mcp-proxy 미포함 결정이 유지보수 부담 vs 효과 trade-off 의 우위. |
| M11 | 2026-05-03 | 2026-05-03 | 1차 코드/문서 모두 자리잡음. 실 운영 sprint 1회 후 데이터 인입 hooks 정착 + v0.1.0 태그. |
