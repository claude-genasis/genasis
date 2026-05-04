> 한국어: [ko/MIGRATION-FROM-GENESIS.md](ko/MIGRATION-FROM-GENESIS.md)

# Migrating from `genesis/` (the bash-script predecessor)

> Status: placeholder for M11.

Projects bootstrapped via the original `create-agentic-team.sh` bash script (commonly referred to as "Genesis") can migrate to Genasis with `genasis migrate-from-genesis`.

## What the migration does

- Converts `.env.agents` into the `genasis.toml` plus a curated `.env.agents` (token-only).
- Rewrites `.mcp.json` so only Playwright remains (Plane / Mattermost use direct API now).
- Extracts the Plane / Mattermost-coupled portions of each `.claude/agents/*.md` into Genasis marker fences and leaves the rest of the prompt untouched.
- Backs up everything to `.genasis-migration-backup-<timestamp>/`.

## What is preserved

- All non-fence agent prompt content (your tribal knowledge stays).
- `.claude/skills/`, `.claude/commands/` outside the Genasis namespace.
- `docs/design-system.md`, `docs/PRD.md`, `docs/ARCHITECTURE.md`, etc.
- The git history of every file (the migration uses `git mv` where possible).

## What changes

- Plane / Mattermost calls become flavor-aware via `genasis.toml [plane.flavor]`, `[mattermost.flavor]`.
- DB workflows route through `genasis db ...` (read-only guard + Atlas / Drizzle Kit / DuckDB raw runner).
- `scripts/agent-monitor.sh` is replaced by `genasis monitor` (Ratatui).
- The agent-context language is now an explicit choice (`genasis attach --lang en|ko`); see ADR-008.

## Mapping table

| Genesis asset | Genasis location |
|---|---|
| `genesis/create-agentic-team.sh` | `genasis init` + `genasis attach` |
| `genesis/setup-agentic-team-v2.sh` | `genasis attach` (incremental) |
| `genesis/rollback-agentic-team.sh` | `genasis detach` |
| `genesis/ls-mm-channel.sh` / `rm-mm-channel.sh` | `genasis mm channel list/rm` |
| Custom agent prompt blocks | marker-fenced overlay in same `.md` |
| `scripts/agent-monitor.sh` | `genasis monitor` |
| `.env.agents` (free-form) | `genasis.toml` + slimmed `.env.agents` (tokens only) |
| Hardcoded language in prompts | `genasis.toml [i18n] active = "en" \| "ko"` |

## Step-by-step

```bash
# 1. Snapshot. (The migration also writes its own backup, but a clean
#    git working tree makes rollback a one-liner.)
git status
git stash -u

# 2. Run the migrator (dry-run first).
genasis migrate-from-genesis --dry-run

# 3. Inspect the diff. When happy:
genasis migrate-from-genesis

# 4. Verify.
genasis doctor
genasis lang status

# 5. Commit.
git add -A
git commit -m "chore: migrate from bash genesis to genasis"
```

`genasis detach` always exists as a safety valve — the marker fences can be removed without affecting any of the surrounding prompt content.
