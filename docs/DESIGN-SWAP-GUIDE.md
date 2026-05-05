# Design Swap Guide

> 한국어: [ko/DESIGN-SWAP-GUIDE.md](ko/DESIGN-SWAP-GUIDE.md) *(coming soon)*

Your project's design system lives in `docs/design-system.md`. When you want
to change it — adopt a new design language, switch component libraries, or
start fresh — genasis handles the transition safely.

## How design-system.md works

- **Single source of truth** for tokens (colors, typography, spacing, motion) and component specs.
- **Frontend agents read it** before writing any UI code.
- **Designer agent curates it** and orchestrates changes.
- **Two modes**: pristine (body is truth) and external (pointer to an external DESIGN.md).

## Swap from the gallery

```bash
# Browse available design systems
genasis design status

# Swap to a new design system by slug
genasis design swap tailwind-minimal
#   → backs up current design-system.md to pristine.bak
#   → fetches new spec via npx getdesign
#   → emits Plane issues for impacted UI areas
#   → posts 🚨 DESIGN CHANGE to Mattermost
```

After the swap, frontend agents pause until they read the updated spec.

## Swap from a local file

If you have your own design spec (e.g., exported from Figma):

```bash
genasis design swap --from ./my-design-spec.md
#   → same flow but reads from local file instead of gallery
```

## Restore to previous design

```bash
genasis design restore
#   → moves external DESIGN.md to archive
#   → restores pristine.bak as design-system.md
#   → clears §B user overrides
```

## User overrides

When a developer needs to deviate from the design system:

```bash
genasis design override add "Use 8px gap instead of 12px in card grids for mobile"
#   → appends to §B of design-system.md
#   → designer agent acknowledges and tracks
```

```bash
genasis design override list    # show all accumulated overrides
genasis design override remove <id>
```

## EPIC mode (automatic when ≥4 areas impacted)

When a design swap affects 4+ areas (color-tokens, typography, spacing,
layout, components, motion), genasis automatically:

1. Creates an **EPIC** issue in Plane
2. Creates child issues per impacted area
3. Assigns frontend agent to each child
4. Posts summary to Mattermost

Use `--per-area` to force one-issue-per-area even when <4 areas change,
or `--full-rewrite` to always create the EPIC structure.

## Verify integrity

```bash
genasis design verify
#   → compares DESIGN.md SHA-256 with .design-state.toml hash
#   → detects unauthorized modifications
```

## Configuration

In `genasis.toml`:

```toml
[design]
gallery_index_url = "https://getdesign.md/"
add_command = "npx getdesign@latest add {slug} --force --out {out}"
disable_telemetry = true    # no analytics sent to getdesign
external_dir = "docs/design-system"
```

## See also

- [ADR-009: Design Catalog Delegation](ADR/ADR-009-design-catalog-delegation.md)
- [`docs/example-design-system.md`](example-design-system.md) — sample design spec
