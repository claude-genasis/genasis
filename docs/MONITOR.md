# `genasis monitor`

> Status: M9 placeholder. Authoritative spec lives in [`../blueprint.md` §11](../blueprint.md).

Ratatui dashboard with widgets for:
- Sprint (current Plane Cycle, todo / in-progress / review / done counts)
- Agents (last activity time, current ticket per role)
- Tokens (RTK savings via `rtk gain --json`, MCP/cache hits, Anthropic cache hits)
- Network (Plane / MM / GitHub byte counters)
- Deploy (dev URL + prod URL LED, manifest-hash REFRESHED badge, visited flag)
- Log tail (`logs/agent-launches/*`)

Actions: `b`uild, `d`eploy, `r`ollback, `o`pen URL, `v` mark visited, `q`uit.

Configuration: `genasis.toml [deploy]` — see template `crates/genasis-templates/templates/genasis.toml.tera`.
