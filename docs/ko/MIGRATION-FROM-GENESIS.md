> English: [../MIGRATION-FROM-GENESIS.md](../MIGRATION-FROM-GENESIS.md) (English version pending — currently a stub)

# Migrating from `genesis/` (the bash-script predecessor)

> Status: placeholder for M11.

Projects bootstrapped via the original `create-agentic-team.sh` bash script (commonly referred to as "Genesis") can migrate to Genasis with `genasis migrate-from-genesis`.

## What the migration does
- Converts `.env.agents` into the `genasis.toml` plus a curated `.env.agents` (token-only).
- Rewrites `.mcp.json` so only Playwright remains (Plane/MM use direct API now).
- Extracts the Plane/Mattermost-coupled portions of each `.claude/agents/*.md` into Genasis marker fences and leaves the rest of the prompt untouched.
- Backs up everything to `.genasis-migration-backup-<timestamp>/`.

## What is preserved
- All non-fence agent prompt content (your tribal knowledge stays).
- `.claude/skills/`, `.claude/commands/` outside Genasis namespace.
- `docs/design-system.md`, `docs/PRD.md`, etc.

## What changes
- Plane / Mattermost calls become flavor-aware via `genasis.toml [plane.flavor]`, `[mattermost.flavor]`.
- DB workflows route through `genasis db ...`.
- `scripts/agent-monitor.sh` is replaced by `genasis monitor`.

## Mapping table

| Genesis asset | Genasis location |
|---|---|
| `genesis/create-agentic-team.sh` | `genasis init` + `genasis attach` |
| `genesis/setup-agentic-team-v2.sh` | `genasis attach` (incremental) |
| `genesis/rollback-agentic-team.sh` | `genasis detach` |
| `genesis/ls-mm-channel.sh` / `rm-mm-channel.sh` | `genasis mm channel list/rm` |
| Custom agent prompt blocks | marker-fenced overlay in same `.md` |
| `scripts/agent-monitor.sh` | `genasis monitor` |
