> 한국어: [ko/PROVIDERS.md](ko/PROVIDERS.md)

# Providers — flavor authoring guide

Genasis abstracts external systems (Plane, Mattermost) behind a *flavor* system: the same trait, multiple implementations for different upstream variants.

> Status: M3 placeholder. Detailed per-trait method docs land alongside the implementations in `crates/genasis-providers/`.

## Why flavors?

Two real-world variants exist:

- **upstream** — the official `plane.so` / `makeplane` SaaS or its self-hosted build with the standard payload schema.
- **agent-aware** — a community fork pattern (originally inspired by the fork that motivated this project). Adds an `agent_user` field on certain endpoints to disambiguate which agent took an action and emits an `x-genasis-agent: true` response header on health endpoints. Any deployment that exposes that header is treated as agent-aware.

The framework must support both without forking the rest of the codebase.

## Adding a new flavor

1. Implement the trait (e.g. `PlaneProvider`) in `crates/genasis-providers/src/plane/<your_flavor>.rs`.
2. Register the variant in `factory.rs` (`FlavorChoice` + `build()`).
3. Update `detect.rs` to recognise the flavor (probe a health endpoint or banner string).
4. Add a row to this file describing the difference vs. upstream.
5. Add a test in `crates/genasis-providers/tests/flavor_parse.rs`.

## Detection rules (`auto`)

`auto` mode tries flavors in order:

1. **agent-aware** — checks `GET /api/v1/health` for the `x-genasis-agent` response header (or `flavor: "agent-aware"` in the body).
2. **upstream** — falls back to the upstream schema if no marker is present.
3. If the network is unreachable, the caller is told to set `flavor =` explicitly in `genasis.toml`.

Detection runs once per `init` / `attach` / `doctor` and is cheap (a single HTTP HEAD-style health probe).

## Mattermost

Same shape as Plane. The interesting differences are mostly around bot account provisioning and personal access token issuance.

- `upstream` uses the standard `/api/v4/users` + token endpoints.
- `agent-aware` adds an `agent_user` property on bot creation that the standard flavor does not.

## Configuration

```toml
# genasis.toml
[plane]
url = "https://plane.example.com"
flavor = "auto"          # "auto" | "upstream" | "agent-aware"
workspace_slug = "demo"

[mattermost]
url = "https://mm.example.com"
flavor = "auto"
team_name = "demo"
```

## See also

- ADR-005 (`docs/ADR/ADR-005-flavor-system.md`) — the original decision and its alternatives.
- ADR-003 — why we use direct REST instead of MCP servers.
