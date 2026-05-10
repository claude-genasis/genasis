# Contributing to Genasis

> 한국어: [CONTRIBUTING.ko.md](CONTRIBUTING.ko.md)

Welcome — this guide walks you through everything you need to install before opening your first PR, plus the workflow we follow once you're set up.

## TL;DR

```bash
# 1. Toolchain (Rust + cargo)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal

# 2. Auxiliary CLIs (optional but recommended)
sudo apt-get install -y build-essential pkg-config libssl-dev   # Linux build deps
brew install gh atlas duckdb                                    # macOS dev tools

# 3. Coverage tooling (only if you touch CI / coverage)
. "$HOME/.cargo/env" && cargo install cargo-llvm-cov

# 4. Clone + build + test
git clone https://github.com/claude-genasis/genasis
cd genasis
cargo test --workspace --no-fail-fast
```

If `cargo test` returns "245+ passed" you're ready to contribute.

---

## Local regression layers

Genasis ships several layers of regression coverage. CI runs L1–L3 + L8 on
every push; the rest are runnable locally and surface different failure
classes. Pick the layer that matches what you touched.

| Layer | Command | What it covers | Time | In CI? |
|---|---|---|---|---|
| **L1** fmt + lint | `cargo fmt --all -- --check` · `cargo clippy --workspace --all-targets` | style + lint | ~10s | ✅ `ci.yml :: test` |
| **L2** unit + integration | `cargo test --workspace --all-targets` | 245+ Rust tests including golden fixtures | ~60s | ✅ `ci.yml :: test` |
| **L3** i18n drift | `scripts/check-i18n-drift.sh` · `scripts/i18n-extract-keys.sh` | EN↔KO mirror parity + i18n key parity | ~5s | ✅ `ci.yml :: lint-i18n` |
| ~~L4~~ trial-app build | _Moved to private agents-pool repo (v0.6+). The trial-app source no longer lives in this repo — see [agents-pool/trial-app/](https://github.com/claude-genasis/agents-pool/tree/main/trial-app)._ | — | — | — |
| ~~L5~~ trial-app E2E | _Same — Playwright specs live alongside the trial-app in agents-pool._ | — | — | — |
| **L6** README-parity E2E | `cargo test -p genasis-e2e` | every command advertised in README (M19) | ~30s | ✅ rolled into L2 |
| **L7** live-server E2E | `scripts/e2e-test.sh [--mock\|--quick]` | full lifecycle vs real Plane + Mattermost | ~10min | ❌ |
| **L8** coverage | `cargo llvm-cov --workspace --lcov --output-path lcov.info` | line coverage → Codecov | ~80s | ✅ `ci.yml :: coverage` |
| **L9** nightly real-server | `scripts/nightly-e2e.sh` (local pre-push gate) | L7 against `servers/docker-compose.yml` | ~10min local | ❌ — local-only by design (GitHub free runners are too slow for the full Plane stack) |
| **L10** build-from-source | `./build.sh` | release binary + `~/.local/bin` install | ~3min | (release verification) |

**Quick path before pushing a PR**: `cargo fmt --all && cargo test --workspace`
covers L1 + L2 + L6 in one go and matches what `ci.yml :: test` will run.

For the full layer-by-layer instructions, scenario-based picks ("I changed
X, what should I run?"), and troubleshooting, see
**[`docs/TESTING.md`](docs/TESTING.md)**.

---

## Why each prerequisite

The list is intentionally short and grouped by which kind of contribution needs it.

### Required for any code change

| Tool | Why we need it | Install |
|---|---|---|
| **rustup** + **cargo** + **rustc** (stable channel) | Genasis is a Cargo workspace of 10 Rust crates. Cargo drives everything (build, test, clippy, fmt). The `rust-toolchain.toml` pins the channel to `stable`, so rustup auto-selects the right compiler when you `cd` into the repo. | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh -s -- -y --default-toolchain stable --profile minimal` |
| **rustfmt** + **clippy** components | CI runs `cargo fmt --all --check` and `cargo clippy --workspace --all-targets`. PRs that don't pass both are rejected automatically. The `--profile minimal` install above adds them through `rust-toolchain.toml` declaration. | (bundled with the rustup install above) |
| **git** | Source control. Every commit/PR happens through git. | `sudo apt-get install git` (Debian/Ubuntu) · `brew install git` (macOS) |
| **C compiler & OpenSSL headers** | A few transitive crate dependencies build native code. On Linux you need `build-essential` (gcc + make) and `libssl-dev`. On macOS the Xcode command-line tools cover it. | Linux: `sudo apt-get install build-essential pkg-config libssl-dev` · macOS: `xcode-select --install` |

### Required only for the install/release path

| Tool | Why we need it | Install |
|---|---|---|
| **bash** ≥ 4 | `install.sh` is POSIX-ish bash. The `--lang both` rejection banner uses heredocs that need bash semantics. | macOS: `brew install bash` (system bash 3.2 works for most things but not the prereq matrix) · Linux: already present |
| **curl**, **tar**, **sha256sum** (or `shasum -a 256` on macOS) | `install.sh` downloads the release binary, verifies the checksum, and untars it. | All present on default Linux + macOS images |
| **gh** (GitHub CLI) | Used by `genasis init`'s GitHub branch-protection helper and the `release-prep` workflow. Optional for local development; required for any PR that touches CI's GitHub-API path. | `brew install gh` · `sudo apt-get install gh` (after adding the GitHub CLI repo) |

### Required only for the runtime monitor + RTK token economics

| Tool | Why we need it | Install |
|---|---|---|
| **rtk** (Rust Token Killer) | The Tokens widget in `genasis monitor` calls `rtk gain --json` to surface RTK's token-saved counters. Genasis works without rtk; the widget just shows zeros. | `cargo install rtk` (or follow the rtk project's README) |
| **node** ≥ 18 + **npm** + (later) **playwright** | `crates/genasis-cli/scripts/provision-plane-users.mjs` is a Node sub-process that automates Plane user provisioning via Playwright. Only relevant if you're working on M4 (Plane user provisioner). | `nvm install 18 && nvm use 18` · `npm install --prefix crates/genasis-cli/scripts` |
| **claude** (Claude Code CLI) | Not needed to build or test. Needed only if you want to dogfood genasis on a real agent team. | `npm install -g @anthropic-ai/claude-code` |

### Required only for the schema kernel

| Tool | Why we need it | Install |
|---|---|---|
| **atlas** | Default migration tool for `genasis db migrate` (postgres / mysql / sqlite). Only needed if you exercise the DB pipeline locally. | `curl -sSf https://atlasgo.sh \| sh` |
| **psql** / **mysql** / **sqlite3** / **duckdb** | One per DB driver you actually use. `genasis db query` shells out to these for the read-only path. | `apt-get install postgresql-client mysql-client sqlite3` · `brew install postgresql mysql sqlite duckdb` |

### Required only for coverage / Codecov work

| Tool | Why we need it | Install |
|---|---|---|
| **cargo-llvm-cov** | The `coverage` CI job runs `cargo llvm-cov --workspace --lcov` and uploads lcov.info to Codecov. Locally, run it before opening a PR that targets coverage thresholds. | `cargo install cargo-llvm-cov` (one-time) and `rustup component add llvm-tools-preview` |

### Required only for documentation work

| Tool | Why we need it | Install |
|---|---|---|
| **markdownlint** (optional) | We don't enforce it in CI but a markdown lint pass before submitting catches stale anchors and broken cross-links. | `npm install -g markdownlint-cli` |
| **ImageMagick** or **rsvg-convert** | `docs/assets/og-image.svg` and `docs/assets/og-image.ko.svg` are the source of truth; the PNG variants for GitHub's social-preview slot are rendered with `convert -background "#0b1320" -density 200 ... 1280x640`. | `apt-get install imagemagick` · `brew install imagemagick` |
| **asciinema** (optional) | `docs/assets/demo.cast` is asciinema v2 format. If you re-record the demo, install asciinema first. | `apt-get install asciinema` · `brew install asciinema` |

---

## Workflow

1. **Fork or branch.** External contributors fork; maintainers branch from `main`. Branch names follow Conventional Commits: `feat/`, `fix/`, `docs/`, `chore/`, `i18n/`.
2. **Build + test locally.** `cargo build --workspace && cargo test --workspace --no-fail-fast`. Both must pass before pushing.
3. **Lint i18n drift.** If you touched any `*.md` outside `docs/ko/` or any `.tera` template, run `scripts/check-i18n-drift.sh --warn` and `scripts/i18n-extract-keys.sh`. CI runs both; release tags hard-fail on drift.
4. **Open a PR.** GitHub will pick up `.github/PULL_REQUEST_TEMPLATE.md`; fill in the i18n checklist if your change touches user-facing strings or docs.
5. **Translation flow.** New `t!()` keys land in **both** `crates/genasis-i18n/locales/en.yml` and `crates/genasis-i18n/locales/ko.yml` in the same commit. English doc edits warn on mirror drift; the `release-prep` workflow opens an automated `[i18n] Translation completion` PR before each release tag.
6. **Adding a new locale.** See [`docs/i18n/CONTRIBUTE-LANG.md`](docs/i18n/CONTRIBUTE-LANG.md).

## House style

- **Conventional Commits** (`feat / fix / docs / chore / i18n`).
- **Squash-merge only** (configured at the repo level).
- **No emoji in code.** Emoji are fine in commit messages and CHANGELOGs.
- **Docs trump comments.** If a behaviour deserves an explanation, put it in the relevant `docs/` page or the function's rustdoc, not a one-off code comment.
- **`unsafe` requires a `// SAFETY:` block.** No exceptions.

## Found a bug?

Open an issue using the bug template at `.github/ISSUE_TEMPLATE/bug.md`. Please include the output of `genasis lang status` and the OS / `uname -a` line.

## Submitting debug-history patches

ADR-012 §8 ("Data-Only PR Model") gives users a way to share the
overlay edits they made in their own projects without forking the
repo. Run:

```bash
genasis debug status      # see what drifted vs the manifest
genasis debug collect     # write an anonymised patch.json under
                          # ~/.genasis/debug-history/<project-hash>/
genasis debug submit      # opens a PR that adds the patch under
                          # debug-history/patches/. Rate-limited to 1
                          # submit per project per 24 h.
```

What contributors **may** include in such a PR:

- Exactly one new file under `debug-history/patches/*.patch.json`.

Anything else on the same PR — template edits, code changes, doc
edits — fails `.github/workflows/debug-history-pr.yml`. Maintainers
process accumulated patches via the `/debug-review` skill (see
`.claude/skills/debug-review/`) and land template fixes in separate,
cited PRs.
