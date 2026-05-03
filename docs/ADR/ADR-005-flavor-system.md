# ADR-005: Provider Flavor system (upstream / agent-aware / auto)

## Status

Accepted (2026-05-03).

## Context

Two real-world variants of both Plane and Mattermost exist in the user base
we expect to serve at 1차 release:

- **upstream** — the official `plane.so` / `makeplane` SaaS, or its standard
  self-hosted build. Standard wire format.
- **agent-aware** — a community fork pattern (originally inspired by the
  fork that motivated this project). Adds an `agent_user` marker field on
  certain endpoints to disambiguate which agent took an action. Adds an
  `x-genasis-agent: true` response header on health endpoints. Any Plane /
  Mattermost deployment that emits this header is treated as agent-aware.

A single hard-coded provider would force one or the other set of users to
fork Genasis. We need both to work out of the box.

## Decision

Each provider crate exposes:

1. A `PlaneProvider` / `MattermostProvider` **trait** as the call surface.
2. A struct per flavor (`UpstreamPlane`, `AgentAwarePlane`, …) that
   implements the trait. The agent-aware variant currently delegates to
   upstream where the wire is identical and overrides only the divergent
   methods.
3. A `detect()` async function that probes the health endpoint and returns
   `DetectedFlavor`.
4. A `factory::build()` that takes a `FlavorChoice::{Upstream,
   AgentAware, Auto}` plus credentials and returns
   `Arc<dyn PlaneProvider>` (resp. Mattermost).

`Auto` runs `detect()` first; if the network is unreachable, the caller is
told to set the flavor explicitly via `genasis.toml`.

Adding a new flavor is a 5-step recipe:
1. New struct file under `flavor/`.
2. Implement the trait, delegating to the closest existing flavor where the
   wire matches.
3. Wire the new variant into `FlavorChoice` and `factory::build()`.
4. Update `detect()` to recognise its banner.
5. Add a row to `docs/PROVIDERS.md`.

## Consequences

**Easier**:
- Users do not need to choose a flavor manually unless they explicitly want
  to override the auto detection.
- New downstream forks have a clear path to first-class support without
  forking Genasis.

**Harder**:
- We must keep flavor probes cheap — they happen on every `init` /
  `attach` / `doctor`.

**Foreclosed**:
- We do not pursue *fully generic* HTTP RPC (à la OpenAPI codegen). The
  flavor surface is small and deliberate.

## References

- Implementation: `crates/genasis-providers/src/plane/`, `…/mattermost/`
- Migration recipe: `docs/PROVIDERS.md`
