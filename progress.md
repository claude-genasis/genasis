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
- [x] User ratify gate — ratified 2026-05-08 (entry point: new `genasis bootstrap` subcommand + `genasis init --bootstrap` alias, auto-chains to `cmd_attach` unless `--no-attach-after`; ADR-010 §3 decision (b)+(d))

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

### M14.3 — CLI wire-up — ✅ commit pending (this commit)
- [x] `crates/genasis-cli/src/cmd_bootstrap.rs` new — `genasis bootstrap [--lang] [--roles] [--no-attach-after] [--dry-run] [--project]`. Loads agents catalog, calls `plan_bootstrap` → `apply_bootstrap`, auto-chains `cmd_attach::pub_run` unless `--no-attach-after`.
- [x] `crates/genasis-cli/src/cmd_init.rs` gets `--bootstrap` alias + `--roles` forwarder. Delegates straight to `cmd_bootstrap::run` so the two entry points stay byte-identical.
- [x] `crates/genasis-cli/src/cmd_attach.rs` empty-dir hint: when `report.agents` and `report.skipped` are both empty, stderr emits `bootstrap.no_agents_hint` and continues (existing non-destructive behaviour preserved).
- [s] `cmd_attach --bootstrap` alternative path — rejected per ADR-010 §3 (b)+(d). Single canonical entry point + `init --bootstrap` alias keeps the surface coherent.
- [x] `genasis-i18n/locales/{en,ko}.yml` keys added: `bootstrap.no_agents_hint`, `bootstrap.scaffolded_summary` (`%{count}`), `bootstrap.skipped_existing` (`%{name}`), `bootstrap.next_step`.
- [x] `--lang` priority — `cmd_bootstrap::run` calls `lang_prompt::decide` identically to `cmd_attach::pub_run`, so the global `--lang` flag picks the same locale for both base and patch trees. Unit tests `parse_roles_*` cover role-subset path.

### M14.4 — `tests/golden/blank/` activation — ✅ commit pending (this commit)
- [x] `tests/golden/blank/input/` — empty mock project (README.md only, no `.claude/`)
- [x] `tests/golden/blank/expected/` — bootstrap+attach output (10 fenced agent files + README.md), populated via `BLESS=1 cargo test`
- [x] `crates/genasis-overlay/tests/golden_blank.rs` — two tests: bootstrap+attach+detach round-trip + expected/ snapshot equality (BLESS=1 to refresh)
- [x] `tests/golden/SHARED.md` table: blank row flipped to **Active** with test path + BLESS hint
- [s] `tests/golden/blank-ko/` — deferred to M18 audit. Per the user's M18 directive ("intent re-check first") we will decide on language variants alongside the rest of the fixture roster instead of adding one ad-hoc here.

### M14.5 — Doctor + retrospective — ✅ commit pending (this commit)
- [x] `cmd_doctor.rs` `[bootstrap]` section:
  - [x] `.claude/agents/` existence + file count report (`doctor.bootstrap.dir_missing` / `doctor.bootstrap.file_count`)
  - [x] Empty dir + bootstrap not run → `doctor.bootstrap.empty_hint` suggestion (i18n)
  - [x] Base files' frontmatter `name:` matches stem; missing/mismatched fields surfaced as warnings
- [x] `progress.md` / `progress.ko.md` retrospective slot — M14 row added below.
- [x] DoD: `cargo test --workspace` green (177 → 179 passed including golden_blank), bootstrap-related drift items cleared. Bigger doctor coverage will be added under M19.

### Risks / TBD
- **(a)** `init --bootstrap` vs `attach --bootstrap` placement: **resolved 2026-05-08** — new `genasis bootstrap` subcommand + `genasis init --bootstrap` alias (ADR-010 §3 (b)+(d)).
- **(b)** ECC `claude-code-templates` differentiation text: README.md
  Comparison table needs "Non-destructive overlay" vs "Bootstrap" as
  two axes to avoid visual confusion.
- **(c)** How far base templates specify `tools:` — too narrow restricts
  user freedom, too broad is meaningless. Starting from ECC default
  (`Bash, Read, Write, Edit, Glob, Grep, Task`) + comment guidance.

---

## v0.1.0 plan (2026-05-08 ratified)

> v0.1.0 cut condition (user decision 2026-05-08): every command advertised
> in `README.md` is exercised by an automated E2E test, and every golden
> fixture in `tests/golden/` either has a populated `expected/` snapshot or
> has been intentionally retired. The roadmap below sequences the remaining
> milestones into commit-sized batches with immediate review after each.

| Order | Milestone | Scope | Status |
|---|---|---|---|
| 1 | M14.0 | ADR-010 ratify gate | done |
| 2 | M14.3 | `cmd_bootstrap.rs` + `init --bootstrap` alias + `attach` empty-dir hint + 4 i18n keys | done |
| 3 | M14.4 | `tests/golden/blank/` activation (input + expected + round-trip) | done |
| 4 | M14.5 | `cmd_doctor.rs [bootstrap]` section + retro entry + DoD | done |
| 5 | M18 | Golden fixture audit — keep / retire / add list, then populate `expected/` for survivors | done |
| 6 | M19 | `tests/e2e/` Rust integration suite covering all 13 README commands (trial flavor as default backend) | in progress (M19.1/.2/.3 done; M19.4 awaits M15/M16) |
| 7 | M20 | `nightly-e2e.yml` workflow restored — `servers/docker-compose.yml` smoke against real Plane + MM | done |
| 8 | M21 | trial-app Playwright suite — full US-001..US-022 acceptance regression | pending |
| 9 | M15 | Manifest + drift detection + `genasis debug {status,log,collect,reset}` | done |
| 10 | M16 | `genasis debug submit` (PR-only per ADR-012 §8) + `debug-history/` repo structure + workflow + skill | done |
| 11 | M17 | Analysis automation + integration | done |
| 12 | v0.1.0 cut | tag + release.yml run + announcement | ready (release notes drafted; tag is a maintainer action) |

### M18 — Golden fixture audit — ✅ commit pending (this commit)

2026-05-08 audit decision: golden fixtures should pin **deterministic
disk-state output** only; any scenario expressible as a unit test
against pure data belongs in the relevant crate. Applied to the seven
existing directories:

| Directory | Decision | Rationale |
|---|---|---|
| `ecc-only/` | **Keep** | Round-trip + idempotent attach anchor (`golden_ecc_only.rs`). Already populated. |
| `blank/` | **Keep** | M14 bootstrap entry point (`golden_blank.rs`). Populated under M14.4. |
| `with-ko-locale/` | **Keep** | Korean overlay body anchor — language-specific disk state worth pinning. |
| `kw-plugins/` | **Retire** | Detector only reads frontmatter `name:`; no code-level distinction from ECC. |
| `legacy-bash-genesis/` | **Retire** | `cmd migrate-from-genesis` is docs-only for v0.1.0 (M11 [s]). No code path to exercise. |
| `with-drizzle/` | **Retire** | Single `detected()` call → covered by new unit tests in `crates/genasis-db/src/adapters/drizzle_kit.rs::tests`. |
| `with-duckdb/` | **Retire** | Single `Driver::parse("duckdb")` → already covered by unit tests in `crates/genasis-db/src/kernel.rs::tests`. |

Optional additions considered (`with-trial/`, `bootstrap-then-attach-{en,ko}/`)
were rejected — they are cheaper to cover via the M19 Rust integration
suite.

Deliverables in this commit:
- `tests/golden/{kw-plugins,legacy-bash-genesis,with-drizzle,with-duckdb}/` removed (`git rm -r`).
- `crates/genasis-db/src/adapters/drizzle_kit.rs` gains 3 unit tests
  (`detected_true_when_ts_config_present`, `_when_js_config_present`,
  `_false_when_no_config`) so the retired `with-drizzle/` scenario is
  not lost.
- `tests/golden/SHARED.md` rewritten with the surviving 3 fixtures +
  retired list + "unit test first, golden second" guidance.
- `cargo test --workspace`: 183 → 186 passed.

### M19 — `tests/e2e/` Rust integration suite (README parity)

Cover every command listed in `README.md §CLI Reference`:
`init`, `init --trial`, `attach`, `detach`, `doctor`, `upgrade`,
`bootstrap`, `agents {browse,install,list,installed,remove}`, `monitor`
(headless smoke), `design swap`, `db {query,migrate}`, `lang switch`,
`debug {status,collect,submit}` (last needs M15+M16 done — gated),
`example`. Default backend is the `trial` flavor against a process-local
`trial-app` instance so suite runs hermetically in CI.

### M20 — `nightly-e2e.yml` workflow restoration

Recreate the workflow declared in M0 (currently missing). On a nightly
schedule: `docker compose up -d` against `servers/docker-compose.yml`,
run M19 suite with `flavor = "plane"` / `flavor = "mattermost"` instead
of `trial`, tear down. Tags: `nightly-real-servers`. Failures open a
labelled issue automatically.

### M21 — trial-app Playwright suite

Per user decision 2026-05-08: rich Playwright coverage matching every
acceptance criterion in `trial-app/ralph/prd.json` US-001..US-022.
- `trial-app/e2e/` directory with `playwright.config.ts`
- One spec file per user story (`us-001.spec.ts` ... `us-022.spec.ts`)
- Wired into `trial-app` `package.json` as `npm run e2e`
- Hooked into M19 (`genasis init --trial` E2E covers Quick Path) +
  separately runnable for trial-app development cycles.

### v0.1.0 cut criteria (DoD)

- [x] `cargo test --workspace --no-fail-fast` green — 222 passed, 2 ignored
- [x] `npm --prefix trial-app run e2e` green (M21 suite) — 14 passed, 1 skipped
- [x] `tests/e2e/` Rust suite green in CI (M19) — 23 tests across lifecycle/agents/supporting/debug
- [ ] Nightly real-server suite green for at least one run (M20) — workflow landed; first scheduled run pending
- [x] All `tests/golden/*/expected/` either populated or directory removed (M18) — survivors: ecc-only, blank, with-ko-locale
- [ ] `lint-i18n-strict` green (release.yml hard fail) — pre-existing drift in 5 docs (CREDITS / DESIGN-SWAP-GUIDE / AGENTS-MARKETPLACE / QUICKSTART / famous-agents) needs Korean mirror landed before tag
- [x] `cargo clippy --workspace --all-targets` clean (no errors) — note: not run with `-D warnings` because of upstream warning surface; all warnings are dead-code style only
- [x] `cargo fmt --all -- --check` clean
- [x] `docs/RELEASE-NOTES-v0.1.0.md` drafted
- [ ] Tag `v0.1.0`, release.yml run, GitHub Release notes published — **maintainer action**

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
- 2026-05-10: **v0.5.1 patch release — monitor text selection restored + tmux Shift+drag hint**. Reported during v0.5.0 dogfooding: drag-select / double-click / triple-click did nothing inside `genasis monitor`. Root cause was `EnableMouseCapture` in [`crates/genasis-monitor/src/app.rs`](crates/genasis-monitor/src/app.rs) without any widget consuming mouse events — terminal stopped routing clicks to its native selection layer. Removed `EnableMouseCapture` + `DisableMouseCapture` (one-line fix, comment noting future widgets must opt-in rather than enable globally). TUI wizard already had capture off; added a dim `Shift+drag select text (in tmux)` hint to [`key_hints.rs`](crates/genasis-tui/src/wizard/widgets/key_hints.rs) so tmux mouse-mode users discover the standard workaround. Three-row troubleshooting table added to [`docs/MONITOR.md`](docs/MONITOR.md) and KO mirror covering the v0.5.0 issue, tmux Shift+drag, and `screen` copy-mode. Workspace version 0.5.0 → 0.5.1; release notes EN+KO. `cargo test --workspace` 245 passed, 4 ignored — no regressions.

- 2026-05-10: **Human roster provisioning — humans as first-class team members (ADR-014)**. Until now `genasis init` / `bootstrap` only auto-provisioned the ten agent bot accounts; humans had to sign up separately, breaking both the "turnkey bootstrap" and "human/agent symmetry" missions. Added `genasis-core::config::HumanEntry` + a `[[humans]]` array in `genasis.toml`, with provisioning side-effects (Mattermost user_id, Plane user_id, temporary password) carved out into `.genasis/humans.lock.toml`. New trait method `MattermostProvider::ensure_human_user(spec, team_id)` with an upstream admin-create implementation (24-char high-entropy temp password covering Mattermost's strictest policy, force-change on first login, idempotent on email). Extended `provision-plane-users.mjs` `ProvisionInput` with `humans: HumanRequest[]` (Playwright UI is still a stub but echoes the humans payload). New CLI `genasis humans add | edit | remove | list | sync`; `cmd_init` now auto-runs `humans sync` when `[[humans]]` is non-empty (failures warn but do not fail init). TUI wizard grew from 6 to 7 steps (Env→Lang→Team→Connect→**Humans**→Overlay→Done) with `a / e / d / s / Enter` for add/edit/delete/sync/advance and a 5-field form modal; re-running the wizard reloads `[[humans]]` for in-place editing ("rerun is the editor"). `agents/GENASIS.md.tera` gains `## Human Roster` table and `### Requirement intake protocol` (registered = binding stakeholder, unregistered = `QUESTION` label + PM verification, bots = existing agent-to-agent flow); `pm.patch.md.tera` / `planner.patch.md.tera` (en/ko) and `commands/check-inbox.md.tera` mirror the protocol. ADR-014 written in EN/KO. New unit tests: `HumansLock` round-trip, upsert case-insensitive match, `derive_mm_username` normalisation, cmd_humans `truncate` / `now_iso`. `cargo test --workspace --lib` green. Out of scope (deferred to v2): invite-email mode for SMTP-enabled environments, Plane Playwright UI port to land real user_ids, OAuth/SSO integration.

- 2026-05-10: **Trial bridge config SSOT cleanup (ADR-013)**. The previous code defined the `[trial]` section but never read it; routing actually used `[plane].url` / `[mattermost].url` plus `MM_ADMIN_TOKEN` / `PLANE_API_KEY` env vars, so `[trial].enabled = false` could not actually disable the bridge and `[trial].url` edits were silently ignored. Added `Option<&TrialConfig>` to `mattermost::factory::build()` / `plane::factory::build()`; trial flavor now sources URL + secret from `[trial]` and rejects `enabled = false`. Added `Config::validate_trial()` for cross-section enforcement at load time. `cmd_init` / `cmd_mm` / `cmd_plane` / `cmd_humans` skip the admin env-var requirement under trial flavor. New unit tests ×10 (factory `build_trial_*`, `validate_trial_*`) + integration `tests/trial_factory_e2e.rs` ×3 (2 `#[ignore]`-marked E2E + 1 negative-path). ADR-013 written in EN/KO. `cargo test --workspace` 245 passed, 4 ignored.
- 2026-05-12: **Explicit team-token gating on Live Trial (ADR-017 §6 amendment)**. Field-feedback after the showcase shipped: anonymous visits silently landed in the `DEFAULT_TEAM_TOKEN` sandbox, which made the multi-partition story confusing — users couldn't tell which kanban "belonged to them," and a per-team landing URL pasted on a different machine would silently drop back to the shared sandbox the moment any navigation cleared the `?team=` query. Removed the auto-fallback rendering. New client component `app/components/TeamTokenBar.tsx` sits at the top of Live Trial as the single owner of token persistence; token resolution in `app/page.tsx` is now URL → cookie → empty (no `DEFAULT_TEAM_TOKEN` default). Empty → LiveBoard renders in `disabled` mode (`pointer-events-none + opacity-40`) with `live.disabled.overlay` banner so the user sees what they'll get once connected. TokenBar validates pasted tokens via `GET /api/trial/team-app/status?team=...` (extended in this amendment to return `team_exists` + `project_name`), persists to a 1-year cookie (`genasis-trial-team`) + localStorage, and `router.replace` navigates to `/?tab=live&team=<token>` so the SSR pass picks up the new tenancy. On the CLI side, `genasis init --trial` now ends with a copy-friendly ASCII-bar summary printing the project name, `team_token`, and pre-filled landing URL — the same language as the TokenBar's "Enter your team token" copy so users get consistent guidance whether they paste the URL or the bare token. README/TUTORIAL EN+KO updated with the paste-token walkthrough; ADR-017 §6 documents the design. 18 new i18n keys (KO+EN) for `live.tokenbar.*` and `live.disabled.overlay`. trial-app `npm run typecheck` + `npm run build` clean; `cargo test -p genasis-cli --bin genasis run_trial` still 3 passed.
- 2026-05-12: **Field-feedback round 2 — install.sh `--lang` doc/impl alignment + Linux musl-static release**. Two unrelated user-reported issues:
  (1) `install.sh --lang ko` (space form) was silently rejected as an unknown flag — only `--lang=ko` (equal form) was parsed. Every doc string (help text, error banners, README) used the space form. Rewrote the arg loop from `for arg in "$@"` to `while [ $# -gt 0 ]` with explicit `shift`, accepting BOTH `--lang ko` and `--lang=ko` (same for `--prefix` and `--version`). Help text updated to spell out the dual-form acceptance.
  (2) Release binaries baked a `GLIBC_2.39` floor because `release.yml` built `x86_64-unknown-linux-gnu` on `ubuntu-latest` (now 24.04). Switched both Linux matrix entries to `*-unknown-linux-musl` via `cross` for fully-static binaries — `cross` auto-selects the musl image so no `apt install musl-tools` boilerplate is needed. Same `rustls-tls` feature flag already in `Cargo.toml` means no OpenSSL/libssl dependency to wrestle with. Added a `Verify static linking` step that runs `file` on the produced binary and fails the build if it reports "dynamically linked" — guards against accidental reintroduction of a glibc dep. Added a `compat-smoke` job that runs the packaged x86_64 binary inside `debian:bullseye` (glibc 2.31) on every tag. Dropped both `macos-latest` matrix entries since Apple Silicon notarisation flow is unresolved — README §Supported Platforms now marks macOS as **TBD** with a roadmap note; `install.sh` prints a clear "build from source" message on Darwin instead of attempting a download that doesn't exist. Bilingual README platform table expanded from 4 rows × 2 cols to 5 rows × 3 cols (Pre-built / Build-from-source / Notes). No new `cargo test` cases — these are infra changes; the actual coverage is the release pipeline itself.
- 2026-05-11: **Trial-app showcase model (ADR-017)**. Closed a credibility gap left open by ADR-013/016 — the scripted `Try it` tab animated the same kanban + chat widgets that the live mode uses, so a first-time visitor couldn't tell which one was "real," and the reference PRD ("Example Feature — Task Status") asked agents to build something visually indistinguishable from the trial-app itself. Four coordinated changes: (1) delete the scripted demo — removed `DemoBoard.tsx`, `ChatThread.tsx`, `KanbanBoard.tsx`, `lib/{use-demo-sprint,demo-script}.ts`, `e2e/demo.spec.ts`, all `demo.*` i18n keys, the `tab=demo` URL handler, and the `TrialTab="demo"` variant; landing tab is now `live`. (2) i18n-aware example PRD — `cmd_example.rs` reads `[i18n].active` from `genasis.toml` and emits either `prd.en.md` or `prd.ko.md`, both describing the new reference app: "I Am a Claude Code Expert" / "나는 Claude Code 전문가" — a mobile-phone-bordered 5-question self-assessment quiz with 3 difficulty levels and a question bank ≥ 15. (3) embedded showcase — new `app/components/QuizApp.tsx` + `lib/quiz-bank.ts` ship the reference quiz inside trial-app, gated per-team by a new `sim_teams.app_status` column (V2 → V3 migration, ADR-016 §3 pattern reused). New `ShowcasePanel.tsx` slides in from the left of LiveBoard when toggled, closes on Esc/click-outside/✕. (4) explicit completion signal — new `genasis trial publish` CLI POSTs `{team_token, status: "complete", project}` to `/api/trial/team-app/status` (the route also unauth'd, ADR-016 §4 token-as-capability extends here). Apply tab → "Borrow real env" / "실환경 빌리기" because the user is literally borrowing a real Plane + MM project on the operator's `mmplane-trial.realstory.blog` infrastructure. README links updated `trial.realstory.blog` → `https://mmplane-trial.realstory.blog/?tab=signup`; same sweep across QUICKSTART (EN+KO), blueprint.ko §22.2, agents-pool/prd/trial-webapp.md, playwright.config.ts, e2e/signup.spec.ts. New tests: cmd_example × 4 (en/ko/explicit-flag/missing-config), cmd_trial × 3 (dry-run/missing-token/missing-config). `cargo test --workspace` 266 passed, 4 ignored (259 → +7). trial-app `npm run typecheck` + `npm run build` clean; `/api/trial/team-app/status` shows in route table.
- 2026-05-11: **ADR-016 follow-up — token propagation + SSE isolation + fallback UX (Phase A + B)**. The initial ADR-016 commit (3760b07) plumbed `team_token` only as far as the config file: Rust trial providers still sent no `X-Genasis-Team-Token`, the new `/api/trial/bootstrap` returned 503 whenever `TRIAL_SHARED_SECRET` wasn't set on the operator-hosted instance (the default), and the SSE event bus was global so cross-tenant updates leaked into every connected tab. Phase A: `TrialPlane` / `TrialMattermost` constructors gain a third `team_token: String` arg; the `headers()` method attaches `X-Genasis-Team-Token` when non-empty; both factories thread `t.team_token.clone().unwrap_or_default()` through from `TrialConfig`. Phase B: `/api/trial/bootstrap` drops `requireTrialContext` — the 32-char hex `team_token` body field is now the sole credential (idempotent + unpredictable, see ADR-016 §4); `lib/events.ts` `subscribe()` accepts an optional `teamToken` filter and `emit()` matches against `event.payload.team_token`; `/api/events/stream` reads `?team=` from the request URL and only forwards matching events; `LiveKanbanBoard` / `LiveChatThread` append `?team=<token>` to their `EventSource` URL (browsers can't attach custom headers to EventSource). `page.tsx` gained a fallback branch for unknown tokens — when `?team=<token>` is present but `getTeam(token)` returns null, the user gets an amber error panel ("Team token not recognised — check `[trial].team_token` in your genasis.toml") instead of silently landing in the default sandbox. New `data-team-token` attribute + colour-coded badge on `LiveBoard` so the user always sees which tenancy they're in. New tests: 5 headers tests across `plane/trial.rs` + `mattermost/trial.rs`. ADR-016 EN+KO extended with §"Auth model — token IS the capability". `cargo test --workspace` 259 passed, 4 ignored (254 → +5). trial-app `npm run typecheck` + `npm run build` clean.
- 2026-05-11: **Trial-app identifier alignment + multi-tenancy (ADR-016)**. ADR-013 wired the *routing* between genasis and the trial-app but said nothing about the *identifiers* flowing through that route — so `genasis init --trial` was hard-coding every Plane/Mattermost field to the literal `"trial"` regardless of the team the user actually wanted, and the trial-app sim had no per-team isolation so concurrent demos on the hosted instance overwrote each other. Three coordinated changes shipped together: (1) real-mode schema gains `[plane].project_name` + `[[mattermost].channels]` (`MattermostChannel { key, name, display_name }`), with `Config::derive_naming_defaults()` synthesising a single `scrum` channel for legacy configs; (2) `[trial].team_token` (32-char hex from `random_team_token()`) becomes the per-team isolation key, written by `genasis init --trial` and falling back to a `"default"` sentinel for pre-ADR-016 configs; (3) the trial-app sim migrates from `user_version = 1` to `2` — every `sim_*` table gains `team_token` plus composite `UNIQUE(team_token, slug|name)`, a new `sim_teams` table records each bootstrap, and a `POST /api/trial/bootstrap` route seeds the project + channels under the token. `lib/trial-auth.ts` resolves the token from `X-Genasis-Team-Token` header → `?team=` query → `DEFAULT_TEAM_TOKEN`. Browser UI (`page.tsx` + `LiveBoard` + `LiveKanbanBoard` + `LiveChatThread`) plumbs the token from `?team=<token>` SSR through to every `fetch()` via `withTeamHeader`. `cmd_init.rs` real-mode no longer string-formats `scrum-{project_name}` — it looks up `cfg.mattermost_channel("scrum")`. `cmd_init.rs --trial` prompts for `--name` (or derives from dirname), generates the token, renders the dynamic Tera-style template, POSTs `/api/trial/bootstrap`, and opens `/?tab=live&team=<token>` in the browser. New tests: 7 in `genasis-core::config` (slugify, random_team_token, derive_naming_defaults, effective_team_token, mattermost_channel lookup, channels TOML round-trip), 3 in `cmd_init::tests` (project name from flag, derived from dirname, idempotent on existing config). ADR-016 written EN/KO. `cargo test --workspace` 254 passed, 4 ignored.
- 2026-05-08: **Phase F audit + checkbox catch-up**. Reconciled `progress.md`/`progress.ko.md` against actual repo state (commits e0683de..5bdaadf, build.sh, CONTRIBUTING.md, docs/CREDITS.md). Phase F status table flipped F.1–F.8 from `planning` → `done`. Trial-app US-001..US-022 all `passes: true` in `trial-app/ralph/prd.json` — corresponding F.5 sub-checkboxes closed, plus F.6 (`genasis init --trial`), F.7 (`cmd_example.rs` + 3 example templates) and F.8 (TUTORIAL.md en/ko + README Quick Path/Step-by-Step restructure + CLAUDE.md mirror table). One item kept as `[s]`: bilingual example documents (active-singularity policy keeps examples English-only until `genasis example --lang` lands). Still open: M14.0 ratify gate, M14.3–M14.5 (CLI bootstrap wire-up + golden blank fixture + doctor section), and the entire Phase F Debug History loop (M15–M17).
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
| M14 | 2026-05-05 | 2026-05-08 | Bootstrap as a subcommand (not an attach side-effect) was the right call — the empty-dir hint in `cmd_attach` is enough to discover the entry point, and ADR-001's non-destructive promise stays intact. `BLESS=1` golden-snapshot pattern (M14.4) generalises to M18 cleanly. Doctor `[bootstrap]` section piggy-backs on existing `Role::ALL` slug list — no new validation infra needed. Frontmatter `name:` ↔ filename stem invariant emerged organically from the test fixtures. |

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

## Phase F — Server Setup + Trial + Docs Refactor (2026-05-06)

Making genasis immediately usable: one-command server install, hosted
trial environment, streamlined README, and design swap guide.

| Sub-milestone | Scope | Status |
|---|---|---|
| F.1 | `servers/` — unified docker-compose.yml (Plane + Mattermost + Caddy) + install guide with key extraction | done |
| F.2 | Trial signup web app PRD (agents-pool) — apply → MM #genasis-trial → admin responds → user gets keys | done |
| F.3 | README refactor — simplify to quickstart + trial link, move details to external guides | done |
| F.4 | `docs/DESIGN-SWAP-GUIDE.md` — how to replace design-system.md via `genasis design swap` | done |
| F.5 | Trial demo app (chat + kanban + signup + status pages, US-001..US-022) | done |
| F.6 | `genasis init --trial` CLI integration | done |
| F.7 | `genasis example {prd|design|prd2}` subcommand | done |
| F.8 | Tutorial documentation (`docs/TUTORIAL.md` + `docs/ko/TUTORIAL.md`) | done |

### F.1 — Server installation guide (`servers/`)

Unified Docker deployment of Plane + Mattermost + Caddy reverse proxy.
Source: existing `/work/plane` and `/work/mattermost` Docker configs on this host.

Deliverables:
- `servers/docker-compose.yml` — single file to bring up all services
- `servers/Caddyfile` — TLS + reverse proxy (plane.domain / mm.domain)
- `servers/README.md` — step-by-step guide including:
  - Prerequisites (Docker, domain, DNS)
  - Environment variables to set
  - How to extract Plane API key + workspace slug
  - How to create Mattermost bot tokens (per agent role)
  - How to obtain Plane user UUIDs for agent assignment
  - How to configure `genasis.toml` with the extracted keys

### F.2 — Trial signup web app (PRD in agents-pool)

Hosted demo at mm.realstory.blog / plane.realstory.blog allowing
potential users to try genasis without self-hosting.

Flow:
1. User visits trial signup page
2. Fills in: name, email, project name, desired team size
3. Submission posts to Mattermost `#genasis-trial` channel
4. Admin (maintainer) responds with provisioned credentials
5. User sees keys/login info on the signup page (or via email)

PRD lives in `agents-pool/prd/trial-webapp.md` (private).
Only the plan reference lives in public progress.md.

### F.3 — README refactor

Principles:
- Above the fold: tagline + one-line value prop + quickstart (3 commands)
- Trial CTA: "Try it now with our hosted Plane + Mattermost" → link
- Move complex docs to external files with links
- Keep SEO-critical content (comparison table, architecture diagram)

External guide files (linked from README):
- `docs/QUICKSTART.md` — full install + first attach walkthrough
- `docs/SERVER-SETUP.md` → `servers/README.md`
- `docs/DESIGN-SWAP-GUIDE.md` — design system replacement
- `docs/AGENTS-MARKETPLACE.md` — browsing + installing agents

### F.4 — Design swap guide

`docs/DESIGN-SWAP-GUIDE.md` covering:
- What is design-system.md and why it matters
- `genasis design swap <slug>` — browsing the gallery
- `genasis design swap --from <path>` — local file
- `genasis design restore` — reverting to pristine
- User overrides (`genasis design override add`)
- EPIC mode (auto-issues for impacted UI areas)

### F.5 — Trial demo app (chat + kanban simulation) — ✅ commits e0683de..de860ad (US-001..US-022) + follow-up UI polish (cc95fa9, 9ca1b43, cffb314, a14fc11, 5bdaadf)

Interactive web app at `trial-app/` (Next.js 15 App Router) covering the full hosted-trial flow:
- [x] Demo kanban board (Todo/InProgress/Done columns, animated card moves) — `app/components/KanbanBoard.tsx` + `DemoBoard.tsx`
- [x] Demo chat thread (scripted agent messages with typing indicator) — `app/components/ChatThread.tsx`
- [x] Pre-scripted 8-step sprint simulation (PM → frontend → reviewer → QA) — `lib/demo-script.ts` + `lib/use-demo-sprint.ts`
- [x] [Run Demo Sprint] / [Reset] buttons — wired in `DemoBoard`
- [x] Signup form (name, email, phone, project, team size) → MM `#genasis-trial` — `SignupForm.tsx` + `/api/submit`
- [x] Status page with token-based credential display — `app/status/[token]/page.tsx` + `CredentialsView.tsx`
- [x] Deploy to trial.realstory.blog — `Dockerfile` + `docker-compose.yml` (deployment config landed; live deploy is a runtime/ops step)
- [x] Live human co-work mode (US-015..US-022): `Trial` flavor + `TrialPlaneProvider` / `TrialMattermostProvider` HTTP forwarders, simulated Plane/MM state schema, `/api/plane/*` + `/api/mattermost/*` bridge endpoints, SSE broadcaster (`/api/events/stream`), `LiveBoard` + `LiveChatThread` + drag-drop kanban + chat composer + chat sidebar
- [x] KO/EN i18n toggle (`LangSwitcher`, `lib/i18n.ts`, Pretendard font, accessibility hardening — commits 572485b, a14fc11)

PRD: `agents-pool/prd/trial-webapp.md` (v2). All 22 user stories `passes: true` in `trial-app/ralph/prd.json`.

### F.6 — `genasis init --trial` CLI integration — ✅ commit de860ad

- [x] `--trial` flag for `cmd_init.rs` (US-013) — `pub trial: bool` clap arg
- [x] Flow: create blank project → bootstrap agents → ask "Launch trial app?" → open browser — `run_trial()` in `cmd_init.rs` writes minimal `genasis.toml` with `[trial]` enabled and offers to spawn the trial-app
- [x] Trial app runs as background process on localhost:3000 — spawn command configurable; default `npm --prefix /work/genasis/trial-app run start`
- [x] i18n keys for trial prompt (en/ko)

### F.7 — `genasis example` subcommand — ✅ commit de860ad

- [x] `genasis example prd` — generate sample PRD.md (todo-app with auth, CRUD, responsive UI)
- [x] `genasis example design` — generate sample design-system.md (color/typography/spacing tokens)
- [x] `genasis example prd2` — generate PRD2.md (login, admin backoffice, user management)
- [x] `cmd_example.rs` — new CLI subcommand (US-014)
- [x] Templates in `crates/genasis-cli/templates/examples/{prd.md,design-system.md,prd2.md}` (PRD said `agents/examples/` — relocated to crate-local because templates are static `include_str!()`-embedded with the binary, not part of the dynamic agents catalog)
- [s] i18n: en/ko versions of each example document — examples ship as English-only per the active-singularity policy (ADR-008). Korean mirror is a future enhancement and would require a `--lang` arg on `cmd_example` to pick the right tree.

### F.8 — Tutorial documentation — ✅ commit d023cd9

- [x] `docs/TUTORIAL.md` (English) — 5-step quick path + 5 exercises
- [x] `docs/ko/TUTORIAL.md` (Korean mirror)
- [x] README restructured: "Quick Path" (5 steps → tutorial link) + "Step-by-Step Guide" (full control) — verified: `## Quick Path — Try Genasis in 5 Minutes` + `## Step-by-Step Guide` headings present in `README.md`; equivalents in `README.ko.md`
- [x] Mirror pair added to CLAUDE.md table (`docs/TUTORIAL.md` ↔ `docs/ko/TUTORIAL.md`)

---

## Releases

First release (`v0.1.0`) cut after F.1 (server guide) enables
end-to-end self-hosted setup. `agents-v1.0.0` already published.
Translation completion gate in `release.yml` passes.
