<div align="center">

# Genasis

**AI Agent Orchestration for Real Team Collaboration**

Bolt a full agentic development team onto the collaboration tools your humans already use — Plane for tickets, Mattermost for chat, sprints + threads + reviews exactly as before.

[![CI](https://img.shields.io/github/actions/workflow/status/claude-genasis/genasis/ci.yml?branch=main&label=CI&style=flat-square&logo=github)](https://github.com/claude-genasis/genasis/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/claude-genasis/genasis?include_prereleases&style=flat-square&logo=github&label=release)](https://github.com/claude-genasis/genasis/releases)
[![License](https://img.shields.io/github/license/claude-genasis/genasis?style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20WSL-blue?style=flat-square)](#supported-platforms)

[**English**](README.md)&nbsp;&nbsp;|&nbsp;&nbsp;[**한국어**](README.ko.md)

</div>

---

<p align="center">
  <img src="docs/assets/genasis-banner-en.png" alt="Genasis — AI-Powered Agentic Team" width="100%">
</p>

## What you get

`genasis` ships a 10-role agentic team (PM, frontend, backend, devops,
designer, QA, planner, architect, code-reviewer, security) that
collaborates with humans through **Plane tickets** and **Mattermost
chat**. A human types into the same channel they always used; Claude
picks up the message as the PM role, dispatches `frontend → devops →
QA`, transitions cards, posts replies in-thread, and ships the result
to a hosted preview.

Two ways to use it:

| Flavor | What it is | Best for |
|--------|------------|----------|
| **Trial** | Browser-only demo. No infrastructure. Chat + kanban + showcase iframe at `mmplane-trial.realstory.blog`. | First contact, demos, learning the workflow. |
| **Real** | Your team's actual Plane workspace + Mattermost server. Agents become first-class members. | Production work. |

Both share the same daemon (`genasis listen`) and the same agent
overlays — only the API URLs differ.

## Quick Path — 5 minutes to a chatting team

```bash
# 1. Install
curl -fsSL https://raw.githubusercontent.com/claude-genasis/genasis/main/install.sh | sh

# 2. Bootstrap a trial team
mkdir my-team && cd my-team
genasis init --trial --name "My Team"
```

That single `init --trial` does four things behind the scenes:
trial-team bootstrap → showcase activation → reactive daemon start →
prints a live URL with your team token already in the query string.

3. **Open the URL** the command printed. The Live Trial tab shows
   your team's kanban + chat + showcase panel.
4. **Type a request** in the chat panel — Korean works fine:

   > 다크모드와 i18n 지원되는 퀴즈 앱 만들어줘

   Within a minute PM acknowledges, frontend scaffolds a Vite + React
   + TS project under your local sandbox, devops runs `npm run build
   && genasis push` to upload the static bundle to the operator's
   trial-app, and the "결과보기" handle on the left flips from
   `준비중` to a live iframe of the built app.
5. **Stop when done**: `genasis stop`.

That's it. The flow is intentionally one main command + one CLI stop.

## Going to production — real Plane + Mattermost

When you outgrow trial, point genasis at your actual stack. You'll
need admin tokens from both:

```bash
export PLANE_URL=https://plane.your-company.com
export PLANE_ADMIN_TOKEN=...        # generate from Plane admin UI
export MM_URL=https://mm.your-company.com
export MM_ADMIN_TOKEN=...           # Mattermost system-admin PAT

genasis provision \
  --team "Marketing Squad" \
  --app "Quiz Demo" \
  --humans "Bravo Kim <gnoopy@gmail.com>"
```

What this does (idempotent — re-run safely):

1. Creates the Plane project, attaches the 10 default agents, invites
   the listed humans.
2. Creates the Mattermost team (`team-<slug>`) and scrum channel
   (`scrum-<slug>`), provisions an MM user + PAT per agent, attaches
   everyone.
3. Writes `genasis.toml` (identifiers) + `.env.local` (per-agent
   tokens, `chmod 600`) into your project root.

Day-2 membership churn doesn't need re-provisioning:

```bash
genasis team list                                   # current roster
genasis team add human "Charlie <charlie@x.com>"
genasis team add agent designer
genasis team remove agent designer                  # history preserved
```

**Slug auto-shortens** to 5 chars (`Marketing Squad` → `ms`); override
with `--team-slug` / `--app-slug` if it collides. Korean team names
get translated via your local `claude` CLI before abbreviation.

See [ADR-019](docs/ADR/ADR-019-real-provisioning.md) for the full
specification.

## For operators (running genasis-as-a-service)

If you're hosting genasis for multiple tenants, keep their secrets
versioned in a private repo:

```bash
export GENASIS_SECRETS_ROOT=/path/to/agents-pool/secrets
genasis provision --team "Tenant Co" --app "Their App" --humans "..."
# writes secrets/teams/<slug>/{genasis.toml.snapshot,.env.local,provision.log}
git -C agents-pool add secrets/ && git commit && git push
```

Snapshot ownership check (alpha.34+) prevents two tenants from
silently inheriting each other's resources if their auto-generated
slugs collide — re-runs from the snapshot dir match by ID and Reuse;
fresh runs into the same slug get an explicit "another tenant owns
this identifier" error.

## CLI reference

```
# Team lifecycle
genasis init --trial --name "X"   # one-command Quick Path
genasis init                       # real Plane/MM (run after `provision`)
genasis provision ...              # real-flavor bootstrap (admin tokens)
genasis team {add|remove|list}     # day-2 membership

# Runtime
genasis status                     # daemon + URL + recent activity
genasis stop                       # stop daemon
genasis logs -f                    # follow daemon log
genasis monitor                    # full TUI dashboard

# Agent catalog
genasis agents                     # browse + install agents from catalog
genasis humans {add|sync|list}     # human collaborators

# Showcase (alpha.39+)
genasis push                       # build static bundle, ship to operator

# Power-user / advanced
genasis listen {start|stop|status|logs|restart}   # daemon long form
genasis publish                    # manual app_status flip
genasis doctor                     # environment check
genasis debug                      # drift / debug-history tooling
```

## How agents collaborate with humans

- **Same channels, same boards.** Agents post in Mattermost threads
  under the human's message and assign themselves Plane tickets like
  any other team member. A human reviewing yesterday's standup can't
  (and shouldn't need to) tell which updates came from agents.
- **PM agent orchestrates.** When a human posts a request, the PM
  picks it up, breaks it into tickets, and dispatches `frontend` or
  `devops` via Claude's `Task` tool. Each sub-agent reports back in
  the same thread.
- **Bring-your-own-agents.** If you already have `.claude/agents/`
  from another framework (ECC, knowledge-work-plugins, custom), the
  overlay attaches via marker-fence patches — your agent definitions
  stay intact.

## Troubleshooting

### "결과보기" handle stuck on "준비중"

`genasis publish` didn't flip `app_status` to `complete`. The
Quick Path runs this automatically inside `init --trial`; if it
failed, run it manually:

```bash
cd <project-dir>
genasis publish
```

### Chat / kanban not updating in real time

alpha.38+ has SSE auto-reconnect on both panels — older builds need
a manual page reload after an operator restart. Verify with:

```bash
~/.local/bin/genasis --version       # alpha.38+ expected
genasis listen restart --trial       # if daemon is from an older binary
```

The daemon also runs a 10-second fallback poll, so a single dropped
SSE event won't leave the board stale.

### Showcase iframe shows "빌드 중" but daemon log is silent

The daemon is **reactive** — it only spawns a Claude session when a
human posts in the chat. The trial seed message ("환경 준비 완료")
is from `publish`, not an actual build. Type a real request in the
chat panel:

> 다크모드 + i18n 지원되는 퀴즈 앱 만들어줘

PM should ack within 30 seconds.

### Plane / Mattermost slug collision (multi-tenant hosting)

```
Error: Plane project identifier "QUIZ" already exists (id=...).
Pick a different `--app-slug`.
```

Two teams hit the same auto-abbreviated slug. Re-run with
`--team-slug` or `--app-slug` explicitly. See ADR-019 §Slug
collision.

## Supported platforms

Linux x86_64, Linux aarch64, macOS (Intel + Apple Silicon), Windows
via WSL2. Static musl binaries on Linux; `cargo install --git`
elsewhere.

## Architecture

Brief: Rust CLI + JSONL hook bus + Plane/MM adapters + reactive
listen daemon that spawns `claude --print` per human message. See
[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the full diagram +
data flow.

## Documentation

- [TUTORIAL](docs/TUTORIAL.md) — hands-on walkthroughs
- [ARCHITECTURE](docs/ARCHITECTURE.md) — components + data flow
- [PROVIDERS](docs/PROVIDERS.md) — Plane / Mattermost adapter contracts
- [ADRs](docs/ADR/) — design decisions ([019 real provisioning](docs/ADR/ADR-019-real-provisioning.md), [020 showcase push](docs/ADR/ADR-020-showcase-push.md))
- [CONTRIBUTING](CONTRIBUTING.md)

## License

MIT — see [LICENSE](LICENSE).
