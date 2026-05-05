# Agents Marketplace Guide

> 한국어: [ko/AGENTS-MARKETPLACE.md](ko/AGENTS-MARKETPLACE.md) *(coming soon)*

Genasis ships with 20+ curated agents sourced from the best community
repositories (ECC, wshobson/agents, VoltAgent, dl-ezo). You browse them
by category, install individually, or use presets.

## Browse agents (CLI)

```bash
# Interactive TUI — category picker → multi-select → install
genasis agents browse

# Filter by category
genasis agents list --category mobile
genasis agents list --search "security"

# See what's installed in your project
genasis agents installed
```

## Browse agents (Claude Code)

```
/install-agent frontend
```

Claude searches the index, shows matching agents with descriptions, and
installs your selection with one click.

## Install by name

```bash
genasis agents install frontend-developer
genasis agents install code-reviewer
genasis agents install ios-expert
```

## Install presets

```bash
# Web app team (9 roles)
genasis agents install --preset web-app

# Full-stack with DevOps (11 roles)
genasis agents install --preset full-stack

# Mobile app team (9 roles)
genasis agents install --preset mobile
```

## Categories

| Category | Agents |
|---|---|
| Core Development | frontend-developer, backend-developer, architect, designer, doc-updater |
| Review & Quality | code-reviewer, qa-tester, security-reviewer, refactor-cleaner, silent-failure-hunter |
| Planning & Management | pm, planner |
| Infrastructure & DevOps | sre-engineer, devops-engineer |
| Mobile | ios-expert, android-expert, react-native-expert, flutter-expert |
| Language Specialists | typescript-reviewer, python-reviewer, rust-reviewer, go-reviewer, nextjs-developer |

## After installing

Run `genasis attach` to inject the Plane/Mattermost overlay protocol
into each installed agent. This wires them into your issue tracker and
team chat automatically.

## Remove an agent

```bash
genasis agents remove frontend-developer
```

## Update the catalog

```bash
genasis agents fetch              # download latest catalog version
genasis agents fetch --version 1.1.0  # specific version
```
