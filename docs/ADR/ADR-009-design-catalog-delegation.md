# ADR-009 — Design catalog delegation (no vendoring)

> 한국어: [ko/ADR/ADR-009-design-catalog-delegation.md](../ko/ADR/ADR-009-design-catalog-delegation.md)

- **Status**: Accepted (2026-05-04)
- **Phase**: D (post-M12)
- **Supersedes**: none
- **Related**: ADR-002 (single static binary), blueprint §7 (M7 design hot-swap)

## Context

After M7 we had a 5-phase `design swap <url> --body <path>` orchestrator
that took a pre-rendered `design-system.md` body, persisted it, diffed it,
and emitted per-area improvement issues. Phase D extends this with two
goals:

1. Make external design systems (Apple, Linear, PostHog, Vercel, ...) a
   first-class swap target — `genasis design swap posthog`.
2. Let agents reference external design specifications transparently, with
   a clear policy for user-driven overrides.

The first goal exposed a forking choice: **vendor 71 brand DESIGN.md files
directly in the genasis repository, or delegate to the upstream `getdesign`
npm CLI?**

## Decision

We **delegate to `getdesign`** (npm package, MIT, by VoltAgent) and **do
not vendor catalog content** in genasis. `genasis design swap <slug>`
shells out to a configurable command template (default
`npx getdesign@latest add {slug} --force --out {out}`) that fetches the
`DESIGN.md` for the chosen brand and drops it at
`docs/design-system/DESIGN.md`. The configurable template is what makes
the gallery URL replaceable — operators can swap in a private fork or a
self-hosted gallery without code changes.

`docs/design-system.md` operates in two modes:

- **`mode: pristine`** — the file body is the truth. No external delegation.
- **`mode: external`** — the file is a thin pointer with three sections:
  §A links to `docs/design-system/DESIGN.md` as the 1st-class reference,
  §B accumulates user overrides under a strict conflict-resolution policy,
  §C documents operator commands.

`docs/.design-state.toml` records the mode, slug, source URL/file, sha256
of the active DESIGN.md, applied_at, previous_slug, override_count, and
the gallery URLs. It is the single source of truth for the monitor widget
and `genasis design status`.

## Consequences

**Positive**

- We do not re-host MIT-licensed content; license compliance stays with
  upstream. The repo stays small.
- 71 brand presets (and any future additions) come for free — getdesign
  already ships them.
- The `add_command` template is the single line operators change to point
  at a self-hosted gallery. genasis never makes assumptions about a
  particular brand list.
- pristine vs external is a clean two-mode model — no "partial"
  states. `restore` is a single, non-destructive operation that archives
  the external dir and brings back the pristine body.

**Negative / risks**

- `genasis design swap <slug>` requires Node ≥18 + npx at runtime. We
  surface this in `genasis doctor` (warning when pristine, error when
  external mode is in use but npx is missing) and in `install.sh` package
  prompts.
- If `getdesign` upstream disappears, slug swaps break. Mitigation: the
  `add_command` template can be repointed at a self-hosted fork; existing
  installations keep working because each project caches its DESIGN.md
  locally under `docs/design-system/`.
- Telemetry: getdesign POSTs install events to
  `https://getdesign.md/api/cli/downloads`. We default to **off** by
  setting `GETDESIGN_DISABLE_TELEMETRY=1` before invoking npx; users opt
  in via `[design].disable_telemetry = false` or `--telemetry`.

## Alternatives considered

1. **Vendor the catalog** — copy the 71 DESIGN.md files (and license)
   into `crates/genasis-design-catalog/THIRD_PARTY/`. Rejected: unbounded
   maintenance (catalog updates), no advantage over delegation since
   getdesign already vendors them with hash-pinned manifests.
2. **Direct REST fetch from getdesign.md** — drop the npx layer, hit
   `https://getdesign.md/<slug>/design-md.txt` (or similar). Rejected:
   the JSON/MD endpoint is not part of getdesign's public contract; the
   npm CLI is. Going through npx keeps us aligned with what the upstream
   project supports.
3. **Format adapter (Stitch DESIGN.md ↔ genasis design-system.md)** —
   parse and rewrite each brand into the existing genasis §0~§n format.
   Rejected: the pointer model is simpler and preserves attribution; the
   external body is read-only and need not be reshaped.

## User-override policy (tied to this ADR)

The pointer body's §B is the only place to record human overrides on top
of the active DESIGN.md. The `design-aware` skill enforces:

1. Quote §A item and user request side-by-side.
2. If the request matches §A → proceed silently.
3. If the request conflicts with §A → ask explicit `[y/N]`.
4. On `y`, run `genasis design override add "<text>"`. The CLI appends
   to §B.2 and bumps `override_count`.
5. On `n`, honour §A.

Conflict resolution is a *human-in-the-loop* step on purpose. Agents must
not silently override the source of truth.

## Open questions for future milestones

- Should swap-induced override re-review be automated? Today, swapping
  to a new slug regenerates the pointer body and resets §B.2. Users
  reapply their overrides under the new design. A future
  `genasis design override migrate` could carry §B over with conflict
  flagging — deferred until we see real usage patterns.
- Should `--from <path>` accept a directory of design docs (to seed both
  pristine and external in a single command)? Deferred.
