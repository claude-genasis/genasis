# ADR-007: `genasis monitor` Ratatui TUI in 1차 release

## Status

Accepted (2026-05-03).

## Context

The user explicitly named Q7-Q8 monitoring requirements as in-scope for
the first release: Sprint counts, RTK token savings, MCP / cache stats,
network call counters, dev/prod URL LEDs with REFRESHED badges, and a
deploy/build/rollback action surface — all in a single TUI.

Putting the dashboard on the back-burner until "later" would have
created a coordination gap: agents do work, but the human has no fast
loop to see what's happening end-to-end.

## Decision

`genasis monitor` ships as a Ratatui + crossterm TUI in the **first**
release, sharing widgets with the install-time `tui_attach` view via
the `genasis-tui` crate.

Widgets included:

1. **Sprint** — current Cycle name, D-day, todo / in-progress / review / done counts.
2. **Tokens** — RTK savings, MCP calls + cache hit %, Anthropic prompt-cache hit %.
3. **Agents** — per-role last activity timestamp, current ticket, status dot.
4. **Deploy** — dev / prod URL LEDs, REFRESHED badge after a build, last build sha.
5. **Network** — Plane / MM / GitHub call counters and bytes.
6. **Log tail** — recent lines from `logs/agent-launches/*`.

Actions (key bindings in scope for 1차):
- `b` build, `d` deploy, `r` rollback, `o` open URL, `v` mark visited, `q` quit.

## Consequences

**Easier**:
- Single screen for the whole agentic cycle. The human can watch one
  pane during a sprint instead of tabbing between Plane, Mattermost,
  GitHub, and the build system.
- Reusing `genasis-tui` widgets keeps the install/attach experience
  visually consistent with the runtime monitor.

**Harder**:
- Ratatui requires terminal control (`crossterm`); we set up the
  alternate-screen / raw-mode dance and must always restore on exit.
- Live data sources (Plane API polling, file watch for build artifacts,
  RTK gain JSON, MCP proxy log) are wired in incrementally — for 1차
  the scaffolding is in place and shows zero values until the data
  channels are connected one-by-one.

**Foreclosed**:
- We do not ship a web dashboard alongside. If the project grows that
  is a separate ADR.

## References

- Implementation: `crates/genasis-monitor/`, `crates/genasis-tui/`
- Blueprint: `blueprint.md` §11 (`genasis monitor`)
