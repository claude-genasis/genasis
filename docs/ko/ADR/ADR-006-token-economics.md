> English: [../../ADR/ADR-006-token-economics.md](../../ADR/ADR-006-token-economics.md) (English version pending — currently a stub)

# ADR-006: Token Economics — RTK + prompt cache + trim hook (no MCP proxy)

## Status

Accepted (2026-05-03).

## Context

Agentic teams burn tokens. The user asked for a token-savings layer
covering both shell tool calls (RTK territory) and MCP / tool-result
volume (cache + trim territory).

Three options were on the table:
1. RTK + Anthropic prompt cache + custom MCP proxy with cache.
2. RTK + Anthropic prompt cache + trim hook (no proxy).
3. No token layer.

Option 1 carries lifecycle complexity (the proxy is a long-running
process; supervision, schema drift). The user explicitly asked us to
ship without the MCP proxy in 1차 (R5 / D13).

## Decision

The 1차 release ships a three-layer token-economics stack:

1. **RTK** — detected during `genasis attach`; if installed, we register
   the appropriate hook in the user's `~/.claude/settings.json`. If not
   installed, we surface the install command on the doctor / attach
   summary but do **not** auto-install it.
2. **Anthropic prompt cache** — `GENASIS.md` is written as a stable
   prefix and remains stable across upgrades except when the fence
   version explicitly changes. CLAUDE.md drift moves volatile content
   into separate sibling files.
3. **`post-tool-trim.sh`** — a PostToolUse hook that summarises any
   tool result larger than `[token_economics] trim_threshold_kb`
   (default 32 KB).

Items deliberately excluded from 1차:

- A custom `genasis-mcp-proxy` — left for a future ADR / release.
- Provider-specific cache layers (mcp-cache, fastmcp) — maintenance
  uncertainty exceeds the win.

## Consequences

**Easier**:
- The integration is opt-in and visible (every save lands in
  `~/.claude/settings.json` where the user can audit).
- The trim hook turns runaway tool outputs into bounded ones without
  hiding important context (head + tail preserved).

**Harder**:
- Without a proxy we have no global view of MCP cache hit ratio. The
  monitor estimates this from per-call telemetry the agents emit, but
  the number is best-effort.

**Foreclosed**:
- Per-tool prompt-stub rewriting. If RTK isn't enough, the user
  installs a richer wrapper themselves.

## References

- Implementation: hook templates under
  `crates/genasis-templates/templates/hooks/`
- Doctor: `crates/genasis-cli/src/cmd_doctor.rs`
- Blueprint: `blueprint.md` §10 (Token Economics)
