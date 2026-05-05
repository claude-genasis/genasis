> 한국어: [progress.ko.md](progress.ko.md)
>
> **Mirror sync policy**: This file and `progress.ko.md` are structural/content
> mirrors. When you modify one, you **must** update the other to match in the
> **same commit** (or the immediately following commit). Full policy:
> [`CLAUDE.md` §Bilingual Mirror Policy](CLAUDE.md).

# Genasis — Progress Tracker

> Follows milestones from [`blueprint.md`](blueprint.md) §15 (first-release scope) and §12 (repo structure).
> Each item uses `[ ]` → `[x]` on completion; blocked items are marked `[!]` with inline reason.

**Started**: 2026-05-03
**Target first release**: v0.1.0 (git tag after M14.0 ratification)
**Current milestone**: **M14 planning** + **Phase F design** (Debug History feedback loop, ADR-012).
M0–M12 + Phase D all complete. Phase F (M15–M17) designed 2026-05-05.
v0.1.0 release tag after M14.0 (ADR-010 ratify).

---

## Convention

- Checkbox notation:
  - `[ ]` = not started
  - `[~]` = in progress
  - `[x]` = completed
  - `[!]` = blocked (state reason)
  - `[s]` = skipped (state reason)
- New ADRs are written immediately upon discovery during work, in `docs/ADR/`
- Decisions requiring `blueprint.md` changes get their own ADR first, then blueprint amendment

---

## M0 — Bootstrap

Repo initial structure and progress tracking infra. **Full source tree under `genasis/`, install.sh is a launcher only.**

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
- [x] `Cargo.lock` — rustup stable installed + cargo build green, then committed (14 crates compiled).

### Crate stubs (Cargo.toml + minimal src skeleton)
- [x] `crates/genasis-cli/` (main.rs + cmd_*.rs stubs ×11 + tui_attach.rs + scripts/)
- [x] `crates/genasis-core/` (lib + config + env + fs + marker + error)
- [x] `crates/genasis-overlay/` (lib + detector + role_inference + merger + validator + dry_run)
- [x] `crates/genasis-providers/` (lib + plane/* + mattermost/* + github)
- [x] `crates/genasis-db/` (lib + kernel + guard + adapters/*) — guard has first unit tests
- [x] `crates/genasis-design/` (lib + extractor + change_protocol + diff + ticket_emitter)
- [x] `crates/genasis-tui/` (lib + theme + layout + widgets/*)
- [x] `crates/genasis-monitor/` (lib + app + state + widgets/* + actions/*)
- [x] `crates/genasis-templates/` (lib + templates/ Tera dir skeleton)

### install.sh launcher
- [x] OS/arch detection (linux x86_64/arm64, macOS arm64/x86_64, Windows→WSL guidance)
- [x] Linux distro detection (`/etc/os-release`)
- [x] Prerequisite check (required: git, curl, tar, bash / optional: node≥18, gh, atlas, psql/mysql/sqlite3/duckdb, rtk, claude)
- [x] Missing-package install commands per OS (apt, dnf, pacman, zypper, apk, brew, port, nvm)
- [x] GitHub Releases asset URL → download → sha256 verify → tar extract
- [x] Install to `~/.local/bin/genasis` or `/usr/local/bin/genasis` + PATH guidance
- [x] `--no-run`, `--prefix=PATH`, `--version=X.Y.Z`, `--skip-prereqs`, `-h/--help` flags
- [x] Auto-invoke `genasis attach` at end (opt-out available)
- [x] Clean exit codes + explicit error messages on failure
- [x] Local smoke test (Ubuntu/apt) — OS/package detect + graceful fail verified

### .github
- [x] `.github/workflows/ci.yml`
- [x] `.github/workflows/release.yml` (cross-rs for linux-arm64 cross-compile)
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
- [x] `tests/golden/{ecc-only,kw-plugins,blank,legacy-bash-genesis,with-drizzle,with-duckdb}/{input,expected}/.gitkeep` (6 fixtures)
- [x] `tests/golden/SHARED.md` (scenario table + conventions)
- [x] Per-fixture `README.md` ×6
- [x] `tests/e2e/.gitkeep`, `tests/unit/.gitkeep`

### templates skeleton (Tera placeholders)
- [x] `crates/genasis-templates/templates/GENASIS.md.tera`
- [x] `crates/genasis-templates/templates/genasis.toml.tera`
- [x] `crates/genasis-templates/templates/env.agents.tera`
- [x] `crates/genasis-templates/templates/mcp.json.tera`
- [x] `crates/genasis-templates/templates/design-system.md.tera`
- [x] `crates/genasis-templates/templates/agent-overlays/README.md` + `.gitkeep`
- [x] `crates/genasis-templates/templates/commands/README.md` + `.gitkeep`
- [x] `crates/genasis-templates/templates/skills/README.md` + `.gitkeep`
- [x] `crates/genasis-templates/templates/hooks/README.md` + `.gitkeep`

### Verification
- [x] `cargo build --workspace` green (14 crates) — verified after rustup stable install.
- [x] `cargo test --workspace --no-fail-fast` → 120 passed (28 suites).
- [x] `bash install.sh --version=v0.0.0-test --no-run` smoke — Ubuntu/apt detected, packages diagnosed, non-existent release graceful.

### Retrospective
- [x] M0 retro: 1) install.sh per-distro package guide matrix took 70% of core effort — 7 managers × 9 packages. 2) `include_dir!()` embed for Tera means single-binary distribution. 3) No local Rust toolchain — increased CI dependency; first push → immediate CI check needed. 4) Marker fence hash uses 4-byte truncation: collision vs readability trade-off — revisit in M2.

---

## M1 — Core Infra (genasis-core, genasis-cli skeleton)

`genasis-core` operational + CLI skeleton's first working command (`version`).

- [x] `crates/genasis-core/` operational
  - [x] `config.rs` — `genasis.toml` schema + load/save + parent-dir walk-up `discover()` (3 unit tests)
  - [x] `env.rs` — `.env.agents` read/write, comment/blank/quoting round-trip preserved (5 unit + 2 integration tests)
  - [x] `fs.rs` — atomic write (sibling tmp + rename + dir fsync), snapshot, optional read (4 unit tests)
  - [x] `marker.rs` — fence parse/serialise/hash/find/inject/replace/upsert/remove, idempotency guaranteed (10 unit + 4 integration tests)
  - [x] `error.rs` — shared error type (NotImplemented, Io, Toml, Json, Config, Overlay, Provider, Db)
- [x] `crates/genasis-cli/` operational
  - [x] `main.rs` + clap v4 dispatch (12 subcommands wired)
  - [s] `cmd_init.rs` — placeholder (actual provisioning in M3)
  - [s] `cmd_attach.rs` — placeholder (M2)
  - [s] `cmd_detach.rs` — placeholder (M2)
  - [s] `cmd_doctor.rs` — placeholder (M8)
  - [s] `cmd_upgrade.rs` — placeholder (M8)
  - [s] `cmd_design.rs` — placeholder (M7)
  - [s] `cmd_db.rs` — placeholder (M5)
  - [s] `cmd_monitor.rs` — placeholder (M9)
  - [x] `cmd_version.rs` — operational (`--json` option, outputs fence v1.0 / build profile / git_sha)
- [x] `crates/genasis-overlay/role_inference.rs` seeded (10 roles + Custom, slug round-trip guaranteed)
- [x] `crates/genasis-db/guard.rs` strengthened — comment removal, string-literal-aware split, EXPLAIN/ANALYZE/PRAGMA/SHOW/DESC/VALUES allowed, transaction control rejected (10 unit + 5 integration tests)
- [x] Unit tests: marker fence idempotency + env round-trip + role inference round-trip + SQL guard
- [x] Integration tests: `crates/genasis-core/tests/{marker_idempotent,env_round_trip}.rs`, `crates/genasis-overlay/tests/role_inference.rs`, `crates/genasis-db/tests/sql_guard.rs`
- [x] CI green — GitHub Actions CI success from commit b7bffaa.
- [x] ADR-001: Overlay = Marker Fence
- [x] ADR-002: Rust single binary

### Retrospective
- [x] M1 retro: 1) Single-fence invariant in `find()` (BEGIN/END exactly one pair, else error) was correct early enforcement — duplicate fences are immediately rejected. 2) `.env.agents` comment preservation needs `Vec<Line>` enum, not IndexMap — lower-level representation. 3) SQL guard's string-literal-aware split uses hand-rolled lexer, simpler than sqlparser-rs dependency (only regex needed). 4) Workspace dev-dependencies declared per-crate — pulled `tempfile` up to workspace dep with `tempfile.workspace = true`.

---

## M2 — Detector + Overlay Merger

Existing team asset recognition and fence injection engine.

- [x] `crates/genasis-core/src/frontmatter.rs` (YAML head/body splitter + scalar reader, 6 unit tests)
- [x] `crates/genasis-overlay/` operational
  - [x] `detector.rs` — `.claude/agents/*.md` scan, classify, has_existing_fence detection (4 unit tests)
  - [x] `role_inference.rs` — 10 roles + Custom (seeded M1, integrated M2)
  - [x] `merger.rs` — `plan_attach` / `plan_detach` / `apply` (3-phase: plan→apply→report) — Tera-based fence body rendering, snapshot then atomic write (3 unit tests)
  - [x] `validator.rs` — `FenceState` (Absent/Pristine/Outdated/Tampered/RoleMismatch) + `WriteDecision` (5 unit tests)
  - [x] `dry_run.rs` — `summary` (one-line glyph format) + `unified_diff` (via `similar`) + counts (3 unit tests)
- [x] Golden fixture: `tests/golden/ecc-only/input/` (3 agent files — frontend canonical / backend canonical / loop-operator custom). Other 4 fixtures deferred to M6 (meaningful expected/ snapshots require all role templates).
- [x] `crates/genasis-templates/templates/agent-overlays/frontend.patch.md.tera` (first real template)
- [x] `cmd_attach.rs` operational — `--project / --dry-run / --diff / --force / --fence-version` options + summary/diff output + apply (Plane/MM calls added in M3)
- [x] `cmd_detach.rs` operational — `--project / --dry-run / --diff` options
- [x] E2E: `crates/genasis-overlay/tests/golden_ecc_only.rs` — round-trip equality + double-attach idempotency (2 integration tests)

### Retrospective
- [x] M2 retro: 1) Tera embed via `include_dir!()` validated — auto-discovered at build time, no manifest needed. 2) Validator's 5-state `FenceState` is essential: Pristine/Outdated/Tampered/RoleMismatch must be explicitly distinguished for `--force` semantics. 3) `MergePlan` plan/apply separation gives dry-run as a natural byproduct. 4) `similar` crate's `TextDiff::from_lines` is sufficient for unified diff — no git-style hunk headers needed. 5) `AppliedReport` returning backup paths enables future `genasis upgrade --rollback`.

---

## M3 — Plane / Mattermost Providers (direct API)

- [x] `crates/genasis-providers/plane/{mod,upstream,agent_aware,detect,factory}.rs` operational
- [x] `crates/genasis-providers/mattermost/{mod,upstream,agent_aware,detect,factory}.rs` operational
- [x] `github.rs` — `gh` CLI wrapper + branch-protection helper
- [x] `cmd_init.rs` operational — config load → Plane health → MM ping → optional `--probe-only`, project + label provisioning
- [x] `cmd_plane.rs` / `cmd_mm.rs` health/ping debug subcommands
- [x] `tests/flavor_parse.rs` integration test
- [x] ADR-003 (direct API) + ADR-005 (Flavor system)

### Retrospective
- [x] M3 retro: agent-aware is near-identical to upstream — delegation pattern overwhelmingly simple. Flavor detection is one response header line; mock-HTTP integration tests deferred (nightly E2E against real instances is more meaningful).

---

## M4 — Plane User Provisioner (Playwright Node sub-process)

- [x] `crates/genasis-cli/scripts/provision-plane-users.mjs` — stdio JSON protocol + Playwright import + stub responses
- [x] `crates/genasis-providers/src/plane/user_provisioner.rs` — Rust spawn + stdin write + stdout parse + exit code handling
- [x] Explicit error messages on failure (Node missing / Playwright missing / JSON parse failure)

### Retrospective
- [x] M4 retro: stdio JSON envelope as the single process-boundary contract — testable on both sides. Actual UI automation code is incrementally ported from Genesis bash assets (first release stays at stub stage).

---

## M5 — Schema Kernel & DB Adapters

- [x] `crates/genasis-db/kernel.rs` — Driver enum + MigrationTool enum + parse + dispatch
- [x] `crates/genasis-db/adapters/{postgres,mysql,sqlite,duckdb,atlas,drizzle_kit,raw_runner}.rs` operational
- [x] `crates/genasis-db/guard.rs` strengthened (seeded M1, M5 integration)
- [x] `cmd_db.rs` operational — query / schema / migrate / diff / status / doctor subcommands
- [x] ADR-004 (DB channel separation)
- [s] Mock HTTP server integration tests deferred (each driver CLI must be host-installed to be meaningful — verified in nightly CI)

### Retrospective
- [x] M5 retro: Atlas is the declarative default; drizzle-kit auto-delegates when `drizzle.config.ts` exists in user repo; DuckDB falls back to raw_runner. URL redaction added to status output to prevent secret leaks.

---

## M6 — Hooks · Skills · Commands templates

- [x] `templates/agent-overlays/*.patch.md.tera` ×10 (frontend from M2 + 9 added)
- [x] `templates/commands/*.md.tera` ×16 (sprint-*, intake-review, issue-*, design-change, db-*, agent-*, check-inbox, record-progress)
- [x] `templates/skills/<name>/SKILL.md.tera` ×6 (scrum-protocol, plane-ops, mm-ops, design-aware, schema-ops, tdd-enforce)
- [x] `templates/hooks/*.tera` ×6 (session-start, pre-tool-branch-guard, pre-tool-worktree-guard, post-tool-mm-sync, post-tool-trim, user-prompt-submit-mm)
- [x] `templates/mcp.json.tera` (Playwright only — authored M0)
- [x] `templates/env.agents.tera` (authored M0)

### Retrospective
- [x] M6 retro: 9 role overlays are thin variants of frontend — only token/bot env-var names differ, lifecycle contract is identical. 16 slash commands are thin pointers delegating to GENASIS.md as single source.

---

## M7 — Design Hot-Swap

- [x] `crates/genasis-design/extractor.rs` — `snapshot_existing` + `write_design_system`
- [x] `crates/genasis-design/diff.rs` — `ImpactArea` enum + keyword categorisation + `changed_areas`
- [x] `crates/genasis-design/ticket_emitter.rs` — `PlannedIssue` plan
- [x] `crates/genasis-design/change_protocol.rs` — 5-phase `run` orchestrator
- [x] `cmd_design.rs swap` / `status` operational
- [s] Golden design-swap fixture deferred (replaced by real migration data at M11)

### Retrospective
- [x] M7 retro: Extractor delegates to designer agent's `ui-style-extractor` skill — Genasis does not self-implement CSS parsing. 6 impact areas (color-tokens, typography, spacing, layout, components, motion) + Other fallback. Line-level keyword categorisation generates one issue per changed area.

---

## M8 — Doctor / Upgrade / Detach polish

- [x] `cmd_doctor.rs` — required/optional tool checks, Genasis asset existence, config load, env secret presence
- [x] `cmd_upgrade.rs` — fence-version arg + dry-run/diff/force options, Tampered/RoleMismatch guard
- [x] `cmd_detach.rs` — completed in M2 (dry-run/diff options included)

### Retrospective
- [x] M8 retro: doctor mirrors install.sh's check matrix in Rust — protects even when users bypass install.sh. upgrade is a thin wrapper on attach but warrants a separate command (version bump is explicitly visible intent).

---

## M9 — Monitor (Ratatui TUI)

- [x] `crates/genasis-monitor/app.rs` — main loop, alternate-screen / raw-mode, 250ms poll
- [x] `widgets/{sprint,tokens,agents,deploy,network,log_tail}.rs` ×6 operational
- [x] `widgets/deploy.rs` — dev/prod LEDs + REFRESHED badge + deploy action key hints
- [x] `state.rs` — AppState + AgentActivity + DeployState + WidgetFocus
- [x] `cmd_monitor.rs` — delegates to `genasis_monitor::app::run`
- [x] ADR-007 (Monitor TUI first-release inclusion)
- [s] Live data source ingest (rtk gain, Plane API poll, manifest watch) is incremental — first release covers widget skeleton

### Retrospective
- [x] M9 retro: ratatui 0.27's `Frame::area()` + `Layout::default().constraints(...)` simplifies 4-row grid. 250ms poll is the right trade-off (CPU <1%, key response immediate). Data ingest will use hook+agent-emitted JSON lines via file-tail, added incrementally.

---

## M10 — Token Economics wrap-up

- [x] `templates/hooks/session-start.sh.tera` — RTK detection + design-bootstrap flag surface
- [x] `templates/hooks/post-tool-trim.sh.tera` — `${GENASIS_TRIM_THRESHOLD_KB:-32}` threshold
- [x] `genasis.toml [token_economics] trim_threshold_kb = 32` schema (authored M0, wired M10)
- [x] ADR-006 (Token Economics)

### Retrospective
- [x] M10 retro: No self-hosted mcp-proxy in v1 is the right call — maintenance burden vs visible impact unfavorable. RTK + Anthropic prompt cache + trim hook 3-tier achieves 80% of the benefit with lifecycle simplicity.

---

## M11 — Migration & Release

- [x] `cmd_plane`, `cmd_mm` debug subcommands (health/ping)
- [x] `docs/ADR/ADR-001` ~ `ADR-007` — all 7 ADRs written
- [x] `docs/PROVIDERS.md` updated (authored M0, flavor guide aligned with M3)
- [x] `docs/MIGRATION-FROM-GENESIS.md` updated (authored M0, mapping table aligned)
- [s] Full `cmd migrate-from-genesis` implementation deferred — requires real Genesis bash team operational data. First-release is docs-only.
- [s] GitHub Release first cross-compile / demo video / v0.1.0 tag — triggers on first PR merge to release pipeline.

### Retrospective
- [x] M11 retro: All milestone code/docs in place. Remaining: (a) run one real sprint to validate data-ingest hooks, (b) verify cross-compile end-to-end via install.sh, (c) cut v0.1.0 tag.

---

## M12 — Internationalization (install-time language selector + active singularity)

> Per blueprint §19 decisions:
> - **User repo agent context is always single-language** (`--lang en|ko`)
> - **Tera template tree split to `templates/{en,ko}/`** + `genasis lang switch` provided
> - **Runtime i18n: rust-i18n** — new crate `genasis-i18n` (lighter than fluent-rs by ~150KB for ~50 messages)
> - **`install.sh` also branches on `--lang`**: inline `case` block (zero dependencies)
> - **`--lang both` rejected** + cites `docs/impact-of-multilang-prompts.md`
> - **CI 3-tier**: normal PR warn / release-prep strict / auto translation-completion PR
>
> Rationale: `docs/impact-of-multilang-prompts.md` (Claude Code language drift bugs
> #46846/#24941, arXiv 2406.20052 Korean line-level confusion, OSS ecosystem
> single-language consensus, prompt cache prefix conflict).
>
> **Required human approval before start** — approved M12 v5, 2026-05-04.

### M12.0 — Human approval gate
- [x] `blueprint.md §19` + `docs/impact-of-multilang-prompts.md` reviewed + approved (M12 v5 plan, 2026-05-04)
- [x] ADR-008 draft written and merged (install-time language selector + active singularity, commit e8b3793)

### M12.1 — i18n infra (runtime — rust-i18n)
- [x] `crates/genasis-i18n/` new crate (commit 9a12ed6)
  - [x] `Cargo.toml` (deps: `rust-i18n = "3"`, `once_cell`)
  - [x] `src/lib.rs` — `Lang` enum (`En`/`Ko`), `resolve()` (CLI flag / toml / env / $LANG / fallback `en`), `install()` calls `rust_i18n::set_locale`, `LangSource` diagnostic enum
  - [x] `i18n!("locales", fallback = "en")` macro root declaration, `t!` re-export
  - [x] `locales/en.yml` (key definition source — 49 keys, 12 namespaces)
  - [x] `locales/ko.yml` (Korean mirror, 100% parity, `_meta.bcp47` specified)
- [x] `Cargo.toml` workspace member added + dependency registered (`rust-i18n = "3"`, `once_cell = "1"`, internal alias)
- [x] Unit tests: `tests/i18n_lookup.rs` — `Lang::parse` (canonical/case-insensitive/locale modifier/friendly names/unknown reject), `resolve()` 5-tier priority + unknown skip-through, `t!` macro en/ko render + fallback semantics, `Lang::code` round-trip, `LangSource::label`. 16 `#[test]`, serial-mutex for process-global state.

### M12.2 — Rust user-facing messages i18n (`t!()` macro)
- [x] `genasis-cli` prose messages wrapped via `t!()` (commit 17b6b99). Structured debug/JSON dump lines intentionally kept in English — grep/IDE friendly + `cmd_doctor` diagnostic key=value form preserved.
  - [x] `cmd_attach.rs` (refused, wrote_summary)
  - [x] `cmd_detach.rs` (wrote_summary)
  - [x] `cmd_upgrade.rs` (refused, wrote_summary)
  - [x] `cmd_init.rs` (resolving_plane, resolving_mm, ensure_project, ensure_channel, next_step, mm_team_id_missing, probe_only_skip)
  - [x] `cmd_doctor.rs` (section headers, pass/warn/error)
  - [x] `cmd_design.rs` (swap messages)
  - [x] `cmd_version.rs` (label strings)
- [x] `genasis-monitor/` TUI labels i18n (widget titles, footer hints)

### M12.3 — Tera template tree split (`templates/{en,ko}/`) — ✅ commit 1fd1e6d
- [x] Flat `templates/` directory reorganised to `templates/{en,ko}/` parallel subtrees
- [x] All overlay / command / skill / hook / top-level templates duplicated to both subtrees
- [x] `genasis-templates::lib.rs` `include_dir!()` unchanged (auto-embeds new structure)
- [x] `build_tera_lang(lang)` plumbed through merger, fetches from `<lang>/agent-overlays/`
- [x] Parity test: `english_and_korean_have_same_top_level_files`

### M12.4 — `genasis init` / `attach` / `detach` `--lang` + interactive prompt — ✅ commits 39d1032, e2e added
- [x] Global `--lang <en|ko>` clap flag on CLI root (not per-subcommand)
- [x] TTY interactive prompt (bilingual banner) when `--lang` omitted + stdin is a TTY
- [x] Non-TTY fallback: `$LANG` → `en` default
- [x] `--lang both` rejection with bilingual banner citing `docs/impact-of-multilang-prompts.md`
- [x] `--non-interactive` / `--yes` bypass for CI
- [x] `genasis.toml [i18n].active` persisted on first attach
- [x] Integration tests: `tests/install_lang_e2e.rs` (flag_en, flag_ko, both_rejected, non_tty_fallback, lang_status, lang_switch_no_op)

### M12.5 — `genasis lang switch <lang>` new command — ✅ commit 39d1032
- [x] Atomic swap: re-renders all `templates/<new-lang>/` → `.claude/genasis/{skills,commands,hooks}/` + `GENASIS.md`
- [x] Fence-internal-only update policy: user edits outside fences preserved
- [x] `genasis.toml [i18n].active` updated
- [x] `genasis lang status` — reports active locale + reference docs path

### M12.6 — `install.sh` `--lang` branch + interactive prompt (Bash) — ✅ commit 54ed32e
- [x] `--lang en|ko|both` parsing
- [x] `--lang both` bilingual rejection message (no dependency)
- [x] Interactive TTY prompt when `--lang` omitted (mirrors Rust CLI UX)
- [x] Passes resolved `--lang` to post-install `genasis attach` invocation
- [x] `bash -n` syntax-check green
- [s] Full round-trip (en → ko → en + fence hash equality) — `git commit` not wrapped inside lang switch (tests run outside git repo), only partial E2E. Revisit at M12.13 release polish.

### M12.7 — Document dual tree (rename + translate + cross-link)

#### M12.7.a Rename pass — ✅ commit ea1e9d6
- [x] `README.md` / `blueprint.md` / `progress.md` → `*.ko.md` (git mv)
- [x] `docs/{ARCHITECTURE,PROVIDERS,MIGRATION-FROM-GENESIS,TOKEN-ECONOMICS,MONITOR}.md` → `docs/ko/`
- [x] `docs/impact-of-multilang-prompts.md` mirror (`docs/ko/impact-of-multilang-prompts.md`)
- [x] `docs/ADR/ADR-000` ~ `ADR-007` (8 files) → `docs/ko/ADR/`

#### M12.7.b Translate pass — ✅ commits ccc1cac, b268d6f, 7c05d94, 23251ae, ea1e9d6
- [x] `README.md` (English) — 18-section SEO structure + bilingual badge row + Star History
- [x] `blueprint.md` (English) — TL;DR + section index + i18n decision summary (full §0–§19 body at release polish stage)
- [x] `progress.md` (English) — milestone summary + M12 sub-step status table
- [x] `docs/ARCHITECTURE.md` — TL;DR + source tree map + ASCII layer diagram + ADR cross-link
- [x] `docs/PROVIDERS.md` — flavor system + 5-step add recipe + detection priority + sample toml
- [x] `docs/MIGRATION-FROM-GENESIS.md` — mapping table + step-by-step CLI flow
- [x] `docs/TOKEN-ECONOMICS.md` — 3-tier model + 1.0 exclusion rationale
- [x] `docs/MONITOR.md` — 6-widget table + key bindings + i18n flow
- [x] `docs/impact-of-multilang-prompts.md` (M12 pre-stage artifact)
- [x] `docs/ADR/ADR-008-i18n-install-time-selector.md` new English + Korean stub mirror
- [s] `docs/ADR/ADR-001` ~ `ADR-007` English body — Korean canonical is single source. English mirrors written at release polish (each ADR's Korean body is short + code/table-heavy, absorbed by release-prep auto-PR).
- [x] Code blocks / env vars / CLI commands / external URLs left untranslated (lint-i18n verifies via grep)

#### M12.7.c Cross-link pass — ✅ commit ea1e9d6, ccc1cac
- [x] All English source files: top cross-link batch
- [x] All Korean mirrors: top `> English: ...` batch (M12.7.b-completed files: "(English version pending)" caveat removed)
- [x] Root `README.md` top bilingual badge row (shields.io English / 한국어 / Add a language) + cross-link batch
- [x] Root `README.ko.md` top same toggle (current language bolded)

### M12.8 — Golden fixture added + cleanup — ✅ commit ea1e9d6
- [x] `tests/golden/with-ko-locale/{input,expected}/` + README new
- [x] Existing 6 fixtures remain English-only
- [x] `tests/golden/SHARED.md` — `with-ko-locale` scenario row added
- [x] `expected/` snapshot populated — `genasis attach --lang ko --non-interactive --yes`. Korean fence body confirmed (`(Genasis Overlay) Plane / Mattermost 프로토콜`).

### M12.9 — `.github` English-only verification — ✅ commit ea1e9d6
- [x] All workflow YAML + issue/PR templates confirmed English-only
- [x] No i18n overhead in CI config

### M12.10 — CI 3-tier guard + drift script + Translation Completion automation — ✅ commit ea1e9d6, 022ca37
- [x] `scripts/check-i18n-drift.sh` (`--warn`/`--strict`/`--list`/`--check-mirror-not-empty`)
- [x] `scripts/i18n-extract-keys.sh` (fluent key parity)
- [x] `ci.yml` `lint-i18n` job (warn)
- [x] `release.yml` `lint-i18n-strict` job (hard-fail)
- [x] `release.yml` auto-opens `[i18n] Translation completion for vX.Y.Z` PR when drift detected

### M12.11 — `genasis doctor [i18n]` extension — ✅ commit ea1e9d6
- [x] Active locale reported
- [x] Template tree presence verified for active locale
- [x] Drift check (calls `check-i18n-drift.sh --list` inline)
- [x] Key parity check (calls `i18n-extract-keys.sh`)
- [x] Integration test: `lang_status_reports_active_locale`

### M12.12 — Retrospective + DoD
- [x] `lint-i18n` CI passes — `lint-i18n` job, `lint-i18n-strict` both authored, CI success verified at commit `b7bffaa`
- [x] `release-prep` workflow — workflow_dispatch with `v0.1.0` trigger success, drift=0 correct `needs_pr=false` branch
- [x] drift 0 — `scripts/check-i18n-drift.sh --strict` clean (all mirrors synced)
- [x] `genasis doctor [i18n]` section green — `lang_status_reports_active_locale` E2E passes
- [x] E2E auto-test scenarios — `tests/install_lang_e2e.rs` 6 tests (flag_en/flag_ko/both_rejected/non_tty_fallback/lang_status/lang_switch_no_op)
- [x] `install.sh --lang ko` Bash branch + `bash -n` passes
- [x] `with-ko-locale` golden fixture (input + README + SHARED.md row)
- [x] `README.md` / `README.ko.md` 18-section SEO + 3-step toggle applied
- [x] GitHub repo Topics ×18 registered (REST API)
- [x] GitHub Pages routing enabled (REST API, from `b7bffaa` build success)
- [x] M12 retro — commit 158aada body + inline retro items in this progress file

### M12.13 — README SEO + multilingual toggle (blueprint §19.13)

#### M12.13.a `README.md` (English) SEO optimisation + structure rewrite — ✅ commits 2e9cdd8, 7c05d94
- [x] 18-section structure (§19.13.5 compliance)
- [x] H1 + tagline + badge row above the fold
- [x] "Why Genasis" narrative section
- [x] Architecture mermaid flowchart
- [x] Comparison table (Genasis vs ECC vs kw-plugins vs claude-code-templates)
- [x] Documentation index table (en/ko dual links)
- [x] Status section linking to progress tracker
- [x] Contributing section with prerequisite pointer

#### M12.13.b Multilingual toggle 3-step fallback — ✅ commit 2e9cdd8
- [x] Badge row (shields.io) at top — English / 한국어 / Add a language
- [x] Cross-link in first paragraph
- [x] Bottom navigation: same 3-way toggle

#### M12.13.c `README.ko.md` (Korean mirror) — ✅ commit 2e9cdd8
- [x] Same 18-section structure, Korean body
- [x] Badge row mirrors English (current language bolded)
- [x] All internal links point to `*.ko.md` / `docs/ko/` counterparts

#### M12.13.d GitHub repo metadata — ✅ API call
- [x] 18 GitHub Topics registered via REST API
- [s] Social preview image upload — REST API unsupported, Web UI only. `docs/assets/og-image.png` prepared for user to upload via Settings → Social preview. M12.13.h Pages OG meta works first.

#### M12.13.e Open Graph + visual assets — ✅ commits 2e9cdd8, 23251ae, follow-up
- [x] `docs/assets/og-image.png` (English)
- [x] `docs/assets/og-image.ko.png` (Korean)
- [x] Jekyll `_config.yml` OG meta tags
- [x] `<head>` OG tags in Pages layout

#### M12.13.f Auto SEO signals (badges) — ✅ commit 2e9cdd8
- [x] CI badge (shields.io + GitHub Actions)
- [x] Release badge (pre-release aware)
- [x] License badge
- [x] Stars badge
- [x] Coverage badge (Codecov)
- [x] Rust version badge
- [x] Star History chart (dark/light mode)

#### M12.13.g Multilingual contributor guide — ✅ commit ea1e9d6
- [x] `docs/i18n/CONTRIBUTE-LANG.md` — 4-step procedure (README + docs + templates + i18n bundle)
- [x] Referenced from CONTRIBUTING.md and README.md

#### M12.13.h GitHub Pages auto-routing — ✅ commits 2e9cdd8 + Pages enable API
- [x] Pages enabled via REST API
- [x] `Accept-Language` header → `/ko/` `/en/` branch (Jekyll `_config.yml`)
- [x] JSON-LD SoftwareApplication schema
- [x] Jekyll sitemap plugin
- [s] Custom domain — user decision (CNAME + DNS setup needed).

#### M12.13.i Measurement + retro hook
- [s] GitHub Insights baseline — repo just published so baseline = 0. First measurement meaningful after 1 week.
- [s] Google Search Console — Pages domain verify (DNS TXT or HTML meta) needed. User proceeds in their GSC account.
- [s] 3-month retro — operational item, calendar reminder. Retrospective issue template written at v0.1.0 tag.

---

## M-D — Design Catalog Integration (post-M12)

> User-approved 2026-05-04. External design provider (`getdesign` npm) delegation +
> two-mode design-system.md (pristine / external-pointer) + user-override
> accumulation + pristine restore + non-npx `--from <path>` entry point.

### Key design decisions
- **No vendoring**: awesome-design-md content is not re-owned. `npx getdesign add <slug>` delegation. License compliance is getdesign upstream's responsibility.
- **Two modes**: `docs/design-system.md` at `mode = pristine` has body as truth; at `mode = external` has §A pointer (external DESIGN.md) + §B user overrides + §C usage manual only.
- **External DESIGN.md location**: `docs/design-system/DESIGN.md`. Read-only treatment.
- **State file**: `docs/.design-state.toml` (mode/slug/source/template_hash/applied_at/previous_slug/gallery_preview/override_count).
- **Backup**: pristine body backed up to `docs/design-system/pristine.bak` before swap. `restore` moves `docs/design-system/` to `docs/design-system.archive-<ts>/` then restores from backup.
- **Issue flood policy**: `changed_areas.len() ≥ 4` triggers auto EPIC mode (1 EPIC + N children). Child descriptions include EPIC ID (Plane upstream compatible). `--per-area` / `--full-rewrite` explicit flags exposed.
- **Telemetry default OFF**: `genasis design swap` auto-sets `GETDESIGN_DISABLE_TELEMETRY=1`. User can enable via `[design].disable_telemetry = false` or `--telemetry on`. No genasis-side collection server.
- **Gallery abstraction**: `genasis.toml [design]` `add_command` template (`{slug}`, `{out}` substitution) allows replacing getdesign with a self-hosted gallery without code changes.

### M-D1 — Pristine/External mode + swap/restore + skill (done 2026-05-04)
- [x] `genasis-core` `Config` gains `[design] DesignConfig` (`gallery_index_url`, `gallery_url_template`, `add_command`, `disable_telemetry`, `external_dir`)
- [x] `genasis-design` crate restructured:
  - [x] `mode.rs` — `Mode::Pristine | Mode::External`, `.design-state.toml` R/W
  - [x] `swap.rs` — slug mode (npx invoke) + `--from <path>` mode (file copy) unified entry
  - [x] `restore.rs` — external→pristine restore (archive move + pristine.bak → design-system.md)
  - [x] `pointer.rs` — design-system.md pointer body render (§A/§B/§C skeleton) — locale branch (en/ko in-source templates)
  - [x] Existing `extractor.rs` / `change_protocol.rs` / `diff.rs` / `ticket_emitter.rs` preserved; legacy entry point aliased as `run_legacy_swap`
- [x] CLI `cmd_design.rs` extended:
  - [x] `swap <slug>` (backward-compatible with `swap <url> --body` — `--body` legacy path kept)
  - [x] `swap --from <path>`
  - [x] `restore`
  - [x] `status` output includes mode/slug/applied_at/override_count/preview URL
- [x] Templates:
  - [s] `design-system.md.tera` two-variant split deferred — pointer body generated by `pointer.rs::render` in code, Tera split unnecessary. Attach emits placeholder; swap overwrites on external mode entry.
  - [x] `templates/{en,ko}/skills/design-aware/SKILL.md.tera` strengthened: reference order (pristine → external §A → §B), user-request conflict procedure, external DESIGN.md direct-edit prohibition, post-swap guidance
- [x] i18n keys: `design.swap.delegating`, `design.swap.from_local`, `design.swap.pristine_backed_up`, `design.swap.design_md_written`, `design.swap.pointer_written`, `design.swap.state_updated`, `design.swap.post_swap_*`, `design.status.mode_pristine`, `design.status.mode_external`, `design.restore.*` etc. (14 keys × 2 locales)
- [x] E2E (`crates/genasis-design/tests/swap_restore_round_trip.rs`): pristine → swap slug → swap slug 2 → restore round-trip + sha256 verification
- [x] cargo test green (132 → 145 passed)

### M-D2 — EPIC plan + Mattermost + user-override accumulation (done 2026-05-04)
- [x] `ticket_emitter` gains `Plan::FullRewrite { epic, children }` + `PlanMode::{Auto, PerArea, FullRewrite}`, auto threshold `DEFAULT_FULL_REWRITE_THRESHOLD = 4` (majority of 7 areas)
- [x] Child description includes EPIC title — Plane upstream (no native parent_id) board bundle visibility
- [x] Mattermost announce template (CLI emits body; actual post is caller/provider): `🚨 DESIGN CHANGE: <from> → <to> | preview: <url> | issues planned: <n>`
- [x] `genasis design verify` (`crates/genasis-design/src/verify.rs`) — `.design-state.toml.template_hash` vs actual `DESIGN.md` sha256, tamper detection
- [x] `genasis design override add "<text>"` (`crates/genasis-design/src/override_log.rs`):
  - [x] Two sentinel comments recognized (en/ko)
  - [x] `#### override-<id> @ <iso>` block appended, `override_count` incremented
  - [s] §A grep+citation is design-aware SKILL agent responsibility — CLI only records body
- [x] `genasis design override list` / `remove <id>`
- [x] E2E (`crates/genasis-design/tests/epic_plan_and_overrides.rs`): full-rewrite EPIC verification, 3 overrides accumulated then swap resets §B.2 (intentional — user re-reviews against new §A, guided by design-aware SKILL) + tamper detection

### M-D3 — Monitor widget + attach prompt + doctor + ADR (done 2026-05-04)
- [x] `AppState.design: DesignWidgetState` (mode/slug/applied_at/override_count/preview_url/gallery_url)
- [x] `widgets/design.rs` — pristine/external branch render, key `7` focus, `Enter` opens preview URL via `open`/`xdg-open`/`cmd /C start`
- [x] `app.rs` layout: Design panel added (below Deploy row, 7-line slot)
- [x] `cmd_attach.rs` auto-seeds `[design]` defaults into `genasis.toml` on first attach (`gallery_index_url`, `gallery_url_template`, `add_command`, `disable_telemetry=true`, `external_dir`). Preserved if already present (idempotent). Interactive prompt omitted for i18n/non-interactive consistency — user edits `genasis.toml` later for gallery swap.
- [x] `cmd_doctor.rs` `[design]` section:
  - [x] Mode output (pristine / external + slug)
  - [x] `npx` availability — optional when pristine, required-missing warning when external
  - [x] External mode: `run_verify` re-invoked for hash match
  - [x] Mode/disk coherence — pointer/external-dir missing detection
- [x] `docs/ADR/ADR-009-design-catalog-delegation.md` (en) + `docs/ko/ADR/ADR-009-...` (ko) — no-vendor rationale / two-mode justification / gallery URL abstraction / telemetry default off / conflict resolution policy / 3 alternatives reviewed
- [x] Doctor i18n keys (en/ko): `doctor.design.section`, `mode_pristine`, `mode_external`, `npx_missing_optional`, `npx_missing_required`, `verify_ok`, `verify_tampered`, `verify_error`, `pointer_missing`, `extdir_missing`. Monitor key hint update (`[1-7] focus`, `[Enter] open URL`)
- [s] Manual TUI smoke — code-path unit verified + state load fallback. Actual key-input testing at first v0.1.0 cross-compile.
- [x] cargo test + lang drift pass

---

## M14 — Default agentic team bootstrap (green-field install)

> 2026-05-05 user-flagged. The overlay engine currently assumes
> `.claude/agents/*.md` files **already exist** — `attach` injects a
> fence into a user-authored file. When a project has no agent team at
> all, the canonical 10 ECC roles cannot be scaffolded, leaving the
> "non-destructive overlay" promise without a green-field entry point.
> M14 fills that gap — **base agent template** (rendered when role file
> is missing) beneath the existing **patch overlay** (rendered into the
> marker fence), forming a 2-layer structure.

### Key design decisions (ADR-010 candidate)

- **Default OFF**: bootstrap is opt-in (`--bootstrap`). Existing users
  running `attach` against an empty `.claude/agents/` get a warning,
  not silent file creation. ADR-001 non-destructive invariant preserved.
- **Base + patch ownership separation**: Base file (post-emit) is fully
  user-owned (free to edit). Only the marker fence inside it is
  genasis-owned (upgraded by `upgrade`). ADR-001 "fence-outside =
  user zone" promise consistently maintained.
- **No ECC vendor**: Base templates are intentionally thin stubs —
  frontmatter (`name/description/tools/model/color`) + 5–10 line
  header. Not a fork of `claude-code-templates` / ECC role definitions.
  Patch fence fills protocol body later.
- **i18n split tree**: `templates/en/agents/<role>.md.tera` +
  `templates/ko/agents/<role>.md.tera` 2 trees. `lang switch` swaps
  base too (but preserves user edits outside fence — existing
  fence-internal-only policy).
- **Role set**: pm / planner / architect / frontend / backend / qa /
  designer / security / devops / code-reviewer (same 10 as M2's
  `Role::ALL`).

### M14.0 — Decision gate + ADR-010
- [x] `docs/ko/ADR/ADR-010-default-team-bootstrap.md` (Korean SSOT):
  context, alternatives (a~f), decision (b+d, e-rejected), consequences,
  references (ADR-001 marker fence + ADR-008 lang precedence)
- [x] `docs/ADR/ADR-010-default-team-bootstrap.md` English mirror
- [x] `blueprint.ko.md §20` new section (M14, ADR-010 cited) + `blueprint.md`
  section index updated
- [x] `blueprint.ko.md §16` ADR table: ADR-008/009/010 rows added
- [ ] User ratify gate — after ADR-010 merge, M14.3 entry (M14.1/M14.2
  can proceed independently — base templates + module are code-only
  additions, entry point exposure is M14.3)

### M14.1 — Base agent templates (`templates/{en,ko}/agents/<role>.md.tera`) — ✅ pending build verification
- [x] `crates/genasis-templates/templates/en/agents/` directory created
  - [x] `pm.md.tera` (frontmatter + 5–10 line role header)
  - [x] `planner.md.tera`
  - [x] `architect.md.tera`
  - [x] `frontend.md.tera`
  - [x] `backend.md.tera`
  - [x] `qa.md.tera`
  - [x] `designer.md.tera`
  - [x] `security.md.tera`
  - [x] `devops.md.tera`
  - [x] `code-reviewer.md.tera`
  - [x] `README.md` — base vs patch boundary explained, user-edit zone specified
- [x] `crates/genasis-templates/templates/ko/agents/` — same 11 files
  (10 base + README, Korean body + identical frontmatter, `description:` in Korean)
- [x] `genasis-templates::lib.rs` `include_dir!()` auto-embeds new directory
  (directory addition only — no manifest update needed)
- [x] `agent_base_subtrees_have_same_roles` test — verifies both locales
  have all 10 required role tera files
- [s] Frontmatter contract unit test covered by `bootstrap.rs::tests::rendered_base_carries_required_frontmatter_keys`
  (rendered base verified for 5 keys + `name:` matches stem)

### M14.2 — `genasis-overlay::bootstrap` module — ✅ pending build verification
- [x] `crates/genasis-overlay/src/bootstrap.rs` new module
  - [x] `BootstrapOptions { lang, roles, context }` + `Default` + builder
    setters (`new`, `with_roles`, `with_context`)
  - [x] `pub fn plan_bootstrap(project_root, opts) -> Result<BootstrapPlan>`
    — `.claude/agents/<role>.md` absent → `Create { body }`, present → `Skip { reason: "exists" }`
  - [x] `BootstrapPlan` (`creates()` / `skips()` iterators) + `BootstrapAction::{Create, Skip}`
  - [x] `pub fn apply_bootstrap(plan) -> Result<BootstrapReport>`
    — `gfs::atomic_write` for new files (`atomic_write` auto-creates parent dir)
- [x] `lib.rs`: `pub mod bootstrap;` + re-export (`apply_bootstrap`,
  `plan_bootstrap`, `BootstrapAction`, `BootstrapChange`, `BootstrapOptions`,
  `BootstrapPlan`, `BootstrapReport`)
- [x] Unit tests (`crates/genasis-overlay/src/bootstrap.rs::tests`):
  - [x] `empty_project_creates_all_ten_roles` — empty project → 10 `Create`
  - [x] `existing_files_are_skipped` — partial roles present → only missing get `Create`, present get `Skip("exists")` + role enum verified
  - [x] `apply_writes_only_create_actions` — `apply_bootstrap` → 10 files exist on disk
  - [x] `rendered_base_carries_required_frontmatter_keys` — frontmatter contract (5 keys + `name: <slug>` match)
  - [x] `korean_locale_subtree_loads` — `--lang ko` works identically
  - [x] `unknown_locale_errors` — unknown locale returns `Error::Overlay`
  - [x] `role_subset_only_plans_chosen_roles` — `with_roles(vec![...])` for partial scaffold
  - [x] `idempotent_second_apply_is_a_noop` — second bootstrap call → all Skip
- [x] Integration tests (`crates/genasis-overlay/tests/bootstrap_then_attach.rs`):
  - [x] `bootstrap_then_attach_injects_into_every_role` — bootstrap → scan → 10 all `Known(_)` → plan_attach → 10 `Inject`
  - [x] `bootstrap_ko_then_attach_ko_injects_korean_overlay` — `--lang ko` chain verified, backend.md attach output contains Korean protocol header "Plane / Mattermost 프로토콜"
  - [x] `bootstrap_partial_then_attach_handles_mix` — user-authored frontend.md preserved byte-identical by bootstrap

### M14.3 — CLI wire-up
- [ ] `crates/genasis-cli/src/cmd_init.rs` gets `--bootstrap` flag:
  - [ ] flag set → `plan_bootstrap` → `apply_bootstrap` → auto-chain `cmd_attach` (or guide user to next step)
  - [ ] flag unset + empty `.claude/agents/` → existing behaviour, but stderr hint: "no agents detected — run `genasis init --bootstrap` to scaffold the default team" (i18n key)
- [ ] Or (alternative) `cmd_attach.rs` gets `--bootstrap` — ADR-010 decides
- [ ] `genasis-i18n/locales/{en,ko}.yml` keys added:
  - `bootstrap.no_agents_hint`
  - `bootstrap.scaffolded_summary` (`{count} default agents created`)
  - `bootstrap.skipped_existing` (`{name} already exists, skipped`)
  - `bootstrap.next_step` (next `attach` invocation guidance)
- [ ] `--lang` priority applies identically to base + patch (base tree also from `templates/<lang>/agents/`)

### M14.4 — `tests/golden/blank/` activation
- [ ] `tests/golden/blank/input/` — empty mock project (README.md only, no `.claude/`)
- [ ] `tests/golden/blank/expected/` — output of `genasis init --bootstrap --lang en` (10 base agent files + GENASIS.md + .claude/genasis/* etc.)
- [ ] `crates/genasis-overlay/tests/golden_blank.rs` new — round-trip (bootstrap → attach → detach → compare)
- [ ] `tests/golden/SHARED.md` table: blank scenario row updated (M2 stub → M14 active)
- [ ] (optional) `tests/golden/blank-ko/` — `--lang ko` variant

### M14.5 — Doctor + retrospective
- [ ] `cmd_doctor.rs` `[bootstrap]` section:
  - [ ] `.claude/agents/` existence + file count report
  - [ ] Empty dir + bootstrap not run → suggestion output (i18n)
  - [ ] Base files' frontmatter `name:` key all canonical role verification
- [ ] `progress.ko.md` retrospective slot added (M14 start/end/learnings)
- [ ] DoD: `cargo test --workspace` green, `lint-i18n` green, golden blank round-trip green

### Risks / TBD
- **(a)** `init --bootstrap` vs `attach --bootstrap` placement: ADR-010
  decides — `init` carries Plane/MM provisioning weight. Separate
  `genasis bootstrap` subcommand entry point considered.
- **(b)** ECC `claude-code-templates` differentiation text: README.md
  Comparison table needs "Non-destructive overlay" vs "Bootstrap" as
  two axes to avoid visual confusion.
- **(c)** How far base templates specify `tools:` — too narrow restricts
  user freedom, too broad is meaningless. Starting from ECC default
  (`Bash, Read, Write, Edit, Glob, Grep, Task`) + comment guidance.

---

## Phase F — Debug History Feedback Loop (ADR-012)

> 2026-05-05 user-designed. Genasis as a meta-tool generates overlay files that
> users inevitably modify. Those modifications are the highest-signal feedback
> for improving genasis. This phase implements a secure, always-on drift
> detection + opt-in submission pipeline that feeds field patches back into
> genasis development via automated Claude Code analysis.
>
> Governance: contributors submit data only (`debug-history/patches/*.patch.json`);
> maintainer processes patches via Claude Code `/debug-review` skill for auto-development.
> See ADR-012 §8.

### M15 — Manifest + Drift Detection + Local Debug Commands

- [ ] `genasis-core` manifest module:
  - [ ] `.manifest.json` schema (genasis_version, agents_catalog_version, attached_at, lang, files map with sha256/template_source/fence_sha256)
  - [ ] `manifest::generate(project_root)` — scan `.claude/genasis/` + marker fences, produce manifest
  - [ ] `manifest::compare(manifest, live_state)` → `Vec<DriftEntry>`
  - [ ] `DriftEntry { file, drift_type, old_hash, new_hash, diff_lines }`
- [ ] Manifest generation wired into `cmd_attach.rs` and `cmd_init.rs` (post-apply)
- [ ] Passive drift detection on every CLI invocation:
  - [ ] `app_preamble()` or equivalent hook runs manifest compare
  - [ ] Appends to `.claude/genasis/.drift-log/current.jsonl`
  - [ ] < 1ms overhead target (SHA-256 per managed file only)
- [ ] `genasis debug` subcommand tree:
  - [ ] `genasis debug status` — drift summary (file count, last collect timestamp)
  - [ ] `genasis debug log` — display `.drift-log/current.jsonl` contents
  - [ ] `genasis debug collect` — anonymise + generate `patch.json`:
    - [ ] Secret stripping (TOKEN/SECRET/KEY/PASSWORD/CREDENTIAL regex)
    - [ ] Path anonymisation (absolute paths → `<PROJECT_ROOT>/...`)
    - [ ] Project identity as one-way hash
    - [ ] Output to `~/.genasis/debug-history/<project-hash>/<timestamp>.patch.json`
  - [ ] `genasis debug reset` — update manifest to current state, clear drift log
- [ ] i18n keys: `debug.status.*`, `debug.collect.*`, `debug.log.*`, `debug.reset.*` (en/ko)
- [ ] Unit tests: manifest generate/compare, drift detection, secret stripping, path anonymisation
- [ ] Integration test: attach → manual file edit → drift detected → collect → patch.json valid

### M16 — Submit + Repo Structure + `/debug-review` Skill

- [ ] `genasis debug submit` command:
  - [ ] `--all | --latest | --file <path>` selection
  - [ ] Full payload preview before confirmation
  - [ ] Interactive confirm prompt (i18n)
  - [ ] Optional `user_comment` field
  - [ ] Submission via `gh issue create` (label: `debug-history`, structured JSON body)
  - [ ] Rate limiting: max 1 submit per project per day
- [ ] `debug-history/` directory structure in genasis repo:
  - [ ] `debug-history/index.jsonl` (patch registry: id, submitted_at, project_hash, status)
  - [ ] `debug-history/patches/` (submitted patch.json files)
  - [ ] `debug-history/analysis/` (auto-generated: clusters.md, proposed-fixes.md)
  - [ ] `debug-history/schema.json` (JSON Schema for patch.json validation)
- [ ] `.github/workflows/debug-history-pr.yml`:
  - [ ] Only allow changes to `debug-history/patches/*.patch.json`
  - [ ] JSON schema validation
  - [ ] Executable content rejection (shebang, suspicious patterns)
  - [ ] Auto-label `[debug-history]` + auto-assign maintainer
- [ ] `.claude/skills/debug-review.md` skill:
  - [ ] Read all unresolved patches from `debug-history/patches/`
  - [ ] Cluster by affected template/file
  - [ ] Identify recurring patterns (threshold: ≥2 patches)
  - [ ] Propose template changes as Edits
  - [ ] Update `debug-history/analysis/clusters.md`
  - [ ] Tag resolved patches in `index.jsonl`
- [ ] i18n keys: `debug.submit.*`, `debug.submit.confirm`, `debug.submit.rate_limited` (en/ko)

### M17 — Analysis Automation + Integration

- [ ] `/debug-review` skill triggers:
  - [ ] Manual: maintainer invokes `/debug-review`
  - [ ] Scheduled: weekly auto-run (GitHub Actions + Claude Code)
- [ ] `debug-history/analysis/clusters.md` auto-generation:
  - [ ] Group patches by template source
  - [ ] Classify: bug_fix / workflow_extension / project_specific
  - [ ] Frequency count + example excerpts
- [ ] `debug-history/analysis/proposed-fixes.md` auto-generation:
  - [ ] For each cluster with ≥2 occurrences: draft template Edit
  - [ ] Link to source patch IDs
  - [ ] Confidence score (based on pattern consistency)
- [ ] Audit trail:
  - [ ] Every merged template fix references motivating patch IDs in commit message
  - [ ] Resolved patches tagged in `index.jsonl` with fix commit SHA
- [ ] Archival policy:
  - [ ] Patches older than 6 months → `debug-history/archive/YYYY-MM/`
  - [ ] Archived patches excluded from active analysis
- [ ] Documentation:
  - [ ] `CONTRIBUTING.md` section on debug-history submissions
  - [ ] `genasis debug --help` comprehensive usage guide
  - [ ] GENASIS.md template updated with debug history section

---

## In-progress notes

(This section records blocks, decision changes, and deferred items inline.)

- 2026-05-03: Initial blueprint agreement complete, M0 start.
- 2026-05-03: M0 complete — 144 files, 9 crate stubs, install.sh smoke verified, 6 golden fixture dirs, 5 Tera templates, 3 GitHub Actions workflows. Ready for M1.
- 2026-05-03: M1 complete — genasis-core 5 modules operational (marker/fs/env/config/error), `cmd_version` first working command, role_inference + SQL guard strengthened, 30+ unit/integration tests, ADR-001/002 written. M2 ready.
- 2026-05-03: M2 complete — frontmatter parser + detector + validator + merger + dry_run + cmd_attach/detach operational, first real template (frontend), ecc-only golden fixture + 2 round-trip integration tests, cumulative 78 `#[test]`. Next: M3.
- 2026-05-03: M3-M11 completed — Plane/MM provider flavor system + GitHub gh wrapper + cmd_init operational, Plane user provisioner Node sub-process, DB schema kernel + 7 adapters + cmd_db, 10+16+6+6 Tera templates (agent overlays / commands / skills / hooks), design hot-swap orchestrator + cmd_design, doctor/upgrade/monitor operational, ADR-003~007 written. First-release code/docs in place.
- 2026-05-04: M12 v1 plan (doc dual tree + CI only, 8 sub-steps). User feedback expanded to v2.
- 2026-05-04: **M12 v2 replan complete** — user request added (a) runtime i18n (Rust CLI/TUI + install.sh), (b) `--lang en|ko` install-time selection, (c) multilingual co-install investigation (`docs/impact-of-multilang-prompts.md` — 13 sources analysed) → **`--lang both` rejection + active singularity** policy. blueprint §19 full rewrite (13 sub-sections), progress M12 expanded to 13 sub-steps. New crate `genasis-i18n`, `templates/{en,ko}/` split, `genasis lang switch` command, `install.sh --lang` branch, `with-ko-locale` golden. **Awaiting human approval**.
- 2026-05-04: **M12 v3 fine-tuning** — 2 user feedback items. (1) Drift gate from 2-tier to **3-tier (PR warn / release-prep strict / auto translation-completion PR)**. (2) Runtime i18n library **fluent-rs → rust-i18n** — ~50 messages / no Korean plural variation / 150KB binary saving / token efficiency.
- 2026-05-04: **M12 v4 — interactive language prompt added, final user approval**. `--lang` arg priority (arg > TTY prompt > `$LANG` fallback). Install shows bilingual banner. `--non-interactive`/`--yes` for CI.
- 2026-05-04: **M12 v5 — README SEO + multilingual toggle added, final approval**. blueprint §19.13 8 sub-sections. **Approved — sequential execution from M12.0**.
- 2026-05-04: **Phase D (Design Catalog Integration) complete** — M-D1/M-D2/M-D3 in one pass. 7 user decisions reflected. No vendoring → `npx getdesign` delegation. Telemetry default OFF. New code: `genasis-design/{mode,pointer,swap,restore,verify,override_log,ticket_emitter}.rs` + `genasis-monitor/widgets/design.rs` + `cmd_design` 5 subcommands. ADR-009 (en+ko). i18n 126 → 144 keys. Cumulative cargo test 145 passed.
- 2026-05-05: **M14 (Default agentic team bootstrap) user-flagged + plan reflected**. User inquiry — can a blank project scaffold a default team? Code audit: `genasis-overlay` only supports attach/detach, no base-agent creation path. Unintentional gap from milestone ordering; blueprint §15 implicitly assumed ECC reference user. M14 established: base + patch 2-layer + ADR-010 + green-field golden activation. v0.1.0 release tag moved post M14.0.
- 2026-05-04: **M12 v6 audit + stale item cleanup**. 154 unchecked items in progress.ko.md audited — nearly all M12.3~M12.13 work was committed but checkboxes not closed. True missing artifacts filled (`logo.svg`, `architecture.svg`, `tests/install_lang_e2e.rs` 6 tests, `cmd_attach.rs --lang` clap conflict resolved). All M12.3~M12.12 sub-steps closed `[x]` or `[s]`. Cumulative cargo test 120 passed.
- TODO: GitHub `<OWNER>` decision needed (install.sh placeholder)
- TODO: Monitor manifest hash comparison — Next.js-only vs Vite/Turbo/plain compatibility check
- TODO: Atlas DuckDB support status recheck (raw runner necessity confirmation)

---

## Future items (post first-release)

- genasis-mcp-proxy
- Multi-project monorepo support
- Web UI dashboard
- Community mcp-cache integration
- VSCode extension
- Plane Pro / GitLab / Linear and other issue-tracker flavors

---

## Retrospective slots

| Milestone | Start | End | Learnings |
|---|---|---|---|
| M0 | 2026-05-03 | 2026-05-03 | install.sh OS matrix was 70% of core effort; Tera `include_dir!()` embed simplifies single-binary distribution; Rust toolchain not installed locally → all crate import consistency reviewed manually. |
| M1 | 2026-05-03 | 2026-05-03 | Single-fence invariant + body hash is the core of overlay safety; `.env.agents` comment preservation requires `Vec<Line>` model; SQL guard uses string-literal-aware split avoiding sqlparser-rs dependency. |
| M2 | 2026-05-03 | 2026-05-03 | `MergePlan` plan/apply separation makes dry-run natural; `FenceState` 5-state clarifies `--force` semantics; `include_dir!()` Tera embed auto-discovers templates; only ecc-only fixture meaningful at M2. |
| M3 | 2026-05-03 | 2026-05-03 | agent-aware is thin delegation over upstream; flavor detection is one header line. |
| M4 | 2026-05-03 | 2026-05-03 | stdio JSON envelope is the single Rust↔Node contract. UI automation stays at stub stage. |
| M5 | 2026-05-03 | 2026-05-03 | Atlas default + Drizzle Kit auto-detect + DuckDB raw_runner fallback; URL redaction prevents secret leaks. |
| M6 | 2026-05-03 | 2026-05-03 | 9 role overlays are thin frontend variants; 16 slash commands are thin pointers to GENASIS.md. |
| M7 | 2026-05-03 | 2026-05-03 | Extractor delegation + 6-area keyword categorisation simplifies issue plan. |
| M8 | 2026-05-03 | 2026-05-03 | doctor mirrors install.sh check matrix — protects even when user bypasses. |
| M9 | 2026-05-03 | 2026-05-03 | ratatui 0.27 `Frame::area()` API simplifies 4-row grid; 250ms poll is right trade-off. |
| M10 | 2026-05-03 | 2026-05-03 | No mcp-proxy in v1 is the right maintenance-vs-impact trade-off. |
| M11 | 2026-05-03 | 2026-05-03 | All first-release code/docs in place. Next: one real sprint for data-ingest hook validation + v0.1.0 tag. |

---

## Phase E — Dynamic Agents Catalog (ADR-011, 2026-05-05)

Architectural shift: remove `include_dir!()` compile-time template embed,
replace with runtime fetch from GitHub Releases (`agents-v1.x.tar.gz`).
Community best-of-breed agents curated via private `agents-pool` submodule.

| Sub-milestone | Scope | Status |
|---|---|---|
| E.0 | ADR-011 written (KO + EN) | done |
| E.1 | `agents/` catalog directory (9 base + 20 overlays + 16 commands + 6 hooks + manifest) | done |
| E.2 | `genasis-templates` crate → fetch+cache+load (include_dir removed) | done |
| E.3 | `genasis-overlay` merger/bootstrap wired to AgentStore | done |
| E.4 | CLI `genasis agents {fetch,status,update,list}` subcommand | done |
| E.5 | `.github/workflows/release-agents.yml` (tag → tarball + sha256) | done |
| E.6 | `agents-pool/` skeleton (config.toml + crawl/verify/publish scripts) | done |
| E.7 | `agents-pool` crawl → verify → publish pipeline live test | done |
| E.7.1 | Separation enforced: agents/base/ removed from public repo, .gitignore blocks it | done |
| E.7.2 | publish.sh → tarball build + gh release upload (not copy to genasis) | done |
| E.7.3 | release-agents.yml → verify-only (tarball built by agents-pool, not CI) | done |
| E.7.4 | agents-pool/CLAUDE.md — curation strategy + privacy rules | done |
| E.8 | Agent marketplace model: individual install + index.json + `/install-agent` command | done |
| E.8.1 | `agents/index.json` — 23 agents, 6 categories, 3 presets (metadata only) | done |
| E.8.2 | CLI `genasis agents install <name>` — individual fetch from release assets | done |
| E.8.3 | CLI `genasis agents browse` — interactive TUI placeholder | done |
| E.8.4 | CLI `genasis agents list --category/--search` — filtered browsing | done |
| E.8.5 | CLI `genasis agents installed` / `remove` — project management | done |
| E.8.6 | `/install-agent` Claude Code slash command template | done |
| E.8.7 | `publish.sh` uploads individual .md as release assets | done |
| E.9 | `agents-pool` pushed to private repo + genasis submodule registration | pending |
| E.10 | Wire `cmd_attach`/`cmd_upgrade` to load AgentStore before plan | pending |
| E.11 | `install.sh` update | pending |
| E.12 | Remove old `crates/genasis-templates/templates/` | pending |
| E.13 | Interactive TUI (dialoguer) for `genasis agents browse` | pending |
| E.14 | First `agents-v1.0.0` release (validates full pipeline) | pending |

### Default 9-role team (famous-agents.md 기반 best-of-breed)

| Role | Source | Category |
|---|---|---|
| pm | genasis + dl-ezo reference | core |
| architect | ECC | core |
| frontend-developer | wshobson | core |
| backend-developer | wshobson | core |
| code-reviewer | ECC (gold standard) | core |
| qa-tester | VoltAgent | core |
| security-reviewer | ECC | core |
| planner | dl-ezo | core |
| designer | genasis | core |

## Releases

No release tagged yet. First release (`v0.1.0`) cut after Phase E.10
(first successful agents catalog release validates the full pipeline).
Translation completion gate in `release.yml` already passes.
