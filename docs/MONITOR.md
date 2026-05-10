> 한국어: [ko/MONITOR.md](ko/MONITOR.md)

# `genasis monitor`

> Status: M9 placeholder. Authoritative spec lives in [`../blueprint.md` §11](../blueprint.md).

Ratatui dashboard for the live state of an agentic team. Six widgets, all bilingual, all driven by data already produced by the rest of the framework (no new collectors needed).

## Widgets

| # | Widget | What it shows | Source |
|---|---|---|---|
| 1 | **Sprint** | Current Plane Cycle name, D-day, todo / in-progress / review / done counts | Plane API (5 s poll) |
| 2 | **Tokens** | RTK saved tokens, MCP call count + cache hit %, Anthropic prompt-cache hit % | `rtk gain --json` (60 s poll) + agent log stream |
| 3 | **Agents** | Last activity time + current ticket per agent role | `.pm-delegations/*`, `logs/agent-launches/*`, git commit log |
| 4 | **Deploy** | dev URL + prod URL LEDs, last build SHA + timestamp, REFRESHED badge when manifest hash changes | dev/prod URL HEAD probe (10 s) + manifest watch |
| 5 | **Network** | Per-system byte / call counters (Plane / MM / GitHub) | `~/.cache/genasis/net.json` |
| 6 | **Log tail** | Real-time tail of `logs/agent-launches/*` | `notify` file watcher |

## Key bindings

| Key | Action |
|---|---|
| `1`–`5` | Focus the corresponding widget |
| `o` | Open the focused URL (`xdg-open` / `open`) |
| `b` | Run `genasis.toml [deploy] build` (output streams into the Deploy widget) |
| `d` | Run `[deploy] cmd_dev` or `cmd_prod` |
| `r` | Open the rollback menu (git tag list) |
| `v` | Mark current dev/prod URLs as visited (clears the REFRESHED badge) |
| `q` | Quit |

## Configuration

```toml
# genasis.toml
[deploy]
build     = "pnpm build"
cmd_dev   = "pnpm dev --port 3000"
cmd_prod  = "vercel deploy --prod"
rollback  = "git revert HEAD && git push"
dev_url   = "http://localhost:3000"
prod_url  = "https://app.example.com"
```

See the bundled template at `crates/genasis-templates/templates/en/genasis.toml.tera` (or `ko/`) for the full schema.

## Localisation

Widget titles, status labels (`idle`, `In Progress`, `In Review`, `Done`), and the keyboard hint line all flow through `genasis-i18n`. The active CLI/TUI locale is resolved from `--lang` / `genasis.toml [i18n] cli_lang` / `$GENASIS_LANG` / `$LANG` (in that order); see ADR-008.

## Performance

- 250 ms event-poll loop (CPU < 1 %, key response immediate).
- All polls are independent tokio tasks; the UI thread only renders.
- The `notify` watcher is debounced so a noisy log file doesn't flood the renderer.

## Troubleshooting

| Symptom | Cause / fix |
|---|---|
| Cannot drag-select / double-click text in the monitor | Fixed in 0.1.x+. The earlier build called `EnableMouseCapture` even though no widget consumed mouse events, which silently disabled native terminal selection. Update to a build that includes the fix; no flag needed. |
| Cannot select text in the `genasis init` / `attach` wizard while inside tmux | tmux's `set -g mouse on` intercepts clicks. Hold **Shift** while dragging to bypass tmux and use the host terminal's native selection. (The bottom hint bar shows this reminder.) |
| Selection works in iTerm2 but not in `screen` | `screen` does not propagate selection like tmux's `mouse on`. Either run without `screen`, or copy via `screen`'s own copy-mode (`Ctrl-a [`). |

## See also

- ADR-007 (`docs/ADR/ADR-007-monitor-tui.md`) — why Ratatui is shipped in 1.0 and not deferred.
- `crates/genasis-monitor/src/widgets/` — one source file per widget.
