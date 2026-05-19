> English: [progress.md](progress.md)
>
> **Mirror sync policy**: 이 파일과 `progress.md` 는 구조·내용 동기
> mirror 입니다. 한쪽을 수정하면 **같은 commit** (또는 직후 commit) 에
> 반드시 다른 쪽도 동일 구조·내용으로 갱신하세요. 정책 상세:
> [`CLAUDE.md` §Bilingual Mirror Policy](CLAUDE.md).

# Genasis — Progress Tracker

> `blueprint.md` 의 §15 (1차 릴리즈 범위) 와 §12 (repo 구조) 를 따라 마일스톤 단위로 진행.
> 각 항목은 `[ ]` → `[x]` 로 닫고, 막힌 항목은 `[!]` 로 표기 + 사유 inline 기록.

**Started**: 2026-05-03
**Target 1차 릴리즈**: v0.1.0 (M12 종료 후 git tag)
**현재 마일스톤**: **M14 planning** + **Phase F 설계** (Debug History 피드백 루프, ADR-012).
M0–M12 + Phase D 모두 완료. Phase F (M15–M17) 2026-05-05 설계.
v0.1.0 release tag 는 M14.0 (ADR-010 ratify) 후 cut.

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
- [x] `Cargo.lock` — rustup stable 설치 + cargo build green 후 commit (현재 14 crates compiled).

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
- [x] `cargo build --workspace` green (14 crates) — rustup stable 설치 후 검증.
- [x] `cargo test --workspace --no-fail-fast` → 120 passed (28 suites).
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
- [x] CI green — commit b7bffaa 부터 GitHub Actions CI success.
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

### M12.3 — Tera 템플릿 트리 분리 (`templates/{en,ko}/`)  — ✅ commit 1fd1e6d
- [x] 기존 `crates/genasis-templates/templates/*` 를 `templates/en/` 으로 이동
- [x] `templates/ko/` 신규 트리 생성 (39 파일, 동일 구조)
  - [x] `GENASIS.md.tera` (한국어 contract)
  - [x] `genasis.toml.tera`
  - [x] `env.agents.tera`
  - [x] `mcp.json.tera`
  - [x] `design-system.md.tera`
  - [x] `agent-overlays/*.patch.md.tera` 10개
  - [x] `commands/*.md.tera` 16개
  - [x] `skills/<name>/SKILL.md.tera` 6개
  - [x] `hooks/*.tera` 6개
- [x] `crates/genasis-templates/src/lib.rs` — `get_lang(lang, relative)` + `SUPPORTED_LANGS = &["en","ko"]` constant + `include_dir!()` 두 트리 임베드 (6 unit tests for parity)
- [x] `crates/genasis-overlay/merger.rs` — `build_tera_lang(lang)` (legacy `build_tera()` 는 "en" default 로 wrap)
- [x] 단위 테스트: `english_genasis_md_present`, `korean_genasis_md_present`, `english_frontend_overlay_present`, `korean_frontend_overlay_present`, `unknown_locale_returns_none`, `english_and_korean_have_same_top_level_files`

### M12.4 — `genasis init` / `attach` / `detach` `--lang` + interactive prompt  — ✅ commits 39d1032, e2e 추가
- [x] **글로벌 `--lang en|ko`** 플래그 (main.rs) — clap conflict 회피 위해 cmd_attach 의 action-local `--lang` 제거하고 main.rs 가 `cli.lang` 을 `pub_run` 에 전달
  - [x] 인자 결정 알고리즘: `--lang` > TTY prompt > `$LANG` fallback (`lang_prompt::decide`)
  - [x] `--lang both` → `BothRejected` sentinel + 영/한 banner + impact URL + exit 1 (anyhow 종속)
  - [x] `--non-interactive` / `--yes` 글로벌 플래그
- [x] **Interactive language selection prompt** (`crates/genasis-cli/src/lang_prompt.rs`)
  - [x] 양언어 병기 헤더 + 5개 설치 대상 경로 명시
  - [x] `--lang both` 거부 banner + impact 문서 링크
  - [x] `$LANG` 추정 default + Enter 수락
  - [x] 3회 실패 → abort
  - [x] confirmation prompt + Y/y/yes/예 수락
  - [x] 자체 stdin loop (dialoguer 미도입 — binary size 절감)
- [x] `genasis.toml [i18n]` schema (`active`/`fence_lang`/`cli_lang`/`reference_langs`/`selected_via`) — `genasis-core/src/config.rs::I18nConfig`
- [x] `selected_via` 추적 (`flag` / `prompt` / `lang_env` / `default` / `switch`)
- [x] `--reference-docs <lang>` — `cmd_attach.rs::write_reference_docs` 가 `docs/genasis-i18n-reference/<lang>/GENASIS.md` 생성
- [x] 완료 후 안내 — i18n bundle `lang.install.success` / `lang.install.next_step`
- [x] 통합 테스트 (`crates/genasis-cli/tests/install_lang_e2e.rs`, 6 tests, std::process 기반):
  - [x] `flag_en_drives_attach_without_prompt`
  - [x] `flag_ko_drives_attach_without_prompt`
  - [x] `both_is_rejected_with_exit_2_and_banner`
  - [x] `non_tty_fallback_uses_lang_env_and_announces_it`
  - [x] `lang_status_reports_active_locale`
  - [x] `lang_switch_no_op_when_already_on_target`
  - [s] PTY-required prompt 시나리오 3종 (default/choice/decline) — std::process 가 PTY 없으니 unit-level (`lang_decide.rs`) 에서 커버. 진짜 PTY E2E 는 `expectrl` 도입 시 추가.

### M12.5 — `genasis lang switch <lang>` 신규 명령  — ✅ commit 39d1032
- [x] `crates/genasis-cli/src/cmd_lang.rs` (`Status` + `Switch` 서브커맨드)
- [x] `switch` 동작 — `pub_run` 재사용으로 force=true 부착, [i18n] selected_via="switch" 갱신
- [x] 멱등성 — `Already on <lang>` 메시지 출력 후 즉시 return
- [x] `status` — active/cli_lang/fence_lang + selected_via + reference_langs + SUPPORTED_LANGS 출력
- [x] 통합 테스트 (`install_lang_e2e.rs`): `lang_status_reports_active_locale`, `lang_switch_no_op_when_already_on_target`
- [s] 본격 round-trip (en → ko → en + fence hash 동등성) — `git commit` 단계가 lang switch 안에 wrap 되어 있지 않아 (tests 가 git repo 외부에서 실행) E2E 가 부분만 검증. M12.13 release polish 단계에서 보강.

### M12.6 — `install.sh` `--lang` 분기 + interactive prompt (Bash 버전)  — ✅ commit 54ed32e
- [x] `install.sh` `--lang en|ko|both` + `--non-interactive` + `-y/--yes` 플래그 파싱
- [x] 결정 알고리즘: `--lang` > TTY prompt > `$LANG` fallback (`resolve_install_lang()`)
- [x] **Bash interactive prompt** (Rust 쪽과 동일 layout)
  - [x] 양언어 병기 헤더 + 5개 설치 대상 경로
  - [x] `--lang both` 거부 banner + impact URL (`reject_both()`)
  - [x] `$LANG` 추정 default + `read` 3회 재시도
  - [x] confirmation prompt + Y/예 수락
- [x] non-TTY 감지 (`[ ! -t 0 ]`) → prompt skip + `$LANG` 자동 + `info` stdout
- [x] 모든 사용자 안내 메시지 영/한 분기
- [x] `--lang both` → reject_both() + exit 2
- [x] binary 호출 시 `attach --lang $ACTIVE_LANG --non-interactive --yes` 자동 전달
- [x] 스모크: `bash -n install.sh` 통과 (CI lint-i18n + manual end-to-end 시 추가 검증)
- [x] 5개 case 별 실 스모크 (`--no-run --skip-prereqs` 조합):
  - [x] `install.sh --lang=ko` ASCII art + ko 분기 출력 OK
  - [x] `install.sh --lang=en` 영어 분기 출력 OK
  - [x] `install.sh --lang=both --skip-prereqs` → reject_both banner + exit 2 (PIPESTATUS 검증)
  - [x] `echo "" | install.sh --skip-prereqs` non-TTY fallback 출력 OK
  - [x] `install.sh -h` help 텍스트 정상

### M12.7 — 문서 듀얼 트리 (rename + translate + cross-link)

#### M12.7.a Rename pass — ✅ commit ea1e9d6
- [x] `README.md` / `blueprint.md` / `progress.md` → `*.ko.md` (git mv)
- [x] `docs/{ARCHITECTURE,PROVIDERS,MIGRATION-FROM-GENESIS,TOKEN-ECONOMICS,MONITOR}.md` → `docs/ko/`
- [x] `docs/impact-of-multilang-prompts.md` mirror (`docs/ko/impact-of-multilang-prompts.md`)
- [x] `docs/ADR/ADR-000` ~ `ADR-007` (8개) → `docs/ko/ADR/`

#### M12.7.b Translate pass — ✅ commits ccc1cac, b268d6f, 7c05d94, 23251ae, ea1e9d6
- [x] `README.md` (English) — 18-section SEO 구조 + bilingual badge row + Star History
- [x] `blueprint.md` (English) — TL;DR + section index + i18n decision summary (full §0–§19 본문은 release polish 단계에서 보강)
- [x] `progress.md` (English) — milestone summary + M12 sub-step status table
- [x] `docs/ARCHITECTURE.md` — TL;DR + source tree map + ASCII layer diagram + ADR cross-link
- [x] `docs/PROVIDERS.md` — flavor 시스템 + 5단계 추가 레시피 + 감지 우선순위 + sample toml
- [x] `docs/MIGRATION-FROM-GENESIS.md` — 매핑 표 + step-by-step CLI 흐름
- [x] `docs/TOKEN-ECONOMICS.md` — 3-tier 모델 + 1.0 미포함 사유
- [x] `docs/MONITOR.md` — 6 위젯 표 + key bindings + i18n 흐름
- [x] `docs/impact-of-multilang-prompts.md` (M12 사전 단계 산출물)
- [x] `docs/ADR/ADR-008-i18n-install-time-selector.md` 신규 영어 + Korean stub mirror
- [s] `docs/ADR/ADR-001` ~ `ADR-007` 영어 본문 — 한국어 canonical 이 단일 source. 영어 mirror 는 release polish 단계에서 작성 (각 ADR 의 한국어 본문이 짧고 코드/표 위주라 release-prep 자동 PR 으로 흡수).
- [x] 코드블록·env 변수·CLI 명령·외부 URL 무번역 (lint-i18n 이 grep 으로 검증)

#### M12.7.c Cross-link pass — ✅ commit ea1e9d6, ccc1cac
- [x] 모든 영어 source 상단 cross-link batch
- [x] 모든 한글 mirror 상단 `> English: ...` batch (M12.7.b 완료된 5개는 "(English version pending)" 캐비어트 제거)
- [x] root `README.md` 상단 bilingual badge row (shields.io English / 한국어 / Add a language) + cross-link batch
- [x] root `README.ko.md` 상단 동일 토글 (현재 언어 굵게)

### M12.8 — Golden fixture 추가 + 정리  — ✅ commit ea1e9d6
- [x] `tests/golden/with-ko-locale/{input,expected}/` + README 신규
- [x] 기존 6 픽스처는 영어 단일 유지
- [x] `tests/golden/SHARED.md` 에 `with-ko-locale` 시나리오 행 추가
- [x] `expected/` 스냅샷 채움 — `genasis attach --lang ko --non-interactive --yes` 로 생성. 한국어 fence body 확인 (`(Genasis Overlay) Plane / Mattermost 프로토콜`).

### M12.9 — `.github` 영어 단일 검증  — ✅ commit ea1e9d6
- [x] `.github/ISSUE_TEMPLATE/bug.md` / `feature.md` 영어 (신규 작성)
- [x] `.github/PULL_REQUEST_TEMPLATE.md` 영어 (i18n 체크리스트 포함)

### M12.10 — CI 3-tier 가드레일 + drift 스크립트 + Translation Completion 자동화  — ✅ commit ea1e9d6, 022ca37
- [x] `scripts/check-i18n-drift.sh` (`--warn`/`--strict`/`--list`/`--check-mirror-not-empty`/`--gen-todo` 5개 모드)
- [x] `scripts/i18n-extract-keys.sh` (`--warn`/`--strict`, surplus 항상 error)
- [x] `.github/workflows/ci.yml` `lint-i18n` job (Korean-in-en source reject + drift warn + key parity warn)
- [x] `.github/workflows/release.yml` `lint-i18n-strict` job (drift+parity hard-fail, 4-arch matrix build)
- [x] `.github/workflows/release-prep.yml` (workflow_dispatch + `release/*` push trigger, peter-evans/create-pull-request 로 PR 생성)
- [s] 별도 `PR_TEMPLATE_i18n_completion.md` 파일 — `--gen-todo` 의 inline body 가 PR 본문으로 충분히 자세함, 별도 파일 불필요. release-prep workflow 의 `body-path: body.md` 가 동일 역할.

### M12.11 — `genasis doctor [i18n]` 확장  — ✅ commit ea1e9d6
- [x] `crates/genasis-cli/src/cmd_doctor.rs` `[i18n]` 섹션
  - [x] CLI/TUI runtime locale + provenance source label
  - [x] active agent locale + reference docs (또는 `(none)` / `not configured`)
- [s] source/mirror parity 인라인 출력 — `scripts/check-i18n-drift.sh --list` 로 위임 (doctor 출력 길이 절약).
- [s] rust-i18n key parity 인라인 — `scripts/i18n-extract-keys.sh` 로 위임.
- [s] `tests/unit/doctor_i18n.rs` — `tests/install_lang_e2e.rs::lang_status_reports_active_locale` 가 i18n 출력의 핵심 contract 를 binary 레벨에서 검증.

### M12.13 — README SEO + 다국어 토글 고도화 (blueprint §19.13)

#### M12.13.a `README.md` (English) SEO 최적화 + 구조 재작성  — ✅ commits 2e9cdd8, 7c05d94
- [x] OSS-grade 구조 (Bun/Tauri/Astro/Vite/Biome 패턴):
  - [x] H1 + tagline + tag chip line + 5 status badges (CI/Release/License/Stars/Rust)
  - [x] Why Genasis (산문 3 단락)
  - [x] Quickstart (1 curl + `--lang both` reject 명시)
  - [x] At a glance (8-row 표)
  - [x] Demo (asciinema cast pointer)
  - [x] Documentation (영/한 분기 표 — 5개 source 모두 영문 링크)
  - [x] Architecture mermaid (GitHub native 렌더)
  - [x] Comparison table (vs ECC / kw-plugins / claude-code-templates)
  - [x] Status / Contributing / Star History (`<picture>` dark-mode variant) / License / bottom navigation

#### M12.13.b 다국어 토글 3-단계 fallback  — ✅ commit 2e9cdd8
- [x] 상단 language badge row (shields.io 3개)
- [x] Cross-link batch (`🇺🇸 English | [🇰🇷 한국어]`)
- [x] Bottom navigation footer (`### Other languages / 다른 언어`)
- [x] `README.ko.md` 동일 토글, 현재 언어 굵게

#### M12.13.c `README.ko.md` (Korean mirror) 작성  — ✅ commit 2e9cdd8
- [x] 18-section 동일 구조 한국어
- [x] 한국어 키워드 (`에이전트`, `클로드`, `한국어`)
- [x] 외부 링크·shields URL·코드블록 영어 source 와 byte-동등

#### M12.13.d GitHub repo 메타데이터  — ✅ API call (이전 응답)
- [x] Repo Settings > Topics 18개 등록 (`agentic-ai`, `claude-code`, ..., `korean` 등)
- [x] About description + homepage URL (Pages URL 로 갱신)
- [s] Social preview image upload — REST API 미지원, Web UI only. `docs/assets/og-image.png` 가 준비되어 있어 사용자가 Settings → Social preview 에서 업로드. M12.13.h Pages OG 메타가 우선 작동.

#### M12.13.e Open Graph + 시각 자산  — ✅ commits 2e9cdd8, 23251ae, 후속
- [x] `docs/assets/og-image.svg` + `og-image.png` (영어, 1280×640)
- [x] `docs/assets/og-image.ko.svg` + `og-image.ko.png` (한국어, 1280×640)
- [x] `docs/assets/demo.cast` (asciicast v2, install + 한국어 prompt + monitor)
- [x] `docs/assets/architecture.svg` (4-layer ASCII-style SVG, GitHub Pages OG fallback)
- [x] `docs/assets/logo.svg` (240×240, dark theme + accent gradient)

#### M12.13.f 자동 SEO 시그널 (badges)  — ✅ commit 2e9cdd8
- [x] shields.io badges: CI, License, Release, Stars, Rust version
- [x] Star History badge (star-history.com `<picture>` dark variant)
- [x] Codecov badge — `cargo-llvm-cov` 설치, `.github/workflows/ci.yml` 에 `coverage` job 추가 (lcov.info 생성 → codecov/codecov-action@v4), README 영/한에 Codecov shield. 로컬 baseline: 54.56% lines / 45.02% fns / 51.36% regions.
- [s] GitHub Sponsors badge — Sponsors 미등록.
- [x] 모든 badge 영/한 mirror 양쪽 동일 배치

#### M12.13.g 다국어 추가 컨트리뷰터 가이드  — ✅ commit ea1e9d6
- [x] `docs/i18n/CONTRIBUTE-LANG.md` (영어, 4-surface PR 레시피)
- [x] `docs/ko/i18n/CONTRIBUTE-LANG.md` (한국어 mirror)
- [x] 4단계 절차 명시
- [s] 새 언어 PR CI 자동 체크리스트 — 현재 `lint-i18n` 이 한국어 source reject + drift + key parity 만 검증. 새 locale PR 검증은 첫 추가 시점에 추가 step.

#### M12.13.h GitHub Pages 자동 라우팅  — ✅ commits 2e9cdd8 + Pages enable API
- [x] `docs/_config.yml` (Jekyll + `jekyll-sitemap` + `jekyll-seo-tag`, locale-only `include` + 광범위 `exclude`)
- [x] `docs/index.html` (navigator.language + `<meta refresh>` fallback + Open Graph + Twitter Card + JSON-LD `SoftwareApplication`)
- [x] `docs/en/index.md`, `docs/ko/index.md` (frontmatter title/description/lang/permalink)
- [x] `docs/robots.txt` (sitemap pointer)
- [x] JSON-LD schema (index.html 인라인)
- [x] Canonical URL (`<link rel="canonical">`)
- [x] Pages activate (REST API `PUT /repos/.../pages` source `main:/docs`)
- [s] Custom domain — 사용자 결정 사항 (CNAME 추가 + DNS 설정).

#### M12.13.i 측정 + 회고 hook
- [s] GitHub Insights baseline — repo 가 막 publish 됐으므로 baseline = 0. 1주일 후 첫 measurement 가 의미.
- [s] Google Search Console — Pages 도메인 verify (DNS TXT 또는 HTML meta) 필요. 사용자 GSC 계정에서 직접 진행.
- [s] 3개월 회고 — 운영성 항목, calendar reminder 로 운영. release v0.1.0 태깅 시 retrospective issue template 작성.

### M12.12 — 회고 + DoD
- [x] `lint-i18n` CI 통과 — `lint-i18n` job, `lint-i18n-strict` 모두 작성, commit `b7bffaa` 에서 CI success 검증
- [x] `release-prep` 워크플로 — workflow_dispatch 로 `v0.1.0` 트리거 success, drift 0 일 때 `needs_pr=false` 정확한 분기 검증
- [x] drift 0건 — `scripts/check-i18n-drift.sh --strict` clean (모든 mirror 동기)
- [x] `genasis doctor [i18n]` 섹션 정상 — `lang_status_reports_active_locale` E2E 통과
- [x] E2E 시나리오 자동 테스트 — `tests/install_lang_e2e.rs` 6 tests (flag_en/flag_ko/both_rejected/non_tty_fallback/lang_status/lang_switch_no_op)
- [x] `install.sh --lang ko` Bash 분기 + `bash -n` 통과
- [x] `with-ko-locale` 골든 픽스처 (input + README + SHARED.md 행)
- [x] `README.md` / `README.ko.md` 18-절 SEO + 3-단계 토글 적용
- [x] GitHub repo Topics 18개 등록 (REST API)
- [x] GitHub Pages 라우팅 활성화 (REST API, `b7bffaa` 부터 build success)
- [x] M12 회고 — commit 158aada 의 body + 본 progress 의 인라인 회고 항목들에 분산 기록

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

## M-D — Design Catalog Integration (post-M12)

> 사용자 승인 2026-05-04. 외부 디자인 provider(`getdesign` npm) 위임 + 두-모드
> design-system.md(pristine / external-pointer) + 사용자 오버라이드 누적 +
> pristine 복원 + 비-npx `--from <path>` 진입점.

### 핵심 설계 결정
- **vendor 안 함**: awesome-design-md 콘텐츠를 다시 짊어지지 않는다. `npx getdesign add <slug>` 위임. 라이선스 컴플라이언스는 getdesign upstream 책임.
- **두 모드**: `docs/design-system.md` 가 `mode = pristine` 일 때는 본문이 진실, `mode = external` 일 때는 §A 포인터(외부 DESIGN.md) + §B 사용자 오버라이드 + §C 사용 매뉴얼만 들어있음.
- **외부 DESIGN.md 위치**: `docs/design-system/DESIGN.md`. read-only 취급.
- **상태 파일**: `docs/.design-state.toml` (mode/slug/source/template_hash/applied_at/previous_slug/gallery_preview/override_count).
- **백업**: pristine 본문은 `docs/design-system/pristine.bak` 으로 swap 직전 자동 백업. `restore` 시 `docs/design-system/` 디렉터리는 `docs/design-system.archive-<ts>/` 로 옮긴 뒤 백업으로 원복.
- **이슈 폭주 정책**: `changed_areas.len() ≥ 4` 자동 EPIC 모드 (EPIC 1 + 자식 N). 자식 description 에 EPIC ID 명시 (Plane upstream 호환). `--per-area` / `--full-rewrite` 명시 플래그 노출.
- **텔레메트리 default OFF**: `genasis design swap` 호출 시 자동으로 `GETDESIGN_DISABLE_TELEMETRY=1` 환경 변수 set. 사용자가 `[design].disable_telemetry = false` 또는 `--telemetry on` 으로 켤 수 있음. genasis 자체 수집 서버는 운영하지 않음.
- **갤러리 추상화**: `genasis.toml [design]` 의 `add_command` 템플릿(`{slug}`, `{out}` 치환)을 통해 getdesign 외 자체 갤러리로 교체 가능.

### M-D1 — Pristine/External 모드 + swap/restore + skill (완료 2026-05-04)
- [x] `genasis-core` 의 `Config` 에 `[design] DesignConfig` 추가 (`gallery_index_url`, `gallery_url_template`, `add_command`, `disable_telemetry`, `external_dir`)
- [x] `genasis-design` 크레이트 재구성:
  - [x] `mode.rs` — `Mode::Pristine | Mode::External`, `.design-state.toml` R/W
  - [x] `swap.rs` — slug 모드(npx invoke) + `--from <path>` 모드(파일 복사) 통합 entry
  - [x] `restore.rs` — external→pristine 복원 (archive 이동 + pristine.bak → design-system.md)
  - [x] `pointer.rs` — design-system.md 포인터 본문 렌더(§A/§B/§C 골격) — locale 분기 (en/ko 인-소스 템플릿)
  - [x] 기존 `extractor.rs` / `change_protocol.rs` / `diff.rs` / `ticket_emitter.rs` 보존, 레거시 진입점은 `run_legacy_swap` 으로 alias
- [x] CLI [`cmd_design.rs`] 확장:
  - [x] `swap <slug>` (기존 `swap <url> --body` 와 호환 — `--body` 레거시 경로 유지)
  - [x] `swap --from <path>`
  - [x] `restore`
  - [x] `status` 출력에 mode/slug/applied_at/override_count/preview URL 포함
- [x] 템플릿:
  - [s] `design-system.md.tera` 두 변종 분리는 보류 — 포인터 본문은 `pointer.rs::render` 가 코드 측에서 생성하므로 Tera 분리 불필요. attach 시 placeholder 가 그대로 들어가고 swap 이 외부 모드 진입 시 덮어쓴다.
  - [x] `templates/{en,ko}/skills/design-aware/SKILL.md.tera` 강화: 참조 순서(pristine → external §A → §B), 사용자 요구 충돌 처리 절차, 외부 DESIGN.md 직접 편집 금지 규칙, 사후 가이드
- [x] i18n keys: `design.swap.delegating`, `design.swap.from_local`, `design.swap.pristine_backed_up`, `design.swap.design_md_written`, `design.swap.pointer_written`, `design.swap.state_updated`, `design.swap.post_swap_*`, `design.status.mode_pristine`, `design.status.mode_external`, `design.restore.*` 등 ko/en (14 키 × 2)
- [x] e2e (`crates/genasis-design/tests/swap_restore_round_trip.rs`): pristine → swap slug → swap slug 2 → restore 라운드트립 + sha256 검증
- [x] cargo test green (132 → 145 passed)

### M-D2 — EPIC plan + Mattermost + 사용자 오버라이드 누적 (완료 2026-05-04)
- [x] `ticket_emitter` 에 `Plan::FullRewrite { epic, children }` + `PlanMode::{Auto, PerArea, FullRewrite}` 추가, 자동 임계치 `DEFAULT_FULL_REWRITE_THRESHOLD = 4` (영역 7 중 과반)
- [x] 자식 description 에 EPIC title 명시 — Plane upstream(native parent_id 없음) 에서도 보드 묶음 가시성 확보
- [x] Mattermost 공지 템플릿 (CLI 가 본문을 emit; 실제 게시는 caller/provider 호출): `🚨 DESIGN CHANGE: <from> → <to> | preview: <url> | issues planned: <n>`
- [x] `genasis design verify` (`crates/genasis-design/src/verify.rs`) — `.design-state.toml.template_hash` 와 실제 `DESIGN.md` sha256 비교, 변조 감지
- [x] `genasis design override add "<text>"` (`crates/genasis-design/src/override_log.rs`):
  - [x] `<!-- genasis design override add appends here. Do not edit by hand. -->` (en) / `... 자동 append. 직접 편집 금지. -->` (ko) 두 sentinel 인지
  - [x] `#### override-<id> @ <iso>` 블록 append, `override_count` 증가
  - [s] §A grep+인용은 design-aware SKILL 의 에이전트 책임 — CLI 는 본문만 받아 기록
- [x] `genasis design override list` / `remove <id>`
- [x] e2e (`crates/genasis-design/tests/epic_plan_and_overrides.rs`): full-rewrite EPIC 검증, 오버라이드 3개 누적 후 swap 시 §B.2 초기화(의도된 동작 — 사용자가 새 §A 기준으로 재검토) + 변조 검증

### M-D3 — Monitor 위젯 + attach 프롬프트 + doctor + ADR (완료 2026-05-04)
- [x] `AppState.design: DesignWidgetState`(mode/slug/applied_at/override_count/preview_url/gallery_url)
- [x] `widgets/design.rs` — pristine/external 분기 렌더, 키 `7` 포커스, `Enter` 시 preview URL → `open`/`xdg-open`/`cmd /C start`
- [x] `app.rs` 레이아웃에 Design 패널 추가 (Deploy 행 아래 7-line 슬롯)
- [x] `cmd_attach.rs` 가 첫 attach 시 `[design]` 기본값을 `genasis.toml` 에 자동 시드 (`gallery_index_url`, `gallery_url_template`, `add_command`, `disable_telemetry=true`, `external_dir`). 이미 있으면 보존(idempotent). 인터랙티브 프롬프트는 i18n / non-interactive 일관성을 위해 시드만 — 사용자는 추후 `genasis.toml` 직접 편집으로 갤러리 교체 가능
- [x] `cmd_doctor.rs` `[design]` 섹션 추가:
  - [x] mode 출력 (pristine / external + slug)
  - [x] `npx` 가용성 — pristine 일 땐 optional, external 일 땐 required-missing 경고
  - [x] external 모드에서 `run_verify` 재호출하여 hash 일치 확인
  - [x] mode 와 디스크 상태 일관성 — 포인터 / 외부 디렉터리 누락 감지
- [x] `docs/ADR/ADR-009-design-catalog-delegation.md` (en) + `docs/ko/ADR/ADR-009-...` (ko) — vendor 안 함 결정 근거 / 두 모드 정당화 / 갤러리 URL 추상화 / 텔레메트리 default off / 충돌 해결 정책 / 대안 검토 3가지
- [x] doctor i18n keys 추가 (en/ko): `doctor.design.section`, `mode_pristine`, `mode_external`, `npx_missing_optional`, `npx_missing_required`, `verify_ok`, `verify_tampered`, `verify_error`, `pointer_missing`, `extdir_missing`. monitor key hint 갱신 (`[1-7] focus`, `[Enter] open URL`)
- [s] manual TUI smoke — 코드 경로 단위 검증 + state load fallback 까지 정합. 실제 키 입력 검증은 첫 v0.1.0 cross-compile 후
- [x] cargo test + lang drift 통과

---

## M14 — Default agentic team bootstrap (green-field install)

> 2026-05-05 사용자 제기. 현재 overlay 엔진은 `.claude/agents/*.md` 를
> **이미 존재하는 파일** 로 가정한다 — `attach` 는 사용자가 직접 작성한
> agent 파일에 fence 만 주입한다. 프로젝트에 agent 팀이 전혀 없을 때,
> ECC canonical 10 역할을 scaffold 하는 경로가 없으므로 "비파괴 overlay"
> 약속에 green-field 진입점이 비어 있다. M14 가 이 갭을 메운다 — **base
> agent template** (역할 파일 부재 시 렌더) 위에 기존 **patch overlay**
> (marker fence 안쪽에 렌더) 가 얹히는 2-layer 구조.

### 핵심 설계 결정 (ADR-010 후보)

- **default OFF**: bootstrap 은 opt-in (`--bootstrap`). `attach` 를 빈
  `.claude/agents/` 에 돌리는 기존 사용자는 silent file 생성이 아니라
  경고를 받음. ADR-001 의 비파괴 invariant 보호.
- **base + patch 소유권 분리**: base 파일 전체는 emit 후 사용자 소유
  (자유 편집). 그 안의 marker fence 만 genasis 소유 (upgrade 가 갱신).
  ADR-001 의 "fence 밖은 사용자 영역" 약속 일관 유지.
- **ECC vendor 안 함**: base 템플릿은 역할별 짧은 스텁 — `claude-code-templates`
  / ECC 역할 정의 fork 가 아니라 frontmatter (`name/description/tools/model/color`)
  + 5~10줄 헤더만. patch fence 가 이후 단계에서 프로토콜 살을 붙임.
- **i18n 분리 트리**: `templates/en/agents/<role>.md.tera` +
  `templates/ko/agents/<role>.md.tera` 2 트리. `lang switch` 시 base 도
  같이 swap (단, 사용자가 fence 밖을 편집했다면 보존 — 기존 `lang switch`
  의 fence-internal-only 정책 그대로).
- **role set**: pm / planner / architect / frontend / backend / qa /
  designer / security / devops / code-reviewer (M2 의 `Role::ALL` 과
  동일 10개).

### M14.0 — Decision gate + ADR-010
- [x] `docs/ko/ADR/ADR-010-default-team-bootstrap.md` (한국어 SSOT) 작성:
  context, alternatives (a~f), decision (b+d, e-rejected), consequences,
  references (ADR-001 marker fence + ADR-008 lang precedence)
- [x] `docs/ADR/ADR-010-default-team-bootstrap.md` 영어 mirror
- [x] `blueprint.ko.md §20` 신설 (M14 섹션, ADR-010 인용) + `blueprint.md`
  section index 갱신
- [x] `blueprint.ko.md §16` ADR 표에 ADR-008/009/010 행 추가
- [x] 사용자 ratify 게이트 — 2026-05-08 ratify 완료 (진입점: 신규 `genasis bootstrap` 서브커맨드 + `genasis init --bootstrap` alias, `--no-attach-after` 가 없으면 `cmd_attach` 자동 chain. ADR-010 §3 결정 (b)+(d))

### M14.1 — Base agent templates (`templates/{en,ko}/agents/<role>.md.tera`)  — ✅ pending build verification
- [x] `crates/genasis-templates/templates/en/agents/` 디렉토리 신설
  - [x] `pm.md.tera` (frontmatter + 5~10줄 역할 헤더)
  - [x] `planner.md.tera`
  - [x] `architect.md.tera`
  - [x] `frontend.md.tera`
  - [x] `backend.md.tera`
  - [x] `qa.md.tera`
  - [x] `designer.md.tera`
  - [x] `security.md.tera`
  - [x] `devops.md.tera`
  - [x] `code-reviewer.md.tera`
  - [x] `README.md` — base vs patch 경계 설명, 사용자 편집 영역 명시
- [x] `crates/genasis-templates/templates/ko/agents/` — 위와 동일 11 파일
  (10 base + README, 한국어 본문 + 동일 frontmatter, `description:` 만 ko)
- [x] `genasis-templates::lib.rs` 의 `include_dir!()` 가 새 디렉토리를
  자동 임베드 (디렉토리 추가만으로 OK — 매니페스트 갱신 불요)
- [x] `agent_base_subtrees_have_same_roles` 테스트 — 양 locale 모두에 10
  required role tera 가 존재함을 검증
- [s] frontmatter contract 단위 테스트는 `bootstrap.rs::tests::rendered_base_carries_required_frontmatter_keys`
  에서 통합 검증 (base 렌더 결과가 5 키를 모두 보유 + `name:` 이 stem
  과 매칭)

### M14.2 — `genasis-overlay::bootstrap` 모듈  — ✅ pending build verification
- [x] `crates/genasis-overlay/src/bootstrap.rs` 신규 모듈
  - [x] `BootstrapOptions { lang, roles, context }` + `Default` + builder
    setters (`new`, `with_roles`, `with_context`)
  - [x] `pub fn plan_bootstrap(project_root: &Path, opts: &BootstrapOptions) -> Result<BootstrapPlan>`
    — `.claude/agents/<role>.md` 부재 → `Create { body }`, 존재 → `Skip { reason: "exists" }`
  - [x] `BootstrapPlan` (`creates()` / `skips()` iterator) + `BootstrapAction::{Create, Skip}`
  - [x] `pub fn apply_bootstrap(plan: &BootstrapPlan) -> Result<BootstrapReport>`
    — `gfs::atomic_write` 로 새 파일 생성 (`atomic_write` 가 자동으로
    부모 디렉토리 `create_dir_all`)
- [x] `lib.rs` 에 `pub mod bootstrap;` + re-export (`apply_bootstrap`,
  `plan_bootstrap`, `BootstrapAction`, `BootstrapChange`, `BootstrapOptions`,
  `BootstrapPlan`, `BootstrapReport`)
- [x] 단위 테스트 (`crates/genasis-overlay/src/bootstrap.rs::tests`):
  - [x] `empty_project_creates_all_ten_roles` — 빈 프로젝트 → 10 `Create`
  - [x] `existing_files_are_skipped` — 일부 역할 존재 → 부재 역할만 `Create`,
    존재 역할은 `Skip("exists")` + role enum 검증
  - [x] `apply_writes_only_create_actions` — `apply_bootstrap` 후 10 파일
    실제 디스크 존재
  - [x] `rendered_base_carries_required_frontmatter_keys` — frontmatter
    contract (5 키 + `name: <slug>` 매칭)
  - [x] `korean_locale_subtree_loads` — `--lang ko` 도 동일하게 동작
  - [x] `unknown_locale_errors` — 미지 locale 은 `Error::Overlay` 반환
  - [x] `role_subset_only_plans_chosen_roles` — `with_roles(vec![...])` 로
    부분 scaffold
  - [x] `idempotent_second_apply_is_a_noop` — bootstrap 두 번 호출 시
    두 번째는 모두 Skip
- [x] 통합 테스트 (`crates/genasis-overlay/tests/bootstrap_then_attach.rs`):
  - [x] `bootstrap_then_attach_injects_into_every_role` — bootstrap → scan
    → 10 모두 `Known(_)` → plan_attach → 10 `Inject`
  - [x] `bootstrap_ko_then_attach_ko_injects_korean_overlay` — `--lang ko`
    chain 검증, backend.md 의 attach 결과에 한국어 프로토콜 헤더
    "Plane / Mattermost 프로토콜" 포함 확인
  - [x] `bootstrap_partial_then_attach_handles_mix` — 사용자 author 한
    frontend.md 가 bootstrap 에 의해 byte-identical 보존됨

### M14.3 — CLI wire-up — ✅ commit pending (이 commit)
- [x] `crates/genasis-cli/src/cmd_bootstrap.rs` 신설 — `genasis bootstrap [--lang] [--roles] [--no-attach-after] [--dry-run] [--project]`. agents 카탈로그 로드 → `plan_bootstrap` → `apply_bootstrap` → `--no-attach-after` 가 없으면 `cmd_attach::pub_run` 자동 chain.
- [x] `crates/genasis-cli/src/cmd_init.rs` 에 `--bootstrap` alias + `--roles` forwarder 추가. 내부적으로 `cmd_bootstrap::run` 으로 위임 — 두 진입점이 byte-identical.
- [x] `crates/genasis-cli/src/cmd_attach.rs` empty-dir hint: `report.agents` 와 `report.skipped` 가 모두 비어 있으면 stderr 에 `bootstrap.no_agents_hint` 출력 후 계속 진행 (기존 비파괴 동작 유지).
- [s] `cmd_attach --bootstrap` 대안 — ADR-010 §3 (b)+(d) 에 따라 거부. 단일 canonical 진입점 + `init --bootstrap` alias 유지.
- [x] `genasis-i18n/locales/{en,ko}.yml` 에 키 추가: `bootstrap.no_agents_hint`, `bootstrap.scaffolded_summary` (`%{count}`), `bootstrap.skipped_existing` (`%{name}`), `bootstrap.next_step`.
- [x] `--lang` 우선순위 — `cmd_bootstrap::run` 이 `cmd_attach::pub_run` 과 동일한 `lang_prompt::decide` 를 호출. 글로벌 `--lang` 플래그가 base/patch 양쪽 트리에 동일하게 적용. `parse_roles_*` 단위 테스트로 role-subset 경로 검증.

### M14.4 — `tests/golden/blank/` 활성화 — ✅ commit pending (이 commit)
- [x] `tests/golden/blank/input/` — README.md 만 있는 빈 mock project (`.claude/` 없음)
- [x] `tests/golden/blank/expected/` — bootstrap+attach 산출물 (펜스가 들어간 10개 에이전트 파일 + README.md), `BLESS=1 cargo test` 로 채움
- [x] `crates/genasis-overlay/tests/golden_blank.rs` — 두 개 테스트: bootstrap+attach+detach round-trip + expected/ snapshot 동치 비교 (`BLESS=1` 으로 갱신)
- [x] `tests/golden/SHARED.md` 표의 blank 행을 **Active** 로 변경 + 테스트 경로 + BLESS 힌트
- [s] `tests/golden/blank-ko/` — M18 audit 으로 미룸. 사용자 지시("의도 재점검 후 결정")에 따라 ad-hoc 으로 추가하지 않고 fixture roster 전체와 함께 결정.

### M14.5 — Doctor + 회고 — ✅ commit pending (이 commit)
- [x] `cmd_doctor.rs` `[bootstrap]` 섹션 추가:
  - [x] `.claude/agents/` 존재 여부 + 파일 수 (`doctor.bootstrap.dir_missing` / `file_count`)
  - [x] 빈 디렉토리 + bootstrap 미실행 → `doctor.bootstrap.empty_hint` 안내 (i18n)
  - [x] base 파일의 frontmatter `name:` 이 파일명 stem 과 일치하는지 + missing/mismatch 경고
- [x] `progress.md` / `progress.ko.md` 회고 표 — 아래에 M14 행 추가.
- [x] DoD: `cargo test --workspace` green (177 → 179 passed; golden_blank 포함), bootstrap 관련 drift 0. doctor 의 더 깊은 coverage 는 M19 에서.

### 리스크 / 미정
- **(a)** `init --bootstrap` vs `attach --bootstrap` 위치: **2026-05-08 해소** — 신규 `genasis bootstrap` 서브커맨드 + `genasis init --bootstrap` alias (ADR-010 §3 (b)+(d)).
- **(b)** ECC `claude-code-templates` 와 차별화 문구: README.md (Comparison
  표) 의 "Non-destructive overlay" vs "Bootstrap" 두 차원으로 분리해야
  시각적 혼동 회피.
- **(c)** base 템플릿이 `tools:` 항목을 어디까지 specify 할지 — 너무 협소
  하면 사용자 자유도 침해, 너무 넓으면 무의미. 우선 ECC default
  (`Bash, Read, Write, Edit, Glob, Grep, Task`) 기준 + comment 로 안내.

---

## v0.1.0 계획 (2026-05-08 확정)

> v0.1.0 컷 조건 (사용자 결정 2026-05-08): `README.md` 에서 소개하는 모든
> 명령이 자동 E2E 테스트로 검증되고, `tests/golden/` 의 모든 fixture 가
> `expected/` 가 채워졌거나 명시적으로 폐기됐을 때. 아래 로드맵은 남은
> 마일스톤을 commit 단위로 쪼개고 각 commit 직후 검토를 받는다.

| 순서 | 마일스톤 | 범위 | 상태 |
|---|---|---|---|
| 1 | M14.0 | ADR-010 ratify gate | done |
| 2 | M14.3 | `cmd_bootstrap.rs` + `init --bootstrap` alias + `attach` empty-dir hint + i18n 키 4개 | done |
| 3 | M14.4 | `tests/golden/blank/` 활성화 (input + expected + round-trip) | done |
| 4 | M14.5 | `cmd_doctor.rs [bootstrap]` 섹션 + retro + DoD | done |
| 5 | M18 | Golden fixture 재점검 — 유지/폐기/추가 결정 후 살아남은 fixture `expected/` 채움 | done |
| 6 | M19 | `tests/e2e/` Rust 통합 스위트 — README 13개 명령 모두 (기본 백엔드 trial flavor) | in progress (M19.1/.2/.3 완료; M19.4 는 M15/M16 후) |
| 7 | M20 | `nightly-e2e.yml` workflow 부활 — `servers/docker-compose.yml` 로 실 Plane + MM 스모크 | done |
| 8 | M21 | trial-app Playwright suite — US-001..US-022 acceptance 풀 회귀 | pending |
| 9 | M15 | Manifest + drift detection + `genasis debug {status,log,collect,reset}` | done |
| 10 | M16 | `genasis debug submit` (PR-only, ADR-012 §8) + `debug-history/` 리포 구조 + workflow + skill | done |
| 11 | M17 | 분석 자동화 + 통합 | done |
| 12 | v0.1.0 cut | 태그 + release.yml 실행 + 공지 | ready (release notes 초안 완료; 태그는 메인테이너 액션) |

### M18 — Golden fixture 재점검 — ✅ commit pending (이 commit)

2026-05-08 audit 결정: golden fixture 는 **결정적 디스크 상태 출력**만
고정하고, 순수 데이터에 대한 단위 테스트로 표현 가능한 시나리오는 해당
crate 로 옮긴다. 기존 7개 디렉토리에 적용:

| 디렉토리 | 결정 | 근거 |
|---|---|---|
| `ecc-only/` | **유지** | round-trip + idempotent attach anchor (`golden_ecc_only.rs`). 이미 채워짐. |
| `blank/` | **유지** | M14 bootstrap 진입점 (`golden_blank.rs`). M14.4 에서 채움. |
| `with-ko-locale/` | **유지** | 한국어 overlay body anchor — 언어별 디스크 상태 고정 가치. |
| `kw-plugins/` | **폐기** | detector 가 frontmatter `name:` 만 읽음 — ECC 와 코드 경로 차이 없음. |
| `legacy-bash-genesis/` | **폐기** | `cmd migrate-from-genesis` 가 v0.1.0 에서 docs-only (M11 [s]). 검증할 코드 경로 없음. |
| `with-drizzle/` | **폐기** | 단일 `detected()` 호출 → `crates/genasis-db/src/adapters/drizzle_kit.rs::tests` 의 신규 unit test 로 cover. |
| `with-duckdb/` | **폐기** | 단일 `Driver::parse("duckdb")` → `crates/genasis-db/src/kernel.rs::tests` 에서 이미 cover. |

검토했던 신규 후보 (`with-trial/`, `bootstrap-then-attach-{en,ko}/`)는
거부 — M19 Rust 통합 스위트가 더 싸게 같은 시나리오 cover.

이 commit 의 산출물:
- `tests/golden/{kw-plugins,legacy-bash-genesis,with-drizzle,with-duckdb}/` 제거 (`git rm -r`).
- `crates/genasis-db/src/adapters/drizzle_kit.rs` 에 unit test 3개
  (`detected_true_when_ts_config_present`, `_when_js_config_present`,
  `_false_when_no_config`) 추가 — 폐기된 `with-drizzle/` 시나리오의 보장 유지.
- `tests/golden/SHARED.md` 를 살아남은 3개 fixture + 폐기 목록 + "unit
  test 우선, golden 차선" 지침으로 재작성.
- `cargo test --workspace`: 183 → 186 passed.

### M19 — `tests/e2e/` Rust 통합 스위트 (README parity)

`README.md §CLI Reference` 의 모든 명령 커버:
`init`, `init --trial`, `attach`, `detach`, `doctor`, `upgrade`,
`bootstrap`, `agents {browse,install,list,installed,remove}`, `monitor`
(headless smoke), `design swap`, `db {query,migrate}`, `lang switch`,
`debug {status,collect,submit}` (마지막은 M15+M16 완료 후 gate),
`example`. 기본 백엔드는 `trial` flavor 와 process-local `trial-app`
인스턴스 — CI 에서 외부 의존성 없이 hermetic 실행.

### M20 — `nightly-e2e.yml` workflow 부활

M0 에서 declare 했지만 실제 파일이 없는 workflow 재작성. nightly
schedule 로: `servers/docker-compose.yml` 을 `docker compose up -d`,
M19 스위트를 `flavor = "plane"` / `flavor = "mattermost"` 로 실행
(trial 대신), 종료 시 tear down. Tag `nightly-real-servers`, 실패 시
라벨 붙은 issue 자동 생성.

### M21 — trial-app Playwright suite

사용자 결정 2026-05-08 — `trial-app/ralph/prd.json` US-001..US-022 의
모든 acceptance criterion 을 Playwright spec 으로 변환.
- `trial-app/e2e/` 디렉토리 + `playwright.config.ts`
- US 당 spec 파일 한 개 (`us-001.spec.ts` ... `us-022.spec.ts`)
- `trial-app` `package.json` 에 `npm run e2e` 등록
- M19 와 hook (`genasis init --trial` E2E 가 Quick Path 커버) +
  trial-app 자체 개발 사이클에서도 단독 실행 가능.

### v0.1.0 컷 기준 (DoD)

- [x] `cargo test --workspace --no-fail-fast` green — 222 passed, 2 ignored
- [x] `npm --prefix trial-app run e2e` green (M21) — 14 passed, 1 skipped
- [x] `tests/e2e/` Rust 스위트 CI green (M19) — lifecycle/agents/supporting/debug 4개 spec, 23 테스트
- [ ] Nightly real-server suite 1회 이상 green (M20) — workflow 등록 완료; 첫 schedule 실행 대기
- [x] `tests/golden/*/expected/` 모두 채워졌거나 디렉토리 제거 (M18) — 살아남은 fixture: ecc-only, blank, with-ko-locale
- [ ] `lint-i18n-strict` green (release.yml hard fail) — 기존 drift 5건 (CREDITS / DESIGN-SWAP-GUIDE / AGENTS-MARKETPLACE / QUICKSTART / famous-agents) 한국어 미러 채워야 태그 가능
- [x] `cargo clippy --workspace --all-targets` clean (errors 0) — `-D warnings` 은 누적된 dead_code 경고 때문에 보류
- [x] `cargo fmt --all -- --check` clean
- [x] `docs/RELEASE-NOTES-v0.1.0.md` 초안 작성
- [ ] `v0.1.0` 태그, release.yml 실행, GitHub Release notes 게시 — **메인테이너 액션**

---

## Phase F — Debug History 피드백 루프 (ADR-012)

> 2026-05-05 사용자 설계. Genasis는 메타 도구로서 overlay 파일을 생성하며 사용자는
> 필연적으로 이를 수정한다. 이 수정사항은 genasis 개선을 위한 최고 가치의 신호다.
> 이 Phase는 안전한 상시 드리프트 감지 + 옵트인 제출 파이프라인을 구현하여
> 필드 패치를 Claude Code 자동 분석으로 genasis 개발에 피드백한다.
>
> 거버넌스: 기여자는 데이터만 제출(`debug-history/patches/*.patch.json`);
> 메인테이너가 Claude Code `/debug-review` 스킬로 패치를 처리해 자동개발.
> ADR-012 §8 참조.

### M15 — 매니페스트 + 드리프트 감지 + 로컬 디버그 명령

- [ ] `genasis-core` 매니페스트 모듈:
  - [ ] `.manifest.json` 스키마 (genasis_version, agents_catalog_version, attached_at, lang, files 맵 with sha256/template_source/fence_sha256)
  - [ ] `manifest::generate(project_root)` — `.claude/genasis/` + marker fence 스캔, 매니페스트 생산
  - [ ] `manifest::compare(manifest, live_state)` → `Vec<DriftEntry>`
  - [ ] `DriftEntry { file, drift_type, old_hash, new_hash, diff_lines }`
- [ ] 매니페스트 생성을 `cmd_attach.rs`와 `cmd_init.rs`에 연결 (apply 후)
- [ ] 매 CLI 호출 시 수동적 드리프트 감지:
  - [ ] `app_preamble()` 또는 동등 hook이 manifest compare 실행
  - [ ] `.claude/genasis/.drift-log/current.jsonl`에 append
  - [ ] < 1ms 오버헤드 목표 (관리 파일당 SHA-256만)
- [ ] `genasis debug` 서브커맨드 트리:
  - [ ] `genasis debug status` — 드리프트 요약 (변경 파일 수, 마지막 collect 시점)
  - [ ] `genasis debug log` — `.drift-log/current.jsonl` 내용 표시
  - [ ] `genasis debug collect` — 익명화 + `patch.json` 생성:
    - [ ] 시크릿 제거 (TOKEN/SECRET/KEY/PASSWORD/CREDENTIAL 정규식)
    - [ ] 경로 익명화 (절대 경로 → `<PROJECT_ROOT>/...`)
    - [ ] 프로젝트 식별자는 단방향 해시
    - [ ] `~/.genasis/debug-history/<project-hash>/<timestamp>.patch.json`에 출력
  - [ ] `genasis debug reset` — 매니페스트를 현재 상태로 갱신, drift log 초기화
- [ ] i18n 키: `debug.status.*`, `debug.collect.*`, `debug.log.*`, `debug.reset.*` (en/ko)
- [ ] 단위 테스트: manifest generate/compare, 드리프트 감지, 시크릿 제거, 경로 익명화
- [ ] 통합 테스트: attach → 파일 수동 편집 → 드리프트 감지 → collect → patch.json 유효

### M16 — Submit + 리포 구조 + `/debug-review` 스킬

- [ ] `genasis debug submit` 명령:
  - [ ] `--all | --latest | --file <path>` 선택
  - [ ] 확인 전 전체 페이로드 미리보기
  - [ ] 인터랙티브 확인 프롬프트 (i18n)
  - [ ] 선택적 `user_comment` 필드
  - [ ] `gh issue create`로 제출 (라벨: `debug-history`, 구조화된 JSON 본문)
  - [ ] 속도 제한: 프로젝트당 하루 최대 1회
- [ ] genasis 리포 내 `debug-history/` 디렉토리 구조:
  - [ ] `debug-history/index.jsonl` (패치 레지스트리: id, submitted_at, project_hash, status)
  - [ ] `debug-history/patches/` (제출된 patch.json 파일)
  - [ ] `debug-history/analysis/` (자동 생성: clusters.md, proposed-fixes.md)
  - [ ] `debug-history/schema.json` (patch.json 검증용 JSON Schema)
- [ ] `.github/workflows/debug-history-pr.yml`:
  - [ ] `debug-history/patches/*.patch.json` 변경만 허용
  - [ ] JSON 스키마 검증
  - [ ] 실행 가능 콘텐츠 거부 (shebang, 의심스러운 패턴)
  - [ ] 자동 라벨 `[debug-history]` + 자동 할당 메인테이너
- [ ] `.claude/skills/debug-review.md` 스킬:
  - [ ] `debug-history/patches/`에서 미해결 패치 모두 읽기
  - [ ] 영향받은 템플릿/파일별 클러스터링
  - [ ] 반복 패턴 식별 (임계값: ≥2 패치)
  - [ ] 템플릿 변경을 Edit으로 제안
  - [ ] `debug-history/analysis/clusters.md` 업데이트
  - [ ] `index.jsonl`에서 해결된 패치 태그
- [ ] i18n 키: `debug.submit.*`, `debug.submit.confirm`, `debug.submit.rate_limited` (en/ko)

### M17 — 분석 자동화 + 통합

- [ ] `/debug-review` 스킬 트리거:
  - [ ] 수동: 메인테이너가 `/debug-review` 호출
  - [ ] 스케줄: 주간 자동 실행 (GitHub Actions + Claude Code)
- [ ] `debug-history/analysis/clusters.md` 자동 생성:
  - [ ] 템플릿 소스별 패치 그룹화
  - [ ] 분류: bug_fix / workflow_extension / project_specific
  - [ ] 빈도 수 + 예시 발췌
- [ ] `debug-history/analysis/proposed-fixes.md` 자동 생성:
  - [ ] ≥2회 발생 클러스터에 대해: 템플릿 Edit 초안
  - [ ] 소스 패치 ID 링크
  - [ ] 신뢰도 점수 (패턴 일관성 기반)
- [ ] 감사 추적:
  - [ ] 모든 머지된 템플릿 수정이 커밋 메시지에 동기 패치 ID 참조
  - [ ] 해결된 패치를 `index.jsonl`에 수정 커밋 SHA로 태그
- [ ] 아카이빙 정책:
  - [ ] 6개월 이상 패치 → `debug-history/archive/YYYY-MM/`
  - [ ] 아카이빙된 패치는 활성 분석에서 제외
- [ ] 문서:
  - [ ] `CONTRIBUTING.md`에 debug-history 제출 섹션
  - [ ] `genasis debug --help` 종합 사용 가이드
  - [ ] GENASIS.md 템플릿에 debug history 섹션 추가

---

## Phase G — 서버 설치 + 체험 신청 + 문서 리팩토링 (2026-05-06)

genasis를 즉시 사용 가능하게 만드는 단계: 원커맨드 서버 설치, 호스팅된
체험 환경, 간결한 README, 디자인 교체 가이드.

| Sub-milestone | Scope | Status |
|---|---|---|
| G.1 | `servers/` — Plane + Mattermost + Caddy 통합 docker-compose + 설치 가이드 (키 추출 방법 포함) | done |
| G.2 | 체험 신청 web app PRD (agents-pool) — 신청 → MM #genasis-trial → 관리자 응답 → 키 제공 | done |
| G.3 | README 리팩토링 — quickstart + 체험 링크 중심, 상세 내용은 외부 가이드로 분리 | done |
| G.4 | `docs/DESIGN-SWAP-GUIDE.md` — design-system.md 교체 방법 가이드 | done |
| G.5 | 체험 데모 앱 (채팅 + 칸반 + 신청 + 상태 페이지, US-001..US-022) | done |
| G.6 | `genasis init --trial` CLI 연동 | done |
| G.7 | `genasis example {prd|design|prd2}` 서브커맨드 | done |
| G.8 | 튜토리얼 문서 (`docs/TUTORIAL.md` + `docs/ko/TUTORIAL.md`) | done |

### G.1 — 서버 설치 가이드 (`servers/`)

통합 Docker 배포: Plane + Mattermost + Caddy reverse proxy.
소스: 현 호스트의 `/work/plane` + `/work/mattermost` Docker 설정.

산출물:
- `servers/docker-compose.yml` — 단일 파일로 모든 서비스 기동
- `servers/Caddyfile` — TLS + 리버스 프록시 (plane.domain / mm.domain)
- `servers/README.md` — 단계별 가이드:
  - 사전 요구 (Docker, 도메인, DNS)
  - 환경 변수 설정
  - Plane API key + workspace slug 추출 방법
  - Mattermost 봇 토큰 생성 (role별)
  - Plane user UUID 확보 (agent 할당용)
  - `genasis.toml`에 추출한 키 입력 방법

### G.2 — 체험 신청 web app (PRD — agents-pool)

mm.realstory.blog / plane.realstory.blog 에서 호스팅. genasis를 자체
서버 없��� 체험 가능.

플로우:
1. 사용자가 체험 신청 페이지 방문
2. 정보 입력 (이름, 이메일, 프로��트명, 팀 규모)
3. 제출 → Mattermost `#genasis-trial` 채널에 알림
4. 관리자(메인테이너)가 환경 프로비���닝 후 응답
5. 사용자가 ���청 페이지에서 발급된 키/로그인 정보 확인

PRD: `agents-pool/prd/trial-webapp.md` (private).

### G.3 — README 리팩토링

원칙:
- Above the fold: 태그라인 + 한 줄 가치 + quickstart (3 명령)
- 체험 CTA: "호스팅된 Plane + Mattermost로 바로 체험" → 링크
- 복잡한 내용은 외부 가이드 파일로 분리 + 링크
- SEO 필수 콘텐츠 유지 (비교 표, 아키텍처 다이어그램)

외부 가이드 (README에서 링크):
- `docs/QUICKSTART.md` — 설치 + 첫 attach 전체 워크스루
- `docs/SERVER-SETUP.md` → `servers/README.md`
- `docs/DESIGN-SWAP-GUIDE.md` — 디자인 시스템 교체
- `docs/AGENTS-MARKETPLACE.md` — agent 브라우징 + 설치

### G.4 — 디자인 교체 가이드

`docs/DESIGN-SWAP-GUIDE.md`:
- design-system.md란 무엇이고 왜 중요한가
- `genasis design swap <slug>` — 갤러리 브라우징
- `genasis design swap --from <path>` — 로컬 파일
- `genasis design restore` — pristine 복원
- 사용자 오버라이드 (`genasis design override add`)
- EPIC 모드 (영향 UI 영역 자동 이슈 생성)

### G.5 — 체험 데모 앱 (채팅 + 칸반 시뮬레이션) — ✅ 커밋 e0683de..de860ad (US-001..US-022) + 후속 UI 다듬기 (cc95fa9, 9ca1b43, cffb314, a14fc11, 5bdaadf)

`trial-app/` (Next.js 15 App Router) 에 호스팅 체험 풀-플로우 구현:
- [x] 데모 칸반 보드 (Todo/InProgress/Done 컬럼, 카드 애니메이션) — `app/components/KanbanBoard.tsx` + `DemoBoard.tsx`
- [x] 데모 채팅 스레드 (스크립트 에이전트 메시지 + 타이핑 인디케이터) — `app/components/ChatThread.tsx`
- [x] 8단계 스프린트 시뮬레이션 (PM → frontend → reviewer → QA) — `lib/demo-script.ts` + `lib/use-demo-sprint.ts`
- [x] [데모 시작] / [초기화] 버튼 — `DemoBoard` 에 와이어
- [x] 신청 폼 (이름, 이메일, 전화, 프로젝트, 팀 규모) → MM `#genasis-trial` — `SignupForm.tsx` + `/api/submit`
- [x] 토큰 기반 인증 정보 표시 페이지 — `app/status/[token]/page.tsx` + `CredentialsView.tsx`
- [x] trial.realstory.blog 배포 — `Dockerfile` + `docker-compose.yml` 배포 설정 완료 (라이브 배포는 운영 단계의 별도 작업)
- [x] 휴먼 협업 라이브 모드 (US-015..US-022): `Trial` flavor + `TrialPlaneProvider` / `TrialMattermostProvider` HTTP forwarder, 시뮬레이션 Plane/MM 상태 스키마, `/api/plane/*` + `/api/mattermost/*` 브릿지, SSE 브로드캐스터 (`/api/events/stream`), `LiveBoard` + `LiveChatThread` + 드래그-드롭 칸반 + 채팅 컴포저 + 채팅 사이드바
- [x] KO/EN i18n 토글 (`LangSwitcher`, `lib/i18n.ts`, Pretendard 폰트, 접근성 강화 — 커밋 572485b, a14fc11)

PRD: `agents-pool/prd/trial-webapp.md` (v2). 22개 user story 모두 `passes: true` (`trial-app/ralph/prd.json`).

### G.6 — `genasis init --trial` CLI 연동 — ✅ 커밋 de860ad

- [x] `cmd_init.rs`에 `--trial` 플래그 추가 (US-013) — `pub trial: bool` clap arg
- [x] 흐름: 빈 프로젝트 생성 → 에이전트 부트스트랩 → "체험 앱 실행?" → 브라우저 열기 — `cmd_init.rs::run_trial()` 가 `[trial]` 가 활성화된 최소 `genasis.toml` 작성 후 trial-app spawn 제안
- [x] 체험 앱은 localhost:3000에서 백그라운드 프로세스로 실행 — spawn 명령 설정 가능, 기본값 `npm --prefix /work/genasis/trial-app run start`
- [x] i18n 키 (en/ko)

### G.7 — `genasis example` 서브커맨드 — ✅ 커밋 de860ad

- [x] `genasis example prd` — 샘플 PRD.md 생성 (인증, CRUD, 반응형 UI를 갖춘 todo-app)
- [x] `genasis example design` — 샘플 design-system.md 생성 (색상/타이포/간격 토큰)
- [x] `genasis example prd2` — PRD2.md 생성 (로그인, 관리자 백오피스, 사용자 관리)
- [x] `cmd_example.rs` — 새 CLI 서브커맨드 (US-014)
- [x] 템플릿: `crates/genasis-cli/templates/examples/{prd.md,design-system.md,prd2.md}` (PRD 의 `agents/examples/` 가 아님 — 정적 `include_str!()` 임베드 자료라 동적 agents 카탈로그가 아닌 crate-local 로 배치)
- [s] i18n: 각 예제 문서의 en/ko 버전 — active-singularity 정책(ADR-008)에 따라 예제는 영어로만 배포. 한국어 미러는 향후 `cmd_example` 에 `--lang` 플래그 추가 시 도입 예정.

### G.8 — 튜토리얼 문서 — ✅ 커밋 d023cd9

- [x] `docs/TUTORIAL.md` (영어) — 5단계 빠른 경로 + 5개 연습
- [x] `docs/ko/TUTORIAL.md` (한국어 미러)
- [x] README 재구성: "빠른 체험" (5단계 → 튜토리얼 링크) + "단계별 가이드" (전체 제어) — `README.md` 의 `## Quick Path — Try Genasis in 5 Minutes` + `## Step-by-Step Guide` 섹션 확인, `README.ko.md` 도 같은 구조
- [x] CLAUDE.md 미러 테이블에 튜토리얼 쌍 추가 (`docs/TUTORIAL.md` ↔ `docs/ko/TUTORIAL.md`)

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
- 2026-05-04: **Phase D (Design Catalog Integration) 완료** — M-D1/M-D2/M-D3 일괄 진행. 사용자 결정 7개 모두 반영(pristine/external 두 모드, 외부 DESIGN.md read-only 강제, restore 명령, swap 후 사후 가이드, 충돌 시 사용자 의사결정→§B 누적, `--from <path>` 비-npx 진입점, EPIC 자동 임계치 4). awesome-design-md vendor 거부 → `npx getdesign` 위임 (manifest sha256 + 71 슬러그). 텔레메트리 default OFF. 신규 코드: `genasis-design/{mode,pointer,swap,restore,verify,override_log,ticket_emitter}.rs` + `genasis-monitor/widgets/design.rs` + `cmd_design` 5 서브커맨드. ADR-009 (en+ko). i18n 126 → 144 키. 누적 cargo test 145 passed (16 design + 1 swap_restore_round_trip + 3 epic_plan_and_overrides 신규). 결정 사항: §B 가 swap 시 초기화되는 동작은 의도된 것 — 새 §A 위에서 사용자가 재검토하도록 design-aware SKILL 이 안내.
- 2026-05-05: **Phase E (Dynamic Agents Catalog) E.0~E.7 완료** — E.7: crawl→verify→publish 파이프라인 실행 확인 (5소스 crawl, 492 파일 verified). agents/base/ genasis public repo에서 삭제 — base agent 파일은 agents-pool이 tarball로만 배포. publish.sh를 tarball 빌드+gh release upload로 변경. release-agents.yml을 verify-only로 전환. agents-pool/CLAUDE.md에 curation 전략·비공개 규칙 정의. .gitignore에 agents-pool/ + agents/base/ 추가.
- 2026-05-05: **Phase E (Dynamic Agents Catalog) 착수 + E.0~E.6 완료** — ADR-011 채택. `include_dir!()` 제거, GitHub Releases tarball 기반 런타임 fetch 모델로 전환. agents/ 디렉토리에 9역할 best-of-breed base agent (ECC/wshobson/VoltAgent/dl-ezo) + overlay .tera 20개 + commands 16 + hooks 6 + manifest 배치 (57 파일). genasis-templates crate를 fetch+cache+load 라이브러리로 리팩토링 (reqwest/flate2/tar/dirs). genasis-overlay merger/bootstrap를 AgentStore 인터페이스에 연결 (plan_attach, plan_bootstrap 시그니처 변경). CLI `genasis agents {fetch,status,update,list}` 서브커맨드 추가. release-agents.yml CI (agents-v* 태그 → tarball + sha256 자동생성). agents-pool skeleton (config.toml 5소스 + crawl/verify/publish scripts + .gitignore). M14 를 Phase E 로 흡수. 다음: E.7 crawl pipeline 실행 검증 → E.8 private repo push.
- 2026-05-05: **M14 (Default agentic team bootstrap) 사용자 제기 + 계획 반영**. 사용자 질의 — 빈 프로젝트에서 default agentic team scaffold 가 가능한지. 코드 audit 결과 `genasis-overlay` 는 attach/detach (기존 파일에 fence 주입/회수) 만 지원, base agent 생성 경로 부재. `templates/{en,ko}/agent-overlays/*.patch.md.tera` 도 patch 본문만 (frontmatter / 역할 헤더 없음). 의도적 누락이 아니라 마일스톤 순서가 닿지 않은 영역. blueprint §15 가 ECC 사실상 reference 사용자로 가정해 "agent 파일 이미 있음" 이 암묵적 전제였음. M14 신설로 base + patch 2-layer 구조 + ADR-010 (소유권 경계) + green-field 골든 픽스처 활성화 계획. v0.1.0 release tag 는 M14.0 (ADR-010 ratify) 이후로 이동.
- 2026-05-04: **M12 v6 audit + 잔여 항목 정리**. progress.ko.md 의 154개 unchecked 가 stale 인지 audit. 결과: M12.3~M12.13 의 거의 모든 작업이 commit 됐지만 per-sub-step 체크박스만 안 닫혀 있었음. 진짜 missing artifacts 채우기 (`logo.svg`, `architecture.svg`, `tests/install_lang_e2e.rs` 6 tests, `cmd_attach.rs --lang` clap conflict 해결 — global `--lang` 만 유지). 모든 M12.3~M12.12 sub-step `[x]` 또는 `[s]` (사유 명시) 로 closure. 누적 cargo test 120 passed. `/work/secusy/genasis/progress.md` (계획 시점 stale 사본) 도 live state 로 sync.
- 2026-05-10: **v0.5.1 패치 릴리즈 — 모니터 텍스트 선택 복구 + tmux Shift+drag 안내**. v0.5.0 dogfooding 중 보고 — `genasis monitor`에서 드래그 선택 / 더블클릭 / 트리플클릭이 전혀 동작하지 않음. 원인은 [`crates/genasis-monitor/src/app.rs`](crates/genasis-monitor/src/app.rs)에서 마우스 이벤트를 소비하는 위젯이 없는데도 `EnableMouseCapture`를 켜둬서 터미널의 native selection 레이어가 차단되던 것. `EnableMouseCapture` + `DisableMouseCapture` 제거(한 줄 수정 + "차후 위젯이 클릭이 필요해도 전역 활성화 대신 opt-in 플래그를 쓰라"는 코멘트). TUI wizard는 이미 캡처 OFF였고, [`key_hints.rs`](crates/genasis-tui/src/wizard/widgets/key_hints.rs) 하단 힌트 바 끝에 dim `Shift+drag select text (in tmux)` 안내 추가 — tmux mouse 모드 사용자가 표준 우회법을 발견하도록. [`docs/MONITOR.md`](docs/MONITOR.md) 와 한국어 미러에 트러블슈팅 3행 표 추가 (v0.5.0 이슈, tmux Shift+drag, screen copy-mode). 워크스페이스 버전 0.5.0 → 0.5.1, 릴리즈 노트 EN+KO 작성. `cargo test --workspace` 245 passed, 4 ignored — 회귀 없음.

- 2026-05-10: **사람 로스터 프로비저닝 — 사람을 일급 팀원으로 (ADR-014)**. 기존 `genasis init`/`bootstrap`은 에이전트 봇 계정 10개만 자동 생성하고 사람은 별도 가입이 필요했음 → "turnkey bootstrap" + "사람-에이전트 대칭" 미션 위배. `genasis-core::config::HumanEntry` + `[[humans]]` 배열 도입, `.genasis/humans.lock.toml` (Mattermost user_id, Plane user_id, 임시 비번) 분리. `MattermostProvider::ensure_human_user(spec, team_id)` 트레잇 메서드 + 업스트림 admin-create 구현 (24자 고엔트로피 임시 비번 + 첫 로그인 시 변경 강제, idempotent on email). `provision-plane-users.mjs`의 `ProvisionInput`에 `humans: HumanRequest[]` 추가 (Playwright 자동화는 stub 유지 + humans echo). `genasis humans add | edit | remove | list | sync` CRUD CLI 신설, `cmd_init`이 `[[humans]]` 비어있지 않으면 자동 sync 호출 (실패는 warning, init 자체는 성공). TUI wizard 6단계 → 7단계 (Env→Lang→Team→Connect→**Humans**→Overlay→Done), `a/e/d/s/Enter` 키로 add/edit/delete/sync/advance + 5필드 form 모달, wizard 재실행 시 `[[humans]]` 자동 로드해 in-place 편집 ("rerun is the editor"). `agents/GENASIS.md.tera`에 `## 사람 로스터` 표 + `### 요구사항 수신 프로토콜` (등록자 = binding stakeholder, 미등록자 = QUESTION 라벨 + PM 검증, 봇 = 기존 에이전트-에이전트). `pm.patch.md.tera` / `planner.patch.md.tera` (en/ko) + `commands/check-inbox.md.tera`도 동일 프로토콜 mirror. ADR-014 EN/KO 작성. 신규 단위 테스트: HumansLock 라운드트립, upsert 케이스 무시 매칭, derive_mm_username 정상화, cmd_humans truncate/now_iso. `cargo test --workspace --lib` 통과. 미구현으로 남기는 영역: invite-email 모드 (SMTP 활성 환경 대상, v2), Plane Playwright UI 포트로 실제 user_id 연결, OAuth/SSO 인테그레이션.

- 2026-05-10: **Trial 브릿지 설정 SSOT 정리 (ADR-013)**. 기존 코드는 `[trial]` 섹션을 정의만 해두고 실제 라우팅에는 `[plane].url` / `[mattermost].url` + `MM_ADMIN_TOKEN` / `PLANE_API_KEY` 환경변수를 사용해, `[trial].enabled = false`로 trial-app을 끌 수 없거나 `[trial].url` 변경이 무시되는 등 죽은 설정 문제. `mattermost::factory::build()` / `plane::factory::build()` 시그니처에 `Option<&TrialConfig>` 추가, `flavor = Trial`일 때 `[trial].url` / `[trial].shared_secret` 사용 + `enabled = true` 강제. `Config::load()`에 `validate_trial()` cross-section 검증 추가. `cmd_init` / `cmd_mm` / `cmd_plane` / `cmd_humans` 모두 trial flavor에서 admin 환경변수 요구 면제. 신규 단위 테스트 10개 (factory build_trial_*, validate_trial_*) + integration `tests/trial_factory_e2e.rs` 3개(2개 #[ignore]ed E2E + 1개 negative path). ADR-013 EN/KO 양쪽 작성. `cargo test --workspace` 245 passed, 4 ignored.
- 2026-05-15: **v0.6.0-alpha.15 — monitor 의 전 widget 데이터 wiring 완성 + Log timestamp + 살아있는 LiveHeartbeat (agents-pool 측)**. 사용자 "Tokens/Network/SESSIONS 가 업데이트 안 됨 + Log 시각 안 보임 + 이미 완료된 카드가 done 안 됨" 보고 따른 monitor 전면 fix.

  **D-073 — JSONL scan 의 root-cause fix** (alpha.14 빌드에 들어갔으나 사용자가 monitor 재시작 안 해서 효과 못 봤던 부분 재확인 + alpha.15 ship):
  - `scan_sessions_dir()` 가 옛 `~/.claude/sessions/*.jsonl` (오늘은 per-PID JSON status 파일 위치) 가 아닌 `~/.claude/projects/<encoded-cwd>/<session>.jsonl` 재귀 스캔.
  - `scan_single_file` 이 `assistant` event 의 `message.usage.{input/output/cache_read/cache_creation}_tokens` 와 `message.model` 추출. 옛 `usage`/`api_response` event 도 fallback.
  - ISO 8601 timestamp (`2026-05-15T01:41:28.718Z`) → epoch 변환. 이전엔 string 을 u64 로 parse 시도해서 0 으로 떨어지고 모든 event 가 5h/7d window 의 from 보다 작아 0 토큰으로 집계.
  - 라이브 검증: files_scanned=216, 5h_input=162K, 5h_output=848K, ctx_model=`claude-opus-4-7`.

  **D-082 — counters never populated 결함 fix**:
  - `state.mcp_calls` / `state.mcp_cache_hits` 가 alpha.14 까지 어디서도 할당 안 되어 영원 0. JSONL 의 `assistant` event 1 건당 mcp_calls +1, `cache_read_input_tokens > 0` 이면 mcp_cache_hits +1. 라이브: mcp_calls=663, hit=648, hit_pct=97.7%.
  - `state.plane_calls` / `state.mm_calls` 도 같은 결함. 사용자 의도 ("trial-app 일 경우 칸반 / 채팅 데이터 기반") 반영: `collect_trial` 가 sim_issues 총합 = `plane_calls`, sim_posts 총합 = `mm_calls` (baseline 차감 — monitor 시작 후 delta). `network_bytes` 도 두 카운트의 합 * 256.
  - `state.anthropic_cache_hit_pct` = cache_read / (cache_read + cache_create + input) 결정적 산식 적용.

  **D-081 — Log widget HH:MM 24h prefix**:
  - listen.log 의 verbose ISO timestamp (`2026-05-15T01:41:28.718972Z  INFO ...`) 를 local timezone 의 `01:41  INFO ...` 으로 reformat. `reformat_with_local_time` 헬퍼가 chrono::DateTime::parse_from_rfc3339 + `%H:%M` 포맷. ISO prefix 없는 line 은 verbatim.

  **D-072 sessions detection 검증 + 재확인** (alpha.14 의 fix 가 사용자 monitor PID 649794 의 deleted-inode 옛 binary 메모리 image 때문에 효과 못 봤음):
  - source 의 `is_project_path_relaxed` 가 빈 `project_root` 일 때 모든 claude 프로세스 표시. 라이브 검증: 165 sessions detected (`/work/genasis` 의 vscode-server claude 등 모두).
  - 사용자 안내: monitor 재시작 (kill + 새 실행) 만 해도 alpha.14 의 D-073/D-072 가 즉시 적용. alpha.15 binary 도 같은 fix + counters wiring 까지 포함.

  **카드 정리** (사용자 "이미 개발 완료인데 #6/#7/#8/#9 가 done 안 가 있어" 보고):
  - sandbox 의 inprogress / inreview 카드 (#327/#328/#329 quiz + #342 scaffold) 를 `/api/trial/bootstrap` 으로 일괄 done 으로 transition. `#313 Test card 1` 만 todo 로 남김 (자가테스트 중 manual 로 만든 흔적).
  - 자율 cleanup 결함 (overlay 에 "QA done 자동" enforce 부재) 은 별 사이클 (D-083).

  **mmplane-trial.realstory.blog/trial-app 측 별 ship** (agents-pool repo commit `2837868`):
  - D-074 ShowcasePanel localhost 입력란 제거.
  - D-077 AppBar 토큰 persistence (URL > cookie > localStorage 3중).
  - D-078 LiveHeartbeat — 3초 폴링 + pulse animation + agent activity surface.
  - D-079 sim_posts / transitionIssue / setTeamApp / announce_dev_server_url 가 sim_agent_activity 로 mirror — heartbeat 가 agent 의 모든 trial-app 흔적 즉시 surface.
  - submit → RealStoryBlog 팀의 `#genasis-cowork-rental` 채널 (genasis-trial-bot + PAT + .env 의 MM_BOT_TOKEN / MM_TRIAL_CHANNEL_ID 갱신).

  **`cargo build --workspace` 0 errors, runtime smoke (files_scanned=216 / mcp_calls_5h=663 / detect_sessions=165) 통과**.

  **다음 사이클 후보**:
  - D-083 — QA / PM overlay 에 "in-review 카드 자동 done transition" enforce.
  - D-075-Rust — daemon `SessionEvent::ToolUse` 직접 `/api/trial/agent-activity` POST forward (sim mirror 보다 더 풍부한 internal Bash / Edit / Read 도 surface).
  - testbed clean reset → README §Quick Path 1-5 새 사용자 시점 재검증.

- 2026-05-14: **v0.6.0-alpha.14 — D-072 monitor 자동 sandbox 발견 + config_hint banner surface**. 사용자가 testbed root 에서 `genasis monitor` (no args) 실행 시 widget 들이 silently 비어 있던 UX 결함 fix.

  **사용자 제기 의심**: testbed root `/work/agenteams/team-ex/` 에 `genasis.toml` / `.claude/agents/` 없는데 agentic team 이 진짜 trial-app 과 연동 중인지 의심.

  **결과**: 의심은 testbed 구조의 한 단계 깊이 (sandbox sub-dir `alpha9-trial/`) 를 못 본 데서 비롯. 실제 결정적 검증:
  - sandbox `/alpha9-trial/.claude/agents/` 에 10 개 agent .md (architect/backend/code-reviewer/designer/devops/frontend/planner/pm/qa/security) 모두 정상
  - `genasis.toml` 의 `[trial]` 섹션 `enabled=true` + `team_token=76b03286…` 정상
  - PM overlay 에 D-062 fix 들어 있음 (`자동 devops dispatch` 1 match)
  - sandbox 안에 frontend 가 작성한 `src/components/QuizApp.tsx` (193 lines) + `src/lib/quiz-bank.ts` (237 lines) 진짜 코드 430 lines
  - 사용자의 "Cyan 색으로 변경해" 메시지가 14:30:08 daemon 도착 → Agent Task → frontend → `src/styles.css:264` `.btn-primary` cyan `#06b6d4` 진짜 수정 → transition_issue done → post_message wrap-up까지 141 초 안에 자율 완료

  **하지만 UX 결함은 별개 사실**: 사용자가 정확한 sandbox path 를 cd 또는 `--project <dir>` 로 지정해야만 monitor 가 trial 데이터 잡음. testbed root 에서 그냥 `genasis monitor` 하면 walk-up 으로 cfg 못 찾고 trial_mode=false → 모든 widget 비어 있음. D-058 fix 가 `state.config_hint` 까진 push 하지만 widget 어디서도 render 안 함 (D-058 incomplete).

  **D-072 (Critical UX) 두 갈래 fix**:
  - `genasis-core::Config::discover_or_descend` 신규 — walk-up 실패 시 cwd 의 자식 1 단계까지 search. `.` 시작 / `node_modules` / `target` 제외 + 정렬 결정성. testbed root 에서 monitor 띄우면 자동으로 `alpha9-trial/genasis.toml` 발견. 발견 시 사용자에게 banner hint ("ℹ Auto-discovered sandbox at …").
  - `monitor::widgets::log_tail` 가 `state.config_hint` 를 widget 상단의 sticky 첫 줄로 surface — 노란색 (⚠ 미발견) / cyan (ℹ 자동 발견). log_tail 이 많이 차도 hint 안 밀려남.

  **D-058 마무리**: 잘못된 dir 에서 monitor 실행 시 hint 가 silently state 에만 남고 화면에 안 보이던 결함 해결. 같은 fix bundle 에 묶음.

  **단위 검증**:
  - `discover_or_descend(/work/agenteams/team-ex)` → `…/alpha9-trial/genasis.toml` ✅
  - `discover_or_descend(/work/agenteams/team-ex/alpha9-trial)` → 같은 path (walk-up zero-hop) ✅
  - `discover_or_descend(/tmp)` → None (hint 띄울 준비) ✅

  **사용자 안내 갱신**: 이제 `cd /work/agenteams/team-ex && genasis monitor` 만 해도 자동으로 sandbox 데이터 (Sprint 카드 카운트 + Log widget 의 listen.log follow) 표시. banner 라인에 자동 발견 path 명시. `--project <dir>` flag 는 여전히 지원 — 여러 sandbox 가 같은 testbed 에 공존할 때 명시적 선택용.

- 2026-05-14: **v0.6.0-alpha.13 — D-061/062/063/065 bundle hotfix + agents-v1.0.3 catalog publish**. 사용자 지시 "남아 있던 오류 수정 모두 진행해" 따라 네 가지 결함 한 사이클에 묶어 ship.

  **D-062 (자율 dispatch chain) — overlay 3개 + system prompt 동시 강화**:
  - `pm.patch.md.tera (ko/en)` Step 3 옆 신규 Step 4 — "코드 변경 동반 요청이면 frontend Task 끝난 직후 같은 turn 안에서 **반드시** devops Task 도 호출". 본문에 구체적 Task() 예시 (npm install → npm run dev `&` → ss verify → announce_dev_server_url → post_message). "PM 본인은 인프라 / server-survival 에 관여 안 함 — devops 의 영역" 명시.
  - `devops.patch.md.tera (ko/en)` — "호출 trigger" 섹션 신설 (PM 의 dispatch chain 마지막 hop), 작업 흐름에 `ls package.json` 사전 확인 + foreground `npm run dev` 금지 + `&` 백그라운드 강제 + `mcp__trial-app__announce_dev_server_url` 호출 명시. 금지 조항에 "announce_dev_server_url 빼먹기" 추가.
  - `frontend.patch.md.tera (ko/en)` — "standalone scaffold 보장" 조항 신설. PRD 가 sub-path 를 가리켜도 그 path 에 `package.json` + `vite.config.ts` 까지 완성. `npm run dev` 는 frontend 본인 안 띄움 (devops 영역) 명시. 작업 완료 보고에 `cwd=<abs path>` 명시 — devops 가 그 path 에서 시작.
  - `session.rs::build_append_system_prompt` — CRITICAL 블록 확장. frontend Task → devops Task → announce_dev_server_url chain 명시 + "Without that announce call, the user's ShowcasePanel iframe shows `localhost refused to connect` and the turn looks broken" failure cost 명시.

  **D-061 (Critical) — `genasis example prd` PRD §5 를 v0.6.0 model 로 재작성**:
  - `crates/genasis-cli/templates/examples/prd.{en,ko}.md` §5 ("Trial-app integration") 전면 rewrite. v0.5.x 의 "agents-pool/trial-app/ source tree 안에 작성" 모델 폐기, v0.6.0 의 "사용자 sandbox 안에 standalone Vite scaffold (package.json + vite.config.ts + src/main.tsx + components/QuizApp.tsx + lib/quiz-bank.ts) + devops 가 `npm run dev` 백그라운드 spawn + `announce_dev_server_url` 호출 → ShowcasePanel iframe 자동 prefill" 흐름으로 변경.
  - 결과: 다음 사이클부터 `genasis example prd` 가 생성하는 PRD 가 dev server 띄울 수 있는 standalone scaffold 를 가리킴 → frontend agent 가 그대로 따르면 devops 가 spawn 가능 → ShowcasePanel iframe 에 진짜 결과 표시.
  - 기존 v0.5.x model 은 ADR-016 참조 안내로 fallback 만 유지.

  **D-063 — trial-app `set_app_features([])` 가 LRU-append 시맨틱 때문에 reset 안 되던 버그**:
  - `app/api/trial/team-app/status/route.ts` 의 조건문 `app_features && app_features.length > 0` → `app_features !== undefined` 로 변경. 빈 배열 명시 시 update 트리거.
  - `db/sim.ts::setTeamApp` 시그니처 `add_features: string[]` → `string[] | null` 로 확장. **null = preserve, [] = explicit reset, [...] = LRU-append** 세 가지 시맨틱 분기 추가.
  - `app/api/trial/bootstrap/route.ts` 호출자도 동일 시맨틱으로 정렬.
  - typecheck PASS. 운영자 trial-app 재배포 필요 (npm run build + restart).

  **D-065 (partial) — `genasis monitor` Log widget 이 daemon listen.log 를 follow 못 해 비어 있던 문제**:
  - 신규 `crates/genasis-monitor/src/collector/listen_log.rs` 작성 — `<project_root>/.genasis/listen.log` 의 마지막 80 줄 (16KB) 을 3 초 tick 으로 read, ANSI escape 제거 후 `state.log_tail` 로 push. 첫 호출은 file 의 끝쪽 16KB 부터 시작 (장기 daemon 의 한 번에 수천 줄 dump 방지). 이후 offset 기억하여 새 line 만 emit.
  - `state.rs` 에 `project_root: Option<PathBuf>` + `listen_log_offset: u64` 추가. `load_trial_config` 가 cfg 발견 시 `cfg_path.parent()` 를 project_root 로 저장.
  - `app.rs` run_loop 에 `LISTEN_LOG_TICK = 3s` 추가.
  - 결과: 사용자가 `genasis monitor --project /path/to/sandbox` 실행 + daemon 떠 있으면 Log widget 에 "session tool_use tool=mcp__trial-app__post_message ..." 같은 실시간 활동이 흐름.
  - Agents widget 의 "wire SessionStart hook" hint + RTK/JSONL/Network counters 는 별개 결함 (**D-066** 다음 사이클).

  **agents-v1.0.3 catalog publish** — `agents-pool/scripts/publish-overlays-only.sh 1.0.3 --from 1.0.2`. `agents-v1.0.3.tar.gz` GitHub Releases 업로드 완료. 사용자 sandbox 에선 `genasis agents update` 또는 `genasis upgrade` 로 새 overlay 받음.

  **빌드 검증**: `cargo build --workspace` 0 errors, trial-app `npx tsc --noEmit` PASS, overlay 6파일 + session.rs + PRD template 2 + monitor 3파일 + trial-app 3파일 변경.

- 2026-05-14: **v0.6.0-alpha.12 — D-060 Task tool dispatch enforce + 추가 결함 D-061/D-062 발굴**. alpha.11 라이브에서 PM 이 카드만 만들고 frontend 호출 안 하는 D-060 증상 확인 → overlay + append_system_prompt 강제 조항 추가. 사용자 라이브 follow-up 으로 검증.

  **D-060 — PM 이 create_issue 후 Task tool 로 sub-agent invoke 안 함**:
  - 사용자가 "@pm — 카드 #327/#328/#329 를 frontend 에게 Task tool 로 디스패치해서 코드 작성 시작" 요청 직접 발송 → PM 이 정상 응답 + Frontend 호출 + **실제 코드 작성**.
  - 산출물 (frontend agent 가 사용자 sandbox 안에 진짜 작성한 파일):
    - `/work/agenteams/team-ex/alpha9-trial/agents-pool/trial-app/app/components/QuizApp.tsx` (503줄, "use client" + useReducer + mulberry32 seed + 3-screen 플로우)
    - `/work/agenteams/team-ex/alpha9-trial/agents-pool/trial-app/lib/quiz-bank.ts` (423줄, 17개 질문 beginner/intermediate/advanced)
  - 카드 #327/#328/#329 → inreview 로 transition. PM 의 wrap-up post (id=310) 가 산출물 경로 + 스코프 충족 항목 자세히 보고.
  - **Fix bundle**:
    - `agents/overlays/{en,ko}/pm.patch.md.tera` 의 §"요구사항 인테이크" Step 3 을 강제 조항으로 재작성 — "create_issue 다음엔 **반드시** Task tool 로 각 카드 assignee 를 invoke. 카드만 만들고 turn 종료 금지. 카드 1 개당 정확히 1 회 `Task(subagent_type=...)`. devops 가 dev server 띄우면 `announce_dev_server_url` 호출 필수."
    - `session.rs::build_append_system_prompt` 에 별도 CRITICAL 블록 추가 — overlay 가 stale 인 환경에서도 system prompt 만으로 enforce.

  **D-061 (Critical) — `genasis example prd` 가 생성하는 PRD §5 가 v0.5.x 시대 경로 가정**:
  - PRD.md §5 본문: "The implementation lives inside the trial-app's source tree at `agents-pool/trial-app/app/components/QuizApp.tsx` … and `agents-pool/trial-app/lib/quiz-bank.ts`". 이건 운영자의 trial-app 본체 source tree path 를 가정. frontend agent 가 PRD 충실하게 따랐고 사용자 sandbox 안 `agents-pool/trial-app/...` subdir 에 그대로 작성. 하지만 그 경로엔 `package.json` / `next.config.js` 가 없어서 standalone Node project 가 아님 → `npm install` + `npm run dev` 가 가능한 위치가 아님 → dev server 안 뜸 → ShowcasePanel iframe `localhost:5173 refused to connect`.
  - 근본 원인: v0.5.x 까지는 trial-app 호스팅 인스턴스가 `QuizApp.tsx` 를 hard-coded 으로 임포트해서 렌더링 → agentic team 이 trial-app 본체 PR 보내는 모델. v0.6.0 부턴 ShowcasePanel 의 LocalDevServerOrFallback 가 사용자 컴퓨터 localhost dev server 를 iframe 으로 받는 모델. **PRD template 이 새 모델과 불일치**.
  - 다음 사이클 fix: `genasis example prd` template 을 수정. PRD §5 가 "사용자 sandbox 안에 standalone Next.js (또는 Vite) scaffold 생성 + dev server (예 `http://localhost:5173`) 띄움 + devops 가 `announce_dev_server_url` 호출" 모델로 재작성.

  **D-062 — PM 이 frontend 완료 후 devops 호출 안 함**:
  - alpha.12 검증 turn 에서 PM 이 frontend Task 까진 했지만 devops agent (`npm install` + `npm run dev` 책임) 호출 안 함. PM wrap-up: "다음 단계: QA 가 inreview 3건 검토 후 done 으로 이행." — dev server / devops 부재.
  - overlay 의 §"배포 (DEPLOY 결정)" 조항이 색상 변경 vs 코드 변경 분기를 명시하지만 PM 이 자동 실행 안 함. D-060 fix 흐름에 devops 호출 강제 1조항 추가 (D-060 fix 안에 포함됨 — `announce_dev_server_url` 필수 명시).
  - 다음 사이클 검증: PRD §5 fix (D-061) 후 frontend → devops 자동 흐름이 실제 도는지 라이브 확인.

  **D-063 — `set_app_features([])` 가 LRU-append 시맨틱이라 reset 안 됨**:
  - PM 이 이전 잘못된 `["dark-mode","i18n"]` 을 비우려 `set_app_features({features:[]})` 호출했지만 trial-app 백엔드의 처리 시맨틱이 LRU-append 라서 빈 배열이 노op 처럼 동작. `/api/trial/team-app/status` 응답에 `["dark-mode","i18n"]` 그대로 남음.
  - 우선순위 낮음 (시각적 우선순위는 "가장 최근 set" 이라 ShowcasePanel 렌더에 영향 작음). trial-app 백엔드에 명시적 reset 엔드포인트 추가 또는 features 빈 배열 시 DELETE 시맨틱으로 보정 필요. 다음 사이클 잡일.

- 2026-05-14: **v0.6.0-alpha.11 — D-059 `bypassPermissions` hotfix**. alpha.10 ship 직후 사용자 라이브 검증에서 새로운 blocker 발견.

  **D-059 (Critical) — PM 이 MCP tool 호출 권한을 거부당해서 chat 패널에 답 못 함**:
  - 사용자가 "D-059 검증 — TODO 앱 만들어줘" 메시지 보냄. daemon log 보니 PM 이 `mcp__trial-app__post_message`, `set_app_kind`, `set_app_features` 를 호출하려 하는데 PM 의 최종 assistant text 가 "trial-app MCP 도구 권한이 필요합니다 (`post_message`, `set_app_kind`, ...)". 결과: PM 의 호출은 전부 reject 됨, sim_posts / sim_issues 에 흔적 안 남음.
  - 원인: `session.rs::spawn` 이 `--permission-mode acceptEdits` 로 claude 띄움. 이 모드는 Edit/Write 같은 file tool 만 자동 허용 — MCP tool 호출은 prompt 흐름으로 빠지는데 stream-json non-interactive 모드라 응답할 곳이 없어 **자동 거부**.
  - Fix — `--permission-mode bypassPermissions`. agentic team 의 cwd 는 사용자 sandbox 로 제한돼 있고 어차피 진짜 코드를 작성하는 게 본 임무라 surface 통제 가능. 한 줄 변경.
  - 라이브 재검증: 같은 메시지 패턴 ("D-059 검증 — TODO 앱 …") 으로 daemon restart 후 재테스트. 결과:
    - 02:54:18 사용자 메시지 수신
    - 02:54:29 PM 이 `post_message` 호출 → sim_posts id=304 actor=pm "ack — TODO 앱(다크모드 + i18n) 작업 시작합니다. 카드 생성 후 프론트엔드로 디스패치." 정상 INSERT
    - 02:54:30 `set_app_kind(kind="todo")` + `set_app_features(["dark-mode","i18n"])` 정상
    - 02:54:31 `create_issue(title="TODO 앱: 다크모드 + i18n", assignee="frontend", state="inprogress")` → sim_issues id=320 inprogress 카드 생성
    - 02:54:33 turn complete success=true duration_ms=12960 (12.9초)
    - `/api/trial/team-app/status` 응답: `{"app_kind":"todo","app_features":["dark-mode","i18n"],"app_status":"complete"}` — ShowcasePanel toggle 이 unlocked 상태로 전환
  - **결정적 검증** — PM 이 채팅 패널, 칸반, 쇼케이스 토글 셋 다 MCP tool 로 정상 갱신.

  **남은 후속 (D-060, 다음 사이클)**: PM 이 카드를 생성하고 frontend 에 assign 했지만, 실제로 Task tool 로 frontend sub-agent 를 invoke 하지는 않음 → 코드 작성 / dev server 띄우기 단계 진입 안 함. overlay 의 "Hand each subordinate role off via Task tool sub-agent invocation" 조항이 enforce 안 되는 듯. 다음 사이클에서 PM prompt 보강 + Task tool 흐름 검증.

- 2026-05-14: **v0.6.0-alpha.10 — D-057 MCP path baking hotfix + D-058 monitor `--project` flag**. alpha.9 ship 직후 자가테스트 + 사용자 채팅에서 두 가지 blocker 발견 → 한 사이클에 hotfix.

  **D-057 (Critical) — MCP server 경로가 CI 머신 경로로 박혀 사용자 환경에서 안 떴다**:
  - daemon log 의 `claude` argv 에 `"args":["/home/runner/work/genasis/genasis/mcp-servers/trial-app/index.mjs"]` — release CI 머신의 경로.
  - 원인: `cmd_listen.rs` 의 default mcp_server_dir 가 `env!("CARGO_MANIFEST_DIR")` 였는데, 이 매크로는 **컴파일 타임** 의 manifest 경로를 박는다. release 바이너리는 GitHub Actions 의 `/home/runner/work/genasis/genasis/...` 가 박힌 채 ship 된다 → 사용자 머신엔 그 경로 없음 → `node` 가 `Cannot find module` 으로 실패 → claude session 의 `mcp__trial-app__*` tool 미등록 → PM 이 사용자에게 답할 수단 자체가 사라짐.
  - 추가 문제: 사용자가 `@modelcontextprotocol/sdk` 를 npm-global 에 설치 안 한 경우 NODE_PATH 가 가리키는 곳에 SDK 가 없어서 같은 증상.
  - Fix — 신규 `crates/genasis-cli/src/mcp_bundle.rs`:
    - 3 개 MCP server 의 `index.mjs` 본문을 `include_str!` 로 바이너리에 임베드 (총 ~30KB)
    - 첫 호출 때 `~/.cache/genasis/mcp-servers/<name>/index.mjs` 로 unpack (sha256 hash 일치하면 skip)
    - 같은 cache 디렉터리에 `package.json` 작성 + `npm install --prefix <cache>` 로 `@modelcontextprotocol/sdk` 받기 (첫 호출만, 그 후 skip)
    - `McpBundle { server_dir, node_modules }` 반환
  - `build_mcp_config` 시그니처에 `node_modules: &Path` 추가 — NODE_PATH 를 cache 의 node_modules 로 고정 (이전엔 `npm root -g` 였음).
  - `GENASIS_MCP_SERVER_DIR` env 는 여전히 override 로 동작 (개발 / 디버깅).
  - 로컬 release 빌드 smoke: `~/.cache/genasis/mcp-servers/` 에 npm install 정상 (~10s), claude session init log 에 `mcp_servers=["trial-app", ...]` 등장, node MCP child PID spawn 확인, 사용자 메시지 받은 후 3 초 만에 PM 응답 (`session turn complete success=true duration_ms=2999`).
  - 남은 후속 (D-058 별개): PM 이 응답 텍스트를 assistant text 로만 emit 하고 `mcp__trial-app__post_message` 호출은 안 함 — chat 패널에 안 보임. overlay 의 5초 ack rule 가 실제로 enforce 안 됨. 다음 사이클에서 D-059 로 추적.

  **D-058 — `genasis monitor` 가 잘못된 디렉터리에서 실행되면 모든 widget 이 빈 상태**:
  - 사용자가 `/work/agenteams/team-ex` 에서 `genasis monitor` 실행 → Sprint Todo:0 In:0 Review:0 Done:0, Agents "no agent activity collected", Log "no log lines yet". 실제 라이브 trial-app 은 카드 4 + 채팅 2 가 보임.
  - 원인: `monitor::app::run` 가 `std::env::current_dir()` 에서 `Config::discover` 호출 → walk-up 으로만 검색 → testbed 루트엔 `genasis.toml` 없음 → silently `load_trial_config` 가 early return → trial_mode=false → 모든 trial widget 빈 상태.
  - Fix:
    - `cmd_monitor::Args` 에 `--project <DIR>` flag 추가.
    - `monitor::app::run(project_root: Option<PathBuf>)` 시그니처 — Some(p) 이면 그 디렉터리부터 walk-up 검색.
    - 검색 실패 시 silently early-return 대신 `state.config_hint` + `state.log_tail` 에 명확한 hint ("⚠ genasis.toml not found walking up from ... Run `genasis monitor` inside your project sandbox, or pass `--project <dir>`.") 푸시.
    - `AppState` 에 `pub config_hint: Option<String>` 추가.

  **빌드 검증**: `cargo build --workspace` 0 errors, `cargo fmt --check` clean, smoke test 으로 daemon → MCP child → claude session init → tool 등록 → PM 응답 turn 전체 통과.

- 2026-05-14: **v0.6.0-alpha.9 — D-054 마무리 + beta real MCP server + M-v6.0.4 multi-team session map**. alpha.8 이후 남아 있던 세 가지 pending 작업을 한 사이클에 ship.

  **D-054 마무리 — 시뮬레이션 시대 잔재 제거**:
  - `crates/genasis-cli/src/listen/{routing.rs, sdk.rs}` 모듈 자체 삭제 (deprecation stub 으로 alpha.7 부터 비어 있던 상태). `mod.rs` 의 `pub mod sdk;` 선언도 함께 제거. v0.5.x marker 파싱 / fresh-spawn `run_claude_agent_sdk` 흔적 완전 소거.

  **beta — real Mattermost / Plane MCP server 구현**:
  - `mcp-servers/mattermost/index.mjs` (+ package.json) — `@modelcontextprotocol/sdk` stdio server. tools: `post_message` (Bearer 인증 + 채널명→UUID 캐시 + `actor` prefix), `list_posts`, `list_channels`, `update_post`. env: `MM_URL` / `MM_ADMIN_TOKEN` / `MM_TEAM_ID` / `MM_DEFAULT_CHANNEL_ID`.
  - `mcp-servers/plane/index.mjs` (+ package.json) — tools: `create_issue` (title idempotency + state UUID 자동 매핑 + PLANE_USER_ID_<ROLE> 환경변수 → assignee UUID), `transition_issue`, `list_issues`, `list_states`. Plane state UUID 매핑: 첫 호출 시 `/states/` GET → "todo"/"inprogress"/"inreview"/"done" alias 를 backlog/started/completed group 으로 fallback.
  - `crates/genasis-cli/src/listen/session.rs::build_mcp_config` 에 `real` / `auto` flavor 분기 추가 — `MM_URL` + `MM_ADMIN_TOKEN` env 있으면 mattermost server 등록, `PLANE_URL` + `PLANE_API_KEY` 있으면 plane server 등록. `PLANE_USER_ID_*` env 는 자동 전파. NODE_PATH 도 trial 과 동일하게 `npm root -g` 자동 탐지.
  - overlay 본문은 trial 과 동일 — 같은 tool 인터페이스 (`mcp__mattermost__post_message`, `mcp__plane__create_issue` 등) 로 flavor swap.

  **M-v6.0.4 — multi-team sandbox (HashMap<team_token, ClaudeTeamSession>)**:
  - `InboundEvent::PostCreated` 에 `team_token: String` 필드 추가. trial_sse / mattermost_ws 두 stream 모두 자기 구독 team_token 을 모든 이벤트에 부여.
  - `MattermostWsStream::connect` 시그니처에 `team_token: String` 추가 — 운영자가 N 개 인스턴스 동시 호스팅 가능. `cmd_listen` 은 `MM_TEAM_ID` env (없으면 mm.url) 를 team key 로 사용.
  - 신규 `pub type SessionFactory = Box<dyn Fn(&str) -> Pin<Box<dyn Future<Output = Result<...>> + Send>> + Send + Sync>;` + `pub async fn run_listen_loop_multi(stream, cfg, factory)`. team_token 별로 `HashMap<String, ClaudeTeamSession>` lookup → 없으면 factory 호출 lazy spawn. drain task 도 team 별로 분리 (`team` field 로깅).
  - `cmd_listen::run_foreground` 가 closure 로 factory 합성 — `project_root` / `flavor` / `trial_url` / `mcp_server_dir` 캡처해서 호출 시점에 team_token 만 받아 session spawn. 단일 team 케이스도 같은 코드 경로 (현재 stream 들은 1 team 만 emit 하므로 HashMap 에 1 key 만 들어감 — 향후 multi-stream 확장 시 그대로 N key).
  - `run_listen_loop_session` 는 `run_listen_loop_multi` 로 대체 (호출자는 cmd_listen 하나).

  **결과**: alpha.9 는 v0.6.0 의 "프로토콜 + 데몬 + MCP server + multi-team" 4 축이 다 갖춰진 첫 빌드. real flavor 도 trial 과 동일 overlay 본문으로 동작 가능. `cargo build --workspace` 0 errors, `cargo fmt --check` clean.

  **남은 작업 (alpha.10 / beta)**:
  - real flavor live 검증 (실제 Mattermost + Plane 인스턴스 대상 end-to-end smoke). 운영자가 PAT 발급 + `[[humans]]` 등록 + `genasis listen --real` 실행 → 같은 PM 메시지가 trial 과 동일한 lifecycle 통과.
  - multi-team end-to-end 검증 — operator 인스턴스 한 개에서 team_token 2 개 SSE 동시 수신 → 두 session 이 독립 spawn / 독립 sandbox 에서 작업.
  - agents-v1.0.3 catalog publish (overlay 변경 없으면 skip).

- 2026-05-14: **v0.6.0-alpha.5 — lazy session init + agents-pool overlay swap + 라이브 결정적 검증 (P3+P4 통합 완성)**. 사용자 §"alpha.5 진행 + 그 뒤 모든 것 다 진행" 명령 따른 본질 검증 사이클.

  **lazy session init (D-047)**:
  - 첫 시도에서 `ClaudeTeamSession::spawn` 이 15s timeout 으로 fail 했던 원인 — claude stream-json 모드는 stdin 으로 첫 NDJSON 메시지가 도착할 때까지 init event 안 발행 (stdin EOF 또는 첫 input 까지 대기)
  - 수정: session spawn 시 init 동기 wait 제거. subprocess + stdin/stdout pipe 만 열고 background drain task 가 stdout 의 NDJSON 을 SessionEvent enum 으로 변환 — init / assistant / tool_use / result 가 같은 stream 에 섞여 옴
  - 결과: spawn 즉시 ready, 첫 사람 메시지 send 후 5-30 초 안에 init + 응답 시작
  - `build_mcp_config` 의 trial-app server env 에 `NODE_PATH` 추가 — server 의 `require('@modelcontextprotocol/sdk')` MODULE_NOT_FOUND 방지

  **사용자 sandbox overlay 수동 sync (publish 우회)**:
  - `/tmp/apply_overlay_v0_6.py`: 본 repo `agents/overlays/en/*.tera` 를 Tera 변수 보간 (project_name=v516 Final, project_slug=v516-final, flavor=trial) 후 사용자 sandbox `.claude/agents/<role>.md` 의 GENASIS:BEGIN/END marker 사이에 직접 삽입
  - `/tmp/patch_frontmatter.py`: frontmatter `tools:` 필드를 role 별 v0.6.0 spec 으로 갱신 (예 pm: Read,Bash,Task; frontend: Read,Edit,Write,Bash,Task)
  - 10 role 모두 갱신 — 정식 publish 워크플로우는 agents-pool 의 verified/ cleanup 후 별도 진행

  **라이브 결정적 검증 (사용자 sandbox `/work/agenteams/team-ex/v516-final/`, 메시지 id=275 "시작 버튼을 진한 보라색으로 바꿔줘")**:
  - session 1개 lazy init, session_id=86d20006, mcp_servers=["trial-app", ...] 등록
  - PM (Task tool 로 sub-agent invocation) → designer + frontend + qa 가 같은 session 컨텍스트 안에서 협업
  - **사용자 sandbox 의 진짜 코드 변경**: `src/styles/tokens.css` (designer — 토큰 재매핑) + `src/components/StartScreen.tsx` (frontend — 컴포넌트 수정). agent text 응답에 `.lilac-cta` 클래스 / `--color-accent-lilac*` 토큰 정확 인용 — Read tool 실제 사용 증거
  - **MCP tool 호출 흔적**: sim_posts id=270-273 (pm/designer/frontend/qa actor, 모두 root_id=269 thread) — `mcp__trial-app__post_message` 호출. sim_teams.app_features 에 `accent-lilac` 추가 — `mcp__trial-app__set_app_features` 호출
  - **session turn complete success=true duration_ms=135076** (~2.25분, 4-role 협업 + 진짜 코드 변경 + 7 MCP tool 호출)
  - v0.5.x marker brittleness 완전 부재 — parsing 자체 없음

  **본질 회복 선언**: 사용자가 v0.5.16 부터 일관 강조한 "trial app 이 시뮬레이션 아닌 실제 동작" 의 본질이 v0.6.0-alpha.5 에서 **결정적 검증**. agent 가 사람 채팅 → 사용자 코드 베이스 → MCP tool 로 외부 시스템 (trial-app/Mattermost/Plane) → 사용자 시각 결과 — 진짜 Claude agentic team.

  **다음 사이클 (alpha.6)**:
  - v0.5.x simulation 코드 일괄 제거: `routing.rs::{parse_pm_routing, apply_pm_routing, build_pm_prompt, build_agent_prompt, build_echo_*}`, `mod.rs::{handle_human_post, run_agent_step, run_claude_print}`, `sdk.rs::run_claude_agent_sdk` — fallback path 폐기
  - agents-pool publish 정식 워크플로우 (verified/ 채움 + publish.sh 또는 우회 publish 스크립트) → `agents-v1.0.1` release → install.sh 재실행
  - mcp-servers/mattermost/ + plane/ 본 구현 (beta)
  - multi-team sandbox (M-v6.0.4)
  - 호스팅 trial-app 의 쇼케이스 패널 = 사용자 로컬 dev server URL iframe / 명시 안내 (M-v6.0.5)

- 2026-05-14: **v0.6.0-alpha.4 — overlay MCP-mode 전환 + 데몬 session 통합 + P5 stub (P3 + P4 + P5 skeleton)**. 사용자 §"남은 모든 과정 다 한 번에 완성" 명령에 따라 v0.6.0 의 전 마일스톤 (P3/P4/P5) 핵심을 한 사이클에 ship.

  **P3 — overlay 20 파일 (ko/en × 10 role)**:
  - 모든 `agents/overlays/{en,ko}/{pm,planner,architect,frontend,backend,designer,qa,devops,security,code-reviewer}.patch.md.tera` 를 marker 출력 → MCP tool 직접 호출 모델로 재작성
  - 공통 패턴: 환경 변수 (`{{ project_name }}` / `flavor`) + tool 권한 (role 별 차이) + workflow (transition_issue → post_message → 진짜 Edit/Write/Bash → post_message done → transition_issue done) + 금지 (marker 출력 / 거짓 보고)
  - trial flavor 와 real flavor 가 같은 tool 인터페이스 (`mcp__trial-app__transition_issue` vs `mcp__plane__transition_issue`) — overlay 본문 한 벌로 양쪽 지원

  **P4 — 데몬 통합**:
  - `session.rs::ClaudeTeamSession::spawn` 시그니처 `Result<(Self, mpsc::Receiver<SessionEvent>)>` — events rx 외부 노출
  - `session.rs::build_mcp_config(flavor, trial_url, team_token, project_slug, project_name, mcp_server_dir)` — inline JSON config 생성 (trial-app server 등록, real flavor 는 P5 자리)
  - `session.rs::build_append_system_prompt(...)` — runtime context (channel/team/role) 주입
  - `mod.rs::run_listen_loop_session` 신규 — 사람 메시지 → `session.send_user_message(text)`. session 의 stdout event 는 background task 가 drain + 로그. **marker 파싱 0**
  - `cmd_listen::run_foreground` 가 echo-only=false + trial flavor 면 session path, 아니면 v0.5.x marker path (fallback). session spawn 실패 시 자동 fallback
  - 결과: 데몬은 broker, agent 가 MCP tool 로 직접 외부 시스템 조작

  **P5 — real Mattermost / Plane MCP stub**:
  - `mcp-servers/mattermost/README.md` + `mcp-servers/plane/README.md` 작성
  - tool 인터페이스 명세 (trial-app MCP 와 동일) + Mattermost/Plane API 매핑 표 + env 명세
  - beta 사이클에 같은 구조로 Node 구현

  **다음 (alpha.5 라이브 검증)**:
  - agents-pool 의 `genasis/agents/overlays/` 동기화 — release tarball 갱신, 사용자 sandbox 의 .claude/agents/*.md 가 새 overlay 받음
  - install.sh 재실행 → 사용자 sandbox 의 GENASIS:BEGIN/END 마커 사이가 MCP-mode overlay 로 교체
  - 데몬 v0.6.0-alpha.4 띄움 → 사람 메시지 → claude session spawn → PM 이 MCP tool 직접 호출 → 라이브 검증
  - 검증 후 v0.5.x marker 코드 (routing.rs parse_pm_routing, mod.rs handle_human_post 의 marker path, sdk.rs run_claude_agent_sdk fresh-spawn) 일괄 제거

  **시뮬레이션 시대 코드 폐기 예정 (alpha.5)**:
  - `routing.rs::parse_pm_routing` / `apply_pm_routing` / `cleanup_stuck_cards` / `build_pm_prompt` / `build_agent_prompt` / `build_echo_*` 일괄 제거
  - `mod.rs::handle_human_post` + `run_agent_step` + `run_claude_print` 폐기 → `run_listen_loop_session` 만 유지
  - `sdk.rs::run_claude_agent_sdk` 폐기 → `session.rs::ClaudeTeamSession` 만
  - D-029~D-046 fix 들 자연 해소 (parsing 자체 없으니 marker brittleness 0)

- 2026-05-14: **v0.6.0-alpha.3 — Long-running session + MCP server 골격 (P1 + P2)**. 사용자 §"현재 alpha.2 의 SDK 분산 호출은 router-style 이고 진짜 Claude agentic team 이 아니다" + §"alpha.4 + beta 통합 진행" 결정 따른 큰 전환.

  **P1 — `crates/genasis-cli/src/listen/session.rs`**:
  - `ClaudeTeamSession::spawn(cwd, mcp_config, append_system_prompt)` 가 long-running `claude -p --input-format stream-json --output-format stream-json --mcp-config <inline-json> --permission-mode acceptEdits` subprocess 띄움
  - team_token 별 1 session — 매 사람 메시지마다 fresh spawn 안 함, 같은 인스턴스에 NDJSON stdin push
  - stdout 의 NDJSON event stream 을 `SessionEvent::{Init, AssistantText, ToolUse, Result, Other}` enum 으로 정규화 + tokio mpsc 로 비동기 송신
  - 인증: claude CLI subprocess 모드 — 사용자 Pro 구독, 별도 API 비용 0 (사용자 명시 강조 `feedback_no_claude_api`)

  **P2 — `mcp-servers/trial-app/`**:
  - Node `@modelcontextprotocol/sdk` 기반 stdio MCP server
  - 7 tools: `post_message`, `list_posts`, `create_issue`, `transition_issue`, `list_issues`, `set_app_features`, `set_app_kind`
  - trial-app REST API 의 thin wrapper — env 로 `GENASIS_TRIAL_URL` / `TEAM_TOKEN` / `PROJECT_SLUG` / `PROJECT_NAME` / `CHANNEL_NAME` 받음
  - 검증: stdio JSON-RPC initialize + tools/list 요청에 7 tools 응답 완료

  **다음 (alpha.4 통합)**:
  - P3: `.claude/agents/*.md` overlay 의 frontmatter 에 `mcpServers: [trial-app]` 추가 + 본문 protocol 을 marker 출력 → MCP tool call 안내로 변경
  - 데몬 mod.rs::handle_human_post 폐기 → `session.send_user_message(text)` 한 줄로
  - routing.rs::parse_pm_routing 폐기 (D-029a/b, D-035-038, D-041 모두 자연 해소)
  - real Mattermost / Plane MCP server (beta)

  **시뮬레이션 시대 폐기 진행률**: alpha.2 (agent 가 진짜 코드 짬) → alpha.3 (long-running session 골격) → alpha.4 (marker 폐기) → beta (real flavor MCP). 본 사이클은 인프라 ship, 다음 사이클에 실 활용 통합 + 라이브 검증.

- 2026-05-14: **v0.6.0-alpha.2 — 모든 role agent SDK + 진짜 코드 작성 검증 (M-v6.0.2 + M-v6.0.3 부분)**. 사용자 §"agent 가 프로젝트 코드를 알고 그 안에서 해결책을 찾아야 한다" 의 본질 검증 결정적 성공.

  **코드 변경**:
  - `mod.rs::run_agent_step` 의 done_reply 호출을 `claude --print` 에서 Agent SDK 모드로 전환 — role 별 tool 권한
  - `agent_tools_for(role)` 헬퍼: frontend/backend = Read/Edit/Write/Bash, designer = Read/Edit/Write, qa = Read/Write/Bash, devops = Read/Bash, pm/planner/architect/code-reviewer = Read/Bash, security = Read/Edit/Bash
  - SDK 호출 timeout = max(180, claude_timeout_secs × 2) — 코드 작업이 PM 응답보다 오래 걸려서 너그럽게
  - fallback chain: SDK 실패 → `claude --print` → echo stub
  - `routing.rs::build_agent_prompt` 본질 강화 — "텍스트 응답이 아닌 진짜 file Write/Edit/Bash 수행", "거짓 보고 금지", role 별 tool 사용 가이드 명시
  - `routing.rs::build_pm_prompt` — "v0.5.x 의 기존 앱 + feature flag 누적 시뮬레이션 폐기", PM 이 `ls`/`cat PRD.md` 로 cwd 인지 + 빈 프로젝트면 frontend (scaffold) + devops (npm install) 포함 분배

  **라이브 검증 (메시지 id=252)**: 사용자 sandbox `/work/agenteams/team-ex/v516-final/` 에 PRD.md 만 있는 빈 프로젝트. 메시지 "PRD.md 를 보고 진짜 React 앱을 처음부터 만들어줘 — frontend 가 scaffold, designer 가 디자인 토큰, qa 가 테스트, devops 가 npm install + dev server" → PM 이 33초 만에 응답:
  - `assignments=4 / new_cards=4` (frontend + designer + qa + **devops**)
  - `[DEPLOY: devops]` ← v0.5.x 의 features-only 시뮬레이션이 아닌 **devops 가 진짜 빌드/배포**
  - 4 agent staggered 진입 → 진짜 작업

  **sandbox 결과**:
  - `package.json` (`name: "i-am-a-claude-code-expert"` — PRD 의 프로젝트명), Vite + TypeScript + React 18 + vitest + testing-library 의존성
  - `src/App.tsx` 본문에 PRD §3, §3.5, §6 정확히 인용한 코드 (5문제 → 결과, restart 시 다른 시드 ≥7/10)
  - `src/components/PhoneChassis.tsx` / `StartScreen.tsx` / `QuestionScreen.tsx` / `ResultScreen.tsx` ← PRD §1 의 phone frame 구조
  - `src/lib/quiz-bank.ts` ← 문제 데이터
  - `src/test/setup.ts` + `src/components/QuizApp.test.tsx` ← qa 가 진짜 테스트 작성
  - **`npm run dev` 실제 실행** → `127.0.0.1:5173` 에서 Vite dev server listen (PID 205965)
  - **사용자가 localhost:5173 접속 시 진짜 PRD 기반 React 앱 시각 확인 가능**

  이게 시뮬레이션이 아니라 **진짜 agentic team 의 작업**. v0.5.x 의 모든 hard-coded 시각화 분기는 폐기 정당화 — agent 가 진짜 코드를 짜기 때문.

  **다음 사이클 결함 (v0.6.0-alpha.3 의 검증·세부 보강)**:
  - **D-043** (SDK timeout 180s 짧음): frontend 의 npm install + 5 컴포넌트 작성에 180s 초과 → fallback claude --print 발동. timeout 을 600s+ 로 늘리거나 진행 상태 stream 으로 받아 partial 보존
  - **D-044** (timeout 시 fallback prompt wrapping 어색): claude --print fallback 이 agent prompt 를 PM 페르소나 + 사람 메시지로 wrapping — 의도 안 한 흐름. fallback 시 echo agent done 으로 바로 가는 게 깔끔
  - **D-045** (devops 의 dev server URL 자동 announce 안 됨): 사용자가 직접 port 확인해야 함. devops agent 응답 끝에 또는 데몬의 deploy announce 에 "사용자는 http://localhost:5173 에서 확인" 자동 노출 필요
  - **D-046** (vite dev server 가 127.0.0.1 only): 외부 접속 안 됨. devops agent prompt 에 `--host 0.0.0.0` 또는 vite.config 의 `host: true` 안내 또는 데몬이 host 옵션 자동 주입

- 2026-05-13: **v0.6.0-alpha.1 — 본질 회복 시작: Agent SDK 통합 (M-v6.0.1)**. 사용자 §"agent 가 진짜 프로젝트 코드를 알고 그 안에서 해결책을 찾아야 함" 정확 지적. v0.5.x 사이클의 모든 fix (D-029b 색상 사전 / D-039 hard-coded CSS 분기 / D-040 announce 문구 / D-041 cleanup 휴리스틱) 가 시뮬레이션 외형 다듬기였음을 인정. v0.6.0 로 본질 회복 시작.

  **새 PLAN_v0.6.0.md**: per-team sandbox + agent code access 모델. 5개 마일스톤 (Agent SDK 통합 / example scaffold / 진짜 코드 변경 / per-team 격리 / 호스팅 trial-app 역할 재정의).

  **M-v6.0.1 (이 사이클)**: `crates/genasis-cli/src/listen/sdk.rs` 신규 — `run_claude_agent_sdk(prompt, cwd, tools, timeout)` 가 Node subprocess `@anthropic-ai/claude-agent-sdk` 띄움. `permissionMode: 'acceptEdits'` + `cwd` + `allowedTools` 보장. **Anthropic API key 없이 로컬 Claude Code 세션 자동 인증** (`feedback_no_claude_api` 준수). `LoopConfig.project_root: PathBuf` 추가. `handle_human_post` PM 호출 분기를 `claude --print` 에서 SDK 모드로 전환 (tools=[Read, Bash] — Edit 는 frontend phase 에서). 실패 시 `claude --print` 또는 echo fallback. echo-only 모드는 CI 검증용 그대로.

  **라이브 검증 (메시지 id=239)**: 사용자 sandbox `/work/agenteams/team-ex/v516-final/` 에서 데몬 띄움 + 메시지 "이 프로젝트의 PRD 내용을 한 줄로 요약해줘" → PM 응답 12초 만에 도착, 본문에 "본 프로젝트(v516 Final)의 PRD 핵심 내용 요약" 라고 정확히 PRD 컨텍스트 인지. Agent SDK 가 cwd 의 PRD.md 를 Read tool 로 진짜 읽음 검증 완료.

  **다음 phase (다음 사이클 이후)**:
  - PM prompt 분기 — 단순 질의 vs 작업 요청 (현재는 항상 fan-out 강제)
  - frontend/designer/qa/devops agent 도 SDK 모드 + Edit/Bash tool
  - `genasis example prd` 가 PRD.md + 실제 React scaffold (npm 프로젝트) 생성
  - sandbox 격리 (per-team) + dev server auto-spawn

  **시뮬레이션 시대 종료 선언**: D-029b ~ D-042 의 hard-coded 사전/분기/휴리스틱은 점진 폐기. 진짜 agent 작업이 동등 기능 대체.

- 2026-05-13: **v0.5.26 릴리스 — deploy/cleanup actor self-trigger 무한루프 방지 (D-042 핫픽스)**. v0.5.25 D-040 검증 중 데몬이 자기가 게시한 `[deploy]` actor 메시지를 SSE 로 다시 받아서 `is_human_actor("deploy") = true` → PM 응답 1회 trigger → assignments=0 빈 응답 (무한루프는 아니지만 무의미 LLM 호출 + 메시지 1개 더 게시). `is_human_actor` 의 KNOWN_BOTS 에 `"deploy"`, `"cleanup"` 추가해서 데몬 자기 게시 system actor 모두 system 으로 분류.

- 2026-05-13: **v0.5.25 릴리스 — accent 적용 범위 확장 + 배포 announce + stuck 카드 정리 (D-039/040/041 + agents-pool@2aa07e9)**. 사용자가 "자가테스트 끝났는데 티켓이 todo/inprogress 에 남아있고, 결과물이 쇼케이스에 안 나타나고, 배포했다고 답한 agent 가 없다" 직보고. 세 문제 동시 수정.

  **D-039 (accent 적용 범위)**: QuizApp 의 accent-* 가 시작 버튼 (`bg-*-600`) 에만 적용되고 타이틀 h2 는 `text-neutral-900` 고정. 사용자가 "타이틀 색 빨강" 요청해도 시각화는 시작 버튼만 변경 → "요청 ≠ 변경 부위" 인지 불일치. agents-pool QuizApp 에 `titleAccentClass` 별도 dict 추가, `text-*-600 dark:text-*-400` 으로 타이틀 h2 도 last-wins resolution 으로 색 변경.

  **D-040 (배포 announce 자동화)**: features-only deploy 모드는 "별도 agent 호출 없음" 이라 채팅에 누구도 "배포 완료" 라고 말하지 않음 → 사용자 인지 끊김. 데몬이 fan-out 끝난 직후 `[deploy]` actor 명의로 "✅ 배포 완료 — sim_teams.app_features = [...]" 자동 announce. by-last-agent / devops 모드도 비슷한 announce.

  **D-041 (stuck 카드 일괄 정리)**: EventSink::cleanup_stuck_cards 메서드 신규. 사용자 메시지에 "정리/마무리/완료해/cleanup" 키워드 감지 시 데몬이 PM 응답 받기 전에 listIssues GET → state=inprogress|todo 카드 전부 done transition 마커로 묶어 bootstrap 한 round-trip. 이전 사이클 stuck 카드들 (#14/15/16/23/24/26 등) 일괄 청소.

- 2026-05-13: **v0.5.24 릴리스 — [CARD:] 정규식 하이픈 매칭 fix (D-038)**. v0.5.23 라이브 검증 중 #26 "accent-yellow 타이틀 렌더 점검" 카드만 todo 에 남는 현상 발견. routing.rs 의 `[CARD: <title> → <state>]` 파서 정규식 `[^→\->\]]+?` 가 character class 에 `\-` 를 포함해서 title 의 하이픈 (`accent-yellow`) 에서 매칭이 끊김 → `title_substring="accent"` 로 잘림 → ensureIssue dedup 실패 → transition 안 됨. 정규식을 `(.+?)\s*(?:→|->)\s*([a-z]+)` 로 단순화해 separator 까지 non-greedy 캡처. 같은 원인의 이전 사이클 stuck 카드들 (#14/15/16/23/24) 도 새 정리 요청 시 정상 처리.

- 2026-05-13: **v0.5.23 릴리스 — echo stub title 정확화 + Plane-compatible sequence_id 전파 (D-036/037)**. 사용자가 v0.5.22 동작 후 보고: (1) 카드가 In Progress 에 쌓이기만 하고 done 으로 안 옮겨짐 (echo "착수" stub 의 `first_three_words` 가 PM seed title 과 다른 사람 — ensureIssue dedup 실패), (2) agent 응답에 "#N" 문자열이 그대로 (LLM 이 placeholder 를 literal 로 옮김 — Plane 호환 위해 진짜 카드 번호 필요).

  **D-036 (echo stub title)**: `build_echo_agent_start` / `build_echo_agent_done` 가 `first_three_words(&assignment.task)` 로 title 잘라쓰던 것을 제거. handle_human_post 가 PM seed 의 정확한 카드 title (`route.new_cards` 에서 같은 role 의 assignee 매칭) 을 찾아 echo stub 에 직접 넘김. 이제 "착수" 마커와 "완료" 마커, PM seed 모두 동일 title 사용 → ensureIssue dedup 일관 매칭 → 카드 transition 안 끊김.

  **D-037 (Plane-compatible sequence_id)**: `EventSink::apply_pm_routing` 시그니처를 `Result<HashMap<String, u64>>` (title → sequence_id) 로 변경. TrialAppSink 가 bootstrap response 의 `demo_issues[]` 에서 sequence_id 추출해 맵 반환. 데몬이 그 맵을 fan-out 에 전달해 `#N` placeholder 를 실제 카드 번호 (예 `#28`) 로 대체. trial 의 sim_issues.sequence_id 와 real Plane 의 issue.sequence_id 가 같은 정수 형식이라 v0.6.0 real Plane 호환 같은 코드 흐름.

- 2026-05-13: **v0.5.22 릴리스 — 시각 KST 표시 + LRU app_features + agent title 정확 보존 (D-033/034/035)**. 사용자가 v0.5.21 동작 후 3개 문제 보고: (1) 채팅 시각이 한국 시각과 9시간 어긋남, (2) "빨강으로 바꿔" 했는데 시작 버튼은 청록 그대로, (3) 채팅 [CARD: → done] 마커가 있는데 칸반 카드는 In Progress 에 머물러 있음.

  **D-033 (시각)**: SQLite `datetime('now')` 는 UTC 를 "YYYY-MM-DD HH:MM:SS" (Z 없음) 으로 저장. 브라우저 `new Date()` 가 로컬 시각으로 잘못 해석. LiveChatThread.formatTime 이 Z/T 없는 입력을 명시적 UTC 로 normalize (`replace(" ", "T") + "Z"`) 후 toLocaleTimeString 호출 — 이제 KST 컴퓨터는 자동으로 KST 로 변환.

  **D-034 (LRU app_features)**: `setTeamApp` 의 set union 이 기존 항목 위치 보존. 사용자가 "빨강→파랑→빨강" 요청 시 두 번째 빨강이 무시되어 array 의 last 가 여전히 파랑. QuizApp 의 last-wins resolution 이 직관 위배. LRU-append (이미 있는 항목 제거 후 끝에 재 append) 로 변경 — 사람이 마지막에 부른 색이 항상 array 끝 = 시각화.

  **D-035 (agent title 정확 보존)**: claude agent 가 PM seed 의 카드 제목을 의역해서 `[CARD: <변형된 title> → state]` 출력 → ensureIssue 의 title equality dedup 가 실패 → 같은 일에 대한 새 카드만 생기고 원래 카드는 transition 안 됨. `build_agent_prompt` 에 `card_title_literal` 변수 직접 보간 + "**한 글자도 다르게 쓰지 말고 정확히 그대로**" 경고 명시.

- 2026-05-13: **v0.5.21 릴리스 — 진짜 LLM 모드 + parser robust + 색상 사전 확장 + 배포 책임 명시 + 병렬 fan-out (D-029/030/032 + agents-pool@7bfb004)**. 사용자 §"trial app은 시뮬레이션 아니라 실제 plane/mattermost 대용" 의 본질적 결함 4종 동시 수정.

  **회고**: v0.5.16~v0.5.20 내내 운영자 데몬을 `--echo-only` 로 띄워와서 사용자의 "시뮬레이션 거부" 요구를 절차적으로 회피. echo-only 빼고 진짜 `claude --print` 모드로 재기동했더니 즉시 두 결함 노출 — (a) parser 가 strict markdown heading (`## 작업 분배`) 만 매칭해서 claude 의 자유 형식 응답을 못 받음, (b) PM prompt 의 features 사전이 좁아 "청록" → "accent-green" 으로 근사. 사용자 §"배포 담당이 누락 아닌지" + §"병렬 처리" 인사이트로 D-030/D-032 도 함께 ship.

  **D-029a (`routing.rs::extract_section`)**: `## 작업 분배` strict match 를 변종 (`# X`, `## X`, `### X`, `**X**`, plain `X`) 모두 인식하는 line-by-line 정규식 매칭으로 전환. claude 가 `##` prefix 빠뜨려도 fan-out 트리거됨.

  **D-029b (사전 확장)**: PM prompt + echo mapper 에 accent-purple/teal/yellow/orange/pink 추가 (한국어/영어 변종 모두 — 청록/teal/민트/cyan, 노란/yellow, 주황/orange/오렌지, 분홍/pink/핑크). PM prompt 가 "매핑 안 되는 요구는 그대로 kebab-case 로 출력" 으로 변경되어 임의 색 (예: 코랄 → accent-coral) 도 데몬이 그대로 sim_teams 에 저장. QuizApp 은 features 배열을 reverse 해서 last-wins resolution — 사람 의도 (가장 최근 요청한 색이 보임) 와 일치.

  **D-030 (`[DEPLOY: <mode>]`)**: PM prompt 에 배포 책임 분기 룰 추가 — `features-only` (feature flag 만 → 즉시 적용) / `by-last-agent role=<role>` (코드 변경 → 마지막 작업 agent 가 배포 announce) / `devops` (인프라/멀티스텝 → devops agent 호출). PmRouting.deploy 필드 + parser. devops agent 는 이미 catalog 에 있음 — 누락 아니라 PM 이 호출 안 했던 게 원인.

  **D-032 (fan-out 병렬화)**: agent 들의 순차 for-loop 을 `FuturesUnordered` + staggered start 로 변환. 각 agent 가 `idx × agent_gap_secs` 만큼 어긋난 시각에 시작 → 동시에 In Progress 상태. 진짜 LLM 모드에서 N×응답시간 → max(응답시간) 으로 약 50% 단축. echo-only 도 사람 눈에는 "여러 agent 동시 작업" 모습.

  **메모리 항목 추가**: `feedback_genasis_no_echo_only.md` — 운영자 데몬은 절대 `--echo-only` 로 띄우지 말 것 (사용자 일관 강조).

- 2026-05-13: **v0.5.20 릴리스 — agentic team 작업을 실시간으로 보이게 (D-028)**. 사용자 직보고: "trial agent 가 요구사항 넣자마자 모두 한 번에 답하는 거 봐서는 시뮬레이션이지 실제로 일하지 않는 것처럼 보임 — 실시간 동작 보이게 개선해."

  근본 원인: `handle_human_post` 가 각 agent 의 inprogress + done transition 을 한 메시지 안에 모두 넣고 한 round-trip 으로 처리. SSE 이벤트가 ms 단위로 연달아 도착해 LiveKanbanBoard 의 React 렌더링이 In Progress 컬럼을 시각적으로 못 보여줌. 사람 눈엔 늘 "비어있는 칸반" + "전부 done" 처럼 보임.

  **수정**: `build_echo_agent_response` 를 `build_echo_agent_start` (inprogress 만) + `build_echo_agent_done` (done 만) 둘로 분리. `handle_human_post` for-루프 가:
  1. agent 의 "착수" 메시지 게시 → SSE `issue.updated` (state=inprogress)
  2. **sleep `agent_work_secs`** (기본 6초) — 사람이 In Progress 카드를 인지할 시간
  3. agent 의 "완료" 메시지 게시 → SSE `issue.updated` (state=done)
  4. **sleep `agent_gap_secs`** (기본 3초) — 다음 agent 로 넘어가기 전 호흡

  CLI 옵션 `--agent-work-secs` / `--agent-gap-secs` 추가, `listen start` 가 child 프로세스로 forward. echo-only 모드의 진행감이 사라지지 않으면서 자연스러운 협업처럼 보임.

- 2026-05-13: **v0.5.19 릴리스 — accent-purple 색상 + agents-pool@0e1c117**. v0.5.18 검증 중 "보라색으로 바꿔주세요" 채팅 → echo PM mapper 가 모르는 색이라 `features=[]`. listen/mod.rs `build_echo_pm_response` 에 보라/purple/퍼플 → accent-purple 추가, QuizApp `accent-purple` → `bg-purple-600` 분기. agents-pool@0e1c117 동시 푸시 + .gitmodules SHA bump.

- 2026-05-13: **v0.5.18 릴리스 — monitor trial slug slugify (D-026 핫픽스)**. v0.5.17 monitor 가 `[plane].project_name = "v516 Final"` (공백 포함 사람-읽기 라벨) 을 그대로 trial-app `/api/plane/issues?project_slug=` 쿼리에 사용해서 0 issue 반환 — D-025 가 의도대로 안 작동. `load_trial_config` 가 `project_name` 을 `genasis_core::config::slugify` 로 정규화 (예: "v516 Final" → "v516-final") 해서 sim DB 의 실제 slug 와 매칭. project_id 가 명시되면 그대로 사용 (real Plane 경로 보존).

- 2026-05-13: **v0.5.17 릴리스 — daemon SSE 자동 재연결 + monitor trial-flavor 데이터 채움 (D-024/025)**. 사용자 보고: "trial-app 도 살아있다고 하고 너도 살아있다고 했지만 monitor 에서는 정보가 안 보이고, 채팅 패널에 요청 줘도 반응이 없다." 두 결함 동시 수정.

  **D-024 (genasis-cli `listen/trial_sse.rs`)**: `reqwest-eventsource::EventSource` 는 stream 이 영구 종료되면 `next()` 가 영원히 `None` 반환 — 기존 코드는 같은 죽은 인스턴스를 계속 재사용해서 listen daemon 이 살아있는데도 받는 메시지가 0건. `TrialAppSseStream` 에 `rebuild()` 추가 (exponential backoff 1s → 2s → 4s → 8s → 16s → 30s), `next_event` 가 `None` 감지 시 swap 후 재시도. `SseEvent::Open` 성공 시 backoff counter 리셋.

  **D-025 (genasis-monitor `collector/trial.rs` + `app.rs` + `state.rs` + `widgets/sprint.rs`)**: `collector::plane::poll_sprint` 가 정의만 되고 어디서도 호출되지 않아 monitor 의 Sprint / Agents / Log-tail 위젯이 항상 비어 있었음. 신규 `collector/trial.rs::poll_trial` 이 trial-app `/api/plane/issues`+`/api/mattermost/posts`+`/api/trial/team-app/status` 폴링해서 `SprintSnapshot`+`Vec<AgentIssue>`+log lines 반환. `app::load_trial_config` 가 `genasis.toml` 읽고 `[trial].enabled && (plane.flavor == "trial" || mattermost.flavor == "trial")` 일 때 `AppState.trial_mode = true` + URL/team_token/project_slug/scrum channel 세팅. `run_loop` 가 5s tick 마다 `collect_trial` 호출 (real Plane 경로는 그대로 미사용 placeholder). Sprint widget 헤더에 `[app_kind · features]` 표시.

  **검증**: monitor 가 trial 모드 일 때 Sprint Cycle/Todo/In/Review/Done 실 카운트, Agents 위젯 pm/designer/frontend/qa 별 카드 표시, Log-tail 위젯 최근 채팅 라인. SSE 가 한 시간 단위 끊겨도 자동 재연결 (`trial SSE rebuild — sleeping then opening new EventSource` 로그).

  **v0.5.17 태그 푸시** = `release.yml`. operator instance daemon 재기동 필요 (D-024 fix 가 적용된 새 바이너리로 갈아끼움).

- 2026-05-13: **v0.5.16 릴리스 — 시나리오 재설계 + thread-grouped 채팅 UI + daemon-guide banner + QuizApp customization (D-021/022/023)**. 사용자가 v0.5.15 의 "TODO 앱 만들어줘" 시나리오를 "무의미" 라고 정확 지적 — example PRD 결과물 (Claude Code 퀴즈) 확인 못 하고 엉뚱하게 TodoApp 으로 교체. 동시에 trial-app 이 root_id 있는데 thread 시각화 안 됨. 자가테스트 후에도 daemon 대기 + 종료 가이드 UI 노출 필요.

  **agents-pool@f9034ec**: LiveChatThread root_id 그룹핑 (사람 root + reply 들여쓰기 + border line), QuizApp `features?: string[]` prop (accent-red/blue/green/larger-text), LiveBoard 상단 amber daemon-guide banner (`genasis listen stop` inline 안내).

  **genasis v0.5.16**: `build_pm_prompt` 가 "쇼케이스는 example PRD 결과물이고 사람 요청은 그 기존 앱 수정" 으로 재설계, `build_echo_pm_response` 도 새 시나리오 (색상/스타일 키워드 → accent-red/dark-mode/share-button feature 매핑) 맞춤. PM 이 `[APP: quiz]` 유지 + `[FEATURES: …]` 누적.

  **검증**: "퀴즈 시작 버튼 색상을 빨간색으로 바꿔줘" → `[APP: quiz]` + `[FEATURES: accent-red]` + pm/designer/frontend/qa fan-out + thread 들여쓰기 + 시작 버튼 빨간색 실 반영. 종료 안내 banner visible.

  **한계 (v0.6.0)**: 드래그/드롭 칸반, 카드 상세 모달, 멘션 자동완성, share-button 실 구현, real Mattermost Plane integration.

  **v0.5.16 태그 푸시** = `release.yml`. operator instance agents-pool@f9034ec 동기화 완료.

- 2026-05-12: **v0.5.15 릴리스 — agentic team 프로토콜 이식 (genesis §9 + §26): multi-agent fan-out + thread root_id + sim_issues 동적 + 쇼케이스 앱 실 갱신 (D-018/019/020 + ADR-018)**. 사용자가 v0.5.14 의 echo-only PASS 결론을 강하게 반박: "쇼케이스가 처음 퀴즈앱 그대로, pm 이 멘션·작업 분배 없이 단일 actor 로만 응답 — 우리 의도가 이거였나?" 직접 답: 의도였지만 transport 파이프라인만 검증하고 control plane (multi-agent routing) 누락. genesis §9 + §26 의 trial 등가물을 본 사이클에서 이식.

  **agents-pool@8872e92**: sim_teams V5 마이그레이션 (`app_kind`+`app_features`), `setTeamApp` 헬퍼, `/api/trial/*` route 가 신규 필드 받음, `PhoneFrame`+`TodoApp` 컴포넌트 신규, `ShowcasePanel` 이 `appKind` 기준 분기.

  **genasis v0.5.15**: 새 `listen/routing.rs` (PM/agent 프롬프트 + 응답 파서), `handle_human_post` 가 multi-agent fan-out (PM → routing 파싱 → sink.apply_pm_routing → 각 role follow-up `claude --print` → 같은 thread reply), `TrialAppSink::reply` 가 `thread_root_id` null 시 source post_id 를 root 로 (strategy.md 의 `root_id = post.root_id || post.id` 그대로).

  **검증**: 사람 "TODO 앱 만들어줘 — 다크모드 + i18n" → 5 응답 (pm + designer + frontend + qa) 모두 root_id=37 스레드, 3 카드 신규 + 일부 transition, sim_teams.app_kind="todo" + features=["dark-mode","i18n"], **쇼케이스 패널이 TodoApp 으로 동적 교체** + feature flag 별 UI 분기. Playwright 11/11 PASS.

  **binary**: +38 KB. **한계**: 데모 앱 1 종, real Mattermost sink Plane integration stub, 진짜 코드 생성은 v0.6.0 로드맵.

  **v0.5.15 태그 푸시** 가 `release.yml` 트리거. operator instance agents-pool@8872e92 동기화 완료.

- 2026-05-12: **v0.5.14 릴리스 — push 기반 reactive bridge (SSE/WebSocket) + daemon lifecycle (`bridgectl` 등가물) + 다라운드 자가테스트**. 사용자가 v0.5.13 의 listen daemon 에 3 follow-up: (1) 앱 "완성" 후에도 reactive loop 가 계속 살아 추가 수정 요청을 받을 수 있어야 함; (2) polling 말고 진짜 이벤트 기반 (SSE/WS) — Mattermost client lib 검토 권장; (3) `flavor=trial` 일 때는 진짜 Mattermost/Plane 인스턴스에 절대 안 닿아야 함.

  **라이브러리 조사** — crates.io 의 mattermost 관련 crate 8 개 살펴봄. `mattermost_api` (DL 11.6k) + `mattermost-rust-client` (DL 11.7k) 가 REST 는 커버하지만 WS 가 patchy, `mattermost-rs` v0.1 은 너무 미성숙 (DL 33), `mattermost-bot` (DL 1.7k) 은 bot framework 전체 끌어옴. 결정: **`reqwest-eventsource` v0.6 (DL 7.5M) + `tokio-tungstenite` v0.29 (DL 177M) 직접 조합**. Mattermost WS 프로토콜은 단순 `authentication_challenge` → JSON event stream 이라 라이브러리 한 단계가 protocol 견고성도 binary 크기도 이득 못 줌.

  **Binary 크기** (사용자가 "DL 177M 이라며 binary 너무 커지는 거 아닌가" 우려 — DL 은 누적 다운로드 카운트 = 인기도 지표일 뿐): v0.5.13 기준 11,640,928 bytes → v0.5.14 12,025,256 bytes. Δ **+384 KB / +3.3%**. 신규 TLS 의존성이 reqwest 의 기존 rustls 그래프와 dedupe 되어서 실 비용은 tungstenite frame parsing + eventsource SSE decoder 뿐.

  **Listen 모듈 재설계** — `crates/genasis-cli/src/listen/` 신규:
  - `mod.rs` — `InboundEvent::PostCreated` + `EventStream` / `EventSink` trait + `run_listen_loop` + `is_human_actor` / `message_requests_done` 휴리스틱.
  - `trial_sse.rs` — `TrialAppSseStream` (`reqwest-eventsource` 가 SSE reconnect/heartbeat 추상화) + `TrialAppSink` (`/api/mattermost/posts` POST + "X 완료" 디렉티브 시 idempotent bootstrap re-seed).
  - `mattermost_ws.rs` — `MattermostWsStream` (`tokio-tungstenite` 직접 사용, `wss://…/api/v4/websocket` 연결 후 `authentication_challenge` 전송, `event="posted"` 필터, exponential backoff 1→30s 재연결) + `MattermostSink` (`/api/v4/posts` POST, Plane transition 은 v0.6.0 후속으로).
  - `lifecycle.rs` — `bridgectl.sh` 등가물. PID 파일 (`.genasis/listen.pid`), 로그 (`.genasis/listen.log`), `/proc/<pid>/cmdline` 매칭 고아 탐지, slug 당 1 프로세스 보장.
  - `cmd_listen.rs` — Args 모두 `global = true` 라 `genasis listen start --trial --echo-only` 자연스러움. Foreground / start / stop / status / restart / logs.

  **SSE 이벤트 포맷 디버깅** — 초기 구현은 `data.kind == "post.created"` 검사. 실제 trial-app SSE 는 `event: post.created\ndata: <SimPost JSON>` 형태 (kind 가 SSE `event:` 라인). reqwest-eventsource 의 `Event::Message { event, data }` 의 `event` 필드 검사하도록 1줄 패치 → reactive 활성화.

  **다라운드 자가테스트** — `playwright-v514-multi.py` 가 호스팅 URL 에서:
  - 1라운드: 사람 "TODO 앱 만들어줘 — 다크모드 지원" → `[pm]` 응답 visible
  - 2라운드: 사람 "방금 만든 거 한국어로도 보이게 해줘" → `[pm]` 추가 응답 visible
  - 데모 카드 4 개 모두 Done
  - Playwright `__ALL_PASS__: true`

  **flavor 격리 보존** — `cmd_listen.rs::build_loop_components` 가 `genasis.toml [trial].enabled` (또는 `--trial` 명시) 검사 → trial 분기는 trial-app `/api/*` 만, real 분기만 `MM_ADMIN_TOKEN` 요구하고 Mattermost 직접. genesis §0 대전제 (사람↔agent 소통 두 채널, 인스턴스 격리) 보존.

  **v0.5.14 태그 푸시**가 `release.yml` 트리거.

- 2026-05-12: **v0.5.13 릴리스 — `genasis listen` reactive bridge + 티켓 상태 정합성 (D-013 + D-014 + TR-3)**. 사용자가 호스팅 Live Trial URL 화면이 자가모순 (Todo + InProgress 에 init 단계 카드가 남은 채 Done 에 "🎉 Example app published" 가 이미 있음) + 11:35 의 사람 채팅 질문에 아무 에이전트도 무응답 이라고 지적. `/work/secusy/genesis/strategy.md` §0/§26/§28 점검 — 원천 genesis 설계는 사람↔에이전트 모든 소통이 Mattermost+Plane 으로만 흐르되, **reactive bridge daemon** (legacy stack 의 `scripts/mattermost-bridge.mjs`) 가 채널을 폴링 → 멘션 감지 → `claude --print` headless 호출. genasis trial flavor 는 그 설계의 정적 자산은 모두 이식 (10 agent overlay, 14 `/sprint-*` `/check-inbox` 명령, `GENASIS.md` 헌장) 했지만 **bridge daemon 자체가 빠짐** — 자율 loop 가 전적으로 수동.

  **D-013 (티켓 상태 정합성)**:
  - `agents-pool/trial-app/db/sim.ts::ensureIssue` 가 `if (existing) return existing` 으로 short-circuit 해서 `input.state` / `input.assignee` 를 silently drop. 호출자가 state/assignee 를 명시했고 기존 row 와 다르면 `transitionIssue` 호출하도록 패치. 새 row 경로 그대로.
  - `genasis publish` `demo_issues` 가 2 카드 (Build done + Example app published done) 에서 3 카드 (Write-PRD done + Build done + Example app done) 로 확장 — publish 후 칸반 종단 상태가 `Done=4, InProgress=0, Todo=0` 으로 narrative 와 정합.
  - agents-pool@8b03654 가 trial-app 측 ship.

  **D-014 (reactive bridge — `genasis listen`)**:
  - 새 `cmd_listen.rs` (361 줄) — async daemon:
    1. `genasis.toml` 의 `[trial]` 섹션 + per-team token 로드.
    2. `GET /api/events/stream?team=<token>` SSE 스트림.
    3. `post.created` 이벤트 중 `actor` 가 사람 패턴 (`KNOWN_ROLE_BOTS` 미매칭) 필터.
    4. 한국어 role-aware 프롬프트로 `claude --print` 호출, 120 초 timeout.
    5. 에이전트 응답을 `--default-actor` (기본 `pm`) 명의로 `/api/mattermost/posts` POST.
    6. Best-effort `maybe_transition_card` 휴리스틱 — 사람이 "X 완료" / "done" 말하면 idempotent path 로 init 4 카드를 Done 으로 재bootstrap → 칸반이 채팅 narrative 따라감.
  - 플래그: `--trial`, `--echo-only` (`claude` 없는 CI 모드), `--default-actor`, `--claude-timeout-secs`, `--max-events`.
  - reqwest `stream` feature + 신규 `futures-util` workspace dep (SSE byte-stream 소비).

  **GENASIS.md 헌장** 에 "Reactive bridge (자동 응답 데몬)" 하위 섹션 추가 — 모든 에이전트 페르소나에게 사람 채팅이 `genasis listen` 으로 와이어드 됐음을 안내. 데몬 없으면 자율 loop 안 돔.

  **TR-3 자가테스트 정책 확장**:
  - `/work/genasis/CLAUDE.md §자가개발 및 테스트` step 4 에 3 가지 검증 항목 명시 (카드 정합성 / reactive loop / 사람 지시 transition).
  - Step 10 종료 조건 강화 — 1·2·3 항목 모두 Playwright 로 PASS 해야 사이클 종료.
  - `/work/agenteams/team-ex/CLAUDE.md` (사용자 SSOT) 에 §14 + §15 미러 — 자가테스트가 PASS 선언 전 `genasis listen --trial` 을 백그라운드 spawn 의무.

  **v0.5.13 태그 푸시**가 `release.yml` 트리거. agents-pool@8b03654 는 `mmplane-trial.realstory.blog` 에 이미 live.

- 2026-05-12: **v0.5.12 릴리스 — D-011 (운영자 dev-mode 빌드) + D-012 (sim DB FK 가 drop 된 legacy 테이블 가리킴) + trial provider 트레이싱 + stale-host 자동 감지**. 사용자가 v0.5.11 사이클의 호스팅 URL 이 여전히 비어있다고 지적 — 로컬 우회 권고 대신 직접 진단 요구. 사용자 추가 지시: 모든 디버깅은 호스팅 도메인 통해 진행, Caddyfile 이 `mmplane-trial.realstory.blog → localhost:2001` 포워딩이므로 동일 머신. 3개 레이어 문제 모두 end-to-end 해결.

  **D-011 — 운영자가 `agents-pool@677bd5d` 위에서 `npm run dev` 직접 돌리는 중**:
  - `ps` 로 프로세스 트리 추적: zsh → `npm run dev` → `next dev --port 2001`. dev-mode Next.js, D-011 trigger 18시간 전 시작, D-009 (`ec7f149`) 보다 2 커밋 이전. zod 의 `safeParse` 가 미정의 키 silently strip 하므로 v0.5.10 의 bootstrap call 이 보낸 `demo_issues` + `welcome_message` 가 DB 레이어까지 도달 못함.
  - 해결: dev 프로세스 트리 종료, `/work/mmplane-trial.realstory.blog/trial-app/` 에서 `git pull` (677bd5d → ec7f149 → 673764b), `npm run build`, `nohup PORT=2001 npm start &`. Caddyfile reverse-proxy 그대로. Bootstrap 응답에 신규 키 echo 확인.

  **D-012 — sim_issues / sim_posts FK 가 drop 된 `sim_projects_legacy` / `sim_channels_legacy` 가리킴**:
  - 재배포 후 `demo_issues:[…]` 포함 첫 bootstrap 이 `SqliteError: no such table: main.sim_projects_legacy` 로 실패. V0→V2 rebuild (`ALTER TABLE sim_projects RENAME TO sim_projects_legacy` 후 `CREATE TABLE sim_projects (… 새 스키마 …)`) 가 SQLite ≥3.25 의 RENAME 자동 FK 재작성 동작 때문에 sim_issues 의 FK 를 `→sim_projects(slug)` 에서 `→"sim_projects_legacy"(slug)` 로 바꿈. 그 다음 rebuild 가 sim_projects_legacy drop 했지만 sim_issues 의 FK 는 그대로. sim_posts → sim_channels 도 동일. pre-v0.5.10 bootstrap 은 sim_issues 에 write 안 했으므로 dormant, D-009 seeding 이 처음 트리거.
  - Fix in `agents-pool@673764b`: `db/index.ts` 에 V3→V4 migration 추가, sqlite 공식 FK-fix recipe (PRAGMA foreign_keys=OFF 아래 테이블 rebuild). Idempotent — `PRAGMA foreign_key_list` 가 깨진 ref 보고할 때만 실행. 운영자 104-project DB (104 projects / 104 channels / 1 issue / 0 posts) 에서 깨끗하게 적용됨.

  **Trial provider 트레이싱 (트랙 3a)**:
  - `tracing::info!(target="trial", provider=…, op=…, …)` 가 trial flavor 의 모든 Plane/Mattermost 호출에 추가: `ensure_project`, `create_issue`, `transition`, `post_root`, `post_thread`. 각 호출이 outbound `→ POST/PATCH …` 라인 + 응답의 sim row id 담은 `← sim row created/updated` 라인 출력. `RUST_LOG=trial=info genasis <cmd>` 로 에이전트 의도 → trial-app DB write 의 end-to-end correlation 확인 가능.

  **Stale-host 자동 감지 (트랙 3b)**:
  - `try_bootstrap_trial_app` 가 bootstrap 응답 JSON 파싱 → `demo_issues` + `welcome_message` 키 부재 시 경고 출력 (배포된 schema 가 ec7f149 이전). README §Known limitations + `GENASIS_TRIAL_URL` 우회 안내. 이게 없으면 사용자는 green CLI output + 빈 칸반을 보고 서버가 seed 데이터 drop 했다는 신호를 못 받음.

  **라이브 검증 (v0.5.12 vs `https://mmplane-trial.realstory.blog`)**:
  - Fresh `genasis init --trial --name "D-012 Hosted Verify"` → bootstrap ok, stale 경고 없음. 새 팀 `35247bd0…` 발급.
  - `RUST_LOG=trial=info genasis init` (bare) → 트레이싱 라인 `→ resolving trial sim project provider="plane" op="ensure_project" team_token_short=35247bd0 project_name="D-012 Hosted Verify"` stderr 에 visible, 이어서 `plane project_id = d-012-hosted-verify`.
  - `genasis publish` → `+ seeded build-complete card + chat message` + landing URL `https://mmplane-trial.realstory.blog/?tab=live&team=35247bd0…`.
  - Playwright 가 호스팅 URL 직접 검증: **`__ALL_PASS__: true`** — 4/4 demo 카드 + 2/2 welcome 메시지 + 쇼케이스 핸들 활성.

  **v0.5.12 태그 푸시**가 `release.yml` 트리거. agents-pool@673764b 는 운영자측 의존성, 포트 2001 의 production 서버는 이미 그 상태.

- 2026-05-12: **v0.5.11 릴리스 — Quick Path inline 점검 단계 + stale-hosting 점프 아웃 (D-010)**. 사용자 재진입 트리거로 자가테스트 사이클 한 번 더. CLI 1-5 단계는 fresh v0.5.10 설치본에서 전부 green 인데, Playwright 진단 결과 **새 사용자가 README 그대로 따라가서 default 운영자 호스팅 URL 을 열면 칸반·채팅 비어있음** — "뭔가 일어났다" 라는 visible 신호가 §"Known limitations" 까지 스크롤해야 발견됨. 동일 흐름을 `GENASIS_TRIAL_URL=http://localhost:2099` 로 돌리면 카드 4 + 환영 메시지 2 + 쇼케이스 핸들 활성. v0.5.10 의 escape hatch 자체는 동작, 단지 5분 경로 안에서 발견 안 됨.

  **Fix (D-010)** — README + README.ko §"Quick Path":
  - Step 2 끝에 **🔍 점검 포인트** 콜아웃 추가: 그 시점에 랜딩 URL 열어서 팀 배지 + 데모 카드 3 + 환영 메시지 확인. 비어있으면 stale hosting → 새로 만든 §"알려진 한계 (v0.5.11)" 앵커로 점프, 한 줄 명령 `export GENASIS_TRIAL_URL=http://localhost:2099` 우회.
  - Step 5 (`genasis monitor` 다음) 에 **🔍 최종 점검** 콜아웃: URL 새로고침, 기대 상태는 카드 5 (Done 의 `🎉 Example app published` 포함) + 메시지 2 + 클릭 가능 쇼케이스 핸들. 사용자가 step 2 점검에서 비어있었고 docker 우회 건너뛴 경우의 "쇼케이스만 활성" 상태도 명시 — 불필요한 되돌이 안 함.
  - `README.ko.md` 미러 동일 콜아웃 반영.

  **라이브 검증 (v0.5.10 binary, 두 시나리오 — 같은 Playwright 실행)**:
  - A. Default hosting (`https://mmplane-trial.realstory.blog`): 배지 ✅, 칸반 0/4, 채팅 0/2, 쇼케이스 핸들 ✅ (publish 가 `app_status` 만 flip 하므로). 새 "Step 2 이후 비어있음" 경고와 일치.
  - B. Local override (`http://localhost:2099`): 배지 ✅, 칸반 4/4, 채팅 2/2, 쇼케이스 핸들 ✅. 새 "Step 5 후 카드 5 개 + 메시지 2 개" 기대치와 일치.
  - 스크린샷: `/work/agenteams/team-ex/screenshots/cycle-{A_default_hosting,B_local_override}.png`.

  **기타 관찰 (블로커 아님)**:
  - D-006 (404 응답) — `page.on('response')` 전수 캡처 결과 11 응답 중 4xx 0건. console 의 "Failed to load resource" 라인은 SSE stream close 노이즈. 비결함 처리.
  - D-007 (caret-color hydration warning) — 로컬 빌드 `ec7f149` 컨테이너에서 재현 안 됨. 호스팅의 dev-mode 빌드 잔재 추정, 재배포 시 사라질 것.

  본 릴리스에 코드 변경 없음 — workspace 버전 bump 후 `Cargo.lock` sync 위한 `cargo build` 만. **v0.5.11 태그 푸시**가 `release.yml` 트리거, install.sh 가 간접 링크하는 README 가 binary 가 받쳐주는 Quick Path 와 함께 ship.

- 2026-05-12: **v0.5.10 릴리스 — `GENASIS_TRIAL_URL` env override + D-009 end-to-end 증명**. 사용자가 v0.5.9 사이클의 "운영자 재배포 대기" 결론을 거부 — "네가 직접 D009 Test 결과를 보고 왜 정보가 사라졌는지 추적해서 문제 해결해". 가정 대신 직접 진단: `https://mmplane-trial.realstory.blog/api/trial/bootstrap` 에 `demo_issues` + `welcome_message` 포함 POST — 응답 JSON 에 신규 필드 **없음** 확인, 즉 호스팅 인스턴스가 `ec7f149` 이전 코드 실행 중. `/api/plane/issues` + `/api/mattermost/posts` probe — 둘 다 401 (pre-D-001 secret-required contract). 호스팅이 진짜 stale.

  **로컬 docker 로 end-to-end 검증** (binary + agents-pool 코드 둘 다 정확함 증명):
  - 현재 agents-pool 트리 (`ec7f149`) 로부터 `mmplane-trial-app:d009-debug` 빌드, port 2099 에서 기동.
  - `http://localhost:2099/api/trial/bootstrap` 에 demo 데이터 직접 POST — 응답에 `demo_issues: [...]` + `welcome_message: {id, channel_id, message}` row 포함.
  - Playwright 로 `http://localhost:2099/?tab=live&team=<token>` 열림 — Done/InProgress/Todo 에 분포된 3 demo 카드 + welcome chat post 1개 visible. D-009 fix 가 운영자 dependency 없이 end-to-end 동작.

  **Fix (실제 binary 변경)**:
  - `cmd_init.rs` 의 하드코딩된 `const TRIAL_APP_URL` 을 `trial_app_url()` 함수로 교체 — `GENASIS_TRIAL_URL` env 읽고 fallback 은 `https://mmplane-trial.realstory.blog`.
  - 이 override 가 일관되게 흘러가는 곳: (1) `genasis.toml [trial].url` 만드는 `render_trial_config`, (2) `try_bootstrap_trial_app` POST endpoint, (3) `--probe-only` 요약, (4) 성공 배너의 per-team open URL, (5) `genasis publish` (이미 `[trial].url` 읽음).
  - README + README.ko `알려진 한계 (v0.5.10)` 가 전체 우회 절차 문서화: sparse-checkout `agents-pool/trial-app` → docker build → docker run -p 2099 → `export GENASIS_TRIAL_URL=http://localhost:2099`.

  **라이브 검증 (v0.5.10 binary vs localhost:2099)**:
  - `genasis init --trial --name "Local D-009 Verify"` → bootstrap ok, 3 demo 카드 seed (`Set up agentic team` Done, `Write PRD…` InProgress, `Build the example app…` Todo) + welcome 메시지 "👋 …팀이 시작됐어요…".
  - `genasis publish` → status complete 으로 flip, 추가 카드 2개 seed (`Build the example app…` Done, `🎉 Example app published` Done) + "✅ 빌드 완료…" 메시지.
  - Playwright: 컬럼 전체에 4/4 카드 visible, 2/2 welcome 메시지, 쇼케이스 핸들 "에이전트가 만든 앱 보기" 활성. Done 컬럼에 `@genasis` assignee 라벨 단 2 항목.

  **v0.5.10 태그 푸시**가 `release.yml` 트리거. 호스팅 `mmplane-trial.realstory.blog` 는 default-URL 경로에서는 여전히 stale 이지만, 사용자는 이제 `GENASIS_TRIAL_URL` 를 통한 완벽 동작 대안 보유.

- 2026-05-12: **v0.5.9 릴리스 — Live Trial UI 가 `init --trial` / `publish` 후 가시적 활동 표시 (D-009)**. 사용자가 직전 자가테스트 사이클의 publish 후 Live Trial URL 열어보니 "여전히 작업 진행했던 흔적이 남아 있지 않아" 라고 보고 — 칸반 비어있음, 채팅 비어있음, showcase 핸들 하나로는 시스템이 뭔가 했다는 신호가 충분치 않음. Root cause: `try_bootstrap_trial_app` 가 team + project + channel row 만 seed, `genasis publish` 가 `app_status` 만 `complete` 로 flip. 실제 카드/메시지 없음.

  **Fix (D-009)**:
  - **agents-pool@ec7f149**: `/api/trial/bootstrap` 에 두 optional 필드 추가:
    - `demo_issues[]` — 초기 칸반 카드 (title + state + assignee), 신규 `ensureIssue()` 헬퍼가 `(team_token, project_slug, title)` 기준 idempotent seed.
    - `welcome_message` — 첫 채널의 root post, 신규 `ensureWelcomePost()` 가 `(actor, message)` 기준 dedup.
  - **genasis**: `try_bootstrap_trial_app` (init --trial 단계) 가 Done/InProgress/Todo 에 걸친 3개 데모 카드 + 다음 단계 안내 한국어 환영 메시지 전송. `run_publish` 는 추가 "build complete" 카드 2개 + publish 확인 메시지 — bootstrap 재호출로 idempotent 보장.
  - `init --trial` 이나 `publish` 재실행해도 trial-app 측 dedup 으로 중복 없음. publish seed 실패는 non-fatal — status flip 이미 성공했으므로 informational warning 만 ("UI may not reflect ... until deployed trial-app catches up").
  - Submodule pointer `agents-pool@ec7f149` 로 bump.

  **검증**:
  - `cargo build --release -p genasis-cli` clean, `cargo test --workspace --lib`: 162 passed
  - `npx tsc --noEmit` on trial-app clean

  **v0.5.9 태그 푸시**가 `release.yml` 트리거. 배포 후, 운영자 호스팅 trial-app 이 `agents-pool@ec7f149` 로 재배포되어야 신규 필드가 honored 됨 (그렇지 않으면 `z.optional()` 스키마로 silently drop — bootstrap 자체는 계속 동작, seeding 만 미반영).

- 2026-05-12: **v0.5.8 릴리스 — install.sh prefix 안전성 + README Self-host 진입점 (D-005 + D-008)**. 자가테스트 사이클 (이번엔 Playwright 브라우저 검증 포함) 이 신규 사용자 시점에서 두 결함을 잡음.

  **Fix (D-005)** — `install.sh --prefix=<신규 경로>` 가 거짓 보고:
  - 사용자가 `--prefix=/some/new/dir` (없는 경로) 를 주면 `mv: cannot stat` 발생, `sudo install` fallback 으로 가는데 non-TTY 환경 (curl|sh, CI 러너) 에서 비대화형 sudo 가 silently 실패 — `set -e` 가 sudo 의 interactive password prompt 너머로 전파 안 됨. 그런데도 `[OK] Installed: <path>` 가 출력. 실제 바이너리는 없음.
  - Fix: `mv` 전 `mkdir -p "$PREFIX" 2>/dev/null || true` 추가. `sudo install` 을 `elif ... 2>/dev/null` 로 감싸서 실패를 명시적으로 catch, writable 기본값 명시한 `die` 로 종료. post-install `[ -x "$install_path" ]` hard check 추가하여 success 라인이 진짜 파일 존재할 때만 출력.
  - Live 검증: `sh install.sh --prefix=/tmp/install-test-new --skip-prereqs --no-run` (없던 prefix) 가 디렉터리 생성 → 11 MB 바이너리 정상 설치 → `genasis --version` 정상 (0.5.7 — 갓 받은 릴리스).

  **Fix (D-008)** — README Self-host Option B 가 Quick Path 에서 도달 불가:
  - "Step-by-Step → Plane & Mattermost 설정 → Option B" 블록이 곧바로 `cd servers && ./scripts/setup-user-env.sh && docker compose up -d` 인데, `install.sh` 는 `genasis` 바이너리만 ship 함 — `servers/` 디렉터리는 받지 않음. README 그대로 따라간 신규 사용자가 `bash: cd: servers: No such file or directory` 에서 막힘.
  - Fix: "먼저 `servers/` 디렉터리를 받습니다" preamble 추가, 2가지 받는 방법 명시 (전체 clone vs sparse-checkout `git sparse-checkout set servers`). `README.ko.md` 미러 동일.

  **검증**:
  - install.sh round-trip end-to-end `--prefix=/tmp/install-test-new` 통과
  - README Quick Path 1-5 가 신규 `install.sh` 설치본 v0.5.7 바이너리로 라이브 `mmplane-trial.realstory.blog` 대상 그대로 green
  - Playwright 브라우저 검증: Live Trial 페이지가 Marketing Squad + scrum-marketing-squad 채널 + 4컬럼 칸반 + 채팅 사이드바 정상 렌더; `genasis publish` 가 `app_status` 를 `complete` 로 flip 하고 쇼케이스 핸들이 활성화됨

  **v0.5.8 태그 푸시**가 `release.yml` 트리거.

- 2026-05-12: **v0.5.7 릴리스 — `genasis attach --upgrade` 플래그가 자체 deprecation 메시지와 일치 (D-003)**. v0.5.6 바이너리 self-test 계속 중, `Upgrade` 서브커맨드의 deprecation 메시지가 사용자를 `genasis attach --upgrade` 로 안내 (README CLI 참조도 동일) 하는데, 그 플래그가 `cmd_attach::Args` 에 실제로 존재하지 않음 — 실행하면 `error: unexpected argument '--upgrade' found` 출력, deprecated 서브커맨드로부터의 마이그레이션 경로가 문서화되어 있지 않은 상태.

  **Fix (D-003)**:
  - `cmd_attach::Args` 에 `--upgrade` boolean 플래그 추가. 현재는 passthrough — 기존 re-attach 머신이 곧 upgrade — 지만 플래그가 사용자 의도를 담고 향후 버전에서 더 보수적인 정책 (예: legacy `cmd_upgrade` 동작 매칭을 위한 Tampered fence 기본 보존) 채택의 기반.
  - 두 internal call site (`cmd_bootstrap`, `cmd_lang`) 에 `upgrade: false` 추가.
  - Live: `genasis attach --upgrade --non-interactive` 가 이제 성공, 표준 re-attach plan 실행.

  **v0.5.7 태그 푸시**가 `release.yml` 트리거.

- 2026-05-12: **v0.5.6 릴리스 — `genasis agents list / browse / status` 가 v1.0.0 카탈로그에 대해 동작 (D-002)**. v0.5.5 에서 D-001 을 닫은 self-test 사이클이 두 번째 오래된 결함을 surface: `genasis agents list` 가 `agents/index.json not found. Run \`genasis agents fetch\` first.` 로 실패하는데, `genasis agents fetch` 는 "카탈로그 이미 cached, 492 agents available" 이라고 응답 — 순환 dead-end. Root cause: v1.0.0 release tarball 은 `manifest.json` (overlay role metadata) 은 ship 하지만 `index.json` (marketplace command 가 기대하는 searchable 카탈로그) 은 안 함, 바이너리도 client-side synthesise 안 함.

  **Fix (D-002)**:
  - 새 helper `load_catalog_index(version, cache_override)` in `cmd_agents.rs`, 3-단계 fallback chain: 프로젝트 로컬 `./agents/index.json` → 캐시 `<dir>/v<ver>/index.json` → **캐시 `base/` frontmatter 에서 synthesise**. Synthesis 가 `<cache>/base/` 의 모든 `.md` 를 walk 해서 기존 `genasis_core::frontmatter` API (single-line scalar reader for `name` / `description` / `category` / `tags`) 로 frontmatter 파싱, 모든 command 가 이미 소비하는 `{agents: [], categories: [], presets: {}}` shape 빌드. `_source: "synthesised-from-cache-base-frontmatters"` 가 index 에 annotate 되어 `agents status` 가 데이터 출처를 surface.
  - `cmd_list`, `cmd_browse`, `cmd_status`, `install_preset` 모두 이제 `load_catalog_index` 경유. `agents-v1.0.1` 이 적절한 `index.json` 을 ship 하면 synthesis 가 dead code 가 되고 바이너리는 추가 변경 없이 richer shape (preset 정의 등) 을 pick up.
  - Presets 는 여전히 비어있음 (frontmatters 에 preset membership 없음), 그래서 `genasis agents install --preset web-app` 가 이제 cryptic `no presets defined in index` context error 대신 `install <name>` 권장하는 한 줄 메시지 출력.

  **검증**:
  - `cargo build --release -p genasis-cli` clean
  - Live: `genasis agents list` → 492 agents, paginated; `genasis agents list --search frontend` → 4 hits (angular-architect, frontend-developer, frontend-security-coder, fullstack-developer); `genasis agents status` → `Index: 492 agents available (synthesised-from-cache-base-frontmatters)` — 직전 dead-end 가 이제 self-explanatory.

  **v0.5.6 태그 푸시**가 `release.yml` 트리거.

- 2026-05-12: **v0.5.5 릴리스 — stale 운영자 배포에 대해 Quick Path 가 self-heal (D-001)**. v0.5.4 바이너리로 자가테스트 사이클을 돌리니, `mmplane-trial.realstory.blog` 호스팅이 `agents-pool@289876c` 보다 옛 버전이면 Quick Path 단계 4 (`--trial` 후 `genasis init`) 가 여전히 `/api/plane/projects` 의 401 로 hard-fail. v0.5.4 릴리스 노트는 이를 운영자 액션으로 문서화했지만, 신규 사용자는 남의 서버를 재배포할 방법이 없다. 이번 릴리스는 그 일을 바이너리로 옮겨 Quick Path 가 self-heal 하도록 만듦.

  **Fix (D-001)**:
  - `TrialPlane::ensure_project` 가 먼저 `GET /api/trial/team-app/status?team=<token>` 를 호출 (auth-free, 모든 배포 버전 수락). `--trial` 단계에서 `try_bootstrap_trial_app` 이 이미 팀을 seed 했으므로 status 가 team_exists=true 를 반환할 것 — bootstrap-canonical `slugify(project_name)` 을 즉시 반환하고 auth-locked `/api/plane/projects` POST 자체를 안 함. 부수 효과로 잠재 slug 일관성 bug 도 해결: 기존 bare `genasis init` 는 slug 를 `slug_to_identifier(name)` (예: "MARK") 로 derive 했는데, 이는 bootstrap 이 쓴 `slugify("Marketing Squad")="marketing-squad"` 와 충돌. 이제 downstream `create_issue` / `transition` 가 Live Trial UI 가 보여주는 동일 sim row 를 가리킴.
  - `TrialMattermost::ensure_channel` 는 auth-free idempotent `/api/trial/bootstrap` 로 라우팅 (대상 채널을 single-element `channels[]` 로 전달). bootstrap 이 기존 채널 row id 를 그대로 돌려줘서 auth-locked `/api/mattermost/channels` POST 불필요.
  - 두 메서드 모두 매우 옛 self-host 배포를 위한 legacy POST fallback 보유, 그 fallback 도 401 시 raw `{"error":"unauthorized"}` 가 아니라 한 줄 remediation 메시지 ("배포된 trial-app 이 이 바이너리보다 옛 버전 — 재배포 또는 self-host 권장") 출력.

  **이번 릴리스 미포함**:
  - 에이전트 런타임이 부르는 호출 (`create_issue`, `transition`, `post_root`, `post_thread`) 은 여전히 legacy `/api/plane/issues` / `/api/mattermost/posts` 대상이라 stale 배포에서는 여전히 401 가능. 이를 닫으려면 trial provider 가 자체 (issue_id → sim row) 매핑을 유지해야 하는 큰 리팩토링 필요, 신규 사용자에게 보이는 Quick Path 외관에는 영향 없음. README §"알려진 한계 (v0.5.5)" 에 정직하게 명시.

  **자가개발 및 테스트**:
  - `CLAUDE.md` 에 `## 자가개발 및 테스트` 섹션 추가, 반복 test→fix→push→monitor→retest 루프 정문화. 테스트 베드는 `/work/agenteams/team-ex/` 와 `PLAN.md` + `genasis-test-log.md`; 프로토콜은 self-contained — 외부 `~/rnd/agenteams/team01/...` 의존성 없음. Claude Code 세션이 이 루프를 자율로 재진입 가능.

  **검증**:
  - `cargo build -p genasis-providers` clean, `cargo test -p genasis-providers` 23 passed
  - `mmplane-trial.realstory.blog` (v0.5.4 시대 배포본) 대상 라이브 통합 테스트: Quick Path 1→4 가 이제 end-to-end 성공. `--trial` 후 `genasis init` 이 `plane project_id = test-v055-live` 반환, 401 없음.

  **v0.5.5 태그 푸시** 가 `release.yml` 트리거 (cross 로 musl-static x86_64 + aarch64, Verify-static-linking + compat-smoke gate, 두 tarball + sha256 첨부된 GitHub Release).

- 2026-05-12: **v0.5.4 릴리스 — v0.5.3 현장 결함 8개 (C1-S1) + 다음 운영자 배포 시 Quick Path 깨끗**. 테스터가 v0.5.3 을 clean WSL2 박스에서 end-to-end 로 돌리고 결함 11개 보고 (C1-3 critical, S1-4 significant, M1-4 medium, D1 low). 그 중 4개 (S2, S4, M4, 부분적 S3) 는 이미 이전 패치에서 처리됨. 이 릴리스가 추가 8개 닫음. **호스팅 `mmplane-trial.realstory.blog` 배포가 agents-pool `289876c` 이후 commit 이라면** (C1 참조), 이제 Quick Path 가 바이너리만으로 end-to-end 동작.

  **Fix**:
  - **C1 (호스팅 trial-app 401)** — 부분. genasis 바이너리 쪽은 v0.5.3 이후 옳음 (`289876c` 가 agents-pool 의 모든 bridge route 에서 shared_secret check 제거), 그러나 deploy 된 `mmplane-trial.realstory.blog` 가 아직 재빌드 안 됐을 수 있음. README §"알려진 한계" 가 운영자에게 재배포 단계를 안내하고, 사용자가 막혔을 때 trial-app 을 self-host 하는 방법을 설명.
  - **C2 (catalog short-name 파일)** — v0.5.3 에서 추가한 `Role::aliases()` + `infer_from_name` 별칭이 이미 bootstrap/attach 를 long-name 파일에 대해 동작하게 함. v0.5.4 는 추가로 `cmd_doctor` 의 frontmatter check 에도 같은 별칭 표를 가르쳐 7 개의 false-positive "name: 가 filename stem 과 불일치" 경고 silence. 카탈로그 자체는 깨끗함을 위해 `agents-v1.0.1` 에서 short-name 파일을 ship 해야 함 — 별도 tracked.
  - **C3 (GENASIS.md 미작성)** — v1.0.0 카탈로그 tarball 이 `<lang>/GENASIS.md.tera` 를 아예 묶지 않는데도 `cmd_attach` 가 요약에 `+ GENASIS.md` 출력. 2 단 fix: (a) genasis 저장소의 `agents/GENASIS.md.tera` 를 바이너리에 `include_str!` fallback 으로 임베드 — 템플릿 누락된 카탈로그여도 GENASIS.md 가 작성됨. (b) 요약 라인이 실제로 작성한 파일 수를 정직하게 보고, GENASIS.md 가 못 만들어졌을 때 별도 경고. `agents-v1.0.1` 이 템플릿 ship 하면 카탈로그 사본 우선.
  - **S1 (채널 슬러그 공백)** — overlay 템플릿이 `project_name` raw 로 interpolate 해서 `#scrum-Marketing Squad` 렌더. 2 단 fix: (a) `build_tera_from_store` 에 `slugify` Tera 필터 등록 — 향후 템플릿 `{{ project_name | slugify }}` 사용 가능. (b) `build_context` 에 `project_slug` (pre-slugified, `genasis_core::config::slugify`) 추가 + `agents/overlays/{en,ko}/` 의 20 개 overlay source 템플릿이 채널 참조에 `project_slug` 사용하도록 업데이트. 다음 카탈로그 릴리스 ship.
  - **M1 (MM 채널 idempotence 추함)** — `UpstreamMattermost::ensure_channel` 이 이제 lookup 먼저 (`GET /teams/{id}/channels/name/{name}`) 하고 404 일 때만 POST. `store.sql_channel.save_channel.exists.app_error` gobbledygook 문자열이 더 이상 사용자에게 도달 안 함. POST 가 id 대신 에러 문자열 반환하는 race 케이스도 follow-up lookup 으로 처리.
  - **M2 (Plane health probe 401 noise)** — probe 를 `/api/v1/workspaces/<slug>/` (auth 게이트, 유효 API key 있어도 401) 에서 `/api/instances/` (unauth metadata 엔드포인트) 로 전환. healthy 서버에서 깨끗한 200/JSON, transport 에러는 여전히 surface. workspace 존재 검증은 `ensure_project` 의 paginated walk 가 처리.
  - **M3 (install.sh `curl | sh` hang)** — lang 프롬프트는 이미 TTY-aware 였지만 install.sh 안의 `genasis attach` 자식 프로세스가 curl pipe stdin 상속. attach 호출에 `</dev/null` 추가해서 자식이 script-pipe 의 사용자-의도 바이트를 절대 못 읽음.
  - **Doctor name_mismatch 경고** — `cmd_doctor` 가 매 clean install 마다 경고 (카탈로그 `name: frontend-developer` ≠ 파일명 `frontend.md`). 이제 `cmd_attach` 와 같은 `infer_from_name` 별칭 해석 사용; stem 과 value 가 같은 role 로 해석되면 경고 안 함 (별칭 케이스), 서로 다른 role 일 때만 경고 (진짜 버그).

  **검증**:
  - `cargo fmt --all` clean
  - `cargo clippy -p genasis-cli -p genasis-overlay -p genasis-providers --tests -- -D warnings` clean
  - `cargo test --workspace`: **269 passed, 4 ignored**
  - trial-app `npm run typecheck` + `npm run build` clean (이 릴리스에 agents-pool 변경 없음)
  - i18n drift gate: 148 keys OK

  **v0.5.4 태그 푸시**가 `release.yml` 트리거 (cross 로 musl-static x86_64 + aarch64, Verify-static-linking + compat-smoke gate, 두 tarball + sha256 첨부된 GitHub Release).

  **다음 minor (v0.6.0) 로 연기**:
  - **S3 (토큰 프로비저닝 헬퍼)**: 신규 `genasis init bootstrap-tokens` 서브커맨드 — Plane + Mattermost admin API ~9 단계 자동 수행.
  - **agents-v1.0.1 카탈로그 리프레시**: short-name base 파일 (`pm.md`, `frontend.md`, …) + slugify-사용 overlay 템플릿 + `<lang>/GENASIS.md.tera`. ship 되는 즉시 이 바이너리의 모든 C2/C3/S1 fallback 이 fast-path 됨.

- 2026-05-12: **v0.5.3 릴리스 — CLI 단순화 (A-E) + v0.5.2 현장 결함 6개 (가-바)**. 테스터가 v0.5.2 를 end-to-end 로 돌려서 v0.5.0 라운드 위에 6 개 결함을 추가 보고했고, 이번 릴리스가 그 6개를 모두 fix 하고 계획된 CLI surface 단순화 (init/publish 를 primary 로, bootstrap/attach/lang/upgrade/plane/mm 를 v0.7.0 제거 마일스톤과 함께 deprecation 처리) 도 함께 ship.

  **디버깅 fix**:
  - **가 — `${#body}` 의 Tera lex 에러가 6개 hook 중 5개를 죽임**. `cmd_attach` 의 `install_genasis_overlay_artifacts` 가 모든 `.tera` 를 `Tera::one_off` 로 렌더하면서 `?` 로 propagate — `${#var}` (bash length 확장, `{#` 가 Tera 주석 시작처럼 보임) 가 들어간 첫 파일에서 전체 루프 bail. body 에 `{{` 또는 `{%` 가 실제로 등장할 때만 Tera 를 호출하는 `render_template_body` 헬퍼로 전환. 나머지는 verbatim passthrough. 파일별 에러는 이제 warning, propagate 안 함. 6개 hook (session-start, branch-guard, MM-sync, worktree-guard, user-prompt-submit-mm, post-tool-trim) 과 17개 command 가 모두 `.claude/genasis/` 에 떨어짐.
  - **나 — 서버측 team_exists: false 인데도 `trial-app bootstrap ok` 가 출력됨**. `try_bootstrap_trial_app` 이 POST status code 만 봤음 — accept 했지만 persist 안 한 서버 (구버전 배포, schema drift 등) 가 silently 거짓말. POST 직후 `GET /api/trial/team-app/status?team=<token>` verify 호출 추가. `team_exists` false 면 "deployed trial-app may be older than the bootstrap contract this binary expects" 명확한 에러 반환.
  - **다 — `/api/plane/projects` 401 (operator-only `shared_secret` 요구)**. ADR-016 §4 를 "bootstrap 만 unauth, 나머지는 secret 필요" 에서 "모든 trial-app route 가 token-as-capability, shared_secret 은 optional defence-in-depth" 로 일반화. `lib/trial-auth.ts` 의 `requireTrialContext` → `resolveTrialContext` 로 rename (legacy alias 한 릴리스 유지). 5개 bridge route (plane/{projects,issues,issues/[id]}, mattermost/{channels,posts}) 모두 secret check drop. ADR-016 §4 업데이트.
  - **라 — 10개 에이전트 중 6개의 frontmatter name: mismatch → overlay fence 미주입**. bootstrap-side 파일명 해석을 위해 `Role::aliases()` 에 추가한 alias-walk symmetry 가 attach-side frontmatter 이름 해석에는 빠져 있었음. `infer_from_name` 을 확장해 v1.0.0 카탈로그의 실제 `name:` 값 (`frontend-developer`, `backend-developer`, `qa-expert`, `product-manager`, `devops-engineer`, `security-engineer`, `design-system-architect` 등) 인식 — 어느 쪽 경계를 넘든 10개 canonical role 모두 해석.
  - **마 — CLAUDE.md, GENASIS.md 미생성**. GENASIS.md install 은 issue 가 fix 로 자동 복구 (install 함수가 단일 템플릿 에러로 전체 abort 안 함). 추가로 `install_genasis_overlay_artifacts` 가 프로젝트 루트에 CLAUDE.md 가 없으면 `@import GENASIS.md` 라인이 든 stub 을 작성 — 그 import 없으면 Claude Code 가 protocol contract 를 로드 안 하고 slash command + hook 들이 orphan 됨. 기존 CLAUDE.md 는 건드리지 않음 (idempotent).
  - **바 — `humans sync` Plane half 가 `PLANE_ADMIN_EMAIL` / `PLANE_ADMIN_PASSWORD` 요구, 문서화 안 됨**. README EN+KO §"Option B" 자격증명 블록을 Step-by-Step §"admin token 발급" 의 god-mode 자격증명을 가리키도록 명시. Plane API-key 인증만으로는 사용자 생성 불가, admin sign-in 필수.

  **CLI 단순화 A-E** (v0.7.0 제거 마일스톤과 함께 deprecation 메시지):
  - **A**: `init` 를 clap help text 에서 **Primary entry point** 로 승격. `bootstrap` 과 `attach` 는 `[Advanced]` 로 재기술 — 손수-author 했거나 partial-scaffold 워크플로에 유용하지만 일상 사용자는 `init` 를 돌리라고 안내.
  - **B**: 새 top-level `genasis publish` subcommand — `genasis trial publish` 와 같은 `PublishArgs` (`--dry-run`, `--project`) 사용. `trial publish` 도 여전히 동작하지만 deprecation note. 신규 `cmd_trial::run_publish_with_project` 가 두 경로에 공유됨.
  - **C**: `genasis lang` 이 실행 전 `genasis attach --lang=<en|ko>` 를 가리키는 deprecation note 출력.
  - **D**: `genasis upgrade` 가 실행 전 `genasis attach --upgrade` 를 가리키는 deprecation note 출력.
  - **E**: `genasis plane` / `genasis mm` 이 `genasis doctor --probe-plane` / `--probe-mm` 를 가리키는 deprecation note 출력.

  Deprecated subcommand 들은 모두 이전과 똑같이 동작 — help-text 분류와 런타임 stderr note 만 추가. v0.7.0 에서 제거.

  검증:
  - `cargo fmt --all` clean
  - `cargo clippy -p genasis-cli -p genasis-overlay --tests -- -D warnings` clean
  - `cargo test --workspace`: **269 passed, 4 ignored**
  - trial-app `npm run typecheck` + `npm run build`: clean
  - i18n drift gate: **148 keys OK**
  - `v0.5.3` 태그 푸시가 `release.yml` 트리거 (cross 로 musl-static x86_64 + aarch64, Verify-static-linking + compat-smoke 게이트, 두 tarball + sha256 첨부된 GitHub Release).

- 2026-05-12: **v0.5.2 릴리스 — v0.5.0 현장 결함 11개 중 8개 fix + 바이너리 배포**. End-to-end Quick Path 테스터가 v0.5.0 에 대해 11개 결함을 보고; 이번 릴리스가 그 중 8개를 닫음 (3개는 workaround 와 함께 known limitations 로 문서화). `workspace.version = "0.5.1"` → `"0.5.2"` bump, `v0.5.2` 태그. 수정:
  - **`run_trial` 에이전트 부트스트랩 체인** (사용자가 보고한 "팀 폴더의 .claude/agents 비어있음" 주증상): `genasis init --trial` 이 빈 `.claude/agents/` 만 만들고 종료했음 → 사용자가 `genasis bootstrap` 을 직접 찾아 실행해야 했음. 이제 `run_trial` 이 `lang_flag` / `non_interactive` / `assume_yes` 를 thread 하고, `genasis.toml` 작성 직후 `cmd_bootstrap::run` (자동 `cmd_attach` chain 포함) 호출. `--probe-only` 는 여전히 bootstrap skip — 테스트가 catalog-fetch 비용 안 치름. 실패는 warning 으로 surface, init 중단 안 함 — team_token 은 여전히 받음.
  - **Issue #11**: `cmd_attach` 가 README 가 늘 약속하던 slash command / hook / skill / `GENASIS.md` 를 한 번도 설치 안 했음 — v1.0.0 카탈로그에 .tera 템플릿 다 있음에도. 새 `install_genasis_overlay_artifacts()` 헬퍼가 `commands/*.tera` → `.claude/genasis/commands/*.md`, `hooks/*.tera` → `.claude/genasis/hooks/*.{sh,md}` (`.sh` 는 0755), `skills/*.tera` → `.claude/genasis/skills/*.md`, `<lang>/GENASIS.md.tera` → 프로젝트 루트 `GENASIS.md` 로 렌더. 18 개 slash command (`/sprint-start`, `/issue-done`, `/db-migrate`, …) + post-tool-trim hook + GENASIS.md 프로토콜 계약이 이제 attach 에서 떨어짐.
  - **Issue #6**: `servers/docker-compose.yml` 에 `networks.default.aliases: [web|api|space|admin|live]` 를 `plane-{web,api,space,admin,live}` 서비스에 추가. Plane 의 내장 proxy Caddyfile 이 bare hostname 을 기대 — alias 없으면 `localhost:${PLANE_PORT}` 모든 요청이 `dial tcp: lookup web: i/o timeout` 으로 502. Fix.
  - **Issue #5**: README + README.ko 의 `Plane at localhost:8080, Mattermost at localhost:8065` 가 거짓 — `setup-user-env.sh` 는 실제로 `38400`/`38500` + `uid % 50` offset 할당. §"Option B" 안내를 allocator 설명 + `.env` 에서 할당된 포트 확인 방법으로 다시 씀.
  - **Issue #8**: `UpstreamPlane::health` 가 `/api/v1/health/` 를 probe 했는데 Plane v1.2.3 가 `{"error":"Page not found."}` 반환 — non-fatal 이지만 `genasis init` 출력에 보기 흉함. `/api/v1/workspaces/<slug>/` 로 전환 — stable, 200/401/404 (모두 "server up") 깨끗하게 반환.
  - **Issue #10**: `UpstreamPlane::ensure_project` 가 존재 여부와 무관하게 `/projects/` POST → `genasis init` 재실행 시 "The project name is already taken" 으로 실패. `find_project_by_name_or_identifier` (paginated `/projects/?next=...` walk) 추가, 매치 시 기존 id 반환.
  - **Issue #9**: `MM_TEAM_ID` 가 채널 프로비저닝에 필수인데 문서화 안 됨. 이제 `[mattermost].team_name` 에서 자동 해석 — `GET /api/v4/teams/name/<name>` 을 이미 in-scope 인 `MM_ADMIN_TOKEN` 으로 호출. lookup 실패 시 legacy "skipped" 메시지로 fallback. README 가 `MM_TEAM_ID` 를 fallback env var 로 명시.
  - **`tera` dep 을 `genasis-cli` 에 추가** — `cmd_attach` 가 command/hook 템플릿 렌더하도록. v1.0.0 카탈로그 템플릿은 사실 Tera 변수 미사용이지만 `Tera::one_off` 는 forward-compatible.
  - **main 에는 있지만 미릴리스** (1, 2, 3, 4): ADR-016 + ADR-017 멀티테넌시 + 쇼케이스 + Linux musl-static release config + `install.sh --lang ko` 공백 형식 파싱 — 로컬 전용 `v0.5.1` 태그와 `v0.5.2` 사이 13 commit 전부 이 릴리스에 land.

  **Known limitations 로 문서화** (다음 패치 미룸):
  - **Issue #7**: Plane 이 plain HTTP 위에서 CSRF 쿠키에 `Secure` 붙임 → 브라우저 sign-up 실패. Workaround 문서화 (호스트 Caddy + self-signed cert, 또는 브라우저 CSRF override).
  - **Issue #5a**: `genasis agents list/install/browse` 실패 — `index.json` 이 `manifest.json` 의 copy 로 publish 됨 (필수 `agents` / `presets` / `categories` 배열 누락). `agents-pool` 에서 tracked — 바이너리 변경 없이 fix land.

  `cargo test --workspace` **269 passed**, 4 ignored (이전 254). `cargo clippy -D warnings` touched crates clean. trial-app `npm run build` clean. `v0.5.2` 태그 푸시가 `release.yml` 트리거 → `x86_64-unknown-linux-musl` + `aarch64-unknown-linux-musl` cross 빌드, `Verify static linking` gate, `compat-smoke` (debian:bullseye) gate, GitHub Release 에 두 tarball + sha256.
- 2026-05-12: **Live Trial UX 정돈 — disconnected 뷰가 ChatSidebar 에 가려지지 않게**. explicit-gating commit 직후 보고: `ChatSidebar`(`absolute right-0 top-0 z-30 h-full`)가 DOM 을 거슬러 올라가며 가장 가까운 `relative` ancestor 를 찾았는데, `<section class="relative ...">`(Live 탭의 외곽 wrapper)를 만나 거기 기준으로 떠 있어서 TokenBar 의 오른쪽 — Connect 버튼 — 위로 떠다녔음. 연결 전·후 모두. 두 가지 변경: (1) `LiveBoard.tsx` 가 kanban+chat 스테이지를 새 `relative h-[630px]` div 로 감싸, ChatSidebar 의 positioning context 를 스테이지로 한정 — 그 위의 TokenBar 는 침범 받지 않음. `disabled` prop 은 완전 제거(disconnected 케이스는 이제 page 레벨에서 처리). (2) `page.tsx` 가 disconnected 일 때 LiveBoard 를 아예 렌더하지 않음. 대신 새 `DisconnectedLive` 가 `max-w-3xl` 중앙 정렬 TokenBar + 작은 "연결 후 활성화" 카드(칸반/채팅/쇼케이스 3줄)만 반환. unknown-token 에러는 같은 쉘 재사용 + amber alert 한 줄 추가. 결과: Connect 버튼이 절대 가려지지 않고, disconnected 페이지가 단일 CTA 에 시각적 집중, connected 페이지는 칸반+채팅 의도된 full height 확보. 신규 i18n 키 (KO+EN): `live.intro.compact`, `live.disconnected.heading`, `live.disconnected.benefits.{kanban,chat,showcase}`. `/` route 20.9 kB → 15.3 kB 로 축소 (placeholder LiveBoard + dim-state CSS 제거). `npm run typecheck` + `npm run build` 깨끗.
- 2026-05-12: **Live Trial 명시적 팀-토큰 게이팅 (ADR-017 §6 amendment)**. 쇼케이스 ship 이후 현장 피드백: 익명 방문이 조용히 `DEFAULT_TEAM_TOKEN` 샌드박스로 떨어져 multi-partition 이야기가 헷갈렸음 — 사용자가 어느 칸반이 "자기 것"인지 분간 못 했고, per-team 랜딩 URL을 다른 머신에 붙여넣었을 때 어떤 네비게이션이라도 `?team=` 쿼리를 비우면 공유 샌드박스로 silently 떨어졌음. auto-fallback 렌더링 제거. 신규 client 컴포넌트 `app/components/TeamTokenBar.tsx` 가 Live Trial 상단에 위치해 토큰 영속성 단일 소유자 역할. `app/page.tsx` 의 토큰 resolution 이 이제 URL → 쿠키 → 빈 값 (`DEFAULT_TEAM_TOKEN` 기본값 없음). 빈 값 → LiveBoard 가 `disabled` 모드 (`pointer-events-none + opacity-40`) + `live.disabled.overlay` 배너로 렌더되어 사용자가 연결 후 받을 화면을 미리 봄. TokenBar 는 붙여넣은 토큰을 `GET /api/trial/team-app/status?team=...` 로 검증 (이 amendment 에서 `team_exists` + `project_name` 반환하도록 확장), 1년 쿠키 (`genasis-trial-team`) + localStorage 에 영속화, `router.replace` 로 `/?tab=live&team=<token>` 네비게이션하여 SSR 패스가 새 tenancy 인식. CLI 쪽: `genasis init --trial` 종료 시 복사 친화 ASCII-bar 요약 출력 — 프로젝트명, `team_token`, pre-fill 된 랜딩 URL. TokenBar 의 "팀 토큰을 입력하세요" 카피와 동일 언어라 사용자가 URL 을 붙여넣든 토큰만 붙여넣든 일관된 가이드 표시. README/TUTORIAL EN+KO 가 paste-token 워크스루로 업데이트, ADR-017 §6 가 디자인 문서화. `live.tokenbar.*` 와 `live.disabled.overlay` 에 신규 i18n 키 18개 (KO+EN). trial-app `npm run typecheck` + `npm run build` 깨끗, `cargo test -p genasis-cli --bin genasis run_trial` 3 passed 유지.
- 2026-05-12: **현장 피드백 2회차 — install.sh `--lang` 문서/구현 정합 + Linux musl-static 릴리스**. 사용자 보고 두 가지 (서로 무관):
  (1) `install.sh --lang ko` (공백 형식) 가 unknown flag로 silently rejected — `--lang=ko` (등호 형식) 만 파싱되고 있었음. 그런데 모든 doc 문자열(help text, error banner, README) 은 공백 형식을 쓰고 있었음. arg 루프를 `for arg in "$@"` 에서 `while [ $# -gt 0 ]` + 명시적 `shift` 로 재작성, `--lang ko` 와 `--lang=ko` 둘 다 수용 (`--prefix`, `--version` 도 동일). help text 가 dual-form 수용을 명시.
  (2) 릴리스 바이너리가 `GLIBC_2.39` floor 를 박고 있었음 — `release.yml` 이 `ubuntu-latest` (현재 24.04) 에서 `x86_64-unknown-linux-gnu` 로 빌드했기 때문. Linux 매트릭스 두 엔트리 모두 `*-unknown-linux-musl` 로 전환, `cross` 가 musl 이미지를 자동 선택하므로 `apt install musl-tools` 보일러플레이트 불필요. `Cargo.toml`에 이미 있는 `rustls-tls` feature flag 덕에 OpenSSL/libssl 의존도 없음 — 깔끔하게 정적 링크. `Verify static linking` 스텝이 `file` 로 빌드 산출물을 확인하고 "dynamically linked"가 나오면 빌드 실패 — 우발적 glibc 의존 재유입을 막는 가드. `compat-smoke` 잡이 매 태그마다 패키징된 x86_64 바이너리를 `debian:bullseye` (glibc 2.31) 컨테이너에서 실행. `macos-latest` 매트릭스 두 엔트리는 삭제 — Apple Silicon notarisation flow 가 정리 안 됨. README §지원 플랫폼이 macOS 를 **TBD** 로 표기 + 로드맵 노트, `install.sh` 가 Darwin 에서는 존재하지 않는 다운로드를 시도하는 대신 "소스에서 빌드하라"는 명확한 메시지 출력. 양국어 README 플랫폼 표가 4행 × 2열 → 5행 × 3열 (Pre-built / Build-from-source / 비고) 로 확장. 신규 `cargo test` 케이스 없음 — 이건 인프라 변경이고 실제 커버리지는 릴리스 파이프라인 자체임.
- 2026-05-11: **trial-app 쇼케이스 모델 (ADR-017)**. ADR-013/016이 남긴 신뢰성 갭을 닫음 — 스크립트 `Try it` 탭이 라이브 모드와 같은 칸반 + 채팅 위젯을 애니메이션하므로 첫 방문자는 어느 쪽이 "진짜"인지 분간 못 했고, 레퍼런스 PRD("Example Feature — Task Status")가 에이전트에게 trial-app 자체와 시각적으로 동일한 것을 만들라고 요구. 네 가지 조정 변경: (1) 스크립트 데모 제거 — `DemoBoard.tsx`, `ChatThread.tsx`, `KanbanBoard.tsx`, `lib/{use-demo-sprint,demo-script}.ts`, `e2e/demo.spec.ts`, 모든 `demo.*` i18n 키, `tab=demo` URL 핸들러, `TrialTab="demo"` 변종 삭제. 랜딩 탭은 이제 `live`. (2) i18n 인식 example PRD — `cmd_example.rs`가 `genasis.toml`의 `[i18n].active`를 읽어 `prd.en.md` 또는 `prd.ko.md`를 emit. 두 PRD가 새 레퍼런스 앱 — "나는 Claude Code 전문가 / I Am a Claude Code Expert" 모바일 폰 프레임 5문제 자가 진단 퀴즈 (3난이도, 문제 은행 ≥ 15) — 를 설명. (3) 임베디드 쇼케이스 — 신규 `app/components/QuizApp.tsx` + `lib/quiz-bank.ts`가 레퍼런스 퀴즈를 trial-app 내부에 ship, 신규 `sim_teams.app_status` 컬럼 (V2 → V3 마이그레이션, ADR-016 §3 패턴 재사용)으로 팀별 게이팅. 신규 `ShowcasePanel.tsx`가 LiveBoard 좌측에서 슬라이드 인, Esc/외부 클릭/✕로 닫힘. (4) 명시적 완료 신호 — 신규 `genasis trial publish` CLI가 `{team_token, status: "complete", project}`를 `/api/trial/team-app/status`에 POST (이 라우트도 unauth, ADR-016 §4의 token-as-capability 모델 확장). Apply 탭 → "실환경 빌리기" / "Borrow real env" — 사용자가 운영자 `mmplane-trial.realstory.blog` 인프라의 실제 Plane + MM 프로젝트를 말 그대로 "빌리는" 것이므로. README 링크 `trial.realstory.blog` → `https://mmplane-trial.realstory.blog/?tab=signup` 으로 업데이트. QUICKSTART (EN+KO), blueprint.ko §22.2, agents-pool/prd/trial-webapp.md, playwright.config.ts, e2e/signup.spec.ts에도 같은 sweep 적용. 신규 테스트: cmd_example × 4 (en/ko/명시-플래그/config-없음), cmd_trial × 3 (dry-run/토큰-없음/config-없음). `cargo test --workspace` 266 passed, 4 ignored (259 → +7). trial-app `npm run typecheck` + `npm run build` 깨끗; `/api/trial/team-app/status`가 route 테이블에 표시.
- 2026-05-11: **ADR-016 후속 — 토큰 전파 + SSE 격리 + fallback UX (Phase A + B)**. 첫 ADR-016 커밋(3760b07)은 `team_token`을 config 파일까지만 전달했음 — Rust trial 프로바이더는 여전히 `X-Genasis-Team-Token`을 안 보내고, 신규 `/api/trial/bootstrap`은 운영자 호스팅 인스턴스 기본 상태(`TRIAL_SHARED_SECRET` 미설정)에서 항상 503, SSE 이벤트 버스는 글로벌이어서 cross-tenant 업데이트가 모든 연결 탭에 새었다. Phase A: `TrialPlane` / `TrialMattermost` 생성자가 3번째 인자 `team_token: String`을 받고, `headers()`가 비어있지 않을 때 `X-Genasis-Team-Token`을 붙임. 두 factory가 `TrialConfig`에서 `t.team_token.clone().unwrap_or_default()`로 전달. Phase B: `/api/trial/bootstrap`이 `requireTrialContext` 제거 — body의 32자 hex `team_token`이 단일 자격증명 (멱등 + 무작위, ADR-016 §4 참조). `lib/events.ts`의 `subscribe()`가 optional `teamToken` 필터를 받고 `emit()`이 `event.payload.team_token`과 매칭. `/api/events/stream`이 `?team=`을 읽어 매칭 이벤트만 forward. `LiveKanbanBoard` / `LiveChatThread`는 `EventSource` URL에 `?team=<token>`을 붙임 (브라우저는 EventSource에 커스텀 헤더 부착 불가). `page.tsx`에 unknown 토큰 fallback 추가 — `?team=<token>`이 있지만 `getTeam(token) === null`일 때 amber 에러 패널 ("팀 토큰을 찾을 수 없습니다 — genasis.toml의 `[trial].team_token` 확인") 표시, default sandbox로 조용히 떨어지지 않음. `LiveBoard`에 `data-team-token` 속성 + color-coded 배지 추가로 현재 어느 테넌시인지 늘 표시. 신규 테스트: `plane/trial.rs` + `mattermost/trial.rs`에 헤더 테스트 5개. ADR-016 EN+KO에 §"인증 모델 — 토큰이 곧 capability" 확장. `cargo test --workspace` 259 passed, 4 ignored (254 → +5). trial-app `npm run typecheck` + `npm run build` 깨끗.
- 2026-05-11: **trial-app 식별자 정렬 + 멀티테넌시 (ADR-016)**. ADR-013은 genasis ↔ trial-app 사이의 *라우팅*을 정의했지만 그 라우트로 흘러가는 *식별자*는 다루지 않아 — `genasis init --trial`이 사용자가 어떤 팀을 만들든 모든 Plane/Mattermost 필드를 리터럴 `"trial"`로 박았고, trial-app sim은 팀별 격리가 없어 호스팅 인스턴스에서 동시 데모가 서로의 데이터를 덮어썼다. 세 가지 변경을 함께 출하: (1) 실모드 schema에 `[plane].project_name` + `[[mattermost].channels]` (`MattermostChannel { key, name, display_name }`) 추가, 레거시 config는 `Config::derive_naming_defaults()`가 단일 `scrum` 채널을 합성; (2) `[trial].team_token` (32자 hex, `random_team_token()`)이 팀별 격리 키로 동작, `genasis init --trial`이 기록하고 ADR-016 이전 config는 `"default"` 센티넬로 fallback; (3) trial-app sim이 `user_version = 1` → `2`로 마이그레이션 — 모든 `sim_*` 테이블에 `team_token` + 복합 `UNIQUE(team_token, slug|name)` 추가, 신규 `sim_teams` 테이블, `POST /api/trial/bootstrap` 라우트로 토큰 아래 프로젝트·채널 시드. `lib/trial-auth.ts`는 `X-Genasis-Team-Token` 헤더 → `?team=` 쿼리 → `DEFAULT_TEAM_TOKEN` 순으로 토큰 해석. 브라우저 UI(`page.tsx` + `LiveBoard` + `LiveKanbanBoard` + `LiveChatThread`)는 `?team=<token>` SSR부터 모든 `fetch()`까지 `withTeamHeader`로 토큰 전파. `cmd_init.rs` 실모드는 더 이상 `scrum-{project_name}` 문자열 포맷을 쓰지 않고 `cfg.mattermost_channel("scrum")`을 조회. `cmd_init.rs --trial`은 `--name`을 받거나(또는 dirname에서 유도) 토큰을 생성, 동적 템플릿을 렌더하고 `/api/trial/bootstrap`에 POST한 뒤 `/?tab=live&team=<token>`을 브라우저로 연다. 신규 테스트: `genasis-core::config` 7개 (slugify, random_team_token, derive_naming_defaults, effective_team_token, mattermost_channel lookup, channels TOML 라운드트립), `cmd_init::tests` 3개 (project name flag, dirname 유도, 기존 config 보존). ADR-016 EN/KO 양쪽 작성. `cargo test --workspace` 254 passed, 4 ignored.
- 2026-05-08: **Phase G audit + 체크박스 catch-up**. `progress.md`/`progress.ko.md` 를 실제 리포 상태(커밋 e0683de..5bdaadf, build.sh, CONTRIBUTING.md, docs/CREDITS.md)에 맞춰 정리. Phase G 상태 표 G.1~G.8 을 `planning` → `done` 으로 일괄 전환. Trial-app US-001..US-022 모두 `passes: true` (`trial-app/ralph/prd.json`) — 대응 G.5 sub-checkbox 닫고, G.6 (`genasis init --trial`), G.7 (`cmd_example.rs` + 3개 예제 템플릿), G.8 (TUTORIAL.md en/ko + README Quick Path/단계별 재구성 + CLAUDE.md 미러 표) 도 모두 `[x]`. `[s]` 로 남긴 항목 1개: 예제 문서의 영/한 양쪽 버전 — `genasis example --lang` 플래그가 도입되기 전까지는 active-singularity 정책에 따라 영어판만 유지. 미구현으로 남는 영역: M14.0 ratify gate, M14.3~M14.5 (CLI bootstrap wire-up + golden blank fixture + doctor 섹션), Phase F Debug History 루프 전체 (M15~M17).
- 2026-05-19: **v0.6.0-alpha.44 — D-131 CLAUDE USAGE 가 `/api/oauth/usage` 의 라이브 값을 가져오도록**. alpha.43 후속: D-130 휴리스틱 fix 적용 뒤에도 사용자의 CLAUDE USAGE 패널은 여전히 `0% / 0% / 0% / 0%` 였고, 함께 첨부된 Anthropic 공식 사용량 페이지(`claude.ai/settings/usage`)는 `9% / 12% / 2% / Claude Design 0%` 였다 — 두 이미지 비교. 원인: D-130 에서 만든 JSONL 집계기는 로컬 토큰 합을 static / tier-derived budget 으로 나눈 값을 보여주지만, 서버 측 rate-limiter 는 다른 가중치(cache rate, multi-window 상호작용, plan-specific carve-out)를 적용한다. 서버 숫자를 직접 읽지 않는 한 % 가 일치할 수 없다. 조사 결과 Claude Code CLI(`@anthropic-ai/claude-code` v2.1.144 x86_64 ELF) 가 `~/.claude/.credentials.json` → `claudeAiOauth.accessToken` 로 `GET /api/oauth/usage` 를 호출한다는 사실 발견 (`strings` + grep 으로 `fetchUtilization: GET /api/oauth/usage` 시그니처 확인, probe `curl` 로 full 응답 캡처). 응답 스키마: `{five_hour: {utilization, resets_at}, seven_day, seven_day_oauth_apps, seven_day_opus, seven_day_sonnet, seven_day_cowork, seven_day_omelette (= Claude Design), tangelo, iguana_necktie, omelette_promotional, extra_usage: {is_enabled, monthly_limit (cents), used_credits (cents), utilization, currency, disabled_reason}}`. probe 응답: `seven_day: 12.0`, `seven_day_sonnet: 2.0` — 설정 페이지 스크린샷과 정확히 일치. **구현:** 신규 `crates/genasis-monitor/src/collector/oauth_usage.rs` 모듈. serde-deserialize `OAuthUsage` / `UsageWindow` / `ExtraUsage` 구조체로 캡처된 스키마 커버. `fetch()` async 함수가 bearer 읽고 `https://api.anthropic.com/api/oauth/usage` 를 `reqwest` (이미 workspace dep, `rustls-tls` + `json` feature 활성) 로 5 s timeout, CLI 와 동일한 `anthropic-beta: claude-code-20250219` header + `User-Agent: genasis-monitor/<ver>` 첨부해 호출. 토큰 만료 가드: `expiresAt` (epoch ms) 를 60 s 마진으로 검사해 곧 만료될 토큰이면 silently `Ok(None)` 리턴 → monitor 는 JSONL fallback 으로 전환, 매분 auth 에러 로깅 없음. 401 도 `Ok(None)` 으로 폴백 (Claude Code 가 다음 interactive run 에서 token 회전하므로 노이즈 로깅 무의미). 4xx/5xx with valid bearer 는 `Err` → `log_tail` 에 `HH:MM (oauth usage fetch failed: ...)` 한 줄. 비활성화 knob: `MONITOR_DISABLE_OAUTH_USAGE=1`. OAuth bearer 는 `ANTHROPIC_API_KEY` 가 아님 — 사용자 메모리 (`feedback_no_claude_api.md`) 의 금지 대상은 Console API key (inference 용); 이건 stat 용 user-scoped OAuth bearer 로 이미 박스에 존재. Claude Code 자체가 같은 메커니즘 사용. **Snapshot 배선:** `UsageSnapshot` 에 신규 옵셔널 / live 필드 11 개 — `oauth_five_h_pct`, `oauth_seven_day_pct`, `oauth_seven_day_opus_pct`, `oauth_seven_day_sonnet_pct`, `oauth_seven_day_design_pct` (= `seven_day_omelette`), `oauth_five_h_resets_at`, `oauth_seven_day_resets_at`, `oauth_extra_used_credits_cents`, `oauth_extra_monthly_limit_cents`, `oauth_extra_pct`, `oauth_fetched_at`. 신규 `apply_oauth_usage()` 가 응답을 snapshot 으로 복사. 신규 `five_h_oauth_countdown()` 가 server `resets_at` 카운트다운 반환. `collect_jsonl()` 이 JSONL 새로고침 시 이 11 개 필드를 carry-over 하도록 수정 — `scan_sessions_dir()` 는 fresh `UsageSnapshot::default()` 를 반환하므로 carry 안 하면 방금 받은 OAuth 값이 매분 지워짐. **App loop:** 신규 `OAUTH_USAGE_TICK = 60 s` (Claude Code 자체 캐싱 cadence 와 일치) + startup 1 회 + `r` (refresh) 키 누름 시 트리거. **위젯:** 게이지들이 이제 `state.usage.oauth_*_pct.unwrap_or_else(|| <JSONL fallback>)` 패턴이라 server 값 우선, OAuth 가 없을 때만 JSONL 사용. 세 번째 게이지가 `7d (Opus)` (서버 보고시) 또는 `7d (Design)` (`seven_day_omelette` 만 있을 때) — Anthropic 페이지의 "Claude Design" 행과 매칭. magenta vs cyan 색상 구분. 비용 라인 재작성: `extra_usage` 가 있으면 `Credits $0.00 / $200 · est 5h $X` (설정 페이지의 "사용 크레딧 / Usage Credits" + "월간 지출 한도" 와 정확히 일치), 없으면 D-130 추정값 표시. Reset 라인에 `source: live` (OAuth) / `source: local` (JSONL) 태그 추가. 빈 상태 hint 로직 갱신: JSONL 비어있고 **동시에** OAuth fetch 미수신일 때만 "No Claude activity in last 5h" 표시 — 새 테스트 베드처럼 로컬 JSONL 은 비어있어도 서버 데이터는 0 이 아닐 수 있음. **실측 검증** (dev host, `cargo run --example usage_dump`): `five_hour: 11.0` (설정 페이지 9% 와 1–2 분 차이), `seven_day: 12.0` (정확 일치), `seven_day_sonnet: 2.0` (정확 일치), `seven_day_omelette: 0.0` ("Claude Design", 정확 일치), `extra_usage.monthly_limit: 20000` ($200, 정확 일치), `extra_usage.used_credits: 0.0` ($0.00, 정확 일치). 신규 테스트: `parses_live_fixture` (캡처된 응답으로 serde 모양 락), `parses_resets_at` (ISO 8601 → epoch), `empty_response_ok` (서버가 `{}` 반환해도 graceful). Workspace 276 passed (273 → +3). 게이트: `cargo fmt --all -- --check` clean; `cargo clippy -p genasis-monitor --all-targets` clean. workspace.version `0.6.0-alpha.43` → `0.6.0-alpha.44`.
- 2026-05-19: **v0.6.0-alpha.43 — D-130 CLAUDE USAGE 위젯 end-to-end fix (monitor)**. 사용자가 새 테스트 베드에서 첨부한 스크린샷이 `5h session 0%`, `7d (all) 0%`, `7d (Sonnet) 0%`, `$0.00 / $200`, `Reset: reset pending` — SESSIONS 패널에는 1일짜리 Claude Code 세션이 다섯 개 잡혀 있는데도 CLAUDE USAGE 의 모든 숫자가 죽어 있었음. `crates/genasis-monitor/src/collector/jsonl.rs` + `widgets/sessions.rs` 감사 결과 결함 4 개. (1) `five_h_cost_usd` 필드가 `UsageSnapshot` 에 선언만 있고 **워크스페이스 어디에서도 대입되지 않음** — 비용은 누구에게나 영구 $0.00. (2) `five_h_reset_epoch` / `week_reset_epoch` 은 레거시 `rate_limit_event` JSONL 레코드에서만 채워지는데, Claude Code v2.1.x 가 더 이상 그 이벤트를 emit 하지 않음 (로컬 156 개 `.jsonl` 전수조사 결과 0 건) → countdown 영구 음수 → `format_countdown(<=0)` 이 "reset pending" 반환. (3) `read_credentials` 는 톱-레벨 `plan` / `planName` / `tier` / `serviceTier` 만 읽었는데 실제 `~/.claude/.credentials.json` 구조는 `{"claudeAiOauth": {"subscriptionType": "max", "rateLimitTier": "default_claude_max_20x"}}` — 모든 사용자에게 `plan: "unknown"` 반환, tier-aware 한도 매핑이 안 걸려서 사용자가 실제로 Max (20x) 든 무엇이든 `MONITOR_5H_TOKEN_LIMIT=7M` 임의 상수가 분모로 들어감. (4) `week_sonnet_*` 는 `model.contains("sonnet")` 일 때만 누적 — Opus 위주 사용자 (genasis dev box 전체가 `claude-opus-4-7`) 는 매일 수백만 Opus 토큰을 써도 Sonnet 바가 영구 0 %. 한 PR 에 3 단계 fix. **Phase A — 죽은 필드 살리기:** 신규 `pricing_for(model)` 테이블 (Claude 4.x: Opus $15/$75, Sonnet $3/$15, Haiku $1/$5 — 입력/출력 per MTok; cache_read = input × 0.1, cache_create = input × 1.25, Anthropic 프롬프트 캐싱 문서값). `scan_single_file` 이 매 `assistant` 이벤트마다 `five_h_cost_usd` + 신규 `week_cost_usd` 누적. `read_credentials` 는 `claudeAiOauth.subscriptionType` + `.rateLimitTier` 우선 읽고, 옛 톱-레벨 키도 fallback 유지 → 레거시 credential 파일도 그대로 동작. 신규 `plan_display_name()` 이 tier 문자열을 사용자 친화 라벨로 매핑 ("Max (20x)", "Pro", "Team", …). 신규 `tier_to_limits(tier) → Option<(5h, week_all, week_opus, week_sonnet)>` 조회 — `app.rs` 가 이 값으로 기본 budget 을 세팅하므로 default 가 실제 플랜과 일치. env override (`MONITOR_*_TOKEN_LIMIT`) 가 있으면 그쪽이 여전히 우선. **Phase B — 정확도:** `UsageSnapshot.five_h_oldest_event_ts` + `week_oldest_event_ts` 가 윈도우 내 가장 오래된 assistant 이벤트 시각을 추적; `five_h_reset_countdown()` 이 `rate_limit_event` epoch 없을 때 `oldest_ts + 5h - now` (가장 오래된 호출이 슬라이딩 윈도우에서 빠져 나가는 순간) 를 반환 — 더 이상 영구 "pending" 아님. `ModelFamily::{Opus, Sonnet, Haiku, Other}` 분류기로 주간 합계를 family 별 카운터에 분리 누적 (`week_opus_input/output/cache_read/cache_create`, `week_haiku_*`, `week_other_*`); 위젯에 `7d (Opus) X%` magenta 게이지 추가 (Max 플랜은 Opus / Sonnet 을 별도 트랙으로 카운트). 신규 `five_h_pct_weighted()` 가 캐시 가중치 적용 % (`input + output + cache_create × 1.25 + cache_read × 0.1`) 를 노출 — 서버 측 limiter 가 실제로 보는 값에 가까움. 신규 `state.limit_week_opus_tokens` 필드. **Phase C — UX 가드:** 신규 `UsageSnapshot::is_empty_5h()` 가 5h 윈도우에 assistant 이벤트 0 건이면 true 반환; 위젯이 4 개 0 % 게이지 대신 두 줄 hint ("No Claude activity in last 5h. / Run `claude` in any project to populate.") 렌더 — 새로 깐 / clean-env 사용자에게 위젯이 "고장난 것처럼" 보이던 문제 제거. CLAUDE USAGE 블록 제목에 plan 라벨 append (` CLAUDE USAGE — Max (20x) `). 레이아웃은 4 × 2-line → 6 × 1-line (5h / 7d-all / 7d-Opus / 7d-Sonnet / 비용 / reset) 로 재배치 — 기존 8 행 슬롯 안에 그대로 fit. 비용 라인을 `5h $X.XX · 7d $Y.YY · cap $Z` 로 재작성하여 오늘과 이번 주 burn 을 동시 비교. Reset 라인은 countdown 이 ≤0 이면 "—" 표시 ("reset pending" 대신). 실측 검증 (dev host): `cargo run --example usage_dump` → `creds plan: Max (20x)`, `tier_to_limits: 5h=4400000 week_all=100000000 week_opus=40000000 week_sonnet=100000000`, `five_h_cost_usd: $91.06`, `week_opus_input: 204817` — 모두 D-130 이전엔 0 / "unknown" 이던 값. 신규 테스트: `scan_populates_cost`, `opus_cost_higher_than_sonnet`, `reset_countdown_from_oldest_event`, `tier_lookup`, `plan_label_max_20x`, `weighted_pct_includes_cache`, `scan_empty_dir_returns_defaults` 에 `is_empty_5h()` 단언 추가. 기존 `examples/detect_dump.rs` 의 자매 도구로 `examples/usage_dump.rs` 신설 — 차후 사이클에서 ratatui 띄우지 않고도 JSONL 집계를 spot-check 가능. 게이트: `cargo fmt --all -- --check` clean; `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo test --workspace --tests --exclude genasis-e2e` monitor 17/17 통과 (인접 크레이트 회귀 없음). workspace.version `0.6.0-alpha.42` → `0.6.0-alpha.43`.
- 2026-05-17: **v0.6.0-alpha.42 — D-124 install.sh 가 cwd 안 건드림 + D-123 부분 hardening (monitor TTY 게이트)**. alpha.41 검증 도중 발견된 현장 follow-up 두 건. **(1) D-124 — CLOSED.** `install.sh` 가 `RUN_AFTER_INSTALL=1` default 로 `genasis attach` 를 사용자 cwd 에 자동 실행했음. `/work/agenteams` 에서 `curl ... | sh` 으로 binary version 만 확인하려던 사용자가 거기에 `.claude/` + `.genasis/` 가 박힌 걸 발견 → 그 디렉터리가 nested sandbox 들로 Claude `.claude/` cascade 통해 overlay 누수 → D-122 와 같은 계열의 cross-contamination. Fix: `install.sh` `RUN_AFTER_INSTALL` default 를 `0` 으로 뒤집음; legacy / CI 흐름용 `--with-attach` flag 신설; 기존 `--no-run` 은 문서화된 no-op 으로 유지 (기존 호출 깨지지 않게). 헤더 코멘트 + help text 갱신, Next steps 배너가 README Quick Path (`cd <project> && genasis init --trial`) 를 가리키도록 정정 (기존엔 그냥 `genasis attach` 언급). **(2) D-123 — STILL OPEN.** 전날 `cargo test --workspace --all-targets` 가 25 GB anon-rss 에서 OOM-kill 당함. `journalctl -k` 가 kill 대상으로 지목한 프로세스 `agents-9360df0d` 는 cargo fingerprint 로 빌드된 `target/debug/deps/agents-<hash>` (= `tests/e2e/tests/agents.rs` integration binary) — 이전 Claude-Code 진단이 짚었던 production CLI `genasis agents browse` 가 아님. **정적 분석으로 `monitor_smoke_without_tty_exits_cleanly` (assert_cmd piped stdin/stdout 으로 `genasis monitor` 호출) 가 진범이라 추정했지만 실측 결과 틀림** — TTY 게이트 추가 후에도 새 `tests/e2e/tests/agents.rs` binary 가 여전히 5 GB RSS 누수, 즉 monitor 는 (적어도 단독으로는) 원인이 아니었음. monitor TTY 게이트는 그래도 defensive hardening 으로 ship (`app::run()` 이 이제 `std::io::stdout().is_terminal()` false 면 `io::ErrorKind::Unsupported` 로 `bail!`, ratatui 루프가 headless 환경에서 도는 걸 막음. monitor 가 누수 원인이 아니더라도 그 자체로 잘못된 코드 경로였음). D-123 진짜 root cause 는 아직 추적 중. **다음 사이클 진단**: `tests/e2e/tests/agents.rs` 9 개 test 를 individual `/usr/bin/time -v` 로 bisect (likely 후보는 `agents_browse_without_index_errors_cleanly` — `cmd_browse` 가 piped stdin 에서 dialoguer FuzzySelect 진입하므로. 다만 이건 실측으로 확인할 사항, 또 다른 정적-분석 추측이 되면 안 됨). 게이트: `cargo fmt --all -- --check` clean; `cargo clippy --workspace --all-targets` clean; `cargo test --workspace --tests --exclude genasis-e2e` 267 passed; `cargo test -p genasis-e2e` 는 D-123 미해결로 인해 이 ship 의 의도적 gate-out. alpha.42 는 사용자 즉시 영향 UX 버그 (D-124) fix + monitor hardening 의 opportunistic 현장 ship 이지 D-123 closure 가 아님. workspace.version `0.6.0-alpha.41` → `0.6.0-alpha.42`.
- 2026-05-17: **v0.6.0-alpha.41 — `app/` subdir 표준 layout 강제 (D-122)**. 사용자 보고: `mkdir my-team && cd my-team && genasis init --trial --name "My Team"` 한 다음 채팅에 작업 요청하면 에이전트가 `my-team/my-team/` 처럼 중첩된 폴더를 만들어 거기서 React 앱을 scaffold 하는 현상. 원인은 (1) frontend overlay 가 `npm create vite@latest .` (cwd 직접) 를 지시했고, (2) PRD 템플릿 §5 가 `<sandbox-root>/` 라는 모호한 표현 + 이미 폐기된 `announce_dev_server_url` v0.5.x 흐름을 그대로 사용, (3) daemon system prompt 도 scaffold 경로를 명시 안 함. 그래서 LLM 이 system prompt 의 `slug: my-team` 을 폴더명으로 해석해 sub-dir 을 만들었다. 한편 `cmd_push.rs` auto-detect 는 이미 `app/dist → app/build → app/out` 을 우선하므로 design intent 는 **항상 `app/` 하위** 였다 — overlay/PRD/system prompt 만 그걸 명시 안 했을 뿐. 수정 4 곳: (1) `agents/overlays/{en,ko}/frontend.patch.md.tera` — `npm create vite@latest app -- --template react-ts --yes` + `(cd app && npm install)`, 모든 `Edit`/`Write` 경로를 `app/src/...` 로, "`npm create vite@latest .` 금지" 명시. (2) `agents/overlays/{en,ko}/devops.patch.md.tera` — `(cd app && npm run build)`, `[ -f app/dist/index.html ]`, `genasis push --dir ./app/dist`, 그리고 layout contract 한 줄 명시. (3) `crates/genasis-cli/templates/examples/prd.{en,ko}.md` §5 — `<sandbox-root>/` 트리를 명시적 `my-team/.claude/+genasis.toml+PRD.md+app/` Claude 표준 layout 으로 교체, 폐기된 `announce_dev_server_url` 흐름 통째로 ADR-020 push-to-operator 모델로 재작성. (4) `crates/genasis-cli/src/listen/session.rs` daemon system prompt — frontend/devops 체인에 `app/` subdirectory 명시 + cwd 직접 scaffold 금지 경고. 결과: 사용자 프로젝트 루트 = `.claude/agents/` + `.genasis/` + `genasis.toml` + `PRD.md` + `app/` (React 앱) — Claude 표준 layout 그대로. `genasis push` 가 `app/dist` 를 auto-detect 하므로 별도 `--dir` 불필요. `cargo fmt --all` clean, `cargo clippy --workspace -- -D warnings` clean, `cargo test --workspace` 회귀 없음. workspace.version `0.6.0-alpha.40` → `0.6.0-alpha.41`.
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
| M14 | 2026-05-05 | 2026-05-08 | Bootstrap 을 attach 의 부수효과가 아닌 별도 서브커맨드로 빼낸 것이 정답 — `cmd_attach` 의 empty-dir 힌트만으로 진입점 발견은 충분하고 ADR-001 의 비파괴 약속도 그대로. `BLESS=1` golden snapshot 패턴(M14.4)이 M18 에 그대로 일반화. Doctor `[bootstrap]` 섹션은 기존 `Role::ALL` slug 리스트에 얹혀 새로운 검증 인프라 없이도 동작. frontmatter `name:` ↔ 파일명 stem 일치 invariant 가 fixture 작성 과정에서 자연스럽게 떠올랐다. |
