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
mkdir my-first-project && cd my-first-project
genasis init --trial
```

This creates a blank project, installs the default agent team, and opens the
**operator-hosted demo at [mmplane-trial.realstory.blog](https://mmplane-trial.realstory.blog)**
in your browser. The demo shows an animated sprint simulation: agents picking
up tickets, posting in chat, moving kanban cards. No local install — the demo
runs on the operator's server.

From the demo app, you can also **request a trial Plane + Mattermost
environment** — no server setup needed.

### Step 3 — Generate a Sample PRD

```bash
genasis example prd
```

This creates `PRD.md` — a realistic product requirements document that your
agent team can immediately start working on. The PRD describes a simple
todo-app with authentication, CRUD operations, and a responsive UI.

### Step 4 — Start Your Agentic Team

```bash
genasis init
```

The PM agent reads `PRD.md`, decomposes it into Plane tickets, assigns roles,
and the team begins working. You can watch the progress in Plane and
Mattermost.

### Step 5 — Monitor the Sprint

```bash
genasis monitor
```

Open the Ratatui TUI dashboard to see real-time progress: which agents are
working on what, token usage, ticket lifecycle, and chat activity.

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
