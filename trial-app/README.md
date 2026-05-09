# Genasis Trial App

Next.js 15 web app that serves both an interactive demo and a trial signup
flow, plus a lightweight Plane / Mattermost simulator that genasis agents can
talk to via the trial bridge.

Three top-level tabs:

| Tab | Purpose |
|---|---|
| 체험하기 | Scripted 8-step sprint demo (PM → Frontend → Code-reviewer → QA). |
| 라이브 트라이얼 | Live kanban + chat backed by `/api/plane/*` and `/api/mattermost/*` simulator endpoints; SSE drives real-time updates from agent calls and human co-work. |
| 신청하기 | Trial environment signup form (POST → `/api/submit` → Mattermost bot notification). |

## Routes

| Route | Purpose |
|---|---|
| `/` | Landing with the three tabs (`?tab=demo|live|signup`). |
| `/status/[token]` | Pending or provisioned status page with credentials + `genasis.toml` snippet. |
| `POST /api/submit` | Signup payload → SQLite + Mattermost notification. |
| `POST /api/webhook` | Admin credential delivery (auth: `X-Genasis-Webhook-Secret`). |
| `POST /api/plane/projects` | Trial bridge — ensure project. |
| `POST /api/plane/issues` | Trial bridge — create issue. |
| `PATCH /api/plane/issues/[id]` | Trial bridge — transition state / assignee. |
| `GET /api/plane/issues?project_slug=…` | Trial bridge — list issues. |
| `POST /api/mattermost/channels` | Trial bridge — ensure channel. |
| `POST /api/mattermost/posts` | Trial bridge — create root or thread post. |
| `GET /api/mattermost/posts?channel_id=… ` | Trial bridge — list posts. |
| `GET /api/events/stream` | Server-Sent Events of every sim mutation. |

The `/api/plane/*` and `/api/mattermost/*` endpoints accept either:

- `X-Genasis-Trial-Secret: $TRIAL_SHARED_SECRET` (server-to-server callers like
  the genasis Rust providers), or
- `Sec-Fetch-Site: same-origin` (the trial-app's own browser UI — set
  automatically by browsers).

## Environment variables

| Var | Required | Description |
|---|---|---|
| `DATABASE_PATH` | no | SQLite file path. Defaults to `./data/trial.db` (Docker: `/data/trial.db`). |
| `MM_BOT_TOKEN` | no | Mattermost bot token used by `/api/submit` to post applicant notifications. If unset, notifications are silently skipped. |
| `MM_TRIAL_CHANNEL_ID` | no | Channel ID where trial-request notifications are posted. |
| `MM_BASE_URL` | no | Mattermost base URL. Defaults to `https://mm.realstory.blog`. |
| `WEBHOOK_SHARED_SECRET` | for `/api/webhook` | Secret expected in the `X-Genasis-Webhook-Secret` header on admin credential POSTs. |
| `TRIAL_SHARED_SECRET` | for trial bridge | Secret expected in `X-Genasis-Trial-Secret` header on `/api/plane/*` and `/api/mattermost/*` requests from server-to-server callers. Browser UI bypasses this via the same-origin check. |

## Genasis-side configuration

When you run `genasis init --trial` the generated `genasis.toml` is already
wired for this app. Manually wiring an existing project requires only this
section (the per-provider `url` fields are placeholders — `[trial]` is the
single source of truth for routing; see ADR-013):

```toml
[plane]
url = "http://localhost:3000"   # ignored when flavor = "trial"
workspace_slug = "trial"
flavor = "trial"

[mattermost]
url = "http://localhost:3000"   # ignored when flavor = "trial"
team_name = "trial"
flavor = "trial"

[trial]
enabled = true
url = "http://localhost:3000"   # actual destination
shared_secret = ""              # must match TRIAL_SHARED_SECRET on this app
```

Setting `[trial].enabled = false` or removing the section while a flavor is
still `"trial"` is rejected at config load time with a clear error — no
silent routing surprises. Trial mode does NOT consult `MM_ADMIN_TOKEN` or
`PLANE_API_KEY`; only `[trial].shared_secret` is sent.

## Local development

```bash
npm install
npm run dev          # http://localhost:3000
npm run typecheck    # tsc --noEmit
npm run build        # next build (standalone output)
```

## Docker

```bash
docker build -t genasis-trial-app:latest .
docker compose up -d
# →  http://localhost:3000
```

The compose file mounts `./data` as `/data` inside the container so the SQLite
file persists across restarts.

## Layout

```
app/
  api/
    submit/         POST → save submission + MM notify
    webhook/        POST → admin credentials
    plane/          Trial bridge — projects, issues
    mattermost/     Trial bridge — channels, posts
    events/stream   SSE
  status/[token]/   Pending / provisioned status page
  components/       AppBar / DemoBoard / SignupForm / Live*Board / CredentialsView …
  page.tsx          Landing (tabs)
  layout.tsx
db/
  index.ts          submissions schema + helpers
  sim.ts            sim_projects / sim_issues / sim_channels / sim_posts
lib/
  events.ts         in-process SSE event bus (subscribe / emit)
  trial-auth.ts     bridge auth (secret OR same-origin)
  demo-script.ts    scripted demo timeline (US-006)
  use-demo-sprint.ts hook driving the scripted demo
  genasis-toml.ts   credential → genasis.toml snippet
  token.ts          24-byte URL-safe token generator
```
