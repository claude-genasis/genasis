> 한국어: [ko/ARCHITECTURE.md](ko/ARCHITECTURE.md)

# Architecture

Authoritative source: [`../blueprint.md`](../blueprint.md). This file is a navigation index for newcomers.

## TL;DR
- **Layer 0** — your existing `.claude/agents/`, `src/`, `docs/` (untouched).
- **Layer 1** — the overlay we add: `GENASIS.md`, `.claude/genasis/{skills,commands,hooks}/`, marker fences inside agent `.md` files.
- **Layer 2** — the `genasis` binary: `init / attach / detach / doctor / upgrade / design / db / monitor / lang`.
- **Layer 3** — Plane, Mattermost, GitHub. Direct API; no MCP servers.

The DB used by the agentic team's target app sits inside Layer 0. Read access goes through `genasis db query` (CLI dispatcher + read-only guard); writes go through `genasis db migrate` (Atlas / Drizzle Kit / DuckDB raw runner).

## Where to look in the source tree
- Marker fence engine — `crates/genasis-overlay/`
- Plane / Mattermost flavors — `crates/genasis-providers/`
- Schema kernel — `crates/genasis-db/`
- Design hot-swap — `crates/genasis-design/`
- Monitor TUI — `crates/genasis-monitor/`
- Templates (Tera) — `crates/genasis-templates/templates/{en,ko}/`
- Runtime i18n bundles — `crates/genasis-i18n/locales/{en,ko}.yml`
- CLI dispatch — `crates/genasis-cli/src/main.rs`

## Layer interaction

```
┌─────────────────────────────────────────────────────────────────┐
│ Layer 3 — Plane · Mattermost · GitHub  (external systems)       │
└────────────────────────────▲────────────────────────────────────┘
                             │ direct REST (reqwest, rustls)
┌────────────────────────────┴────────────────────────────────────┐
│ Layer 2 — `genasis` Rust binary                                 │
│   cmd_init  cmd_attach  cmd_detach  cmd_doctor  cmd_upgrade     │
│   cmd_design  cmd_db  cmd_monitor  cmd_lang  cmd_version        │
└────────────────────────────▲────────────────────────────────────┘
                             │ writes only inside marker fences
┌────────────────────────────┴────────────────────────────────────┐
│ Layer 1 — Overlay artefacts                                     │
│   GENASIS.md (@import'd by CLAUDE.md)                           │
│   .claude/genasis/{skills,commands,hooks}/                      │
│   marker-fenced blocks inside .claude/agents/*.md               │
└────────────────────────────▲────────────────────────────────────┘
                             │ preserves
┌────────────────────────────┴────────────────────────────────────┐
│ Layer 0 — Your existing team                                    │
│   .claude/agents/*.md  src/  docs/  package.json  target-app DB │
└─────────────────────────────────────────────────────────────────┘
```

## Decision rationale

For the *why* behind each architectural choice, read the matching ADR in [`ADR/`](ADR/):

- ADR-001 — Overlay = marker fence
- ADR-002 — Rust single binary
- ADR-003 — Direct API (not MCP)
- ADR-004 — DB channel separation (read vs. write)
- ADR-005 — Provider flavor system
- ADR-006 — Token economics tiers
- ADR-007 — Monitor = Ratatui in 1.0
- ADR-008 — i18n install-time selector + active-language singularity
