# ADR-019 — Real Plane + Mattermost Provisioning (`genasis provision`)

> 한국어: [docs/ko/ADR/ADR-019-real-provisioning.md](../ko/ADR/ADR-019-real-provisioning.md)

Status: **Proposed (alpha.26 scaffold landed; REST adapters incoming)**
Date: 2026-05-15

## Context

Genasis's trial path (ADR-017) lets users spin up a fake Plane + Mattermost
environment in seconds — perfect for the "let me see what this looks like"
moment but not where real work happens. Once a team decides to actually
collaborate with agents, they need real Plane projects, real Mattermost
channels, and real per-agent service accounts wired up to the same
`genasis listen` daemon.

The previous answer was a Python `provision.py` script invoked out-of-band.
That fragments the tooling story (one binary for trial, a separate script
for real), keeps the user installing Python plus a `requests` shim, and
leaks credentials handling outside Rust's type system.

ADR-019 folds the entire workflow into the existing `genasis` binary as
a first-class subcommand pair: **`genasis provision`** for the initial
setup, **`genasis team add | remove | list`** for ongoing membership
churn.

## Decision

### 1. Single-binary `genasis provision`

```
genasis provision
  [--team "Marketing Squad"]
  [--app "Quiz Demo"]
  [--humans "Bravo Kim <gnoopy@gmail.com>,Alice <alice@x.com>"]
  [--humans-file ./humans.json]
  [--agents pm,frontend,backend,devops,designer,qa,planner,architect,code-reviewer,security]
  [--output ./]
  [--non-interactive]
  [--dry-run]
```

Required environment variables:

```
PLANE_URL          # https://plane.realstory.blog OR http://localhost:8080
PLANE_ADMIN_TOKEN  # admin API token issued from the Plane admin UI
MM_URL             # https://mm.realstory.blog    OR http://localhost:8065
MM_ADMIN_TOKEN     # Mattermost system-admin PAT
```

Two execution modes:

- **Interactive (default when flags are missing)** — stdin prompts walk
  the user through team name, app name, human members (one at a time
  with `name?`/`email?`), and a confirmation review. CI and headless
  installs skip prompts with `--non-interactive`; missing required
  values then turn into errors instead of hanging waiting for stdin.

- **Fully scripted** — every input via flag or `--humans-file`.

### 2. Slug abbreviation rule

Team and app names are abbreviated to **5-character lowercase slugs**
via `genasis_core::slug::slugify_abbrev`:

| Input                         | Slug    | Reason                       |
|-------------------------------|---------|------------------------------|
| `Marketing Squad`             | `ms`    | 2 words → initials           |
| `Marketing Communications`    | `mc`    | 2 words → initials           |
| `Quiz Demo`                   | `qd`    | 2 words → initials           |
| `Quiz`                        | `quiz`  | 1 word → first 5 chars       |
| `Pomodoro`                    | `pomod` | 1 word → first 5 chars       |
| `팀협업` (Hangul)              | `tc` (translated→"team collab" → `tc`) — or `the` via deunicode if `claude` is not on PATH |

Hangul input is first sent through the user's local `claude -p` CLI
asking for a short English phrase (1-3 words), then abbreviated.
`claude` invocation has a 30-second timeout; on failure we fall back
to `deunicode` phonetic transliteration.

### 3. Identifier patterns

| Resource                       | Pattern                             | Example                          |
|--------------------------------|-------------------------------------|----------------------------------|
| Plane workspace (preferred)    | `<team-slug>`                       | `ms`                             |
| Plane workspace (fallback)     | `agentic`                           | shared, if per-team unsupported  |
| Plane project name             | `<app-name>` _or_ `<team>-<app>`    | `Quiz Demo`                      |
| Plane project identifier       | `<APP-SLUG>` (uppercase)            | `QUIZ`                           |
| Mattermost team                | `team-<team-slug>`                  | `team-ms`                        |
| Mattermost scrum channel       | `scrum-<app-slug>`                  | `scrum-quiz`                     |
| Agent user (Plane + MM)        | `<role>-<team-slug>@genasis.bot`    | `pm-ms@genasis.bot`              |
| Human username (suggested)     | local-part of email                 | `gnoopy@gmail.com` → `gnoopy`    |

### 4. Output files

Two files written to the working directory (or `--output`):

**`genasis.toml`** — identifiers and URLs only, safe to commit if the
project repo is private:

```toml
[provision]
provisioned_at = "2026-05-15T22:35:00+09:00"
team_slug = "ms"
app_slug = "quiz"

[plane]
flavor = "real"
url = "https://plane.realstory.blog"
workspace_slug = "ms"
project_id = "01HABC..."
project_identifier = "QUIZ"

[mattermost]
flavor = "real"
url = "https://mm.realstory.blog"
team_id = "z4t5..."
scrum_channel_id = "ch-..."
scrum_channel_name = "scrum-quiz"

[[humans]]
name = "Bravo Kim"
email = "gnoopy@gmail.com"
username = "gnoopy"
plane_user_id = "..."
mm_user_id = "..."

[[agents]]
role = "pm"
email = "pm-ms@genasis.bot"
plane_user_id = "..."
mm_user_id = "..."
```

**`.env.local`** — credentials, chmod 600, gitignored. Flat KEY=VALUE
shape (no JSON blobs) per user preference:

```env
PLANE_URL=https://plane.realstory.blog
PLANE_WORKSPACE_SLUG=ms
PLANE_PROJECT_ID=01HABC...

PLANE_AGENT_TOKEN_PM=plk_...
PLANE_AGENT_TOKEN_FRONTEND=plk_...
PLANE_AGENT_USERID_PM=...
PLANE_AGENT_USERID_FRONTEND=...
# ... one pair per agent, plus humans

MM_URL=https://mm.realstory.blog
MM_TEAM_ID=z4t5...
MM_SCRUM_CHANNEL_ID=ch-...

MM_AGENT_PAT_PM=mm_pat_...
MM_AGENT_USERID_PM=...
# ... etc

HUMAN_PLANE_USERID_GNOOPY=...
HUMAN_MM_USERID_GNOOPY=...
```

The daemon's existing D-098 env-passthrough infrastructure already
forwards these to MCP servers and to the orchestrator claude.

### 5. Plane workspace strategy

Two paths:

1. **Per-team workspace** (preferred). `genasis provision` first tries
   `POST /api/v1/workspaces/` with the team slug. If the admin token
   has that permission this succeeds; the team gets full isolation
   from every other Genasis team on the same Plane instance.

2. **Shared `agentic` workspace + naming convention** (fallback). If
   workspace creation 403s, we land the project inside a pre-existing
   `agentic` workspace with the project name set to `<team>-<app>`
   (e.g. `ms-quiz`). The user is told which path was taken in the
   final summary.

### 6. Post-provision: `genasis team`

`genasis provision` only runs once. For day-2 changes:

```
genasis team add human "Charlie <charlie@x.com>"   # invite a new human
genasis team add agent designer                     # hire an additional agent
genasis team add agent custom-role                  # hire a brand-new role
genasis team remove human alice@x.com               # deactivate a human
genasis team remove agent designer                  # retire an agent
genasis team list                                   # show roster + health
```

All operations are idempotent: re-adding an existing member is a no-op
with an informational message, and removing a missing one is also a
no-op. The Plane and Mattermost backends keep deactivated accounts'
history intact (issues created, posts authored remain visible).

### 7. Failure handling

No automatic rollback. The flow is structured so each REST call is
either idempotent (GET-before-create) or safely re-runnable (PATCH
that converges on the target state). On partial failure the user
re-runs the same command; already-created resources are detected and
skipped, work resumes from the next pending step.

## Consequences

- **Single Rust binary owns the entire team lifecycle**: trial,
  real provisioning, ongoing membership. No Python or shell scripts
  to install separately, no language drift between operator-hosted
  and self-host paths.

- **`.env.local` is the credential boundary.** Anything in
  `genasis.toml` is shareable (identifiers, URLs, role names);
  anything in `.env.local` is per-agent secrets that must never
  leave the user's machine. The daemon explicitly loads both files
  at startup and forwards only the env vars it needs.

- **Slug collisions exist but are bounded.** Two teams both named
  "Marketing Squad" would both slug to `ms`; the per-team workspace
  path stops one of them with a 409, the fallback path stops the
  second's project name from creating with a 409. The user is asked
  to pick a different team name. We don't auto-disambiguate (`ms2`,
  `ms3`) because the resulting identifier would be opaque.

- **Hangul translation depends on the user's `claude` CLI.** Same
  dependency as `genasis listen`, so we're not introducing a new
  installation requirement. If `claude` is unavailable the
  transliteration fallback preserves uniqueness but loses meaning
  — users with non-Latin team names who care about the slug being
  readable should make sure their `claude` is installed before
  running `provision`.

- **The trial path stays unchanged.** `genasis init --trial` and
  `genasis listen --trial` continue to work against
  `mmplane-trial.realstory.blog` exactly as before. ADR-017 and
  ADR-019 are complementary, not competing.

## Implementation status

- **alpha.26** — `genasis-core::slug` (8 tests),
  `genasis-cli::cmd_provision` scaffold (dry-run preview).
- **alpha.27** — `cmd_provision` interactive prompts +
  `--humans-file` JSON + `cmd_team` scaffold + ADR + README.
- **alpha.28** — `plane::real_provisioner` REST adapter; LIVE
  smoke against plane.realstory.blog (whoami → ensure_project →
  member attach → invitation, all idempotent).
- **alpha.29** — `mattermost::real_provisioner` REST adapter; LIVE
  smoke against mm.realstory.blog (whoami → team → channel → user
  → membership → PAT, all idempotent).
- **alpha.30** — `provision_writer` (`.env.local` + snapshot toml +
  provision.log) + `GENASIS_SECRETS_ROOT` redirect for the
  operator's agents-pool/secrets/ tree (6 tests).
- **alpha.31** — `cmd_provision` live flow orchestrating PR-1/2/3;
  end-to-end LIVE PASS on plane+mm with team `gpt` + 2 agents +
  1 human (Created → Reused on re-run).
- **alpha.32** — `cmd_team` body (list / add human / add agent /
  remove human / remove agent) + snapshot loader; LIVE PASS for
  list, add agent designer, remove agent designer, add human.
  Operator cheat sheet added to README.

All ADR-019 PRs complete. Next on the roadmap is per-tenant token
rotation (`genasis team rotate <slug>`) and self-host
docker-compose smoke against a fresh stack.
