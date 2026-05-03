# ADR-002: Rust single static binary

## Status

Accepted (2026-05-03).

## Context

The genesis predecessor was an 11k-line bash script. Pain points reported
by the user:

- 11k lines in one file: hard to review, hard to test, hard to fork.
- Any per-OS divergence (apt vs brew, GNU sed vs BSD sed) becomes invasive.
- Distribution: clone-and-run, no checksum or signing story.
- Onboarding: contributors have no entry point; the script is *one* thing.

Genasis must ship `genasis init / attach / detach / doctor / upgrade /
design / db / monitor`. Two parts of that surface impose hard constraints:

- **TUI**: install-time visualisation (Q1) and `genasis monitor` (Q7-Q8) must
  both run as full-screen interactive terminal apps. The user explicitly
  named `Ratatui + tui-textarea` (Rust) as the required TUI stack.
- **Distribution**: `curl … | sh` followed by zero compilation. Users should
  not need a compiler, a package manager, or a runtime to be installed.

## Alternatives

| Alternative | Why rejected |
|---|---|
| Python + Typer + Textual | Adds runtime requirement (Python 3.11+); Textual ≠ Ratatui per user spec; pip / uv / pipx fragmentation. |
| Node + Ink | Same runtime burden; agentic-team users may not have Node 18+. |
| Bash with helper Python scripts | Reverts to exactly the genesis pain. |
| Go + Bubbletea | Excellent option, but the user named Ratatui specifically. |
| Rust + Ratatui + smaller helper scripts in Python | Hybrid distribution (rustup *and* uv/pipx) defeats the "single curl install" goal. |

## Decision

The entire Genasis codebase is a **single Rust workspace** that produces one
static binary `genasis`. The binary embeds Tera templates via `include_dir!`
so installation is one file.

Exceptions (deliberately scoped):

- **Playwright Plane user provisioning** runs as a Node sub-process spawned
  from Rust. Rust has no first-class Playwright binding; rewriting Playwright
  is out of scope. The Node script lives at
  `crates/genasis-cli/scripts/provision-plane-users.mjs`. The Rust side
  spawns it, parses JSON on stdout, and surfaces errors. Node 18+ is a
  documented optional prerequisite.
- **Atlas / drizzle-kit / psql / mysql / sqlite3 / duckdb** are dispatched as
  external CLIs when present. Rust does not embed DB clients.

## Consequences

**Easier**:
- One curl-install command. No runtime to bootstrap.
- `cargo test --workspace` runs every test, including golden fixtures.
- Cross-compilation targets via `release.yml` cover Linux x86_64/arm64 and
  macOS arm64/x86_64.
- Tooling: `clippy`, `rustfmt`, `cargo deny`, `cargo audit` form a coherent
  CI story.

**Harder**:
- Contributors need Rust 1.78+. We pin via `rust-toolchain.toml`.
- TUI development on Rust is more verbose than Textual or Ink. Acceptable
  trade-off given the user's stack mandate.
- The Node sub-process boundary is the *only* place where two languages
  meet. We keep its surface to a single JSON-stdout protocol so it stays
  inspectable.

**Foreclosed**:
- We do not ship a Python or Node distribution. If the community wants
  language bindings, they can be added as separate, optional crates that
  call into the Rust core.

## References

- `Cargo.toml` (workspace), `rust-toolchain.toml`
- `install.sh` (binary download path)
- `.github/workflows/release.yml` (cross-compile pipeline)
- Blueprint: `blueprint.md` §4.2 (install.sh launcher), §12 (repo layout)
