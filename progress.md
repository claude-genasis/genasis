# Genasis — Progress Tracker

> 한국어: [progress.ko.md](progress.ko.md)

> Tracker for the milestones described in [`blueprint.md`](blueprint.md).
> The detailed per-step checklist (M0–M12) lives in
> [`progress.ko.md`](progress.ko.md) until M12.7.b ports it into English.

**Started**: 2026-05-03
**Current milestone**: M12 — Internationalization (in progress)

## Milestone summary

| Milestone | Scope | Status |
|---|---|---|
| M0 | Bootstrap (workspace, install.sh launcher, CI, doc skeletons) | done |
| M1 | Core infra (genasis-core, CLI skeleton, version cmd) | done |
| M2 | Detector + overlay merger | done |
| M3 | Plane/Mattermost provider with flavor system | done |
| M4 | Plane user provisioner (Node sub-process) | done |
| M5 | Schema kernel + DB adapters | done |
| M6 | Hooks / skills / commands templates | done |
| M7 | Design hot-swap | done |
| M8 | Doctor / upgrade / detach polish | done |
| M9 | Monitor (Ratatui TUI) | done |
| M10 | Token economics wrap-up | done |
| M11 | Migration & release docs | done |
| **M12** | **Internationalization (install-time selector + active singularity)** | **in progress** |

## M12 sub-step status (English summary; see `progress.ko.md` for the full checklist)

- ☑ M12.0 — ADR-008 written and accepted
- ☑ M12.1 — `genasis-i18n` crate (rust-i18n + en/ko bundles + Lang::resolve)
- ☑ M12.2 — CLI + monitor user-facing prose wrapped via `t!()`
- ☑ M12.3 — `templates/` split into `templates/{en,ko}/` parallel subtrees
- ☑ M12.4 — Interactive `--lang` prompt + `--reference-docs` + `--lang both` rejection
- ☑ M12.5 — `genasis lang switch <lang>` atomic locale swap
- ☑ M12.6 — `install.sh` `--lang` flag + bilingual Bash prompt
- ☑ M12.7.a — Document rename pass (Korean source → `*.ko.md` / `docs/ko/`)
- ☐ M12.7.b — Translate pass (English source authored from Korean baseline)
- ☑ M12.7.c — Cross-link batches (`> 한국어: ...` / `> English: ...`)
- ☑ M12.8 — Golden fixture `with-ko-locale/`
- ☑ M12.9 — `.github` template English-only verification
- ☑ M12.10 — `lint-i18n` CI 3-tier + drift script + key parity script + automated translation-completion PR
- ☑ M12.11 — `genasis doctor [i18n]` extension
- ☐ M12.12 — Retrospective + DoD signoff (post-translation)
- ☐ M12.13 — README SEO + 3-step language toggle (post-translation)

## Phase D — Design Catalog Integration (post-M12)

External design provider integration on top of the existing M7 hot-swap.
Two-mode `docs/design-system.md` (pristine vs external-pointer), getdesign
delegation (no vendoring), local `--from <path>` non-npx entry, user-override
accumulation with conflict prompts, and pristine restore.

User-approved 2026-05-04. Detailed sub-step checklist mirrors in
`progress.ko.md` §M-D.

| Sub-milestone | Scope | Status |
|---|---|---|
| M-D1 | `[design]` config schema; `cmd_design swap <slug>` (npx delegate) and `swap --from <path>`; `restore`; `.design-state.toml`; design-system.md pointer template (en/ko); design-aware SKILL strengthened; e2e round-trip | in progress |
| M-D2 | EPIC plan (auto when ≥4 areas) + Mattermost announce; `verify` (sha256); `override add/list/remove` with conflict prompt; user-override §B accumulator | pending |
| M-D3 | Monitor "Design" widget (key 7, Enter→preview); attach prompts `[design]` keys; doctor checks (npx availability, hash match, mode coherence); ADR-009; post-swap i18n guidance keys | pending |

Design decisions:
- Telemetry default OFF — `genasis design swap` sets `GETDESIGN_DISABLE_TELEMETRY=1`
  before invoking npx. No genasis-side collection server.
- No vendoring of awesome-design-md content — fully delegated to `getdesign` npm
  package. License compliance owned by getdesign upstream.
- `add_command` template is configurable so a self-hosted gallery can replace
  getdesign without code changes.

## Releases

No release tagged yet. First release will be cut after M12.7.b
(translation completion) and M12.13 (README SEO) close, with the
`release.yml` translation-completion gate gating the tag.
