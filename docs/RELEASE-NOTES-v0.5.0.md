# Release Notes — v0.5.0

> 한국어: [`ko/RELEASE-NOTES-v0.5.0.md`](ko/RELEASE-NOTES-v0.5.0.md)

**Released**: 2026-05-10

The first public binary release of Genasis. Skips through 0.1.0–0.4.x
because the codebase has matured well past the M0–M11 scope originally
planned for v0.1.0; v0.5.0 reflects the actual feature set landed
across milestones M0–M21 plus ADRs 1 through 15.

---

## What is Genasis?

Genasis turns AI agents into **first-class team members** alongside
humans, embedded in the collaboration tools your team already uses
(Plane for issues, Mattermost for chat). One command provisions a
full ten-role agentic team, wires it into your Plane workspace and
Mattermost team, and hands you the same sprint workflow you've been
running with humans — only now agents pick up tickets, run TDD, and
reply in threads alongside you.

---

## Highlights

### Bolt-on agentic team (ADR-001, ADR-010)
- `genasis init` / `genasis bootstrap` scaffolds a canonical 10-role
  team (PM, Planner, Architect, Frontend, Backend, QA, Designer,
  Security, DevOps, Code-reviewer) into `.claude/agents/` of any
  project — green-field or existing.
- Marker-fence overlay model: existing agentic teams (ECC,
  knowledge-work-plugins, claude-code-templates) are augmented
  non-destructively; remove the fence and your originals come back
  byte-identical.

### Real-team integration
- **Plane**: native REST integration with three flavors (`upstream`,
  `agent-aware`, `auto`) — automatic detection at probe time.
- **Mattermost**: bot account provisioning, channel ensuring, threaded
  replies, post-tool sync hooks.
- **Trial bridge** (ADR-013): a bundled trial-app at
  `http://localhost:3000` simulates Plane + Mattermost so you can
  evaluate Genasis without standing up either.

### Human roster (ADR-014, new)
- `[[humans]]` array in `genasis.toml` registers human stakeholders.
- `genasis humans add | edit | remove | list | sync` CRUD CLI.
- TUI wizard 7-step Humans modal with `a/e/d/s/Enter` keybindings.
- Mattermost admin-create + 24-char temp password (force change on
  first login).
- Agents recognise registered humans via `GENASIS.md § Human Roster`
  and treat their messages as **binding stakeholder requirements**;
  unregistered senders go through PM verification.

### Multi-host server stack (ADR-015, new)
- `servers/docker-compose.yml` boots Plane + Mattermost with **shared
  PostgreSQL** (role + database isolation) — saves ~400 MB RAM vs
  twin-postgres.
- `servers/init/init-databases.sh` (least-privilege role grants) and
  `servers/scripts/setup-user-env.sh` (per-operator credential
  bootstrap).
- `docs/MIGRATE-PG-CONSOLIDATION.md` — operator runbook for migrating
  from twin-postgres.

### Slash commands + hooks
- 17 slash commands (`/check-inbox`, `/sprint-start`, `/issue-link`,
  `/intake-review`, `/db-migrate`, `/design-change`, etc.) thin-pointer
  to `GENASIS.md` and per-role overlay fences.
- 6 hooks (post-tool MM sync, post-tool token-trim, pre-tool branch
  guard, pre-tool worktree guard, session-start, user-prompt-submit).

### Database channel (ADR-004)
- Read-only `genasis db query "SELECT ..."` with SQL guard rejecting
  DDL/DML/transactions.
- `genasis db migrate` auto-detects Atlas, Drizzle Kit, or DuckDB
  raw_runner.

### Design swap (ADR-009)
- `genasis design swap <slug>` hot-swaps `design-system.md` from any
  external getdesign.md catalogue or local file.
- Override log + verify + ticket emitter for redesign-driven
  per-area issue plans.

### Token economics (ADR-006)
- Auto-trim of tool results above `[token_economics] trim_threshold_kb`
  (default 32 KB).
- RTK token-counter wraps shell tool calls when installed.

### Bilingual (ADR-008)
- Every doc, agent overlay, slash command, and runtime string ships
  in English **and** Korean.
- `genasis lang switch en|ko` flips the agent context atomically.
- Mirror-drift CI gate prevents EN/KO documentation from divergence.

### Debug history feedback loop (ADR-012)
- Drift between baseline manifest and live `.claude/genasis/` files is
  silently tracked.
- `genasis debug collect` produces an anonymised patch.json; `submit`
  opens a PR against the genasis repo's `debug-history/` directory.
- The data-only-PR gate enforces contributor scope (only patches/*.patch.json).

### Monitor TUI (ADR-007)
- `genasis monitor` — ratatui dashboard with sprint status, token
  spend, agent sessions, deploy state, network, logs.

### Catalog of best-of-breed agents (ADR-011)
- `genasis agents browse | install | list | remove` against a
  versioned catalog of community agents (ECC, wshobson, VoltAgent,
  dl-ezo).
- `agents-pool` private repo curates and republishes verified
  catalog entries.

---

## Install

```bash
# Pre-built binary (recommended)
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/claude-genasis/genasis/main/install.sh | sh

# From source
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/claude-genasis/genasis/main/build.sh | sh
```

Supported platforms: **Linux x86_64**, **Linux aarch64**, **macOS
x86_64 (Intel)**, **macOS aarch64 (Apple Silicon)**. Windows users:
WSL2 with the Linux x86_64 binary.

After install:

```bash
genasis doctor          # verify environment
genasis init --trial    # try Genasis without a real Plane/MM stack
genasis init            # set up agentic team against a real Plane + MM
```

---

## Breaking changes

None — this is the first public release.

---

## Known limitations

- `genasis migrate-from-genesis` is documentation-only; live
  migration tooling lands post-v0.5 once we have real Genesis bash
  team operational data.
- `genasis agents status` invokes `reqwest::blocking` inside
  `#[tokio::main]` and panics at runtime shutdown. README does not
  list it; tracked as a v0.6 follow-up.
- `genasis debug submit`'s actual `gh pr create` invocation remains
  gated behind `--dry-run`; the canonical contract is asserted in
  E2E.
- The Plane Playwright user provisioner (`provision-plane-users.mjs`)
  is a stub — real UI automation lands in v0.6+. Until then, agent
  user accounts must be created by hand or the `--trial` flavor
  is used.
- The `nightly-e2e` real-server smoke is a **local-only pre-push
  gate** (`scripts/nightly-e2e.sh`) — not a GitHub Actions workflow.
  See [`docs/TESTING.md` § L9](TESTING.md#l9--real-server-smoke-scriptsnightly-e2esh).
- TUI wizard test coverage is currently 0% (1,200 lines uncovered).
  Headless ratatui harness planned for v0.6.

---

## Coverage and tests

- **245 tests** across 10 workspace crates, all green
- **53.97%** line coverage (CI's `coverage` job uploads to Codecov)
- The bulk of uncovered lines is the TUI wizard (intentional — see
  [`docs/TESTING.md`](TESTING.md))

---

## Upgrade path

This is v0.5.0 — there is no prior release to upgrade from. v0.5 →
v0.6 will keep the marker fence v1.0 contract; users may run
`genasis upgrade --fence-version 1.0` and expect a no-op result.

---

## Acknowledgements

See [`docs/CREDITS.md`](CREDITS.md) for the full list of upstream
projects and contributor agents. Special thanks to the maintainers of
ECC, knowledge-work-plugins, claude-code-templates, awesome-design-md,
and the wshobson / VoltAgent / dl-ezo agent collections that seed the
catalog.

The trial-app's initial 22 user stories were authored by iterations
of the [Ralph autonomous-loop pattern](https://github.com/snarktank/ralph)
([§11 of `docs/famous-agents.md`](famous-agents.md)) before the
Ralph state directory was retired in favour of a normal feature-PR
flow.

---

## Full commit list (since project inception)

The full milestone history (M0 through M21) is preserved in
[`progress.md`](../progress.md) (English) /
[`progress.ko.md`](../progress.ko.md) (Korean SSOT). 81 commits land
in v0.5.0.
