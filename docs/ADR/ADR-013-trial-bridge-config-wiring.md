> 한국어: [`../ko/ADR/ADR-013-trial-bridge-config-wiring.md`](../ko/ADR/ADR-013-trial-bridge-config-wiring.md)

# ADR-013: `[trial]` Section as the Single Source of Truth for Trial-Bridge Routing

## Status

Accepted (2026-05-10). Cleans up the trial-app integration introduced on
the `ralph/trial-webapp` branch where the `[trial]` section was defined
in config but not actually consulted at runtime.

## Context

`genasis init --trial` spins up the trial-app (Next.js) so users can
exercise the agent workflow without installing Plane or Mattermost. The
initial implementation scattered trial settings across two places:

1. `[plane].url`, `[plane].flavor = "trial"`, `[mattermost].url`,
   `[mattermost].flavor = "trial"` — actually used for routing.
2. `[trial].enabled`, `[trial].url`, `[trial].shared_secret` — defined
   but read by no Rust code. Effectively dead config.

This caused several silent or surprising failures:

- Setting `[trial].enabled = false` while `flavor = "trial"` did NOT
  disable trial-app routing (the user could not actually turn trial
  mode off via the config field that documented itself as doing so).
- Editing `[trial].url` had no effect; users had to mirror the change
  into both `[plane].url` and `[mattermost].url`.
- Filling in `[trial].shared_secret` was ignored; the providers used
  `MM_ADMIN_TOKEN` / `PLANE_API_KEY` env vars as the secret. Trial-mode
  users had to export Mattermost admin tokens that did not actually
  authenticate against anything real.

## Decision

**The `[trial]` section is the single source of truth for trial routing.**

1. **Add `Option<&TrialConfig>` to factory signatures.**
   - `mattermost::factory::build()` and `plane::factory::build()` consult
     `[trial].url` and `[trial].shared_secret` when `flavor = Trial`.
   - The argument is ignored for non-trial flavors.
   - Passing `flavor = Trial` with `trial = None` or `enabled = false`
     fails with an explicit `Error::Config`.

2. **`[plane].url` / `[mattermost].url` are documented placeholders in
   trial mode.**
   - The template emitted by `genasis init --trial` carries explicit
     `# Ignored when flavor = "trial"` comments on those fields.
   - They are kept so the operator can later flip `flavor` to a real
     backend without re-discovering the URL.

3. **Cross-section validation in `Config::load()`.**
   - New `validate_trial()` method requires that any `flavor = "trial"`
     declaration is matched by a `[trial]` section with `enabled = true`.
   - Catches partial edits at load time instead of as a runtime HTTP
     failure against a stale URL.

4. **Trial mode no longer requires backend env vars.**
   - `cmd_init`, `cmd_mm`, `cmd_plane`, and `cmd_humans` skip the
     `MM_ADMIN_TOKEN` / `PLANE_API_KEY` requirement when the active
     flavor is `Trial`.
   - The trial bridge has no admin-token concept; the only secret is
     `[trial].shared_secret`.

## Consequences

**Easier**:
- Trial users can exercise the full workflow with `genasis init --trial`
  alone — zero env vars to export.
- Editing `[trial].url` immediately re-routes traffic, which matters
  for hosting trial-app on a shared port or pointing at a remote demo
  instance.
- Misconfigured configs surface fast, clear errors instead of silently
  hammering the wrong URL.

**Harder**:
- The factory `build()` signature gained one parameter. Only four
  callers internally, but a minor-version bump if external SDKs ever
  link against this crate.

**Foreclosed**:
- A hybrid where the *same* provider is half-trial / half-real is not
  supported. (Independent half-trial — Plane real, Mattermost trial —
  remains possible via per-section flavors.)

## Verification

- Unit tests: `build_trial_*` series in
  `crates/genasis-providers/src/{mattermost,plane}/factory.rs` and
  `validate_trial_*` series in `crates/genasis-core/src/config.rs`.
- E2E: `crates/genasis-providers/tests/trial_factory_e2e.rs`, marked
  `#[ignore]` and gated on `TRIAL_BASE` / `TRIAL_SECRET` env vars so it
  only runs against a live trial-app.

## References

- Implementation:
  `crates/genasis-providers/src/{mattermost,plane}/factory.rs`,
  `crates/genasis-core/src/config.rs`,
  `crates/genasis-cli/src/{cmd_init,cmd_mm,cmd_plane,cmd_humans}.rs`.
- Related ADR: ADR-005 (Flavor system) — Trial is the fourth flavor in
  that taxonomy.
- The trial-app itself: `trial-app/` (Next.js; the destination
  `[trial].url` resolves to).
