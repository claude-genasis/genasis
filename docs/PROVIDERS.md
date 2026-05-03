# Providers — flavor authoring guide

Genasis abstracts external systems (Plane, Mattermost) behind a *flavor* system:
the same trait, multiple implementations for different upstream variants.

> Status: M3 placeholder. Detailed per-trait method docs land alongside the
> implementations in `crates/genasis-providers/`.

## Why flavors?

Two real-world variants exist:
- **upstream**: the official `plane.so` / `makeplane` SaaS or its self-hosted build with the standard payload schema.
- **agent-aware**: a fork with extra fields needed to disambiguate agent users. API endpoints have the same paths but differ in payload keys / required headers.

The framework must support both without forking the rest of the codebase.

## Adding a new flavor

1. Implement the trait (e.g. `PlaneProvider`) in `crates/genasis-providers/src/plane/<your_flavor>.rs`.
2. Register it in `factory.rs`.
3. Update `detect.rs` to recognise the flavor (probe a health endpoint or banner string).
4. Document the differences in this file.

## Detection rules

`auto` mode tries flavors in order:
1. `agent-aware` — checks `GET /api/v1/health` for the custom marker.
2. `upstream` — checks the upstream version banner.
3. Falls back to error and asks the user to set `flavor =` in `genasis.toml`.

## Mattermost

Same shape as Plane. The interesting differences are mostly around bot account
provisioning and personal access token issuance.
