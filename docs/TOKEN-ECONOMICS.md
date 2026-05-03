# Token Economics

> Status: M10 placeholder. Authoritative spec lives in [`../blueprint.md` §10](../blueprint.md).

Three layers of token-saving:

1. **RTK (Rust Token Killer)** — wraps shell tool calls; 60–90% reduction on dev operations. Genasis detects RTK during `attach` and offers to register the appropriate hook in `~/.claude/settings.json`.
2. **Anthropic prompt cache** — Genasis writes `GENASIS.md` as a *stable prefix* so the Claude Code cache stays valid. Volatile sections of your `CLAUDE.md` should stay outside this prefix.
3. **`post-tool-trim.sh`** — a `PostToolUse` hook that summarises tool results larger than `[token_economics] trim_threshold_kb` (default 32 KB).

Genasis 1차 release explicitly **does not ship** an MCP proxy. That option remains available for future versions if community demand justifies it.
