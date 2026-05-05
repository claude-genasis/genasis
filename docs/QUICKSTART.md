# Quickstart — Full Walkthrough

> 한국어: [ko/QUICKSTART.md](ko/QUICKSTART.md)

This guide walks you through installing genasis, setting up a team,
and running your first sprint with agents — from zero to agents
collaborating with humans on Plane and Mattermost.

## Prerequisites

- **Claude Code** installed and authenticated
- **Plane** instance (self-hosted or [trial](https://trial.realstory.blog))
- **Mattermost** instance (self-hosted or [trial](https://trial.realstory.blog))
- (Optional) Rust toolchain if building from source

## Step 1: Install genasis

```bash
curl -fsSL https://raw.githubusercontent.com/claude-genasis/genasis/main/install.sh | sh
```

The installer:
- Downloads the latest genasis binary
- Fetches the agents catalog (492 curated agents)
- Asks for your preferred language (English/Korean)

## Step 2: Browse and install agents

```bash
genasis agents browse
```

The interactive TUI shows categories. Select the agents you need.
Or use a preset:

```bash
genasis agents install --preset web-app
# Installs: pm, architect, frontend-developer, backend-developer,
#           code-reviewer, qa-tester, security-reviewer, planner, designer
```

## Step 3: Configure Plane + Mattermost

Create `genasis.toml` (or let `genasis init` scaffold it):

```toml
[project]
name = "my-project"

[plane]
url = "https://plane.yourdomain.com"
workspace_slug = "my-workspace"

[mattermost]
url = "https://mm.yourdomain.com"
```

Set environment variables (`.env.agents`):

```bash
PLANE_API_KEY=your-plane-api-token
MM_ADMIN_TOKEN=your-mm-admin-token
MM_TEAM_ID=your-team-id
```

> **Using the trial?** Paste the credentials you received from
> [trial.realstory.blog](https://trial.realstory.blog).

## Step 4: Initialize

```bash
genasis init
```

This:
- Pings Plane + Mattermost to verify connectivity
- Creates a Plane project + labels
- Creates a Mattermost `#scrum-{project}` channel
- Injects overlay protocol into each agent file

## Step 5: Verify

```bash
genasis doctor
```

All checks should show ✓.

## Step 6: Start working

Open Claude Code in your project. Your agents are ready:

```
> /sprint-start
> /install-agent mobile     (if you need more agents later)
```

Agents will pick up Plane tickets, post in Mattermost, run through
the lifecycle (Todo → In Progress → In Review → Done), and
coordinate with you in the same channels you already use.

## Next steps

- [Design Swap Guide](DESIGN-SWAP-GUIDE.md) — customize your design system
- [Server Setup](../servers/README.md) — self-host Plane + Mattermost
- [Agents Marketplace](AGENTS-MARKETPLACE.md) — explore all available agents
- [Monitor TUI](MONITOR.md) — dashboard for sprint/tokens/agents/deploy
