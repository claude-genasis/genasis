> English: [../../ADR/ADR-001-overlay-marker-fence.md](../../ADR/ADR-001-overlay-marker-fence.md) (English version pending — currently a stub)

# ADR-001: Overlay = HTML-comment Marker Fence

## Status

Accepted (2026-05-03).

## Context

Genasis must add a Plane / Mattermost / TDD / Design / DB protocol contract to
arbitrary existing agentic teams (`.claude/agents/*.md`, possibly authored by
ECC, knowledge-work-plugins, claude-code-templates, or hand-rolled).

The constraint is non-negotiable:

- The user's existing agent prompts are tribal knowledge. We **cannot rewrite
  them**. Doing so would block adoption: "Genasis ate my custom planner.md".
- We must still inject a small protocol block per agent, because Claude Code
  sub-agents do not inherit `CLAUDE.md` automatically — anything the agent
  must know has to live inside its own `.md`.
- Whatever we inject must be **reversible** — `genasis detach` returns the
  files to byte-identical state.
- It must also be **idempotent** — `genasis attach` running twice yields the
  same result, regardless of intervening edits to the rest of the file.
- We must be able to detect and refuse to overwrite a fence whose body the
  user has hand-edited.

## Alternatives

| Alternative | Why rejected |
|---|---|
| Replace the agent `.md` entirely with a Genasis-managed file | Destructive; loses user prompt. |
| Add a *separate* agent (e.g. `genasis-protocol.md`) that everyone defers to | Sub-agents don't read sibling agent files; no enforcement. |
| Append protocol to a sidecar `<agent>.genasis.md` and import via `@` | Claude Code does not currently support `@` import inside sub-agent prompts. |
| Live-patch the agent `.md` only at runtime via hooks | Hooks fire only on the main session, not sub-agents. |
| Use YAML frontmatter `description:` field appendix | Frontmatter is small, single-line semantics; can't carry the full protocol. |

## Decision

Inject a **marker-fenced HTML-comment block** with the exact form:

```markdown
<!-- GENASIS:BEGIN role=<slug> version=<X.Y> hash=<sha256[:8]> -->
... protocol body ...
<!-- GENASIS:END -->
```

- The fence is placed **immediately after the YAML frontmatter terminator**
  (`---\n`) when present, otherwise at byte 0.
- A file may contain **at most one** fence; the merger refuses to inject a
  second.
- The `hash` records the SHA-256 of the rendered body, hex-truncated to 8
  characters.
- HTML comments are universal in markdown ecosystems and are not visible to
  the rendered prompt context, but Claude Code sees the raw `.md` text.

## Consequences

**Easier**:
- `attach` and `detach` are pure text rewrites — fast, deterministic, easy to
  test (golden fixtures).
- Diff review on a PR clearly delineates Genasis-owned vs human-owned regions.
- Future protocol changes ship as fence version bumps; users opt in via
  `genasis upgrade`.

**Harder**:
- Two competing tools that both want to inject blocks must agree on a
  separate fence syntax. (We pick `GENASIS:` as a unique sentinel.)
- Hash truncation to 8 chars accepts a small collision probability — fine for
  tamper detection, not a security boundary. Documented explicitly in
  `marker.rs`.

**Foreclosed**:
- We do *not* attempt to "merge" a Genasis fence with a hand-edited body.
  Detected hash mismatch causes the upgrade to skip that file with a warning.
  Users must either accept the new fence (overwrite their edits) or stay on
  the previous version.

## References

- Implementation: `crates/genasis-core/src/marker.rs`
- Tests: `crates/genasis-core/tests/marker_idempotent.rs`
- Blueprint: `blueprint.md` §3 (Marker Fence Spec)
