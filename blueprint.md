# Genasis — Blueprint v1.0

> 한국어: [blueprint.ko.md](blueprint.ko.md)

> Successor to the original 11k-line bash script (`create-agentic-team.sh`,
> commonly referred to as "Genesis") that bootstrapped a single agentic
> team for a single project. **Genasis** generalises that pattern as a
> Rust framework that **non-destructively overlays** Plane / Mattermost /
> TDD / Design hot-swap / Schema-as-code / Monitoring onto **any**
> existing Claude Code agentic team.

> ⚠ **Translation status**: This is the canonical English blueprint.
> The full body content has been authored in Korean
> ([blueprint.ko.md](blueprint.ko.md)) ahead of the English translation.
> M12.7.b (translate pass) ports the §0–§19 sections; until then,
> contributors should treat blueprint.ko.md as the source of truth and
> consult `docs/ADR/ADR-008-i18n-install-time-selector.md` for the
> i18n architecture decision (which is fully English-native).

## Overview

Genasis attaches a non-destructive **overlay** onto a user's existing
Claude Code agent team. The overlay covers:

| Layer | What |
|---|---|
| **L0** | The user's `.claude/agents/*.md`, `src/`, `package.json`, target-app DB — preserved untouched. |
| **L1** | Marker-fence overlay inside agent files + `.claude/genasis/{skills,commands,hooks}/` + `GENASIS.md` (`@import`'d by `CLAUDE.md`). |
| **L2** | The `genasis` Rust binary itself: `init`, `attach`, `detach`, `doctor`, `upgrade`, `db`, `design`, `monitor`, `lang`. |
| **L3** | External systems: Plane, Mattermost, GitHub. |

## Sections (full body lives in `blueprint.ko.md` until M12.7.b)

- §0 Premises, goals, non-goals
- §1 Personas
- §2 3-layer architecture (overlay model)
- §3 Marker fence specification
- §4 CLI surface + `install.sh` launcher
- §5 Provider adapters & flavor system
- §6 Schema kernel & DB operations
- §7 Design-system hot-swap
- §8 Hooks / skills / commands catalogue
- §9 TDD / SDD / security enforcement
- §10 Token economics
- §11 `genasis monitor` (Ratatui TUI)
- §12 Repository structure (Rust workspace)
- §13 Migration from the bash Genesis predecessor
- §14 Testing strategy
- §15 First-release scope (DoR)
- §16 ADR index
- §17 Risks & mitigations
- §18 Next steps
- §19 Internationalization (M12) — read together with
  `docs/impact-of-multilang-prompts.md` and
  `docs/ADR/ADR-008-i18n-install-time-selector.md`.
- **§20 Default agentic team bootstrap (M14)** — green-field install
  path; introduces base + patch 2-layer structure. Read together with
  `docs/ADR/ADR-010-default-team-bootstrap.md` (planning).
- **§21 Debug History — Field Feedback Loop (Phase F)** — always-on
  drift detection + opt-in submission + data-only contributor governance
  + maintainer auto-development via Claude Code. Read together with
  `docs/ADR/ADR-012-debug-history-feedback-loop.md`.

## §21 — Debug History: Field Drift Feedback Loop (Phase F)

Genasis is a meta-tool that generates overlay files users inevitably
modify. Those modifications are the highest-signal feedback for
improvement. Phase F implements:

1. **Manifest** (`.manifest.json`) — SHA-256 snapshot at attach time
2. **Passive drift detection** — every CLI call compares live vs manifest (~1ms)
3. **`genasis debug collect`** — anonymised, secret-stripped `patch.json`
4. **`genasis debug submit`** — opt-in submission (GitHub Issue or PR)
5. **`/debug-review` skill** — Claude Code clusters patches, proposes template fixes
6. **Data-Only Contributor Governance** — contributors submit patch.json only;
   maintainer processes via automated Claude Code development

Security: only overlay-scoped diffs, secret redaction, path hashing,
opt-in with full payload preview. See
[`ADR-012`](docs/ADR/ADR-012-debug-history-feedback-loop.md) for full design.

## i18n decision (M12)

Active agent context is **always exactly one language**, chosen at
install time (`--lang en|ko`). `--lang both` is rejected with a
bilingual banner that cites `docs/impact-of-multilang-prompts.md`.
Documentation in this repository is dual-tree: English source-of-truth
(this file) + Korean mirror (`*.ko.md` / `docs/ko/`). Mirror drift is a
warning on regular PRs and a hard-fail on release-prep PRs (see
ADR-008).

## Status

This file is the English entry point. For the full design treatment in
its current canonical form, read [`blueprint.ko.md`](blueprint.ko.md).
