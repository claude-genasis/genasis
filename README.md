# Genasis

<p align="center">
  <a href="README.md"><img src="https://img.shields.io/badge/lang-English-blue?style=flat-square" alt="English"></a>
  <a href="README.ko.md"><img src="https://img.shields.io/badge/%EC%96%B8%EC%96%B4-%ED%95%9C%EA%B5%AD%EC%96%B4-red?style=flat-square" alt="한국어"></a>
  <a href="docs/i18n/CONTRIBUTE-LANG.md"><img src="https://img.shields.io/badge/+-add%20language-lightgrey?style=flat-square" alt="Add a language"></a>
</p>

> 🇺🇸 **English** | [🇰🇷 한국어](README.ko.md)

> **Plane × Mattermost × TDD × Design × DB × Monitor — overlay (not rewrite) for any Claude Code agent team.** Install with one curl command. Korean and English supported.
>
> Tags: `claude-code` · `agentic-team` · `agent-orchestration` · `plane-issues` · `mattermost-bot` · `tdd` · `rust-cli` · `multi-agent` · `ratatui` · `i18n` · `한국어` · `에이전트`

**Status:** v0.0.1 (M12 — internationalization). See [progress.md](progress.md).

---

## What is this

The predecessor was an 11k-line bash script (`create-agentic-team.sh`) that scaffolded a single team for a
single project. **Genasis** is the polyglot, modular successor:

- A **single Rust binary** (no Python, no Node runtime requirement on the target machine for the core CLI).
- **Non-destructive overlay** — it does not rewrite your existing `.claude/agents/*.md`. It injects a small
  marker-fenced block per agent and keeps the rest of your team untouched.
- **Reversible** — `genasis detach` removes everything.
- **Idempotent** — running `attach` twice yields the same result.
- **Rich TUI** for both attach-time visualisation and the runtime `genasis monitor` dashboard.

Read [blueprint.md](blueprint.md) for the full design.

---

## Install (end-user)

```bash
curl -fsSL https://raw.githubusercontent.com/OWNER/genasis/main/install.sh | sh
```

The installer:

1. Detects your OS / arch (Linux x86_64/arm64, macOS arm64/x86_64; WSL on Windows).
2. **Checks prerequisites** (git, curl, tar; optionally node ≥18, gh, atlas, psql/mysql/sqlite3/duckdb, rtk, claude).
3. Prints **OS-specific install commands for any missing packages** — it does not install them automatically.
4. Downloads the matching release binary, verifies sha256, extracts to `~/.local/bin/genasis`.
5. Optionally runs `genasis attach` to bolt onto your current project.

Flags:

```
install.sh [--no-run] [--prefix=PATH] [--version=vX.Y.Z]
```

---

## Build from source (contributors)

```bash
git clone https://github.com/OWNER/genasis
cd genasis
cargo build --release
./target/release/genasis --help
```

Toolchain pinned via [rust-toolchain.toml](rust-toolchain.toml) (Rust 1.78+).

---

## Usage at a glance

```bash
genasis init        # blank project → ECC team + overlay + Plane/MM provisioning
genasis attach      # existing team → bolt overlay on, leaving original files mostly intact
genasis detach      # remove overlay (marker fences only)
genasis doctor      # verify env/tools/permissions
genasis upgrade     # bump overlay version (fence-hash diff)

genasis monitor     # Ratatui TUI: sprint, tokens, agents, deploy, network, logs

genasis design swap <reference-url>
genasis db query "SELECT ..."     # read-only with SQL guard
genasis db migrate                 # delegates to Atlas / Drizzle Kit / DuckDB raw runner
```

---

## Why "Genasis"?

`genesis` (the script) → `genasis` (the framework). Same root, broader scope.

---

## License

MIT — see [LICENSE](LICENSE).

---

## Status

This repository is in its bootstrap phase. The functionality described above is the **target**, not what is
currently executable. Tracker: [progress.md](progress.md).

`<OWNER>` placeholders will be replaced with the actual GitHub owner once the repository is published.

---

### Other languages / 다른 언어
- 🇺🇸 [English](README.md)
- 🇰🇷 [한국어](README.ko.md)
