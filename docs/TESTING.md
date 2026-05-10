# Testing Guide

> 한국어: [`ko/TESTING.md`](ko/TESTING.md)

How to verify a Genasis change is safe before pushing it. This guide
covers every test layer the project ships — from a 10-second `cargo
fmt` to a 10-minute live-server smoke against the full Plane +
Mattermost docker stack — and tells you which layer to run for which
kind of change.

---

## TL;DR

| You touched… | Run before pushing |
|---|---|
| Rust code (any crate) | `cargo fmt --all && cargo test --workspace` |
| `crates/genasis-providers/` | + `scripts/nightly-e2e.sh` |
| `crates/genasis-cli/` (init / attach lifecycle) | + `scripts/nightly-e2e.sh` |
| `servers/docker-compose.yml` | + `scripts/nightly-e2e.sh` |
| trial-app frontend | _Now in private [agents-pool/trial-app/](https://github.com/claude-genasis/agents-pool/tree/main/trial-app) (v0.6+) — runs separately. Use `genasis init --trial` to consume the operator-hosted instance at https://genasis-trial.realstory.blog._ |
| `*.md` outside `docs/ko/` | `scripts/check-i18n-drift.sh --warn` |
| `crates/genasis-i18n/locales/*.yml` | `scripts/i18n-extract-keys.sh` |
| `tests/golden/**` fixtures | `BLESS=1 cargo test -p genasis-overlay` then verify diff |

The single command that mirrors what CI runs:

```bash
cargo fmt --all -- --check && \
  cargo clippy --workspace --all-targets && \
  cargo test --workspace --all-targets
```

---

## Test layer overview

Genasis ships 10 layers of regression coverage. CI runs L1–L3 + L8 on
every push. The remaining layers run locally — either because they're
too heavy for free-tier CI runners (L9 boots a full Plane stack) or
because they need a developer-controlled environment (L7 hits live
services with credentials).

| Layer | What | When to run | Time | CI? |
|---|---|---|---|---|
| **L1** fmt + lint | `cargo fmt --check`, `cargo clippy` | every commit | ~10 s | ✅ `ci.yml :: test` |
| **L2** unit + integration | `cargo test --workspace --all-targets` | every PR | ~60 s | ✅ `ci.yml :: test` |
| **L3** i18n drift | `check-i18n-drift.sh`, `i18n-extract-keys.sh` | when docs/templates change | ~5 s | ✅ `ci.yml :: lint-i18n` (warn-only on PRs) |
| ~~L4~~ trial-app build | _Moved to agents-pool/trial-app (v0.6+) — see "L4/L5 retirement" note below._ | — | — | — |
| ~~L5~~ trial-app E2E | _Same as L4 — see retirement note._ | — | — | — |
| **L6** README parity | `cargo test -p genasis-e2e` | when CLI surface changes | ~30 s | ✅ rolled into L2 |
| **L7** live-server | `scripts/e2e-test.sh` | before a release | ~10 min | ❌ |
| **L8** coverage | `cargo llvm-cov` | informational | ~80 s | ✅ `ci.yml :: coverage` → Codecov |
| **L9** real-server smoke | `scripts/nightly-e2e.sh` | provider / init / servers changes | ~10 min | ❌ — local-only by design |
| **L10** build-from-source | `./build.sh` | release verification | ~3 min | (release pipeline) |

---

## L1 — `cargo fmt --check` + `cargo clippy`

**What it catches**: style drift, unidiomatic Rust, common bug patterns.

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets
```

**Failure example**:

```
Diff in crates/genasis-cli/src/cmd_init.rs:195
```

**Fix**: `cargo fmt --all` (no `--check`) then re-stage. Both commands
must pass before pushing — `ci.yml :: test` runs them before any
test, so a fmt failure short-circuits the entire CI run.

---

## L2 — `cargo test --workspace --all-targets`

**What it catches**: regressions in any of the 10 workspace crates +
the `tests/e2e` lifecycle harness + golden fixture drift.

```bash
cargo test --workspace --all-targets
# or, faster, drop --all-targets if you only want lib + integration
cargo test --workspace
```

Current floor: **245 tests across 10 crates**.

**Common failures**:
- Schema test failure → check `crates/genasis-core/src/config.rs` and
  the corresponding test module.
- Golden fixture drift → see [Golden fixtures](#golden-fixtures) below.
- Integration test failure under `tests/e2e/` → run that one test
  with `cargo test -p genasis-e2e <name> -- --nocapture` for full output.

---

## L3 — i18n drift (en ↔ ko mirror, en.yml ↔ ko.yml)

Genasis is a strict bilingual project (CLAUDE.md §Bilingual Mirror
Policy). Two scripts enforce parity:

```bash
scripts/check-i18n-drift.sh --warn   # prints drift, exit 0 (warn mode)
scripts/check-i18n-drift.sh          # prints drift, exit 1 if any
scripts/i18n-extract-keys.sh         # ensure en.yml ↔ ko.yml key parity
```

CI runs both in `--warn` mode on PRs, but **release-prep** hard-fails on
drift. Resolve drift by either:
1. Updating the older mirror to match (preferred), or
2. Recording the intentional asymmetry in `progress.{md,ko.md}`.

Reject Korean text in English source files: CI also greps `[가–힯]` in
`*.md`, `*.tera`, `*.rs`, `*.sh`, `*.yml` outside `docs/ko/`. If a
Korean character slips into an English-source file, CI fails outright.

---

## L4 + L5 — retired (v0.6+)

Trial-app source code moved out of this repo into the private
[claude-genasis/agents-pool](https://github.com/claude-genasis/agents-pool/tree/main/trial-app)
in v0.6. The genasis main repo no longer ships the Next.js app, the
Playwright suite, or the `trial-app/` directory.

Operator-hosted deployment lives at
**https://genasis-trial.realstory.blog**, served by Caddy from
`/work/genasis-trial/` on the operator host (see the deployment
README inside that directory).

### What replaces L4/L5 inside this repo
- **For Genasis CLI users**: `genasis init --trial` now points at the
  hosted instance. No local install of trial-app is needed to evaluate
  Genasis.
- **For trial-app contributors**: clone `claude-genasis/agents-pool`
  (private), edit `trial-app/`, run `npm run typecheck && npm run
  build && npm run e2e` there. The operator runs `deploy.sh` to ship
  changes to `/work/genasis-trial/`.

The original layer numbering is preserved (L1, L2, L3, L6, L7, L8,
L9, L10) so that prose references in older PRs / commit messages
still resolve. New layer numbers will not reuse 4 or 5.

---

## L6 — README parity (`tests/e2e/`)

```bash
cargo test -p genasis-e2e
```

**What it catches**: every command advertised in `README.md` /
`README.ko.md` (e.g. `genasis init`, `genasis attach`, `genasis
doctor`) has a smoke test under `tests/e2e/tests/`. If you add a new
CLI command or change a flag's behaviour, add a matching test here so
the README and code can never drift.

This crate is part of the workspace, so `cargo test --workspace` (L2)
already runs it. Calling it out separately is useful when you're
debugging a single failing command.

---

## L7 — Live-server E2E (`scripts/e2e-test.sh`)

```bash
# Mock mode — no live services needed (~30 s)
scripts/e2e-test.sh --mock

# Quick mode — skips Plane provisioning (~5 min)
scripts/e2e-test.sh --quick

# Full lifecycle against live services (~10 min)
scripts/e2e-test.sh
```

**Prerequisites for live mode**:
- `target/release/genasis` (run `cargo build --release` first)
- Either `agents-pool/.env.e2e` (private file in the agents-pool
  submodule) **or** the following env vars exported:
  - `PLANE_URL`, `PLANE_API_KEY`, `PLANE_WORKSPACE_SLUG`
  - `MM_URL`, `MM_ADMIN_TOKEN`, `MM_TEAM_ID`

Results are recorded in `test-results/e2e-<timestamp>.log` and the
latest summary lives in `test-results/e2e-latest.json`. `--mock`
mode is useful for verifying script plumbing without touching real
services; it short-circuits the API calls but exercises the wiring.

---

## L8 — Coverage (`cargo llvm-cov` → Codecov)

```bash
cargo install cargo-llvm-cov   # first time only
cargo llvm-cov --workspace --lcov --output-path lcov.info
```

CI uploads `lcov.info` to Codecov on every push to `main`; the
[![Coverage](https://img.shields.io/codecov/c/github/claude-genasis/genasis?branch=main&style=flat-square&logo=codecov)](https://codecov.io/gh/claude-genasis/genasis)
badge in README reflects the latest result. There's no enforced
floor — coverage is informational, not a gate.

To inspect coverage locally without uploading:

```bash
cargo llvm-cov --workspace --html --open
```

---

## L9 — Real-server smoke (`scripts/nightly-e2e.sh`)

The heaviest layer. Boots the **entire** Plane + Mattermost docker
stack from `servers/docker-compose.yml` and probes it via `genasis
init --probe-only`. **This is local-only by design** — GitHub free
runners (7 GB RAM, 2 vCPU) cannot host the full Plane stack
reliably (postgres + redis + minio + rabbitmq + plane-api/web/space/
admin/live + caddy + Mattermost + mm-postgres). A developer
workstation handles it in ~10 min; GitHub typically times out at
~30 min on health waits alone.

```bash
# Full run (~10 min): build → boot → cargo test (release) → probe → tear down
scripts/nightly-e2e.sh

# Faster (~3 min): boot + probe only, skips cargo test
scripts/nightly-e2e.sh --skip-test

# Inspection mode: leaves the docker stack running after probe
scripts/nightly-e2e.sh --no-down
# remember to clean up afterwards:
( cd servers && docker compose down -v --remove-orphans )

# Inline help
scripts/nightly-e2e.sh --help
```

**Prerequisites**:
- `docker` + `docker compose` v2 plugin
- `cargo` (stable toolchain)
- ~10 GB free disk for the docker images on first pull
- ~6 GB available RAM for the running stack

The script uses an `EXIT` trap so the docker stack always comes back
down (unless you pass `--no-down`).

**When to run**:
- Before pushing changes to `crates/genasis-providers/` (real-server
  flavor logic)
- Before pushing changes to `crates/genasis-cli/` (init / attach
  lifecycle)
- Before pushing changes to `servers/docker-compose.yml`
- Before cutting a release tag

For other changes (frontend, agent overlays, docs), `cargo test
--workspace` is sufficient — L9 is overkill.

---

## L10 — Build-from-source (`./build.sh`)

```bash
./build.sh
```

Validates the install path advertised in README — installs Rust if
absent, builds release binary, copies to `~/.local/bin/genasis`.
Useful before cutting a release to confirm a fresh-machine install
still works.

---

## Test by scenario

| I changed… | Minimum gate | Confidence-boost optional |
|---|---|---|
| One Rust function in genasis-core | L1 + L2 | — |
| Anything in genasis-providers | L1 + L2 | **L9** |
| `cmd_init.rs` / `cmd_attach.rs` | L1 + L2 + L6 | **L9** |
| New `[[humans]]`-like config field | L1 + L2 + golden fixtures | L9 |
| Agent overlay `.tera` template | L2 + L3 | — |
| New `t!()` key | L2 + L3 (i18n key parity) | — |
| `docs/**` only (no code) | L3 | — |
| trial-app frontend (in agents-pool repo) | (run agents-pool's trial-app L4/L5 there) | — |
| `servers/docker-compose.yml` | L9 | — |
| GitHub workflow file | (none — push and observe) | — |
| Release prep | L1 + L2 + L6 + **L7** + **L9** + L10 | — |

---

## CI vs local — split rationale

CI (`.github/workflows/ci.yml`) is the cheap-and-fast loop. It runs
on every push and every PR, taking ~50 s. It's tuned to catch the
common failure classes: fmt, clippy, broken tests, missing
translations, coverage drop. It deliberately does **not** run the
heavy layers because:

1. ~~**L4/L5 (trial-app)**~~ no longer apply — trial-app source moved
   to the private agents-pool repo in v0.6+ (see "L4 + L5 — retired").
2. **L7 (live-server)** would need real Plane + Mattermost
   credentials in repo secrets; security risk for a public repo.
3. **L9 (real-server smoke)** can't fit on free-tier runners —
   moving it to GitHub burned ~30 min/run with no green outcome.

These layers run on the developer's workstation instead, where:
- Resources are available (laptop has 16+ GB RAM, fast disk)
- The developer has the right credentials / env vars
- Failures surface immediately, not in an asynchronous CI run

This split is intentional and codified in CONTRIBUTING.md.

---

## Adding a new test

### A new unit test
Add it next to the code, in a `#[cfg(test)] mod tests` block. CI's L2
will pick it up automatically.

### A new integration test in tests/e2e
Add a new `#[test]` function under `tests/e2e/tests/` and use the
helpers in `tests/e2e/src/lib.rs` (the `harness` module exposes
`spawn_genasis()` etc.). CI's L2 will pick it up automatically.

### A new golden fixture
1. Create `tests/golden/<name>/input/` with the input project state
2. Create `tests/golden/<name>/README.md` describing the fixture
3. Run `BLESS=1 cargo test -p genasis-overlay` to generate
   `tests/golden/<name>/expected/`
4. Inspect the diff, commit if correct, then run without `BLESS=1`
   to confirm the fixture is stable.

### A new Playwright spec
Trial-app Playwright specs live in the **private agents-pool repo**
under `trial-app/e2e/` (not this repo). See
[agents-pool/trial-app](https://github.com/claude-genasis/agents-pool/tree/main/trial-app)
for the test runner setup.

### A new test helper
Helpers shared across crates go in `tests/e2e/src/lib.rs`. The
`tests/unit/` directory is reserved for tiny utility tests that
don't fit a specific crate.

---

## Troubleshooting

### `cargo test` is slow (>5 min)
- Run targeted: `cargo test -p <crate-name>` skips the other 9
  crates.
- Use `cargo test --no-run` to compile without running, then iterate
  on a single test with `cargo test --no-fail-fast <test_name>`.

### Mattermost health check times out in `nightly-e2e.sh`
The Mattermost container does first-run schema migrations on cold
boot (~2-3 min). The script waits 15 min total. If it consistently
times out, run `( cd servers && docker compose logs mm )` to see
what's stuck.

### `npm run e2e` hangs at "starting webServer"
Port 3000 collision. Kill any stray Next.js dev server:

```bash
lsof -i :3000
kill <pid>
```

### Codecov badge shows "unknown"
The `CODECOV_TOKEN` repo secret might be missing or the most recent
push didn't touch any Rust files (so coverage didn't re-upload).
Trigger ci.yml manually or push any cargo-touching change.

### `BLESS=1` fixture regeneration produces noise
Some fixtures contain timestamps or temp paths. The `genasis-overlay`
crate's golden harness sanitises these before comparison; if you see
diffs you didn't intend, check the harness's normalisation rules in
`crates/genasis-overlay/tests/golden_helpers.rs`.

---

## See also

- **[CONTRIBUTING.md](../CONTRIBUTING.md)** — the same L1-L10 table
  in concise form, plus general contribution flow
- **[docs/PROVIDERS.md](PROVIDERS.md)** — flavor system (upstream /
  agent-aware / trial), useful for understanding why L9 needs the
  full stack
- **[docs/MIGRATION-FROM-GENESIS.md](MIGRATION-FROM-GENESIS.md)** —
  legacy bash → Rust test parity rules
- **[docs/ADR/ADR-013-trial-bridge-config-wiring.md](ADR/ADR-013-trial-bridge-config-wiring.md)**
  — why CI's `flavor=trial` path can't replace L9
