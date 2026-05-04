> English: [../ARCHITECTURE.md](../ARCHITECTURE.md)

# Architecture

Authoritative source: [`../blueprint.md`](../blueprint.md). This file is a navigation index for newcomers.

## TL;DR
- **Layer 0** — your existing `.claude/agents/`, `src/`, `docs/` (untouched).
- **Layer 1** — the overlay we add: `GENASIS.md`, `.claude/genasis/{skills,commands,hooks}/`, marker fences inside agent `.md` files.
- **Layer 2** — the `genasis` binary: `init / attach / detach / doctor / upgrade / design / db / monitor`.
- **Layer 3** — Plane, Mattermost, GitHub. Direct API; no MCP servers.

The DB used by the agentic team's target app sits inside Layer 0. Read access goes through `genasis db query` (CLI dispatcher + read-only guard); writes go through `genasis db migrate` (Atlas / Drizzle Kit / DuckDB raw runner).

## Where to look in the source tree
- Marker fence engine — `crates/genasis-overlay/`
- Plane / Mattermost flavors — `crates/genasis-providers/`
- Schema kernel — `crates/genasis-db/`
- Design hot-swap — `crates/genasis-design/`
- Monitor TUI — `crates/genasis-monitor/`
- Templates (Tera) — `crates/genasis-templates/templates/`
- CLI dispatch — `crates/genasis-cli/src/main.rs`

For decision rationale, see `docs/ADR/`.
