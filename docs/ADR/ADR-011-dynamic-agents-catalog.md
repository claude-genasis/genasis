> 한국어: [`../ko/ADR/ADR-011-dynamic-agents-catalog.md`](../ko/ADR/ADR-011-dynamic-agents-catalog.md)

# ADR-011: Dynamic Agents Catalog — remove binary embed, runtime fetch

## Status

Accepted (2026-05-05).

## Context

The architecture through ADR-001 (marker fence) + M6 (10 overlay
templates) + M14 (bootstrap) embeds **all agent instruction files into
the binary at compile time** via `include_dir!()`.

Problems:
1. **Update lag**: modifying agent instructions requires a full genasis
   binary release. Agent content lifecycle is coupled to tool lifecycle.
2. **Cannot reflect community best-of-breed**: per `docs/famous-agents.md`
   survey, we want to curate and rapidly deploy agents from ECC /
   wshobson / VoltAgent / dl-ezo, but binary rebuild blocks this.
3. **Curation separation impossible**: a private repo (agents-pool) for
   crawl → verify → publish pipeline is needed, but templates are
   coupled to code, making PRs/CI heavyweight.
4. **Base agent files unnecessarily use .tera**: analysis shows zero Tera
   variables used — they are static text. Plain .md suffices.

## Alternatives

| Alternative | Decision | Reason |
|---|---|---|
| (a) Keep `include_dir!()` as fallback (hybrid) | Rejected | Dual management complexity. Version mismatch risk between embedded and fetched. Unnecessary binary size. |
| (b) GitHub Releases tarball (`agents-v1.x.tar.gz`) | **Accepted** | Fully independent versioning from binary. Leverages existing release infra. SHA-256 verifiable. |
| (c) npm package (`@genasis/agents`) | Rejected | Node.js dependency. Violates ADR-002 (Rust single binary). |
| (d) GitHub raw URL fetch (branch-based) | Rejected | Rate limits, no version pinning, cache invalidation difficulty. |
| (e) Keep agent base as .tera | Rejected | Zero variables used. Unnecessary rendering step adds complexity only. |
| (f) Convert overlays to plain .md too | Rejected | Overlays use 7–10 project-specific variables (`{{ project_name }}`, `{{ mm_url }}`). Tera required. |

## Decision

Adopt **(b) + (e-rejected) + (f-rejected)**:

### 1. Structural separation

| Layer | Format | Source | Distribution |
|---|---|---|---|
| Agent base files | **plain .md** | agents-pool (private) → genasis/agents/base/ | GitHub Release tarball |
| Overlay patches | **.tera** (kept) | genasis/agents/overlays/{en,ko}/ | Same tarball |
| Commands/Skills/Hooks | .tera (kept) | genasis/agents/{commands,skills,hooks}/ | Same tarball |

### 2. Distribution flow

```
agents-pool (private, submodule) → crawl → verify → publish.sh
  → genasis/agents/ copy → commit → tag agents-v1.x.0
  → CI (release-agents.yml) builds tarball → GitHub Release asset
```

### 3. CLI fetch model

- `genasis agents fetch [--version]`: explicit download
- `genasis attach` / `upgrade`: auto-check (pinned version vs cache)
- Cache: `~/.cache/genasis/agents/v{version}/`
- Pin: `genasis.toml [agents].version = "1.0.0"`

### 4. Complete `include_dir!()` removal

No template/agent content in the binary. Network unavailable + no
cache → error + "run `genasis agents fetch` first" guidance.

### 5. Default 9 roles (per famous-agents.md)

pm, architect, frontend-developer, backend-developer, code-reviewer,
qa-tester, security-reviewer, planner, designer — community
best-of-breed (ECC/wshobson/VoltAgent/dl-ezo) + genasis-authored
(pm, designer).

### 6. Communication protocol injection guarantee

Every base agent receives an overlay fence with the Plane/MM lifecycle
protocol + GENASIS.md reference link. PM and Designer are the
communication protocol hubs (issue assignment / design change alerts).

## Consequences

**Easier:**
- Agent content updates independent of binary releases. One tag
  (`agents-v1.1.0`) lets all users `genasis agents update`.
- Community best-of-breed curated and deployed rapidly.
- Binary size reduced (template embed removed).
- Agent base management simplified to plain .md.

**Harder:**
- First use requires network (`install.sh` includes `genasis agents fetch`).
- CI environments need cache warm-up (CI cache integration recommended).
- `genasis-templates` crate undergoes major refactoring (include_dir →
  HTTP client + cache).

**Foreclosed:**
- Offline-only deployment (impossible without pre-cached catalog).
- Binary-alone agent installation (catalog fetch always required).

## References

- `docs/famous-agents.md` — community agent survey (selection rationale)
- ADR-001 (marker fence) — overlay injection mechanism preserved
- ADR-002 (Rust single binary) — npm dependency rejection basis
- ADR-010 (default team bootstrap) — subsumed by this ADR
- `blueprint.ko.md` §20 → updated to §21
