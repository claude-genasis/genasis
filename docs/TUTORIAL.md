> 한국어: [ko/TUTORIAL.md](ko/TUTORIAL.md)

# Genasis Tutorial

## Quick Path — Experience Genasis in 5 Steps

Complete Steps 1-5 and you'll have a fully operational agentic team running
a real sprint. This is the fastest way to understand what genasis does.

### Step 1 — Install

```bash
curl -fsSL https://raw.githubusercontent.com/claude-genasis/genasis/main/install.sh | sh
```

### Step 2 — Initialize with Trial Mode

```bash
mkdir marketing-squad && cd marketing-squad
genasis init --trial --name "Marketing Squad"
```

This creates a blank project and writes a `genasis.toml` whose `[plane]`,
`[mattermost]`, and `[trial]` sections all carry **your** team name —
`workspace_slug = "marketing-squad"`, `team_name = "marketing-squad"`,
a `scrum-marketing-squad` channel under `[[mattermost.channels]]`, and a
freshly generated `team_token` under `[trial]`. Omit `--name` and the CLI
either prompts (interactive) or humanises the directory name
(`marketing-squad` → "Marketing Squad").

The command then opens the **operator-hosted demo at
[mmplane-trial.realstory.blog](https://mmplane-trial.realstory.blog)**
straight into *your* team's per-token URL —
`/?tab=live&team=<token>` — so the kanban header and chat sidebar show
"Marketing Squad" instead of a generic shared sandbox. The `team_token`
isolates your sim rows from any other concurrent demos on the hosted
instance (ADR-016).

From the trial app's **Borrow real env** tab you can also request a
real Plane + Mattermost project on the operator's shared infrastructure
— no server setup needed (ADR-017).

### Step 3 — Generate a Sample PRD

```bash
genasis example prd
```

This creates `PRD.md` — a realistic product requirements document that
your agent team can immediately start working on. The reference PRD
describes **"I Am a Claude Code Expert"** — a mobile-phone-bordered
quiz app that grades the user's Claude Code knowledge as Beginner /
Intermediate / Advanced (ADR-017).

The file is locale-aware: a project initialised in Korean (`[i18n].active
= "ko"`) gets a Korean PRD describing "나는 Claude Code 전문가"; English
projects get the English version. Override with `genasis example prd
--lang en|ko`.

### Step 4 — Start Your Agentic Team

```bash
genasis init
```

The PM agent reads `PRD.md`, decomposes it into Plane tickets, assigns
roles, and the team begins working. You can watch the progress in
Plane and Mattermost — or, for trial-mode projects, in the live trial
app at the URL `genasis init --trial` printed.

### Step 5 — Reveal the Showcase

When agents finish the build:

```bash
genasis trial publish
```

This flips your team's `app_status` to `complete` on the trial-app.
Reload the live trial URL and the **"See the app the agents built"**
button on the LiveBoard becomes active — clicking it slides a panel in
from the left with the embedded quiz so you can play through the same
deliverable the agents just shipped (ADR-017 §3-§4).

### Step 6 — Monitor the Sprint

```bash
genasis monitor
```

Open the Ratatui TUI dashboard to see real-time progress: which agents
are working on what, token usage, ticket lifecycle, and chat activity.

---

**You've mastered the basics.** Your agentic team just ran a sprint from PRD
to working code. Below are optional exercises to explore further.

---

## Going Further

These exercises build on the project you created above. Each one demonstrates
a different genasis capability.

### Exercise 6 — Expand with PRD2

```bash
genasis example prd2
```

Generates `PRD2.md` with additional features: user login, admin backoffice,
and user management. The agents read PRD2, create new tickets, and extend
the existing codebase. This demonstrates incremental development with an
agentic team.

### Exercise 7 — Swap the Design System

```bash
genasis example design
genasis design swap --from docs/design-system.md
```

Generates a new `docs/design-system.md` with different design tokens
(colors, typography, spacing). The swap triggers Plane issues for every UI
area that needs updating. Frontend agents pick up these issues automatically.

### Exercise 8 — Add Specialized Agents

```bash
genasis agents browse
```

Browse the agent catalog and add specialists:
- `seo-specialist` — audits pages for SEO, generates meta tags
- `sre-engineer` — sets up monitoring, health checks
- `ios-expert` — adds mobile app support

### Exercise 9 — Start a Brand New Project

```bash
mkdir another-project && cd another-project
genasis init --bootstrap
```

This time without `--trial` — connect to your own Plane and Mattermost
(self-hosted or trial server).

### Exercise 10 — Attach to an Existing Project

```bash
cd /path/to/existing-project
genasis attach
```

genasis detects your existing `.claude/agents/*.md` files and non-destructively
overlays Plane/Mattermost integration. Your agent definitions stay untouched
outside the marker fences.
