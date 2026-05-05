# Credits & Open-Source Acknowledgments

> 한국어: [ko/CREDITS.md](ko/CREDITS.md) *(coming soon)*

Genasis is built on the shoulders of excellent open-source projects.
We curate, integrate, and add value — we don't reinvent wheels.

## Agent Sources

These projects provide the curated agent definitions available in
the genasis agents marketplace. All are used under their stated licenses.

| Project | Stars | What genasis uses | License |
|---|---|---|---|
| [everything-claude-code (ECC)](https://github.com/affaan-m/everything-claude-code) | 173K+ | `code-reviewer` (gold standard — 80% confidence filtering), `architect` (system + code dual-layer), `security-reviewer`, `silent-failure-hunter`, `a11y-architect`, `refactor-cleaner`, language-specific reviewers (TypeScript, Python, Rust, Go) | MIT |
| [wshobson/agents](https://github.com/wshobson/agents) | 34K+ | `frontend-developer` (React 19, Next.js 15, RSC), `backend-developer` (API + event-sourcing + TDD), `full-stack-orchestration` (deployment + performance + security) | MIT |
| [VoltAgent/awesome-claude-code-subagents](https://github.com/VoltAgent/awesome-claude-code-subagents) | 19K+ | `qa-tester` (test-automator + accessibility), `sre-engineer`, `devops-engineer`, infrastructure cluster (17 agents), language specialists | MIT |
| [dl-ezo/claude-code-sub-agents](https://github.com/dl-ezo/claude-code-sub-agents) | 184+ | `planner` (project-planner + requirements-analyst), full PM lifecycle (user-story-generator, progress-tracker, risk-manager, stakeholder-communicator) | MIT |
| [0xfurai/claude-code-subagents](https://github.com/0xfurai/claude-code-subagents) | 884+ | Mobile specialists: `ios-expert`, `swiftui-expert`, `android-expert`, `kotlin-expert`, `react-native-expert`, `flutter-expert` | MIT |

## Infrastructure

| Project | What genasis uses it for | License |
|---|---|---|
| [Plane](https://github.com/makeplane/plane) | Issue tracking + project management — agents own tickets and transition lifecycle | AGPL-3.0 |
| [Mattermost](https://github.com/mattermost/mattermost) | Team messaging — one bot per agent role, threaded discussions alongside humans | Various (Apache-2.0 / proprietary) |
| [Caddy](https://github.com/caddyserver/caddy) | Reverse proxy + automatic TLS for self-hosted Plane + Mattermost | Apache-2.0 |

## Rust Ecosystem

| Crate | Purpose in genasis | License |
|---|---|---|
| [Ratatui](https://github.com/ratatui/ratatui) | Monitor TUI dashboard (sprint, tokens, agents, deploy) | MIT |
| [Tera](https://github.com/Keats/tera) | Template engine for overlay rendering (Plane/MM protocol injection) | MIT |
| [Clap](https://github.com/clap-rs/clap) | CLI argument parsing | MIT / Apache-2.0 |
| [Reqwest](https://github.com/seanmonstar/reqwest) | HTTP client for Plane/MM API + agents catalog fetch | MIT / Apache-2.0 |
| [Tokio](https://github.com/tokio-rs/tokio) | Async runtime | MIT |
| [rust-i18n](https://github.com/longbridgeapp/rust-i18n) | Runtime internationalization (en/ko) | MIT |
| [Dialoguer](https://github.com/console-rs/dialoguer) | Interactive CLI prompts (agent marketplace TUI) | MIT |

## Research & Community References

| Resource | How it informed genasis |
|---|---|
| [docs/famous-agents.md](../docs/famous-agents.md) | Comprehensive survey of Claude Code agent ecosystem (methodology, comparison, recommendations) |
| [obra/superpowers](https://github.com/obra/superpowers) | Workflow methodology (TDD → plan → subagent-driven) — informed our sprint protocol design |
| [BMAD-METHOD](https://github.com/bmad-code-org/BMAD-METHOD) | Scrum/agile lifecycle skills — informed our slash command design |
| [Anthropic Claude Code docs](https://code.claude.com/docs) | Official sub-agent specification + MCP integration patterns |
| [docs/impact-of-multilang-prompts.md](../docs/impact-of-multilang-prompts.md) | Research on why single-language agent context is essential (Claude drift bugs, arXiv 2406.20052) |

## How we use these projects

1. **Agent definitions** are curated (selected, verified for compatibility)
   and distributed via the genasis agents marketplace. Source attribution
   is preserved in each agent file's `## Source` section.

2. **Overlay protocol** (Plane/MM lifecycle rules) is genasis-original
   content — it is injected INTO the community agents via marker fences,
   not derived from any of the above projects.

3. **Infrastructure** (Plane, Mattermost) is used as deployment targets,
   not vendored or forked. genasis communicates via their public REST APIs.

4. **Rust crates** are standard dependencies declared in `Cargo.toml`.

## License compliance

All curated agents are sourced exclusively from MIT-licensed repositories.
genasis itself is MIT-licensed. The overlay protocol, CLI tool, and all
genasis-original content are original works.

Plane (AGPL-3.0) and Mattermost are used as external services via API —
genasis does not include or distribute their source code.
