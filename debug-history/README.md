# debug-history/

This directory accumulates anonymised field patches submitted by
genasis users via `genasis debug submit`. Per ADR-012 §8 (Data-Only
PR Model), contributors **only** add files under
`debug-history/patches/`. The maintainer processes accumulated patches
via the `/debug-review` Claude Code skill and lands template fixes in
follow-up PRs.

## Layout

| Path | Purpose |
|---|---|
| `index.jsonl` | Patch registry — one JSON object per line: `id`, `submitted_at`, `project_hash`, `status`, optional `fix_commit`. |
| `patches/*.patch.json` | The submitted payloads. JSON-Schema validated; executable content rejected. |
| `analysis/` | Auto-generated artifacts from `/debug-review` (`clusters.md`, `proposed-fixes.md`). |
| `schema.json` | JSON Schema for `patches/*.patch.json`. CI uses this to gate every PR. |
| `archive/YYYY-MM/` | Patches older than 6 months — excluded from active analysis. |

## Workflow

1. User runs `genasis debug submit` after editing overlay files.
2. The CLI prepares an anonymised `patch.json`, opens a PR that adds it
   under `patches/`, and labels it `debug-history`.
3. `.github/workflows/debug-history-pr.yml` validates the PR:
   - changes restricted to `debug-history/patches/*.patch.json`
   - JSON Schema match
   - no shebang / executable patterns
4. Maintainer merges → patch enters the active set.
5. Periodically (manual `/debug-review` or weekly cron), Claude Code
   reads `patches/`, clusters them, drafts template fixes, and opens
   a follow-up PR.
6. Resolved patches are tagged in `index.jsonl` with the fix commit SHA.
