# CLAUDE.md — Project Instructions for Genasis

> This file is loaded automatically by Claude Code. It contains project-wide
> conventions and constraints that all agents / sessions must follow.

---

## Core Philosophy — The North Star

Genasis exists to **make AI agents first-class team members alongside
humans** by embedding them into the collaboration tools humans already use
(Plane for issues, Mattermost for chat). Every feature, design decision,
and code change must be evaluated against this mission:

1. **Human-agent seamless collaboration**: Agents must participate in the
   same messengers, issue boards, and sprint ceremonies that human team
   members use — not in a separate "AI sandbox." The goal is that a human
   reviewing a Plane board or Mattermost channel cannot (and need not)
   distinguish whether a given update came from a human or an agent.

2. **Non-destructive adoption for existing teams**: Teams already running
   agentic workflows (ECC, knowledge-work-plugins, claude-code-templates,
   custom `.claude/agents/`) must be able to bolt Genasis on **without
   rewriting their existing agent definitions**. The overlay model (marker
   fences) exists precisely to honour this.

3. **Turnkey bootstrap for new teams**: Teams with zero agentic experience
   must get a fully functional agentic team (`genasis init`) that is
   immediately wired into Plane + Mattermost, ready to collaborate with
   humans from minute one.

4. **Agents operate through human-facing channels only**: Every agent
   action (status update, question, code review request, blocker
   escalation) flows through the same Plane tickets and Mattermost threads
   that humans read. No hidden side-channels.

### How to apply this philosophy

When proposing a new feature, refactor, or architectural change:
- Ask: "Does this bring agents closer to being natural team members in
  human collaboration tools?"
- If the answer is no, the proposal must include a justification for why
  the deviation serves the mission indirectly, or it should be rejected.
- If an alternative exists that better serves human-agent collaboration,
  propose it as a critical counter-suggestion — even if the original idea
  is technically elegant.
- Favour designs where a human PM, designer, or developer interacts with
  agents **through the same UX they already use** rather than through
  CLI-only or developer-only interfaces.

---

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

---

## Debug History — Field Feedback Loop

Genasis is a **meta-tool** that generates and manages agentic team
configurations inside real projects. Inevitably, users modify the
generated overlay files to fix bugs or adapt to project-specific needs.
These modifications are **invaluable signal** for improving Genasis itself.

### Concept: Drift-as-Feedback

```
User project (.claude/genasis/)
  │
  ├── Initial state (recorded at attach time as manifest)
  ├── Current state (live files)
  └── Drift = diff(manifest, current)
        │
        ▼
  genasis debug collect
        │  (strips source code, keeps only overlay-scoped diffs)
        ▼
  ~/.genasis/debug-history/<project-hash>/<timestamp>.patch
        │
        ▼
  genasis debug submit  (opt-in: pushes anonymised patches to genasis repo)
        │
        ▼
  genasis/debug-history/  (in genasis repo — curated field patches)
        ▼
  Claude Code reads debug-history/ when working on genasis to inform fixes
```

### Security constraints

- **NEVER** include user source code (`src/`, `lib/`, `app/`, etc.)
- **NEVER** include secrets (`.env`, tokens, credentials)
- **ONLY** diff files within `.claude/genasis/` and marker-fenced
  sections of `.claude/agents/*.md`
- Project identity is a one-way hash (not reversible to repo name/path)
- `debug submit` is always **opt-in** and shows the exact payload before
  sending

### How this enables genasis self-improvement

1. `debug-history/` patches in this repo serve as **regression seeds** —
   Claude Code can read them to understand what real users needed to fix.
2. A `/debug-review` skill (planned) will summarise accumulated patches,
   propose template/overlay improvements, and draft PRs automatically.
3. The manifest comparison runs **by default** (debug mode always on) so
   drift is silently tracked locally even if never submitted — zero
   developer effort to collect the data.

### Contribution governance (Data-Only PR Model)

- **Contributors** may ONLY submit `debug-history/patches/*.patch.json`
  files via PR. They must NOT modify templates, overlay source, or
  analysis files based on debug data.
- **Maintainer** processes accumulated patches via Claude Code automated
  development (`/debug-review` skill), reviews auto-generated PRs, and
  merges fixes.
- This separation ensures: zero supply-chain risk from contributors,
  consistent fix quality across all users, minimal review burden.
- See `docs/ADR/ADR-012-debug-history-feedback-loop.md` §8 for full
  rationale.
