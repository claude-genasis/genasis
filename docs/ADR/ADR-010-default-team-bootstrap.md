# ADR-010: Default agentic team bootstrap (M14) — base + patch 2-layer

> 한국어: [`../ko/ADR/ADR-010-default-team-bootstrap.md`](../ko/ADR/ADR-010-default-team-bootstrap.md)

## Status

Proposed (2026-05-05). User-ratification gate — promotes to Accepted
once the M14 plan is signed off.

## Context

After ADR-001 (marker fence) + M2 (overlay merger) + M6 (10 patch
overlay templates), the operating premise of Genasis is:

> The user's `.claude/agents/<role>.md` files **already exist**.

`attach` injects a marker fence into a pre-existing file, `detach`
removes the fence, and `upgrade` rewrites the fence body on hash diff.
This is exactly right for users who already have a team — ECC,
knowledge-work-plugins, or hand-authored.

But for **a project with no agent team at all** — i.e. the first
target of `genasis init` — there is no scaffolding path. `cmd_init`
provisions Plane/Mattermost only; the user has to write
`.claude/agents/*.md` themselves before any overlay can attach.
Consequences observed today:

- blueprint §15 (first-release scope) implicitly assumed ECC as the
  reference user, so "agent files already exist" became unspoken.
- `tests/golden/blank/` has been a stub since M0 — `input/` and
  `expected/` are empty; the README is the only artifact.
- The README Comparison table emphasises "Non-destructive overlay"
  but has no "Bootstrap" axis, so ECC `claude-code-templates`
  differentiation is invisible.

User-flagged 2026-05-05; this ADR closes the gap.

## Alternatives

| Alternative | Decision | Reason |
|---|---|---|
| (a) `attach` auto-scaffolds when `.claude/agents/` is empty (default ON) | Rejected | Existing users running `attach` for the first time would get silent file creation. Conflicts with the spirit of ADR-001's non-destructive invariant. |
| (b) Opt-in `--bootstrap` flag (default OFF) | **Accepted** | User-protective + explicit intent + still a one-line entry for green-field teams. |
| (c) `init --bootstrap` (sub-flag of init) | Partially accepted | `init` already carries Plane/MM provisioning weight; risks confusion as the only entry. |
| (d) Standalone `genasis bootstrap` subcommand | **Accepted** | Clear entry point. `init --bootstrap` retained as alias. |
| (e) Vendor ECC `claude-code-templates` content | Rejected | License obligation + maintenance overhead. Base templates stay deliberately thin (5–10 lines + frontmatter); patch overlay fills the protocol body. |
| (f) Sidecar `.claude/agents/<role>.genasis.md` | Rejected | Claude Code sub-agents do not read sibling files (see ADR-001 §Alternatives). |

## Decision

Adopt **(b) + (d) + (e-rejected)**:

1. **Default OFF.** `genasis attach` does not silently scaffold when
   `.claude/agents/` is empty. Instead it emits a stderr hint:
   > no agents detected — run `genasis bootstrap` (or `genasis init
   > --bootstrap`) to scaffold the default team.

2. **Two-layer structure:**

   | Layer | Location | Owner | Update trigger |
   |---|---|---|---|
   | **Base** | Outside the marker fence in `.claude/agents/<role>.md` (frontmatter + 5–10 lines of role header) | User | One-shot emit during `bootstrap`; user-editable thereafter |
   | **Patch** | Inside the same file's marker fence (Plane/MM protocol body) | Genasis | Rewritten by `attach` / `upgrade` via hash diff |

3. **Entry points:** new subcommand `genasis bootstrap [--lang en|ko]
   [--roles <list>]`, plus `genasis init --bootstrap` as an alias.
   `bootstrap` chains into `attach` automatically so the fence is
   injected in the same step (`--no-attach-after` to separate them).

4. **Base template contract:**
   `templates/{en,ko}/agents/<role>.md.tera` carries only the 5-key
   frontmatter (`name / description / tools / model / color`) and a
   5–10-line role header. No ECC content is vendored — the protocol
   meat lives in the patch overlay rendered into the marker fence.

5. **Role set:** the same 10 roles as `Role::ALL` from M2 — pm,
   planner, architect, frontend, backend, qa, designer, security,
   devops, code-reviewer.

6. **i18n:** parallel `templates/en/agents/` and
   `templates/ko/agents/` trees. The existing `lang switch` machinery
   for `templates/<lang>/` already covers the new directory. User
   edits outside the fence are preserved across `lang switch` —
   identical to the existing fence-internal-only update policy.

## Consequences

**Easier:**
- One-line scaffold of an ECC-compatible team in a green-field
  project (`genasis bootstrap`).
- `tests/golden/blank/` finally has a meaningful round-trip
  (bootstrap → attach → detach).
- README Comparison table can add a "Bootstrap" axis to make the ECC
  `claude-code-templates` differentiation visible.
- ADR-001's marker-fence invariant is preserved verbatim — bootstrap
  is just a thin "drop a base file when the file itself is missing"
  stage in front of the existing flow.

**Harder:**
- 20 base templates (10 roles × 2 langs) need a frontmatter
  consistency unit test (M14.1).
- Partial-scaffold semantics need a clear definition: missing roles
  get `Create`, present roles get `Skip("exists")`. `--overwrite` is
  intentionally absent — users who really want to start over are
  expected to `detach` first, then `bootstrap`.

**Foreclosed:**
- Auto-scaffold (default ON) — incompatible with protecting existing
  users.
- Vendoring ECC content — license + maintenance trap.

## References

- Implementation: `crates/genasis-overlay/src/bootstrap.rs` (M14.2)
- Templates: `crates/genasis-templates/templates/{en,ko}/agents/` (M14.1)
- CLI: `crates/genasis-cli/src/cmd_bootstrap.rs` or
  `cmd_init.rs --bootstrap` (M14.3)
- Golden fixture: `tests/golden/blank/` (M14.4)
- Blueprint: `blueprint.ko.md` §20
- Progress: `progress.ko.md` §M14
- Predecessor ADRs: ADR-001 (marker fence — invariant preserved),
  ADR-008 (i18n install-time selector — `--lang` precedence reused)
