# CLAUDE.md — Project Instructions for Genasis

> This file is loaded automatically by Claude Code. It contains project-wide
> conventions and constraints that all agents / sessions must follow.

## Bilingual Mirror Policy

This project maintains **English ↔ Korean parallel documents**. When one
file in a mirror pair is modified, the corresponding file in the other
language **must be updated in the same commit** (or the immediately
following commit at most) to maintain structural and content parity.

### Mirror pairs (authoritative list)

| English (source of truth for code docs) | Korean (source of truth for planning) |
|---|---|
| `README.md` | `README.ko.md` |
| `blueprint.md` | `blueprint.ko.md` |
| `progress.md` | `progress.ko.md` |
| `CONTRIBUTING.md` | `CONTRIBUTING.ko.md` |
| `docs/ARCHITECTURE.md` | `docs/ko/ARCHITECTURE.md` |
| `docs/PROVIDERS.md` | `docs/ko/PROVIDERS.md` |
| `docs/MIGRATION-FROM-GENESIS.md` | `docs/ko/MIGRATION-FROM-GENESIS.md` |
| `docs/TOKEN-ECONOMICS.md` | `docs/ko/TOKEN-ECONOMICS.md` |
| `docs/MONITOR.md` | `docs/ko/MONITOR.md` |
| `docs/impact-of-multilang-prompts.md` | `docs/ko/impact-of-multilang-prompts.md` |
| `docs/ADR/ADR-*.md` | `docs/ko/ADR/ADR-*.md` |

### Rules

1. **Structural parity**: Both files must have the same section
   headings, the same sub-step items (translated), and the same
   status markers (`[x]`, `[ ]`, `[s]`, etc.).
2. **Content parity**: Meaning must match. Verbatim word-for-word
   translation is not required, but no section may be present in one
   file and absent in the other.
3. **Single-commit rule**: If you edit `progress.ko.md`, you must
   also bring `progress.md` to parity before finishing. The same
   applies in the reverse direction.
4. **Cross-link header**: Every mirror file must start with a
   cross-link to its counterpart:
   - English: `> 한국어: [filename.ko.md](filename.ko.md)`
   - Korean: `> English: [filename.md](filename.md)`
5. **CI enforcement**: `scripts/check-i18n-drift.sh` warns on PRs and
   hard-fails on release-prep when drift is detected.
6. **New mirror files**: When creating a new English doc that warrants
   a Korean mirror (or vice versa), create both in the same commit
   and add the pair to this table.

### Scope

This policy applies to **all** `.md` documentation files that have a
declared mirror pair. It does NOT apply to:
- Code comments (English only)
- Commit messages (English only, Conventional Commits)
- Rust doc comments (English only)
- Template `.tera` files (already split by `templates/{en,ko}/`)
- `i18n/*.yml` locale bundles (managed by `lint-i18n` key parity)

## Conventions

- Rust: `cargo fmt` + `cargo clippy` before commit.
- Commits: Conventional Commits (`feat / fix / docs / chore / i18n`).
- New user-facing strings: `t!()` macro, both `en.yml` and `ko.yml`.
- ADRs: Korean SSOT in `docs/ko/ADR/`, English mirror in `docs/ADR/`.
- Progress tracking: `progress.ko.md` is the operational SSOT (full
  checklists); `progress.md` is its structural mirror in English.
  Both must stay in sync per the bilingual mirror policy above.
