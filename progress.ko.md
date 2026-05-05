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
- [ ] 사용자 ratify 게이트 — ADR-010 머지 후 M14.3 진입 (M14.1/M14.2 는
  ratify 와 독립적으로 선행 가능 — base 템플릿 + 모듈은 코드만 추가,
  진입점 노출은 M14.3)

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

### M14.3 — CLI wire-up
- [ ] `crates/genasis-cli/src/cmd_init.rs` 에 `--bootstrap` flag 추가:
  - [ ] flag set 시 `plan_bootstrap` → `apply_bootstrap` → 자동 후속
    `cmd_attach` 호출 (또는 사용자에게 다음 step 안내)
  - [ ] flag unset + 빈 `.claude/agents/` → 기존 동작 유지하되 stderr 에
    "no agents detected — run `genasis init --bootstrap` to scaffold the
    default team" 안내 (i18n key)
- [ ] 또는 (대안) `cmd_attach.rs` 에 `--bootstrap` 추가 — ADR-010 에서 결정
- [ ] `genasis-i18n/locales/{en,ko}.yml` 에 키 추가:
  - `bootstrap.no_agents_hint`
  - `bootstrap.scaffolded_summary` (`{count} default agents created`)
  - `bootstrap.skipped_existing` (`{name} already exists, skipped`)
  - `bootstrap.next_step` (다음 `attach` 호출 안내)
- [ ] `--lang` 우선순위가 base + patch 양쪽에 동일하게 적용되는지 확인
  (base 트리도 `templates/<lang>/agents/` 에서 픽업)

### M14.4 — `tests/golden/blank/` 활성화
- [ ] `tests/golden/blank/input/` — 비어있는 mock project (Cargo.toml
  대신 README.md 정도만, `.claude/` 자체 없음)
- [ ] `tests/golden/blank/expected/` — `genasis init --bootstrap --lang en`
  실행 후 산출물 (10 base agent 파일 + GENASIS.md + .claude/genasis/* 등)
- [ ] `crates/genasis-overlay/tests/golden_blank.rs` 신규 — round-trip
  (bootstrap → attach → detach → 비교)
- [ ] `tests/golden/SHARED.md` 표에 blank 시나리오 행 갱신 (M2 의 stub
  → M14 active)
- [ ] (옵션) `tests/golden/blank-ko/` — `--lang ko` 변종

### M14.5 — Doctor + 회고
- [ ] `cmd_doctor.rs` `[bootstrap]` 섹션 추가:
  - [ ] `.claude/agents/` 존재 여부 + 파일 수 보고
  - [ ] 빈 디렉토리 + bootstrap 미실행 → suggestion 출력 (i18n)
  - [ ] base 파일들의 frontmatter `name:` 키가 모두 canonical role 인지 검증
- [ ] `progress.ko.md` 회고 슬롯 추가 (M14 시작/완료/학습한 것)
- [ ] DoD: `cargo test --workspace` green, `lint-i18n` green, golden blank
  round-trip green

### 리스크 / 미정
- **(a)** `init --bootstrap` vs `attach --bootstrap` 위치: ADR-010 에서
  결정 — `init` 은 Plane/MM provisioning 까지 묶여있어 무거움. 별도
  `genasis bootstrap` 서브커맨드 진입점 검토.
- **(b)** ECC `claude-code-templates` 와 차별화 문구: README.md (Comparison
  표) 의 "Non-destructive overlay" vs "Bootstrap" 두 차원으로 분리해야
  시각적 혼동 회피.
- **(c)** base 템플릿이 `tools:` 항목을 어디까지 specify 할지 — 너무 협소
  하면 사용자 자유도 침해, 너무 넓으면 무의미. 우선 ECC default
  (`Bash, Read, Write, Edit, Glob, Grep, Task`) 기준 + comment 로 안내.

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
