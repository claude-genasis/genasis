> 한국어: [ko/TOKEN-ECONOMICS.md](ko/TOKEN-ECONOMICS.md)

# Token Economics

> Status: M10 placeholder. Authoritative spec lives in [`../blueprint.md` §10](../blueprint.md).

Three layers of token-saving, applied in order:

1. **RTK (Rust Token Killer)** — wraps shell tool calls; 60–90% reduction on dev operations. Genasis detects RTK during `attach` and offers to register the appropriate hook in `~/.claude/settings.json`. If RTK is missing, Genasis still works — the wrap is purely additive.
2. **Anthropic prompt cache** — Genasis writes `GENASIS.md` as a *stable prefix* so the Claude Code cache stays valid across turns. Volatile sections of your `CLAUDE.md` should stay outside this prefix; the `attach` flow surfaces a warning if it detects high-churn content inside the cache window.
3. **`post-tool-trim.sh`** — a `PostToolUse` hook installed under `.claude/genasis/hooks/`. Summarises tool results larger than `[token_economics] trim_threshold_kb` (default 32 KB) by keeping a head + tail slice and appending `(... truncated K lines)`.

## What is intentionally not shipped (1.0)

- **Custom MCP proxy.** ADR-006 weighs the maintenance cost of running a Genasis-specific MCP proxy against the marginal token saving over the three tiers above and concludes the trade-off does not justify shipping it in the first release.
- **Per-call token billing accounting.** The Monitor TUI's Tokens widget reports RTK-saved tokens and prompt-cache hit rate; per-call dollar accounting is left to whatever billing dashboard you already use.

## Configuration

```toml
# genasis.toml
[token_economics]
trim_threshold_kb = 32   # default — bump up if your tool output is structured JSON you actually want
```

## Measuring

The Monitor TUI's Tokens widget calls `rtk gain --json` and pulls Anthropic prompt-cache hit rate from the agent log stream. See [`MONITOR.md`](MONITOR.md) for widget detail.

## See also

- ADR-006 (`docs/ADR/ADR-006-token-economics.md`) — the original three-tier decision and the alternatives that were considered.
