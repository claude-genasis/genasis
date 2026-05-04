> English: [../../ADR/ADR-003-direct-api-not-mcp.md](../../ADR/ADR-003-direct-api-not-mcp.md) (English version pending — currently a stub)

# ADR-003: Plane / Mattermost via direct REST API, not MCP

## Status

Accepted (2026-05-03).

## Context

Genesis (the predecessor bash script) registered an `@plane-mcp/server` and a
Mattermost MCP server in `.mcp.json`, but the agent prompts in practice fell
back to direct `curl` calls. We surveyed the original Genesis-bootstrapped
codebase that motivated this project: every `lifecycle` state transition was
implemented as a hand-rolled curl, not via the MCP tool surface.

We also observed:

- Self-hosted Plane / Mattermost users (the agent-aware flavor in
  particular) extend the wire format. Standard MCP servers built against
  the upstream schema mismatch the custom payload.
- The MCP tool-call wrapping adds tokens and an extra hop. Genasis already
  invests in a token-economics layer (RTK + prompt cache + trim hook).
- MCP-server lifetime management is a separate concern (process supervision,
  reconnection, schema drift across server versions).

## Decision

Genasis itself talks to Plane and Mattermost over **direct REST** through
`reqwest` (rustls TLS, JSON bodies). The provider trait absorbs the wire
differences via the flavor system (ADR-005). Agents continue to use `curl`
in their overlay protocol when they need ad-hoc calls.

The only MCP server we still ship is **Playwright**, because it's the
mechanism we drive Plane's web UI for agent-user provisioning (M4).

## Consequences

**Easier**:
- One adapter per flavor, in one language, in one crate.
- Test with mock HTTP servers (e.g. wiremock, httpmock) instead of having to
  start MCP processes.
- Provisioning latency drops because every call is one round-trip.

**Harder**:
- We don't get the "discoverability" benefit MCP gives free — agents can't
  list available Plane operations through their tool catalog.
- Schema upgrades (when Plane adds a field) are visible only inside the
  provider crate; we need to keep CHANGELOG entries and a flavor probe.

**Foreclosed**:
- We do not ship a `genasis-mcp-server` 1차 — see Q5 / R5 in the planning
  conversation. Future versions may add one as an *optional* adjacent crate.

## References

- Implementation: `crates/genasis-providers/src/{plane,mattermost}/`
- Blueprint: `blueprint.md` §5 (Provider Adapters & Flavor 시스템)
