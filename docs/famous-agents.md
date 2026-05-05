# Famous OSS Claude Code Agent Collections — Field Survey

> Reference doc for evaluating `.claude/agents/*.md` providers.
> Compiled 2026-05-05. Star counts and "last pushed" timestamps captured live from the GitHub REST API on this date — re-pull before quoting numbers in conversation.

---

## 1. Methodology

I weighted candidates by (a) raw popularity (GitHub stars + fork count via `GET /repos/{o}/{r}` on api.github.com), (b) commit recency (`pushed_at` within last 60 days = active), (c) presence in Anthropic's *official* plugin marketplace (`anthropics/claude-plugins-official` `marketplace.json`), (d) inclusion in the de-facto curation lists (`hesreallyhim/awesome-claude-code` 42.6K★, `hesreallyhim/a-list-of-claude-code-agents`), and (e) actual file format — I fetched representative agent files from each repo and verified they ship valid `.claude/agents/*.md` with YAML frontmatter (`name`, `description`, optional `tools`, `model`). Repos that turned out to be *skills* frameworks, methodology guides, or CLAUDE.md-only patterns are flagged and demoted, since they are not drop-in replacements for ECC's `.claude/agents/`. Discussion-quality signals (HN thread #44742081, Augment Code's running ECC star-count series, daily.dev posts, dev.to writeups) were used as tiebreakers, not primary evidence.

---

## 2. Comparative table — top OSS Claude `.claude/agents/*.md` collections

| Repo | Stars | Last commit | Format | Install method | Strengths | Weaknesses | Best roles |
|---|---|---|---|---|---|---|---|
| [affaan-m/everything-claude-code](https://github.com/affaan-m/everything-claude-code) (ECC) | **173,533** | 2026-05-03 | Plain `.md` agents + plugin manifest + skills + hooks + rules | `git clone … && bash install.sh` (also `/plugin marketplace add affaan-m/everything-claude-code`) | 48 ships agents, **strong language-reviewer matrix** (typescript, python, rust, go, java, kotlin, c++, c#, dart, flutter), code-reviewer with 80%-confidence filtering, opensource-* utilities, harness-optimizer; cited as "community standard"; multilingual READMEs | Big surface (997 tests, multi-runtime support); some Augment Code reviews call it overkill for solo devs; opinionated about workflow | code-reviewer, planner, architect, language-specific reviewers, refactor, docs |
| [obra/superpowers](https://github.com/obra/superpowers) | **178,678** | 2026-05-04 | **Skills, not agents** (no `.claude/agents/`); ships methodology + skills like `subagent-driven-development`, `using-git-worktrees`, `writing-plans` | Anthropic-official: `/plugin install superpowers@claude-plugins-official` | Author Jesse Vincent; *officially listed* in Anthropic's plugin marketplace; brainstorming → plan → TDD → subagent-driven-development pipeline; cross-harness (Codex, Gemini CLI, Cursor) | **Not an agent collection** — if you're looking to swap `.claude/agents/*.md`, this is the wrong shape. Pair it with one of the agent collections | Workflow harness, TDD enforcement, plan→execute |
| [bmad-code-org/BMAD-METHOD](https://github.com/bmad-code-org/BMAD-METHOD) | **46,362** | 2026-05-04 | Skills-as-folders under `src/bmm-skills/` (analysis → plan → solutioning → implementation); not raw `.claude/agents/*.md` | `npx bmad-method install` | Strong scrum/agile lifecycle: PRD → architecture → story → dev story → QA E2E; sprint-planning + retrospective skills | Skill-graph format, not agent files; integration into existing `.claude/agents/` requires translation | Sprint planning, PRD writing, scrum/PM workflow |
| [wshobson/agents](https://github.com/wshobson/agents) | **34,783** | 2026-05-02 | **Plugin marketplace** that bundles `.md` agents under `plugins/<plugin-name>/agents/` | `/plugin marketplace add wshobson/agents` then `/plugin install <plugin-name>` | 80 focused plugins, 185 agents, 153 skills, 100 commands; covers SEO, blockchain, fintech, quant, embedded; *plugin-isolated context* (only loads what you install) | Some HN/issue feedback says agents are "too generic" (issue #24, discussion #42); install requires plugin per role | frontend, backend, mobile (Flutter+iOS+RN), DB, K8s, security, full-stack-orchestrator |
| [VoltAgent/awesome-claude-code-subagents](https://github.com/VoltAgent/awesome-claude-code-subagents) | **19,103** | 2026-04-20 | Plain `.md` per category + plugin manifest | `claude plugin marketplace add VoltAgent/awesome-claude-code-subagents` or clone + `./install-agents.sh` | 131+ agents, 10 clean categories (core dev / lang / infra / quality / data-AI / DX / specialized / business / meta-orch / research); explicit `mobile-developer`, `flutter-expert`, `expo-react-native-expert`, `swift-expert`, `kotlin-specialist`, `ios-developer` (under multi-platform) | Less "production-tested" reputation than wshobson; some agents include heavy `Communication Protocol` JSON request blocks that pad context | Language specialists, frontend, mobile, security-auditor, accessibility-tester |
| [hesreallyhim/awesome-claude-code](https://github.com/hesreallyhim/awesome-claude-code) | 42,554 | 2026-04-27 | **Curation list (meta)**, not a shipped agent set | `git clone` to read | The de-facto "where do I look first" index for Claude Code resources | Reorg in progress per current README; doesn't ship agents itself | Discovery |
| [hesreallyhim/a-list-of-claude-code-agents](https://github.com/hesreallyhim/a-list-of-claude-code-agents) | 1,239 | 2025-11-10 | Community-submitted agent index (links + samples) | `git clone` | Pure agent curation (sister to awesome list) | Slower-moving (last commit 2025-11) | Discovery |
| [vijaythecoder/awesome-claude-agents](https://github.com/vijaythecoder/awesome-claude-agents) | 4,234 | 2025-10-30 | Plain `.md` agents organized as `agents/{core,orchestrators,specialized,universal}` | `git clone + cp` | Stack-specialized teams: Django / Laravel / Rails / React / Vue specialists; `tech-lead-orchestrator`, `team-configurator`, `project-analyst` | Last commit 2025-10 — slowing; smaller surface | Stack-specific full-team scaffolding |
| [iannuttall/claude-agents](https://github.com/iannuttall/claude-agents) | 2,049 | 2025-07-25 | Plain `.md` under `agents/` | `git clone + cp` | Tight 7-agent set (`prd-writer`, `frontend-designer`, `code-refactorer`, `security-auditor`, `content-writer`, `vibe-coding-coach`, `project-task-planner`); often quoted in tutorials | **Stale** (last push 2025-07); low coverage | PRD writing, refactor — historic reference only |
| [lst97/claude-code-sub-agents](https://github.com/lst97/claude-code-sub-agents) | 1,558 | 2025-08-15 | Plain `.md` | `git clone + cp` | Personal full-stack collection, well-loved on dev.to | Stale (2025-08); single-maintainer | Solo full-stack developer template |
| [0xfurai/claude-code-subagents](https://github.com/0xfurai/claude-code-subagents) | 884 | 2025-10-15 | Plain `.md` | `cd ~/.claude && git clone …` (auto user-global) | **Best dedicated mobile coverage** of any pure-agent repo: `ios-expert`, `swiftui-expert`, `android-expert`, `react-native-expert`, `flutter-expert`, `expo-expert`, `swift-expert`, `kotlin-expert`, `dart-expert` | Stale-ish (Oct 2025); low star count vs. surface area | Mobile dev, language specialists |
| [dl-ezo/claude-code-sub-agents](https://github.com/dl-ezo/claude-code-sub-agents) | 184 | 2025-07-30 | Plain `.md` (35 agents) | `git clone + cp` | **Strongest PM / requirements lifecycle**: `requirements-analyst`, `user-story-generator`, `business-process-analyst`, `requirements-validator`, `risk-manager`, `progress-tracker`, `stakeholder-communicator`, `uat-coordinator`, `training-change-manager` | Stale (July 2025); small audience | Project planning, requirements, PM coordination |
| [senaiverse/claude-code-reactnative-expo-agent-system](https://github.com/senaiverse/claude-code-reactnative-expo-agent-system) | 114 | 2025-10-05 | Plain `.md` tiered (S/1/2/3) | `git clone` | RN-specific: `Design Token Guardian`, `A11y Compliance Enforcer`, `Performance Budget Enforcer`, `Performance Prophet`, `Security Penetration Specialist` | Niche; small | React Native + Expo only |
| [keskinonur/claude-code-ios-dev-guide](https://github.com/keskinonur/claude-code-ios-dev-guide) | 655 | 2026-01 | **Guide doc, not agents** (templates for you to author) | n/a | PRD-driven workflow + Swift 6 / SwiftUI / @Observable patterns; XcodeBuildMCP integration | You build the agents; doesn't ship them | iOS Swift reference |
| [anthropics/claude-plugins-official](https://github.com/anthropics/claude-plugins-official) | **18,566** | 2026-05-04 | Marketplace registry (178 plugins) — Anthropic-curated | Auto-loaded by Claude Code | Vendor integrations (Atlassian, Auth0, AWS, Cloudflare, Cloudinary, Asana, Box…) + workflow plugins; **only mobile-related entries: `expo`, `kotlin-lsp`, `swift-lsp`** | Mostly *integrations*, not generic dev-role agents; sparse coverage of frontend/backend/QA/architect roles | Vendor SDK glue, LSP plumbing |

Repos **considered and dropped** as either dead or low-signal: `rahulvrane/awesome-claude-agents` (334★, low fork ratio, supplanted by VoltAgent), `rshah515/claude-code-subagents` (49★, last push 2025-08), `chusri/claude-code-agents` (mirror of wshobson), `mvandermeulen/wshobson-agents` and `destenson/wshobson--agents` (forks).

---

## 3. De-facto standard per role — recommendation table

> Rule used: if a role has a **well-known dedicated agent file** in a repo with >5K stars *and* commit activity in 2026, it is named as the standard. If no clear standard exists, the best available is named with the proviso.

| Role | Recommended source | File path inside repo | Why this one wins | Runner-up |
|---|---|---|---|---|
| Frontend developer (web) | wshobson/agents | [`plugins/frontend-mobile-development/agents/frontend-developer.md`](https://github.com/wshobson/agents/blob/main/plugins/frontend-mobile-development/agents/frontend-developer.md) | React 19 / Next 15 / RSC explicit; Anthropic-marketplace-shaped; isolated plugin context | VoltAgent `categories/01-core-development/frontend-developer.md` |
| Backend developer | wshobson/agents | [`plugins/backend-development/agents/backend-architect.md`](https://github.com/wshobson/agents/blob/main/plugins/backend-development/agents/backend-architect.md) | Splits backend-architect / event-sourcing / graphql / temporal-python-pro into one plugin; tdd-orchestrator co-located | VoltAgent `backend-developer.md` + `microservices-architect.md` |
| Full-stack | wshobson/agents | [`plugins/full-stack-orchestration/agents/`](https://github.com/wshobson/agents/tree/main/plugins/full-stack-orchestration) (deployment-engineer, performance-engineer, security-auditor, test-automator) | Multi-agent workflow orchestrator, designed for parallel execution | VoltAgent `categories/01-core-development/fullstack-developer.md` |
| Mobile (cross-platform) | VoltAgent | [`categories/01-core-development/mobile-developer.md`](https://github.com/VoltAgent/awesome-claude-code-subagents/blob/main/categories/01-core-development/mobile-developer.md) + [`categories/02-language-specialists/flutter-expert.md`](https://github.com/VoltAgent/awesome-claude-code-subagents/blob/main/categories/02-language-specialists/flutter-expert.md) + [`expo-react-native-expert.md`](https://github.com/VoltAgent/awesome-claude-code-subagents/blob/main/categories/02-language-specialists/expo-react-native-expert.md) | Largest single-repo mobile surface in a 19K★ active repo | 0xfurai/claude-code-subagents `agents/{ios,android,flutter,expo,react-native}-expert.md` |
| Mobile iOS (Swift native) | 0xfurai/claude-code-subagents | [`agents/ios-expert.md`](https://github.com/0xfurai/claude-code-subagents/blob/main/agents/ios-expert.md), [`agents/swiftui-expert.md`](https://github.com/0xfurai/claude-code-subagents/blob/main/agents/swiftui-expert.md), [`agents/swift-expert.md`](https://github.com/0xfurai/claude-code-subagents/blob/main/agents/swift-expert.md) | Only repo shipping all three iOS-specific agents (UIKit, SwiftUI, Swift lang) as actual files | VoltAgent `swift-expert.md` (no UIKit-specific) |
| Mobile Android (Kotlin/Compose) | 0xfurai/claude-code-subagents | `agents/android-expert.md`, `agents/kotlin-expert.md` | Dedicated Android-Compose agent | VoltAgent `categories/02-language-specialists/kotlin-specialist.md` |
| Mobile React Native / Expo | senaiverse/claude-code-reactnative-expo-agent-system | tiered agents under `.claude/agents/` | Only repo specifically targeting RN/Expo with accessibility + perf budget agents pre-wired | VoltAgent `expo-react-native-expert.md` |
| QA / test automation | VoltAgent | [`categories/04-quality-security/test-automator.md`](https://github.com/VoltAgent/awesome-claude-code-subagents/blob/main/categories/04-quality-security/test-automator.md), [`qa-expert.md`](https://github.com/VoltAgent/awesome-claude-code-subagents/blob/main/categories/04-quality-security/qa-expert.md), [`ui-ux-tester.md`](https://github.com/VoltAgent/awesome-claude-code-subagents/blob/main/categories/04-quality-security/ui-ux-tester.md) | 6 quality/security agents in one folder; explicit accessibility-tester | wshobson `plugins/api-testing-observability` + `plugins/unit-testing` |
| Security reviewer | ECC | [`agents/security-reviewer.md`](https://github.com/affaan-m/everything-claude-code/blob/main/agents/security-reviewer.md) | Cited as standout in awesome-claude-code; pairs with `silent-failure-hunter`, `ad-security-reviewer` | VoltAgent `security-auditor.md` (also strong) |
| DevOps / infra | VoltAgent | [`categories/03-infrastructure/`](https://github.com/VoltAgent/awesome-claude-code-subagents/tree/main/categories/03-infrastructure) — full set: cloud-architect, kubernetes-specialist, terraform-engineer, sre-engineer, devops-incident-responder, network-engineer, platform-engineer | Most *complete* infra cluster (17 agents, separated cleanly) | wshobson `plugins/kubernetes-operations` + `plugins/cloud-infrastructure` |
| Architect | ECC | [`agents/architect.md`](https://github.com/affaan-m/everything-claude-code/blob/main/agents/architect.md) and [`agents/code-architect.md`](https://github.com/affaan-m/everything-claude-code/blob/main/agents/code-architect.md) | Two-layer architect (system + code); type-design-analyzer co-located | VoltAgent `microservices-architect.md` + wshobson `architect-review` |
| Code reviewer | **ECC** | [`agents/code-reviewer.md`](https://github.com/affaan-m/everything-claude-code/blob/main/agents/code-reviewer.md) | **Most-praised single agent in the ecosystem**: 80%-confidence filtering + auto-consolidation + security-first ordering. Multiple writeups call it the gold standard. | wshobson `plugins/comprehensive-review/agents/code-reviewer.md` (model: opus) |
| Planner / PM | dl-ezo/claude-code-sub-agents | [`project-planner.md`](https://github.com/dl-ezo/claude-code-sub-agents/blob/main/project-planner.md), [`requirements-analyst.md`](https://github.com/dl-ezo/claude-code-sub-agents/blob/main/requirements-analyst.md), [`user-story-generator.md`](https://github.com/dl-ezo/claude-code-sub-agents/blob/main/user-story-generator.md), [`progress-tracker.md`](https://github.com/dl-ezo/claude-code-sub-agents/blob/main/progress-tracker.md), [`risk-manager.md`](https://github.com/dl-ezo/claude-code-sub-agents/blob/main/risk-manager.md), [`stakeholder-communicator.md`](https://github.com/dl-ezo/claude-code-sub-agents/blob/main/stakeholder-communicator.md) | **Only repo with a complete requirements→PM lifecycle as `.md` agents.** ECC's `planner.md` is good but standalone | BMAD-METHOD (skill-shaped, needs translation) |
| Performance | wshobson/agents | [`plugins/application-performance/agents/`](https://github.com/wshobson/agents/tree/main/plugins/application-performance) + [`performance-engineer`](https://github.com/wshobson/agents/blob/main/plugins/full-stack-orchestration/agents/performance-engineer.md) | Application-tier focus + tied into orchestration | ECC `performance-optimizer.md` |
| Refactor | ECC | [`agents/refactor-cleaner.md`](https://github.com/affaan-m/everything-claude-code/blob/main/agents/refactor-cleaner.md) + [`code-simplifier.md`](https://github.com/affaan-m/everything-claude-code/blob/main/agents/code-simplifier.md) | Two complementary agents; pairs naturally with `silent-failure-hunter` | VoltAgent `refactoring-specialist.md` |
| Docs | ECC | [`agents/doc-updater.md`](https://github.com/affaan-m/everything-claude-code/blob/main/agents/doc-updater.md) + [`docs-lookup.md`](https://github.com/affaan-m/everything-claude-code/blob/main/agents/docs-lookup.md) | doc-updater specifically wired for sync after code changes | wshobson `plugins/code-documentation` |
| Accessibility | ECC | [`agents/a11y-architect.md`](https://github.com/affaan-m/everything-claude-code/blob/main/agents/a11y-architect.md) | Sole "a11y-architect" framing in any major repo | VoltAgent `accessibility-tester.md` |
| Workflow / methodology | obra/superpowers | skills (not agents) — install via `/plugin install superpowers@claude-plugins-official` | Anthropic-blessed; layers cleanly *over* any agent collection above | BMAD-METHOD |

---

## 4. Installation recipes

> All targets assume Claude Code v2.0+ in repository `/work/secusy`. `.claude/agents/` is the **project-scoped** location (committed); `~/.claude/agents/` is **user-global** (not committed). Pick deliberately.

### A. wshobson/agents (plugin marketplace, RECOMMENDED for most roles)

```bash
# In Claude Code, register the marketplace once
/plugin marketplace add wshobson/agents

# Then install per-role plugins as needed
/plugin install backend-development@claude-code-workflows
/plugin install frontend-mobile-development@claude-code-workflows
/plugin install comprehensive-review@claude-code-workflows
/plugin install kubernetes-operations@claude-code-workflows
/plugin install full-stack-orchestration@claude-code-workflows
```

Equivalent raw clone (when `.md` files need to live in your repo for editing):

```bash
git clone --depth=1 https://github.com/wshobson/agents.git /tmp/wshobson
mkdir -p .claude/agents
cp /tmp/wshobson/plugins/comprehensive-review/agents/code-reviewer.md .claude/agents/
cp /tmp/wshobson/plugins/frontend-mobile-development/agents/*.md .claude/agents/
cp /tmp/wshobson/plugins/backend-development/agents/*.md .claude/agents/
```

License: MIT (Seth Hobson). Free to vendor.

### B. VoltAgent/awesome-claude-code-subagents

```bash
# As a Claude Code plugin
claude plugin marketplace add VoltAgent/awesome-claude-code-subagents
claude plugin install voltagent-core-dev
claude plugin install voltagent-lang
claude plugin install voltagent-infra
claude plugin install voltagent-meta

# Or interactive installer (clone form)
git clone https://github.com/VoltAgent/awesome-claude-code-subagents.git
cd awesome-claude-code-subagents
./install-agents.sh   # installs to ~/.claude/agents/

# Or per-file copy into project
cp categories/01-core-development/frontend-developer.md /work/secusy/.claude/agents/
cp categories/02-language-specialists/flutter-expert.md /work/secusy/.claude/agents/
cp categories/02-language-specialists/swift-expert.md /work/secusy/.claude/agents/
cp categories/04-quality-security/{code-reviewer,security-auditor,test-automator,qa-expert,accessibility-tester}.md /work/secusy/.claude/agents/
cp categories/03-infrastructure/{kubernetes-specialist,terraform-engineer,sre-engineer}.md /work/secusy/.claude/agents/
```

License: MIT (VoltAgent, 2025). Free to vendor.

### C. ECC (current incumbent)

```bash
# Plugin form (Anthropic-style)
/plugin marketplace add affaan-m/everything-claude-code

# Or direct install script (clones & wires up agents/skills/hooks/rules)
git clone https://github.com/affaan-m/everything-claude-code.git /tmp/ecc
cd /tmp/ecc && bash install.sh

# Cherry-pick the standout files into your repo
cp /tmp/ecc/agents/code-reviewer.md       /work/secusy/.claude/agents/
cp /tmp/ecc/agents/architect.md           /work/secusy/.claude/agents/
cp /tmp/ecc/agents/security-reviewer.md   /work/secusy/.claude/agents/
cp /tmp/ecc/agents/silent-failure-hunter.md /work/secusy/.claude/agents/
cp /tmp/ecc/agents/typescript-reviewer.md /work/secusy/.claude/agents/
```

License: MIT.

> Note: `obra/superpowers` is installed as a *skill* plugin (`/plugin install superpowers@claude-plugins-official`) and **does not** populate `.claude/agents/`. It complements any of A/B/C above.

### D. 0xfurai/claude-code-subagents (mobile gap-fill)

```bash
# User-global install (drops directly into ~/.claude/agents/)
cd ~/.claude && git clone https://github.com/0xfurai/claude-code-subagents.git

# Or selective copy into project
git clone --depth=1 https://github.com/0xfurai/claude-code-subagents.git /tmp/furai
cp /tmp/furai/agents/{ios-expert,swiftui-expert,swift-expert}.md /work/secusy/.claude/agents/
cp /tmp/furai/agents/{android-expert,kotlin-expert}.md /work/secusy/.claude/agents/
cp /tmp/furai/agents/{react-native-expert,expo-expert,flutter-expert,dart-expert}.md /work/secusy/.claude/agents/
```

### E. dl-ezo/claude-code-sub-agents (PM lifecycle gap-fill)

```bash
git clone --depth=1 https://github.com/dl-ezo/claude-code-sub-agents.git /tmp/dlezo
cp /tmp/dlezo/{project-planner,requirements-analyst,user-story-generator,progress-tracker,risk-manager,stakeholder-communicator,business-process-analyst,requirements-validator,uat-coordinator}.md /work/secusy/.claude/agents/
```

---

## 5. Verdict — is ECC still the best?

**Short answer: ECC is still the strongest single-repo *foundation*, but the optimal 2026 setup for `secusy` is a hybrid.**

Reasoning:

1. **ECC wins on canonical quality** for `code-reviewer`, `security-reviewer`, `architect`, `planner`, `refactor-cleaner`, `silent-failure-hunter`, `a11y-architect`, `doc-updater`, language reviewers (`typescript-reviewer.md`, `python-reviewer.md`, `rust-reviewer.md`, `go-reviewer.md`, `flutter-reviewer.md`, `kotlin-reviewer.md`, `dart-reviewer.md`). These are the most-cited individual agents in the ecosystem (Augment Code series, dev.to writeups, awesome-claude-code).
2. **ECC has gaps** — by design it's a **harness**, not a per-stack scaffold:
   - No frontend-developer / backend-developer *implementer* agents (only reviewers)
   - No mobile coverage at all (no iOS/Android/RN/Expo agent)
   - Thin DevOps/infra (only `e2e-runner`, no kubernetes / terraform / cloud-architect)
   - No GraphQL/WebSocket specialists
   - PM/requirements layer is one file (`planner.md`) vs. dl-ezo's 8-file lifecycle
3. **wshobson/agents is the natural complement** for implementers (frontend, backend, mobile, K8s, cloud, full-stack-orchestration). The plugin model means context isn't bloated — install per role.
4. **VoltAgent fills the language-specialist + accessibility + DevOps holes** when a *raw `.md` cherry-pick* (not a plugin) is wanted.
5. **0xfurai is unique** for native-iOS (`ios-expert`, `swiftui-expert`) and Android (`android-expert`) — no other major repo ships those as named files.
6. **dl-ezo** is unique for full PM lifecycle (`requirements-analyst`, `user-story-generator`, `risk-manager`, `progress-tracker`, `stakeholder-communicator`, `uat-coordinator`).
7. **obra/superpowers** is the workflow skin — install on top, regardless of which agents are picked. It's the only `.claude/agents/`-adjacent thing officially endorsed by Anthropic via `claude-plugins-official`.

### Recommended hybrid for `/work/secusy`

| Layer | Source | What to copy |
|---|---|---|
| Reviewers + harness (keep) | ECC | `code-reviewer.md`, `security-reviewer.md`, `architect.md`, `silent-failure-hunter.md`, `a11y-architect.md`, `doc-updater.md`, `tdd-guide.md`, `e2e-runner.md`, `refactor-cleaner.md`, `typescript-reviewer.md` |
| Implementers (add) | wshobson/agents (plugins) | `backend-development`, `frontend-mobile-development`, `full-stack-orchestration` plugins via `/plugin install` |
| Mobile (add if relevant) | 0xfurai | `ios-expert.md`, `swiftui-expert.md`, `android-expert.md`, `react-native-expert.md`, `expo-expert.md` |
| Language specialists (gap-fill) | VoltAgent | `react-specialist.md`, `nextjs-developer.md`, `swift-expert.md`, `kotlin-specialist.md`, `python-pro.md` |
| PM lifecycle (replace ECC's single `planner.md`) | dl-ezo | `requirements-analyst.md`, `user-story-generator.md`, `progress-tracker.md`, `risk-manager.md` |
| Workflow skin | obra/superpowers | `/plugin install superpowers@claude-plugins-official` |

This is consistent with the project's existing scrum + Plane workflow: ECC reviewers stay as the quality bar, wshobson plugins become the implementer pool agents called by `pm` during sprint dispatch, dl-ezo augments the planner, superpowers enforces the brainstorming → plan → TDD pipeline already encoded in the project's CLAUDE.md.

**ECC alone is no longer the best choice** for a project that needs frontend/backend/mobile *implementers* in addition to reviewers — and `secusy` does, given the agent roster in `.claude/agents/`. Augmenting with wshobson + VoltAgent + dl-ezo + (optional) 0xfurai/superpowers closes every observable gap with no overlap.

---

## 6. Sources

### GitHub repositories (verified live 2026-05-05)
- [affaan-m/everything-claude-code](https://github.com/affaan-m/everything-claude-code) — 173,533★, pushed 2026-05-03
- [obra/superpowers](https://github.com/obra/superpowers) — 178,678★, pushed 2026-05-04
- [obra/superpowers-marketplace](https://github.com/obra/superpowers-marketplace) — 931★
- [bmad-code-org/BMAD-METHOD](https://github.com/bmad-code-org/BMAD-METHOD) — 46,362★, pushed 2026-05-04
- [hesreallyhim/awesome-claude-code](https://github.com/hesreallyhim/awesome-claude-code) — 42,554★
- [wshobson/agents](https://github.com/wshobson/agents) — 34,783★, pushed 2026-05-02
- [VoltAgent/awesome-claude-code-subagents](https://github.com/VoltAgent/awesome-claude-code-subagents) — 19,103★, pushed 2026-04-20
- [anthropics/claude-plugins-official](https://github.com/anthropics/claude-plugins-official) — 18,566★, pushed 2026-05-04
- [vijaythecoder/awesome-claude-agents](https://github.com/vijaythecoder/awesome-claude-agents) — 4,234★, pushed 2025-10-30
- [iannuttall/claude-agents](https://github.com/iannuttall/claude-agents) — 2,049★, pushed 2025-07-25
- [lst97/claude-code-sub-agents](https://github.com/lst97/claude-code-sub-agents) — 1,558★, pushed 2025-08-15
- [hesreallyhim/a-list-of-claude-code-agents](https://github.com/hesreallyhim/a-list-of-claude-code-agents) — 1,239★
- [0xfurai/claude-code-subagents](https://github.com/0xfurai/claude-code-subagents) — 884★, pushed 2025-10-15
- [keskinonur/claude-code-ios-dev-guide](https://github.com/keskinonur/claude-code-ios-dev-guide) — 655★ (guide, not agents)
- [dl-ezo/claude-code-sub-agents](https://github.com/dl-ezo/claude-code-sub-agents) — 184★, pushed 2025-07-30
- [senaiverse/claude-code-reactnative-expo-agent-system](https://github.com/senaiverse/claude-code-reactnative-expo-agent-system) — 114★
- [anthropics/claude-cookbooks](https://github.com/anthropics/claude-cookbooks)

### Anthropic official docs
- [Create custom subagents — Claude Code Docs](https://code.claude.com/docs/en/sub-agents)
- [Subagents in the SDK — Claude API Docs](https://platform.claude.com/docs/en/agent-sdk/subagents)
- [Claude Code Advanced Patterns: Subagents, MCP, and Scaling (PDF)](https://resources.anthropic.com/hubfs/Claude%20Code%20Advanced%20Patterns_%20Subagents,%20MCP,%20and%20Scaling%20to%20Real%20Codebases.pdf)
- [How and when to use subagents in Claude Code (Anthropic blog)](https://claude.com/blog/subagents-in-claude-code)
- [Building agents with the Claude Agent SDK (Anthropic engineering)](https://www.anthropic.com/engineering/building-agents-with-the-claude-agent-sdk)
- [Discover and install prebuilt plugins through marketplaces](https://code.claude.com/docs/en/discover-plugins)
- [Plugin Marketplace & Discovery — DeepWiki](https://deepwiki.com/anthropics/claude-code/4.1-plugin-marketplace-and-discovery)

### Industry / community writeups (used for reception, not raw fact)
- Augment Code series tracking ECC growth: [82K](https://medium.com/@tentenco/everything-claude-code-inside-the-82k-star-agent-harness-thats-dividing-the-developer-community-4fe54feccbc1), [100K](https://www.augmentcode.com/learn/everything-claude-code-github), [118K](https://www.augmentcode.com/learn/everything-claude-code-118k-stars), [163K](https://www.augmentcode.com/learn/everything-claude-code-hits-163k-stars), [170K](https://www.augmentcode.com/learn/everything-claude-code-github-stars)
- [Everything Claude Code Cheatsheet — dev.to](https://dev.to/shimo4228/everything-claude-code-ecc-complete-cheatsheet-24ok)
- [Bridgers — ECC explained](https://bridgers.agency/en/blog/everything-claude-code-explained)
- [ClaudePluginHub — ECC entry](https://www.claudepluginhub.com/plugins/affaan-m-everything-claude-code)
- [KDnuggets — 10 GitHub repositories to master Claude Code](https://www.kdnuggets.com/10-github-repositories-to-master-claude-code)
- [Claude Code productivity nuke (wshobson) — AI Monks Medium](https://medium.com/aimonks/claude-code-productivity-nuke-one-click-summons-112-agents-16-team-workflows-b83921719c80)
- [Ry Walker — wshobson/agents research](https://rywalker.com/research/wshobson-agents)
- [tom-doerr — Wshobson Agents showcase](https://tom-doerr.github.io/repo_posts/2025/07/31/wshobson-agents.html)
- [VoltAgent dev.to — 100+ Claude Code Subagent Collection](https://dev.to/voltagent/100-claude-code-subagent-collection-1eb0)
- [Builder.io — Claude Code subagents how-to](https://www.builder.io/blog/claude-code-subagents)
- [PubNub — Best practices for subagents](https://www.pubnub.com/blog/best-practices-for-claude-code-sub-agents/)
- [AntStack — Claude Agents/Subagents/Skills field guide](https://www.antstack.com/blog/claude-agents-subagents-agent-teams-skills-and-mcp-a-developer-s-field-guide/)
- [sankalp's blog — Claude Code 2.0 guide](https://sankalp.bearblog.dev/my-experience-with-claude-code-20-and-how-to-get-better-at-using-coding-agents/)
- [InfoQ — Anthropic agent-based code review](https://www.infoq.com/news/2026/04/claude-code-review/)
- [systemprompt.io — Anthropic marketplace install guide](https://systemprompt.io/guides/getting-started-anthropic-marketplace)
- [Pete Gypps — Official 36-plugin marketplace breakdown](https://www.petegypps.uk/blog/claude-code-official-plugin-marketplace-complete-guide-36-plugins-december-2025)
- [Superprompt — Best Claude Code agents](https://superprompt.com/blog/best-claude-code-agents-and-use-cases)
- [ClaudeFast — 9 best skills](https://claudefa.st/blog/tools/skills/best-claude-code-skills)
- [subagents.cc directory](https://subagents.cc/)
- [claudemarketplaces.com — mobile category](https://claudemarketplaces.com/skills/category/mobile)

### Hacker News / Reddit threads
- [Ask HN: Experience with Claude Code subagents](https://news.ycombinator.com/item?id=44742081)
- [Claude Code introduces specialized sub-agents (HN)](https://news.ycombinator.com/item?id=44686726)
- [How I'm Productive with Claude Code (HN)](https://news.ycombinator.com/item?id=47494890)
- [How to use Claude Code subagents to parallelize development (HN)](https://news.ycombinator.com/item?id=45181577)
- [Claude Code hits #1 on HN — dev.to writeup](https://dev.to/max_quimby/claude-code-just-hit-1-on-hacker-news-heres-everything-you-need-to-know-j74)
- [Weekly Reddit AI Agent Newsletter (substack)](https://agenticlife.substack.com/p/weekly-reddit-ai-agent-newsletter)
- [Superpowers blog — Jesse Vincent (fsck.com)](https://blog.fsck.com/2025/10/09/superpowers/)

### Single-agent file references used in §3
- [VoltAgent frontend-developer.md](https://github.com/VoltAgent/awesome-claude-code-subagents/blob/main/categories/01-core-development/frontend-developer.md)
- [VoltAgent code-reviewer.md](https://github.com/VoltAgent/awesome-claude-code-subagents/blob/main/categories/04-quality-security/code-reviewer.md)
- [VoltAgent security-auditor.md](https://github.com/VoltAgent/awesome-claude-code-subagents/blob/main/categories/04-quality-security/security-auditor.md)
- [VoltAgent flutter-expert.md](https://github.com/VoltAgent/awesome-claude-code-subagents/blob/main/categories/02-language-specialists/flutter-expert.md)
- [VoltAgent mobile-developer.md](https://github.com/VoltAgent/awesome-claude-code-subagents/blob/main/categories/01-core-development/mobile-developer.md)
- [VoltAgent mobile-app-developer.md (specialized-domains)](https://github.com/VoltAgent/awesome-claude-code-subagents/blob/main/categories/07-specialized-domains/mobile-app-developer.md)
- [wshobson code-reviewer.md (model: opus)](https://github.com/wshobson/agents/blob/main/plugins/comprehensive-review/agents/code-reviewer.md)
- [wshobson error-detective.md](https://github.com/wshobson/agents/blob/main/error-detective.md)
- [wshobson docs-architect.md](https://github.com/wshobson/agents/blob/main/docs-architect.md)
- [wshobson data-engineer.md](https://github.com/wshobson/agents/blob/main/data-engineer.md)
- [0xfurai ios-expert.md](https://github.com/0xfurai/claude-code-subagents/blob/main/agents/ios-expert.md)
- [0xfurai flutter-expert.md](https://github.com/0xfurai/claude-code-subagents/blob/main/agents/flutter-expert.md)
- [vijaythecoder universal agents folder](https://github.com/vijaythecoder/awesome-claude-agents/tree/main/agents/universal)

### Discussion / criticism (used for "Weaknesses" column)
- [wshobson/agents Discussion #42 — "Very generic agents"](https://github.com/wshobson/agents/discussions/42)
- [wshobson/agents Issue #24 — code-reviewer.md question](https://github.com/wshobson/agents/issues/24)

---

## 7. Curated `.claude/agents/` plan — every dev role covered, single folder

> Goal: a **single `.claude/agents/` folder** that covers all common SW-development roles by cherry-picking from the repos surveyed above. No plugin marketplace dependency at runtime — just `.md` files committed to this repo. Sources stay traceable per-file (§9).
>
> Naming convention: `{role}.{origin-tag}.md` where `origin-tag` ∈ `{secusy, ecc, wshobson, voltagent, furai, dlezo, senaiverse, vijaythecoder}`. This makes provenance grep-able (`ls .claude/agents/ | cut -d. -f2 | sort -u`) and prevents same-name collisions when two repos ship a `code-reviewer.md`. **`secusy` is reserved for files genuinely written from scratch by this project's team — files derived from ECC (even if heavily modified) are tagged `ecc` so attribution is correct.**
>
> **Total: 47 cherry-picked files + 7 secusy-owned orchestrators = 54 agents** in one folder.

### 7.1 Origin audit of files currently in `.claude/agents/`

Before designing the new layout, verify what each *existing* file's true upstream is. This is the only honest basis for the §7.2 plan and §8 script. Origin determined by (a) presence in ECC's published `agents/` directory at the time of secusy's `genesis/create-agentic-team.sh` STEP 2 clone, (b) git history of the file in this repo, (c) content inspection.

| Current file | True origin | Status today | Tag in new layout |
|---|---|---|---|
| `architect.md` | **ECC** (modified by secusy 2026-04-13 to add Plane/Mattermost lifecycle) | ECC-derived | `architect.ecc.md` |
| `backend.md` | secusy-custom (no ECC counterpart — ECC only ships reviewers, not implementers) | secusy-original | `backend.md` |
| `build-error-resolver.md` | **ECC** | unmodified ECC | `build-error-resolver.ecc.md` |
| `chief-of-staff.md` | **ECC** | unmodified ECC; off-scope for team scrum | dropped |
| `code-reviewer.md` | **ECC** (modified 2026-04-13) | ECC-derived | `code-reviewer.ecc.md` |
| `database-reviewer.md` | **ECC** | unmodified ECC | `database-reviewer.ecc.md` |
| `designer.md` | secusy-custom (Gumroad design system orchestrator — no ECC counterpart) | secusy-original | `designer.md` |
| `devops.md` | secusy-custom (Plane/CI orchestrator — no ECC counterpart) | secusy-original | `devops.md` |
| `docs-lookup.md` | **ECC** | unmodified ECC | `docs-lookup.ecc.md` |
| `doc-updater.md` | **ECC** | unmodified ECC | `doc-updater.ecc.md` |
| `e2e-runner.md` | **ECC** | unmodified ECC | `e2e-runner.ecc.md` |
| `frontend.md` | secusy-custom (Plane/Mattermost orchestrator — no ECC counterpart) | secusy-original | `frontend.md` |
| `harness-optimizer.md` | **ECC** | unmodified ECC | `harness-optimizer.ecc.md` |
| `loop-operator.md` | **ECC** | unmodified ECC; off-scope | dropped |
| `performance-optimizer.md` | **ECC** | unmodified ECC | `performance-optimizer.ecc.md` |
| `planner.md` | **ECC** (modified 2026-04-12 to reference Plane) | ECC-derived | `planner.ecc.md` |
| `pm.md` | secusy-custom (Sprint/Plane Cycle orchestrator — no ECC counterpart) | secusy-original | `pm.md` |
| `qa.md` | secusy-custom (E2E ownership matrix — no ECC counterpart, distinct from ECC's `e2e-runner`) | secusy-original | `qa.md` |
| `refactor-cleaner.md` | **ECC** | unmodified ECC | `refactor-cleaner.ecc.md` |
| `security.md` | secusy-custom (security domain ticket owner, distinct from ECC's `security-reviewer`) | secusy-original | `security.md` |
| `security-reviewer.md` | **ECC** | unmodified ECC | `security-reviewer.ecc.md` |
| `tdd-guide.md` | **ECC** | unmodified ECC | `tdd-guide.ecc.md` |
| `typescript-reviewer.md` | **ECC** | unmodified ECC | `typescript-reviewer.ecc.md` |

**Summary**: of 23 current files, **16 are ECC-origin** (14 unmodified + 2 modified) and **7 are secusy-original** (the orchestrators). The new §7.2 inventory tags them accordingly.

### 7.2 Full inventory — every file in the planned `.claude/agents/`

Each row carries a clickable upstream URL pointing to the exact upstream blob. License column is MIT for every row (audited live in §10).

| # | Category | File in `.claude/agents/` | Origin | Upstream URL (full path) | Lic. |
|---|---|---|---|---|---|
| 1 | Orchestrator | `pm.md` | **secusy** | (no upstream — written by secusy team) | — |
| 2 | Orchestrator | `frontend.md` | **secusy** | (no upstream — written by secusy team) | — |
| 3 | Orchestrator | `backend.md` | **secusy** | (no upstream — written by secusy team) | — |
| 4 | Orchestrator | `qa.md` | **secusy** | (no upstream — written by secusy team) | — |
| 5 | Orchestrator | `security.md` | **secusy** | (no upstream — written by secusy team) | — |
| 6 | Orchestrator | `devops.md` | **secusy** | (no upstream — written by secusy team) | — |
| 7 | Orchestrator | `designer.md` | **secusy** | (no upstream — written by secusy team) | — |
| 8 | Architect | `architect.ecc.md` | **ECC** | https://github.com/affaan-m/everything-claude-code/blob/main/agents/architect.md | MIT |
| 9 | Architect | `code-architect.ecc.md` | **ECC** | https://github.com/affaan-m/everything-claude-code/blob/main/agents/code-architect.md | MIT |
| 10 | Architect | `architect-review.wshobson.md` | **wshobson** | https://github.com/wshobson/agents/blob/main/plugins/comprehensive-review/agents/architect-review.md | MIT |
| 11 | Architect | `microservices-architect.voltagent.md` | **VoltAgent** | https://github.com/VoltAgent/awesome-claude-code-subagents/blob/main/categories/01-core-development/microservices-architect.md | MIT |
| 12 | Code review | `code-reviewer.ecc.md` | **ECC** | https://github.com/affaan-m/everything-claude-code/blob/main/agents/code-reviewer.md | MIT |
| 13 | Code review | `code-reviewer.wshobson.md` | **wshobson** | https://github.com/wshobson/agents/blob/main/plugins/comprehensive-review/agents/code-reviewer.md | MIT |
| 14 | Code review | `silent-failure-hunter.ecc.md` | **ECC** | https://github.com/affaan-m/everything-claude-code/blob/main/agents/silent-failure-hunter.md | MIT |
| 15 | Code review | `typescript-reviewer.ecc.md` | **ECC** | https://github.com/affaan-m/everything-claude-code/blob/main/agents/typescript-reviewer.md | MIT |
| 16 | Code review | `python-reviewer.ecc.md` | **ECC** | https://github.com/affaan-m/everything-claude-code/blob/main/agents/python-reviewer.md | MIT |
| 17 | Code review | `rust-reviewer.ecc.md` | **ECC** | https://github.com/affaan-m/everything-claude-code/blob/main/agents/rust-reviewer.md | MIT |
| 18 | Code review | `go-reviewer.ecc.md` | **ECC** | https://github.com/affaan-m/everything-claude-code/blob/main/agents/go-reviewer.md | MIT |
| 19 | Code review | `kotlin-reviewer.ecc.md` | **ECC** | https://github.com/affaan-m/everything-claude-code/blob/main/agents/kotlin-reviewer.md | MIT |
| 20 | Code review | `flutter-reviewer.ecc.md` | **ECC** | https://github.com/affaan-m/everything-claude-code/blob/main/agents/flutter-reviewer.md | MIT |
| 21 | Security | `security-reviewer.ecc.md` | **ECC** | https://github.com/affaan-m/everything-claude-code/blob/main/agents/security-reviewer.md | MIT |
| 22 | Security | `security-auditor.voltagent.md` | **VoltAgent** | https://github.com/VoltAgent/awesome-claude-code-subagents/blob/main/categories/04-quality-security/security-auditor.md | MIT |
| 23 | Frontend (web) | `frontend-developer.wshobson.md` | **wshobson** | https://github.com/wshobson/agents/blob/main/plugins/frontend-mobile-development/agents/frontend-developer.md | MIT |
| 24 | Frontend (web) | `react-specialist.voltagent.md` | **VoltAgent** | https://github.com/VoltAgent/awesome-claude-code-subagents/blob/main/categories/02-language-specialists/react-specialist.md | MIT |
| 25 | Frontend (web) | `nextjs-developer.voltagent.md` | **VoltAgent** | https://github.com/VoltAgent/awesome-claude-code-subagents/blob/main/categories/02-language-specialists/nextjs-developer.md | MIT |
| 26 | Backend | `backend-architect.wshobson.md` | **wshobson** | https://github.com/wshobson/agents/blob/main/plugins/backend-development/agents/backend-architect.md | MIT |
| 27 | Backend | `graphql-architect.wshobson.md` | **wshobson** | https://github.com/wshobson/agents/blob/main/plugins/backend-development/agents/graphql-architect.md | MIT |
| 28 | Backend | `python-pro.voltagent.md` | **VoltAgent** | https://github.com/VoltAgent/awesome-claude-code-subagents/blob/main/categories/02-language-specialists/python-pro.md | MIT |
| 29 | Backend | `nodejs-developer.voltagent.md` | **VoltAgent** | https://github.com/VoltAgent/awesome-claude-code-subagents/blob/main/categories/02-language-specialists/nodejs-developer.md | MIT |
| 30 | Full-stack | `fullstack-developer.voltagent.md` | **VoltAgent** | https://github.com/VoltAgent/awesome-claude-code-subagents/blob/main/categories/01-core-development/fullstack-developer.md | MIT |
| 31 | Full-stack | `deployment-engineer.wshobson.md` | **wshobson** | https://github.com/wshobson/agents/blob/main/plugins/full-stack-orchestration/agents/deployment-engineer.md | MIT |
| 32 | Mobile X-plat | `mobile-developer.voltagent.md` | **VoltAgent** | https://github.com/VoltAgent/awesome-claude-code-subagents/blob/main/categories/01-core-development/mobile-developer.md | MIT |
| 33 | Mobile X-plat | `flutter-expert.voltagent.md` | **VoltAgent** | https://github.com/VoltAgent/awesome-claude-code-subagents/blob/main/categories/02-language-specialists/flutter-expert.md | MIT |
| 34 | Mobile X-plat | `expo-react-native-expert.voltagent.md` | **VoltAgent** | https://github.com/VoltAgent/awesome-claude-code-subagents/blob/main/categories/02-language-specialists/expo-react-native-expert.md | MIT |
| 35 | iOS native | `ios-expert.furai.md` | **0xfurai** | https://github.com/0xfurai/claude-code-subagents/blob/main/agents/ios-expert.md | MIT |
| 36 | iOS native | `swiftui-expert.furai.md` | **0xfurai** | https://github.com/0xfurai/claude-code-subagents/blob/main/agents/swiftui-expert.md | MIT |
| 37 | iOS native | `swift-expert.furai.md` | **0xfurai** | https://github.com/0xfurai/claude-code-subagents/blob/main/agents/swift-expert.md | MIT |
| 38 | Android native | `android-expert.furai.md` | **0xfurai** | https://github.com/0xfurai/claude-code-subagents/blob/main/agents/android-expert.md | MIT |
| 39 | Android native | `kotlin-expert.furai.md` | **0xfurai** | https://github.com/0xfurai/claude-code-subagents/blob/main/agents/kotlin-expert.md | MIT |
| 40 | DB / data | `database-optimizer.wshobson.md` | **wshobson** | https://github.com/wshobson/agents/blob/main/plugins/comprehensive-review/agents/database-optimizer.md | MIT |
| 41 | DB / data | `database-reviewer.ecc.md` | **ECC** | https://github.com/affaan-m/everything-claude-code/blob/main/agents/database-reviewer.md | MIT |
| 42 | DB / data | `data-engineer.wshobson.md` | **wshobson** | https://github.com/wshobson/agents/blob/main/data-engineer.md | MIT |
| 43 | DevOps / infra | `cloud-architect.voltagent.md` | **VoltAgent** | https://github.com/VoltAgent/awesome-claude-code-subagents/blob/main/categories/03-infrastructure/cloud-architect.md | MIT |
| 44 | DevOps / infra | `kubernetes-specialist.voltagent.md` | **VoltAgent** | https://github.com/VoltAgent/awesome-claude-code-subagents/blob/main/categories/03-infrastructure/kubernetes-specialist.md | MIT |
| 45 | DevOps / infra | `terraform-engineer.voltagent.md` | **VoltAgent** | https://github.com/VoltAgent/awesome-claude-code-subagents/blob/main/categories/03-infrastructure/terraform-engineer.md | MIT |
| 46 | DevOps / infra | `sre-engineer.voltagent.md` | **VoltAgent** | https://github.com/VoltAgent/awesome-claude-code-subagents/blob/main/categories/03-infrastructure/sre-engineer.md | MIT |
| 47 | DevOps / infra | `devops-incident-responder.voltagent.md` | **VoltAgent** | https://github.com/VoltAgent/awesome-claude-code-subagents/blob/main/categories/03-infrastructure/devops-incident-responder.md | MIT |
| 48 | QA / test | `test-automator.voltagent.md` | **VoltAgent** | https://github.com/VoltAgent/awesome-claude-code-subagents/blob/main/categories/04-quality-security/test-automator.md | MIT |
| 49 | QA / test | `qa-expert.voltagent.md` | **VoltAgent** | https://github.com/VoltAgent/awesome-claude-code-subagents/blob/main/categories/04-quality-security/qa-expert.md | MIT |
| 50 | QA / test | `ui-ux-tester.voltagent.md` | **VoltAgent** | https://github.com/VoltAgent/awesome-claude-code-subagents/blob/main/categories/04-quality-security/ui-ux-tester.md | MIT |
| 51 | QA / test | `accessibility-tester.voltagent.md` | **VoltAgent** | https://github.com/VoltAgent/awesome-claude-code-subagents/blob/main/categories/04-quality-security/accessibility-tester.md | MIT |
| 52 | QA / test | `e2e-runner.ecc.md` | **ECC** | https://github.com/affaan-m/everything-claude-code/blob/main/agents/e2e-runner.md | MIT |
| 53 | QA / test | `tdd-guide.ecc.md` | **ECC** | https://github.com/affaan-m/everything-claude-code/blob/main/agents/tdd-guide.md | MIT |
| 54 | Performance | `performance-engineer.wshobson.md` | **wshobson** | https://github.com/wshobson/agents/blob/main/plugins/full-stack-orchestration/agents/performance-engineer.md | MIT |
| 55 | Performance | `performance-optimizer.ecc.md` | **ECC** | https://github.com/affaan-m/everything-claude-code/blob/main/agents/performance-optimizer.md | MIT |
| 56 | Refactor | `refactor-cleaner.ecc.md` | **ECC** | https://github.com/affaan-m/everything-claude-code/blob/main/agents/refactor-cleaner.md | MIT |
| 57 | Refactor | `code-simplifier.ecc.md` | **ECC** | https://github.com/affaan-m/everything-claude-code/blob/main/agents/code-simplifier.md | MIT |
| 58 | Docs | `doc-updater.ecc.md` | **ECC** | https://github.com/affaan-m/everything-claude-code/blob/main/agents/doc-updater.md | MIT |
| 59 | Docs | `docs-lookup.ecc.md` | **ECC** | https://github.com/affaan-m/everything-claude-code/blob/main/agents/docs-lookup.md | MIT |
| 60 | Docs | `docs-architect.wshobson.md` | **wshobson** | https://github.com/wshobson/agents/blob/main/docs-architect.md | MIT |
| 61 | Accessibility | `a11y-architect.ecc.md` | **ECC** | https://github.com/affaan-m/everything-claude-code/blob/main/agents/a11y-architect.md | MIT |
| 62 | Planner / PM | `planner.ecc.md` | **ECC** | https://github.com/affaan-m/everything-claude-code/blob/main/agents/planner.md | MIT |
| 63 | Planner / PM | `requirements-analyst.dlezo.md` | **dl-ezo** | https://github.com/dl-ezo/claude-code-sub-agents/blob/main/requirements-analyst.md | MIT⚠ |
| 64 | Planner / PM | `user-story-generator.dlezo.md` | **dl-ezo** | https://github.com/dl-ezo/claude-code-sub-agents/blob/main/user-story-generator.md | MIT⚠ |
| 65 | Planner / PM | `progress-tracker.dlezo.md` | **dl-ezo** | https://github.com/dl-ezo/claude-code-sub-agents/blob/main/progress-tracker.md | MIT⚠ |
| 66 | Planner / PM | `risk-manager.dlezo.md` | **dl-ezo** | https://github.com/dl-ezo/claude-code-sub-agents/blob/main/risk-manager.md | MIT⚠ |
| 67 | Planner / PM | `stakeholder-communicator.dlezo.md` | **dl-ezo** | https://github.com/dl-ezo/claude-code-sub-agents/blob/main/stakeholder-communicator.md | MIT⚠ |
| 68 | Planner / PM | `uat-coordinator.dlezo.md` | **dl-ezo** | https://github.com/dl-ezo/claude-code-sub-agents/blob/main/uat-coordinator.md | MIT⚠ |
| 69 | Build / utility | `build-error-resolver.ecc.md` | **ECC** | https://github.com/affaan-m/everything-claude-code/blob/main/agents/build-error-resolver.md | MIT |
| 70 | Build / utility | `harness-optimizer.ecc.md` | **ECC** | https://github.com/affaan-m/everything-claude-code/blob/main/agents/harness-optimizer.md | MIT |
| 71 | Build / utility | `error-detective.wshobson.md` | **wshobson** | https://github.com/wshobson/agents/blob/main/error-detective.md | MIT |

> **Footnote on `MIT⚠`** (rows 63-68): dl-ezo declares MIT in its README but has no LICENSE file in the repo. Treated as MIT for §10.2 compatibility, mitigated per §10.4.

> **Origin breakdown (54 imported + 7 secusy-original = 61 unique* files / 71 rows)**:
> - secusy-original: 7
> - ECC: 23 (rows 8, 9, 12, 14–20, 21, 41, 52, 53, 55, 56, 57, 58, 59, 61, 62, 69, 70)
> - wshobson: 11
> - VoltAgent: 18
> - 0xfurai: 5
> - dl-ezo: 6
>
> *Numbering went 1–71 to keep distinct rows; some upstream sources contribute multiple files.

### 7.3 What changes vs. today

- **Keep as-is** (secusy-original orchestrators, hand-tuned for Plane/Mattermost/Sprint workflow): `pm.md`, `frontend.md`, `backend.md`, `qa.md`, `security.md`, `devops.md`, `designer.md`. These have **no upstream** — they were written by the secusy team and the `secusy` tag here is correct.
- **Re-import from upstream and rename with `.ecc.md` suffix** (so future ECC updates can be re-pulled cleanly, and the §9.1 provenance header is reset to upstream truth even for the two files that were locally modified): `architect.md` → `architect.ecc.md`, `code-reviewer.md` → `code-reviewer.ecc.md`, `security-reviewer.md` → `security-reviewer.ecc.md`, `tdd-guide.md` → `tdd-guide.ecc.md`, `e2e-runner.md` → `e2e-runner.ecc.md`, `doc-updater.md` → `doc-updater.ecc.md`, `docs-lookup.md` → `docs-lookup.ecc.md`, `database-reviewer.md` → `database-reviewer.ecc.md`, `refactor-cleaner.md` → `refactor-cleaner.ecc.md`, `performance-optimizer.md` → `performance-optimizer.ecc.md`, `planner.md` → `planner.ecc.md`, `typescript-reviewer.md` → `typescript-reviewer.ecc.md`, `build-error-resolver.md` → `build-error-resolver.ecc.md`, `harness-optimizer.md` → `harness-optimizer.ecc.md`. **The two locally-modified files (`architect.md`, `code-reviewer.md`, `planner.md`) lose their secusy-specific edits in this step — those edits should live in the orchestrators (`pm.md`, `frontend.md`, etc.) instead, not inside the imported file.**
- **Drop entirely** (ECC-origin but not relevant to team scrum): `chief-of-staff.md` (already noted as misfit in `genesis/create-agentic-team.sh:1343`), `loop-operator.md`
- **Add 33 new specialists** spanning frontend implementer, backend implementer, full-stack, mobile (iOS/Android/RN/Flutter/Expo), infra/DevOps, QA, performance, PM lifecycle

### 7.4 Why this composition

1. ECC stays the **review/quality bar** (every reviewer agent is ECC's). Empirically the most-praised single-repo reviewer matrix in the ecosystem (§5).
2. wshobson supplies **implementer + cross-cutting orchestration** (frontend-developer, backend-architect, deployment-engineer, performance-engineer, error-detective, docs-architect, data-engineer). Plugin source structure preserved as bare files.
3. VoltAgent fills **language specialists, infra cluster, and quality testers** that ECC and wshobson are thin on (kotlin/swift specialists exist in voltagent and furai both — voltagent picked for cross-platform, furai for native-only).
4. 0xfurai is the **only repo** with `swiftui-expert` + `ios-expert` + `android-expert` as named files — used solely for mobile-native gap.
5. dl-ezo's PM lifecycle agents replace the gap left by ECC's single `planner.md` for projects with real requirements/UAT phases.
6. senaiverse and vijaythecoder are excluded from default cherry-pick — niche enough that they should be added per-project, not in baseline.
7. obra/superpowers is **NOT** in `.claude/agents/` — it's installed as a skill plugin. Listed in §8 as a separate post-install step.
8. **secusy-original tag is reserved for the 7 orchestrators** (`pm`, `frontend`, `backend`, `qa`, `security`, `devops`, `designer`). Any other file currently sitting under bare names in `.claude/agents/` originated from ECC and is correctly retagged in §7.1.

---

## 8. Cherry-pick install shell script

Save as `genesis/install-curated-agents.sh`. Idempotent — re-running cleans `.claude/agents/_imported/` and re-pulls.

```bash
#!/usr/bin/env bash
# install-curated-agents.sh — populate .claude/agents/ from the curated §7 inventory.
# License: see §10. Each imported file retains its upstream `Source-*:` header (§9).
# Re-run: safe. Removes only files under .claude/agents/_imported/ marker dir.

set -euo pipefail

# ─── 0. Guards ──────────────────────────────────────────────────────────────
[[ -d ".claude/agents" ]] || { echo "Run from repo root with .claude/agents/"; exit 1; }
command -v git >/dev/null || { echo "git required"; exit 1; }
command -v rsync >/dev/null || { echo "rsync required"; exit 1; }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
TARGET=".claude/agents"
MARK="$TARGET/.imported-manifest"
mkdir -p "$TARGET"

# ─── 1. Clean prior import (orchestrators kept) ─────────────────────────────
# Anything tagged with one of our origin tags = previously imported, safe to delete
echo "[1/6] Cleaning prior import…"
find "$TARGET" -maxdepth 1 -type f \
  \( -name '*.ecc.md' -o -name '*.wshobson.md' -o -name '*.voltagent.md' \
     -o -name '*.furai.md' -o -name '*.dlezo.md' -o -name '*.senaiverse.md' \
     -o -name '*.vijaythecoder.md' \) -delete
rm -f "$MARK"

# ─── 2. Clone upstreams (shallow) ───────────────────────────────────────────
echo "[2/6] Cloning upstream repos (shallow)…"
git clone --depth=1 --quiet https://github.com/affaan-m/everything-claude-code.git    "$TMP/ecc"        &
git clone --depth=1 --quiet https://github.com/wshobson/agents.git                    "$TMP/wshobson"   &
git clone --depth=1 --quiet https://github.com/VoltAgent/awesome-claude-code-subagents.git "$TMP/voltagent" &
git clone --depth=1 --quiet https://github.com/0xfurai/claude-code-subagents.git      "$TMP/furai"      &
git clone --depth=1 --quiet https://github.com/dl-ezo/claude-code-sub-agents.git      "$TMP/dlezo"      &
wait

# ─── 3. Helper: copy + inject provenance header ─────────────────────────────
# Adds an HTML-comment provenance block at the very top of each .md.
# Claude Code's frontmatter parser ignores HTML comments before the YAML block,
# so this is a runtime no-op — but on GitHub it makes origin clickable.
import_file () {
  local src_repo="$1" src_path="$2" src_tag="$3" dst_name="$4" src_license="${5:-MIT}"
  local src_full="$TMP/$src_tag/$src_path"
  local dst_full="$TARGET/$dst_name"
  [[ -f "$src_full" ]] || { echo "  MISSING: $src_repo : $src_path"; return 1; }

  local commit_sha
  commit_sha="$(git -C "$TMP/$src_tag" rev-parse HEAD)"
  local source_url="https://github.com/$src_repo/blob/$commit_sha/$src_path"
  local source_url_main="https://github.com/$src_repo/blob/main/$src_path"
  local imported_at
  imported_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

  {
    cat <<EOF
<!--
Source-Repo: $src_repo
Source-Path: $src_path
Source-URL: $source_url_main
Source-Pinned-URL: $source_url
Source-License: $src_license
Source-Commit: $commit_sha
Imported-At: $imported_at
-->
EOF
    cat "$src_full"
  } > "$dst_full"

  printf '%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$dst_name" "$src_repo" "$src_path" "$commit_sha" "$src_license" "$source_url_main" \
    >> "$MARK"
}

echo "[3/6] Importing curated files…"
echo "# .claude/agents import manifest — generated $(date -u +%Y-%m-%dT%H:%M:%SZ)" > "$MARK"
printf '# columns: imported-name\tsource-repo\tsource-path\tcommit-sha\tlicense\tsource-url\n' >> "$MARK"

# ── ECC ────────────────────────────────────────────────────────────────────
import_file affaan-m/everything-claude-code agents/architect.md              ecc architect.ecc.md
import_file affaan-m/everything-claude-code agents/code-architect.md         ecc code-architect.ecc.md
import_file affaan-m/everything-claude-code agents/code-reviewer.md          ecc code-reviewer.ecc.md
import_file affaan-m/everything-claude-code agents/silent-failure-hunter.md  ecc silent-failure-hunter.ecc.md
import_file affaan-m/everything-claude-code agents/security-reviewer.md      ecc security-reviewer.ecc.md
import_file affaan-m/everything-claude-code agents/typescript-reviewer.md    ecc typescript-reviewer.ecc.md
import_file affaan-m/everything-claude-code agents/python-reviewer.md        ecc python-reviewer.ecc.md
import_file affaan-m/everything-claude-code agents/rust-reviewer.md          ecc rust-reviewer.ecc.md
import_file affaan-m/everything-claude-code agents/go-reviewer.md            ecc go-reviewer.ecc.md
import_file affaan-m/everything-claude-code agents/kotlin-reviewer.md        ecc kotlin-reviewer.ecc.md
import_file affaan-m/everything-claude-code agents/flutter-reviewer.md       ecc flutter-reviewer.ecc.md
import_file affaan-m/everything-claude-code agents/database-reviewer.md      ecc database-reviewer.ecc.md
import_file affaan-m/everything-claude-code agents/refactor-cleaner.md       ecc refactor-cleaner.ecc.md
import_file affaan-m/everything-claude-code agents/code-simplifier.md        ecc code-simplifier.ecc.md
import_file affaan-m/everything-claude-code agents/performance-optimizer.md  ecc performance-optimizer.ecc.md
import_file affaan-m/everything-claude-code agents/doc-updater.md            ecc doc-updater.ecc.md
import_file affaan-m/everything-claude-code agents/docs-lookup.md            ecc docs-lookup.ecc.md
import_file affaan-m/everything-claude-code agents/a11y-architect.md         ecc a11y-architect.ecc.md
import_file affaan-m/everything-claude-code agents/planner.md                ecc planner.ecc.md
import_file affaan-m/everything-claude-code agents/tdd-guide.md              ecc tdd-guide.ecc.md
import_file affaan-m/everything-claude-code agents/e2e-runner.md             ecc e2e-runner.ecc.md
import_file affaan-m/everything-claude-code agents/build-error-resolver.md   ecc build-error-resolver.ecc.md
import_file affaan-m/everything-claude-code agents/harness-optimizer.md      ecc harness-optimizer.ecc.md

# ── wshobson ───────────────────────────────────────────────────────────────
import_file wshobson/agents plugins/comprehensive-review/agents/code-reviewer.md       wshobson code-reviewer.wshobson.md
import_file wshobson/agents plugins/comprehensive-review/agents/architect-review.md    wshobson architect-review.wshobson.md
import_file wshobson/agents plugins/comprehensive-review/agents/database-optimizer.md  wshobson database-optimizer.wshobson.md
import_file wshobson/agents plugins/frontend-mobile-development/agents/frontend-developer.md wshobson frontend-developer.wshobson.md
import_file wshobson/agents plugins/backend-development/agents/backend-architect.md    wshobson backend-architect.wshobson.md
import_file wshobson/agents plugins/backend-development/agents/graphql-architect.md    wshobson graphql-architect.wshobson.md
import_file wshobson/agents plugins/full-stack-orchestration/agents/deployment-engineer.md  wshobson deployment-engineer.wshobson.md
import_file wshobson/agents plugins/full-stack-orchestration/agents/performance-engineer.md wshobson performance-engineer.wshobson.md
import_file wshobson/agents data-engineer.md         wshobson data-engineer.wshobson.md
import_file wshobson/agents docs-architect.md        wshobson docs-architect.wshobson.md
import_file wshobson/agents error-detective.md       wshobson error-detective.wshobson.md

# ── VoltAgent ──────────────────────────────────────────────────────────────
import_file VoltAgent/awesome-claude-code-subagents categories/01-core-development/microservices-architect.md voltagent microservices-architect.voltagent.md
import_file VoltAgent/awesome-claude-code-subagents categories/01-core-development/fullstack-developer.md     voltagent fullstack-developer.voltagent.md
import_file VoltAgent/awesome-claude-code-subagents categories/01-core-development/mobile-developer.md        voltagent mobile-developer.voltagent.md
import_file VoltAgent/awesome-claude-code-subagents categories/02-language-specialists/react-specialist.md    voltagent react-specialist.voltagent.md
import_file VoltAgent/awesome-claude-code-subagents categories/02-language-specialists/nextjs-developer.md    voltagent nextjs-developer.voltagent.md
import_file VoltAgent/awesome-claude-code-subagents categories/02-language-specialists/python-pro.md          voltagent python-pro.voltagent.md
import_file VoltAgent/awesome-claude-code-subagents categories/02-language-specialists/nodejs-developer.md    voltagent nodejs-developer.voltagent.md
import_file VoltAgent/awesome-claude-code-subagents categories/02-language-specialists/flutter-expert.md      voltagent flutter-expert.voltagent.md
import_file VoltAgent/awesome-claude-code-subagents categories/02-language-specialists/expo-react-native-expert.md voltagent expo-react-native-expert.voltagent.md
import_file VoltAgent/awesome-claude-code-subagents categories/03-infrastructure/cloud-architect.md           voltagent cloud-architect.voltagent.md
import_file VoltAgent/awesome-claude-code-subagents categories/03-infrastructure/kubernetes-specialist.md     voltagent kubernetes-specialist.voltagent.md
import_file VoltAgent/awesome-claude-code-subagents categories/03-infrastructure/terraform-engineer.md        voltagent terraform-engineer.voltagent.md
import_file VoltAgent/awesome-claude-code-subagents categories/03-infrastructure/sre-engineer.md              voltagent sre-engineer.voltagent.md
import_file VoltAgent/awesome-claude-code-subagents categories/03-infrastructure/devops-incident-responder.md voltagent devops-incident-responder.voltagent.md
import_file VoltAgent/awesome-claude-code-subagents categories/04-quality-security/test-automator.md          voltagent test-automator.voltagent.md
import_file VoltAgent/awesome-claude-code-subagents categories/04-quality-security/qa-expert.md               voltagent qa-expert.voltagent.md
import_file VoltAgent/awesome-claude-code-subagents categories/04-quality-security/ui-ux-tester.md            voltagent ui-ux-tester.voltagent.md
import_file VoltAgent/awesome-claude-code-subagents categories/04-quality-security/accessibility-tester.md    voltagent accessibility-tester.voltagent.md
import_file VoltAgent/awesome-claude-code-subagents categories/04-quality-security/security-auditor.md        voltagent security-auditor.voltagent.md

# ── 0xfurai (mobile native) ────────────────────────────────────────────────
import_file 0xfurai/claude-code-subagents agents/ios-expert.md         furai ios-expert.furai.md
import_file 0xfurai/claude-code-subagents agents/swiftui-expert.md     furai swiftui-expert.furai.md
import_file 0xfurai/claude-code-subagents agents/swift-expert.md       furai swift-expert.furai.md
import_file 0xfurai/claude-code-subagents agents/android-expert.md     furai android-expert.furai.md
import_file 0xfurai/claude-code-subagents agents/kotlin-expert.md      furai kotlin-expert.furai.md

# ── dl-ezo (PM lifecycle) — see §10.4: MIT declared in README only, no LICENSE file
DLEZO_LICENSE="MIT (declared in upstream README; no LICENSE file — see _licenses/LICENSE.dl-ezo-claude-code-sub-agents.NOTE)"
import_file dl-ezo/claude-code-sub-agents requirements-analyst.md       dlezo requirements-analyst.dlezo.md     "$DLEZO_LICENSE"
import_file dl-ezo/claude-code-sub-agents user-story-generator.md       dlezo user-story-generator.dlezo.md     "$DLEZO_LICENSE"
import_file dl-ezo/claude-code-sub-agents progress-tracker.md           dlezo progress-tracker.dlezo.md         "$DLEZO_LICENSE"
import_file dl-ezo/claude-code-sub-agents risk-manager.md               dlezo risk-manager.dlezo.md             "$DLEZO_LICENSE"
import_file dl-ezo/claude-code-sub-agents stakeholder-communicator.md   dlezo stakeholder-communicator.dlezo.md "$DLEZO_LICENSE"
import_file dl-ezo/claude-code-sub-agents uat-coordinator.md            dlezo uat-coordinator.dlezo.md          "$DLEZO_LICENSE"

# ─── 4. Vendor LICENSE files ────────────────────────────────────────────────
echo "[4/6] Vendoring upstream LICENSE files into .claude/agents/_licenses/"
mkdir -p "$TARGET/_licenses"
cp "$TMP/ecc/LICENSE"        "$TARGET/_licenses/LICENSE.affaan-m-everything-claude-code"
cp "$TMP/wshobson/LICENSE"   "$TARGET/_licenses/LICENSE.wshobson-agents"
cp "$TMP/voltagent/LICENSE"  "$TARGET/_licenses/LICENSE.VoltAgent-awesome-claude-code-subagents"
cp "$TMP/furai/LICENSE"      "$TARGET/_licenses/LICENSE.0xfurai-claude-code-subagents"
# dl-ezo: README declares MIT but no LICENSE file — vendor the README excerpt instead
{
  echo "# License notice for dl-ezo/claude-code-sub-agents"
  echo
  echo "Upstream repository does not ship a LICENSE file."
  echo "README.md declares (verbatim):"
  echo
  echo "    ## 📄 License"
  echo "    MIT License - feel free to use, modify, and distribute these agents for any purpose."
  echo
  echo "Source: https://github.com/dl-ezo/claude-code-sub-agents/blob/main/README.md"
  echo "Captured: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
} > "$TARGET/_licenses/LICENSE.dl-ezo-claude-code-sub-agents.NOTE"

# ─── 5. Print manifest summary ─────────────────────────────────────────────
echo "[5/6] Manifest:"
column -t -s $'\t' "$MARK" | head -10
echo "  …(see $MARK for full list)"

# ─── 6. Optional: install superpowers as skill plugin ──────────────────────
echo "[6/6] To install obra/superpowers as a workflow skill plugin, in Claude Code run:"
echo "      /plugin marketplace add anthropics/claude-plugins-official"
echo "      /plugin install superpowers@claude-plugins-official"
echo
echo "Done. $(/bin/ls "$TARGET" | wc -l) files in $TARGET/"
```

Make executable and run:

```bash
chmod +x genesis/install-curated-agents.sh
./genesis/install-curated-agents.sh
```

Re-running pulls the latest commits and rewrites the imported files. Files that begin with the `Source-Repo:` HTML comment are treated as "imported" and removable on next run; orchestrators (no header) are preserved.

---

## 9. Attribution rules — how to cite each imported file

Goal: when this `.claude/agents/` is published as a public GitHub repo, each imported file's origin is unambiguous and auditable, and Anthropic's runtime parser is undisturbed.

### 9.1 Per-file provenance header (required)

Every imported file gets a 7-line HTML comment **above any YAML frontmatter** — Claude Code's parser ignores HTML comments before the frontmatter, so this is invisible at runtime but visible on GitHub. `Source-URL` always points at the upstream `main` branch (so the link keeps working as the upstream evolves); `Source-Pinned-URL` is the immutable commit-pinned permalink (so the *exact* version that was imported can always be retrieved):

```markdown
<!--
Source-Repo: wshobson/agents
Source-Path: plugins/frontend-mobile-development/agents/frontend-developer.md
Source-URL: https://github.com/wshobson/agents/blob/main/plugins/frontend-mobile-development/agents/frontend-developer.md
Source-Pinned-URL: https://github.com/wshobson/agents/blob/7f3a4b2c.../plugins/frontend-mobile-development/agents/frontend-developer.md
Source-License: MIT
Source-Commit: 7f3a4b2c…
Imported-At: 2026-05-05T09:50:00Z
-->
---
name: frontend-developer
description: …
---
…rest of agent body…
```

This is what `import_file` in §8 injects automatically.

### 9.2 Top-level `NOTICE.md`

Add `NOTICE.md` to the repo root listing every upstream + commit SHA + license + clickable file-level URL. Generated from `.claude/agents/.imported-manifest` (6 columns, written by §8 step 3):

```bash
# regenerate-notice.sh
{
  echo "# Third-Party Notices"
  echo
  echo "This repository's \`.claude/agents/\` directory contains files imported from"
  echo "the OSS projects below. Each imported file retains its origin in an HTML"
  echo "comment header (\`Source-Repo\`, \`Source-URL\`, \`Source-Pinned-URL\`,"
  echo "\`Source-License\`, \`Source-Commit\`, \`Imported-At\`), and the upstream"
  echo "LICENSE files are vendored under \`.claude/agents/_licenses/\`."
  echo
  echo "| # | Imported file | Upstream repo | Upstream file (link) | Commit | License |"
  echo "|---|---|---|---|---|---|"
  i=0
  # Skip the 2 header lines, then format. Column order in manifest:
  #   1=imported-name 2=source-repo 3=source-path 4=commit-sha 5=license 6=source-url
  tail -n +3 .claude/agents/.imported-manifest | while IFS=$'\t' read -r name repo path sha license url; do
    i=$((i+1))
    printf '| %s | `%s` | [%s](https://github.com/%s) | [`%s`](%s) | `%s` | %s |\n' \
      "$i" "$name" "$repo" "$repo" "$path" "$url" "${sha:0:8}" "$license"
  done
} > NOTICE.md
```

### 9.3 LICENSE vendoring

Step 4 of the install script copies each upstream LICENSE verbatim into `.claude/agents/_licenses/`. **MIT requires preserving the copyright notice and the permission-notice text** (clauses 1–2). Vendoring the file itself is the minimal-friction way.

### 9.4 README disclosure

Add to repo root `README.md`:

```markdown
## Attribution

Many `.claude/agents/*.md` files in this repo were imported from upstream OSS projects. See [`NOTICE.md`](./NOTICE.md) for the full per-file attribution table and [`/.claude/agents/_licenses/`](./.claude/agents/_licenses/) for the vendored upstream LICENSE files. Imported files carry an HTML comment header with their `Source-Repo`, `Source-Path`, `Source-Commit`, and `Source-License`.
```

### 9.5 Modifications to imported files

If you modify an imported file (e.g., adapt `frontend-developer.wshobson.md` to your project), append a `Modified-By:` line under `Imported-At:` in the header. **Do NOT remove the Source-* lines** — that would breach MIT's attribution requirement. Example:

```markdown
<!--
Source-Repo: wshobson/agents
Source-Path: plugins/frontend-mobile-development/agents/frontend-developer.md
Source-URL: https://github.com/wshobson/agents/blob/main/plugins/frontend-mobile-development/agents/frontend-developer.md
Source-Pinned-URL: https://github.com/wshobson/agents/blob/7f3a4b2c.../plugins/frontend-mobile-development/agents/frontend-developer.md
Source-License: MIT
Source-Commit: 7f3a4b2c…
Imported-At: 2026-05-05T09:50:00Z
Modified-By: secusy team — adapted to Plane/Mattermost workflow on 2026-05-12
-->
```

Better practice for secusy: **don't modify imported files at all**. If a wshobson `frontend-developer.wshobson.md` needs project-specific tweaks (Plane lifecycle, Mattermost mentions), those tweaks belong in the secusy-original orchestrator (`frontend.md`) which then *delegates* to the unmodified wshobson specialist. That preserves the ability to re-run §8's install script and pick up upstream improvements without merge conflicts.

---

## 10. License compatibility — MIT inbound, MIT outbound

### 10.1 Live audit — every candidate repo

Pulled directly from each repo's LICENSE file via `raw.githubusercontent.com` on 2026-05-05:

| Repo | License file | License | Copyright holder | Notes |
|---|---|---|---|---|
| affaan-m/everything-claude-code | `LICENSE` | **MIT** | © 2026 Affaan Mustafa | Standard MIT |
| wshobson/agents | `LICENSE` | **MIT** | © 2024 Seth Hobson | Standard MIT |
| VoltAgent/awesome-claude-code-subagents | `LICENSE` | **MIT** | © 2025 VoltAgent | Standard MIT |
| 0xfurai/claude-code-subagents | `LICENSE` | **MIT** | © 2025 0xfurai | Standard MIT |
| dl-ezo/claude-code-sub-agents | *(no LICENSE file)* | **MIT (declared in README)** | (no holder named) | ⚠ Less defensible — see §10.4 |
| vijaythecoder/awesome-claude-agents | `LICENSE` | **MIT** | © 2024 Awesome Claude Agents Contributors | Standard MIT |
| iannuttall/claude-agents | `LICENSE` | **MIT** | © 2025 ian nuttall | Standard MIT |
| lst97/claude-code-sub-agents | `LICENSE` | **MIT** | © 2025 SIO TOU LAI [Nelson] | Standard MIT |
| senaiverse/claude-code-reactnative-expo-agent-system | `LICENSE` | **MIT** | © 2025 SenaiVerse | Standard MIT |
| bmad-code-org/BMAD-METHOD | `LICENSE` | **MIT** | © 2025 BMad Code, LLC | Not used in §7 (skill-shaped) but listed for completeness |
| obra/superpowers | `LICENSE` | **MIT** | © 2025 Jesse Vincent | Used as plugin, not vendored |

### 10.2 Compatibility verdict

**No conflicts.** Every source is MIT (one is MIT-by-README — see §10.4 mitigations). MIT is *permissive* and explicitly compatible with redistributing under MIT, including modified versions, provided the original copyright notice and permission notice are preserved (the two-paragraph LICENSE body). Outbound MIT for your repo is the natural choice — combining MIT-licensed inputs into an MIT-licensed output requires only:

1. **Preserve original copyright notices** → §9.1 per-file headers + §9.3 vendored LICENSE files
2. **Preserve the MIT permission notice** → §9.3 vendored LICENSE files
3. **Disclaim warranty / liability** → satisfied by *your* MIT LICENSE in the repo root (the standard MIT body covers this)

There is **no copyleft contamination, no GPL ambiguity, no Apache NOTICE-file requirement, no patent-grant divergence**. MIT-into-MIT is the trivial case.

### 10.3 What you would need to change for *any* of these scenarios

If you pulled future agents from a repo with a different license, the picture changes:

| Upstream license | Compatible with MIT outbound? | What you'd need |
|---|---|---|
| MIT / BSD-2 / BSD-3 / ISC | ✅ Yes | Just preserve copyright + license notice |
| Apache-2.0 | ✅ Yes (MIT can redistribute Apache code) | Vendor `LICENSE` **and** any `NOTICE` file; record any patent grants |
| MPL-2.0 | ⚠ File-level copyleft | Modified files stay MPL; can keep original-MPL files alongside MIT files but cannot relicense them to MIT |
| LGPL | ⚠ Copyleft on modifications + linking quirks | Avoid for simple text agents; legal review needed |
| GPL-2.0 / GPL-3.0 | ❌ **Incompatible with MIT outbound** | Cannot relicense; whole repo would have to be GPL |
| AGPL-3.0 | ❌ As GPL plus network use | Same: whole repo would have to be AGPL |
| CC-BY-4.0 | ✅ Compatible (similar attribution) | Preserve attribution; not ideal mixed with code MIT but works for prose/docs |
| CC-BY-SA-4.0 | ❌ ShareAlike clause | Cannot relicense to MIT |
| CC-BY-NC | ❌ Non-commercial restriction | Cannot use in any commercial context |
| Custom / unclear | ❌ Treat as All Rights Reserved | Skip until clarified by upstream |

Translation: as long as you only cherry-pick from MIT/BSD/ISC/Apache-2.0 upstreams, the MIT outbound strategy holds. Any addition outside that set should trigger a review against this table.

### 10.4 dl-ezo edge case — MIT declared in README only

dl-ezo/claude-code-sub-agents has **no LICENSE file** in the repo. `README.md` says "MIT License - feel free to use, modify, and distribute these agents for any purpose." This is the maintainer's clear intent but is legally weaker than a LICENSE file because:

- A judge reading the bare repo without README context might default to "all rights reserved" (GitHub's default for unlicensed repos)
- The README-declared MIT lacks a copyright holder name → ambiguous about whose attribution to preserve

**Mitigation strategy** (already implemented in §8 step 4):

1. Vendor `.claude/agents/_licenses/LICENSE.dl-ezo-claude-code-sub-agents.NOTE` quoting the README verbatim with timestamp + permalink
2. In §9.1 file headers for those 6 imports, set `Source-License: MIT (declared in upstream README)`
3. (Optional, recommended) Open a PR upstream adding a proper `LICENSE` file. Keeps everyone safer
4. (Optional) If 3 doesn't land in 30 days, replace the dl-ezo agents with hand-written equivalents derived from §3's runner-up (BMAD-METHOD skills translated to agents) — that removes the legally weaker dependency entirely

For now the risk is **low** (clear public intent + small specialized files + §10.5 final defense), but worth tightening if the resulting repo is used in commercial contexts.

### 10.5 Recommended outbound license setup for your public repo

```
your-repo/
├── LICENSE                              ← MIT, your copyright, repo-wide
├── NOTICE.md                            ← §9.2 generated table
├── README.md                            ← §9.4 attribution section
└── .claude/
    └── agents/
        ├── architect.ecc.md             ← imported, with §9.1 header
        ├── …                            (53 more)
        ├── _licenses/                   ← §9.3 vendored upstream LICENSEs
        │   ├── LICENSE.affaan-m-everything-claude-code
        │   ├── LICENSE.wshobson-agents
        │   ├── LICENSE.VoltAgent-awesome-claude-code-subagents
        │   ├── LICENSE.0xfurai-claude-code-subagents
        │   └── LICENSE.dl-ezo-claude-code-sub-agents.NOTE
        └── .imported-manifest           ← §8 SHA-pinned manifest
```

Root `LICENSE` text (standard MIT, with one extra paragraph at the end making the layered nature explicit):

```
MIT License

Copyright (c) 2026 <YOUR NAME>

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

---

This repository includes third-party content imported from MIT-licensed
upstreams. Each such file carries a `Source-Repo:` header naming its origin,
and the upstream LICENSE files are vendored under
`.claude/agents/_licenses/`. See `NOTICE.md` for the full attribution table.
```

That's the full discharge of MIT inbound + MIT outbound obligations. Compatible. Auditable. No surprises.

### 10.6 Quick checklist before publishing

- [ ] Run `./genesis/install-curated-agents.sh` clean (no MISSING errors). Expect 47 imported rows.
- [ ] Run `./genesis/regenerate-notice.sh` and review `NOTICE.md` — every imported file should appear with a clickable upstream link.
- [ ] Confirm `.claude/agents/_licenses/` has 4 LICENSE files + 1 NOTE
- [ ] Confirm root `LICENSE` (MIT, your copyright)
- [ ] Confirm `README.md` has the §9.4 Attribution section
- [ ] `grep -L 'Source-Repo:' .claude/agents/*.{ecc,wshobson,voltagent,furai,dlezo}.md` — should return empty (every imported file must have a header).
- [ ] `grep -L 'Source-URL: https://' .claude/agents/*.{ecc,wshobson,voltagent,furai,dlezo}.md` — should also be empty (every imported file must have a clickable URL).
- [ ] `grep -L 'Source-Pinned-URL: https://' .claude/agents/*.{ecc,wshobson,voltagent,furai,dlezo}.md` — should also be empty (commit-pinned link required for archival).
- [ ] `grep -l 'Source-Repo:' .claude/agents/{pm,frontend,backend,qa,security,devops,designer}.md` — should return empty (orchestrators are secusy-original, must NOT carry an upstream header).
- [ ] `git diff` — confirm no edits to vendored LICENSE files
- [ ] (Optional) Open PR upstream at dl-ezo to add a proper LICENSE file

---

## 11. Ralph (snarktank/ralph) — fit assessment & positioning

> Ad-hoc analysis added 2026-05-05 in response to a question: can Ralph's functionality be expressed as `.claude/agents/*.md` and slotted into the §7 architecture? Conclusion below; verdict is **partial fit — Ralph is mostly a runtime loop and a workflow skill, not an agent. It belongs alongside the agent set as a thin orchestrator layer, not inside it.**

### 11.1 What Ralph actually is

Source: [github.com/snarktank/ralph](https://github.com/snarktank/ralph), MIT-licensed, based on [Geoffrey Huntley's Ralph pattern](https://ghuntley.com/ralph/). Five components:

| Ralph component | Form | Role |
|---|---|---|
| `ralph.sh` (~3.5KB bash) | Outer driver | `for i in 1..N; spawn fresh AI instance with prompt.md/CLAUDE.md; check for `<promise>COMPLETE</promise>`; archive on branch change` |
| `prompt.md` / `CLAUDE.md` | Single-page agent prompt | The instructions each fresh iteration receives — read PRD, pick highest-priority story where `passes:false`, implement, run checks, commit, append to `progress.txt`, set `passes:true`, repeat |
| `prd.json` | Persistent state | Backlog of user stories with `{id, title, acceptanceCriteria, priority, passes:false}` |
| `progress.txt` | Persistent state | Append-only log + a "Codebase Patterns" header section that future iterations read first |
| `skills/prd/` + `skills/ralph/` | Two Claude Code skills | (a) generate PRD from feature description, (b) convert PRD markdown → `prd.json` |

**Operating model**: each iteration is a *fresh* Claude/Amp process — no persistent agent context. State carries through git history + `progress.txt` + `prd.json`. Stop condition is the LLM emitting `<promise>COMPLETE</promise>`. The "loop until all stories pass" loop is in bash, not in the agent.

### 11.2 Does Ralph slot into §7's `.claude/agents/*.md` shape?

**Mostly no — and that's the right answer.**

Ralph is structurally three things, and only one of them is an agent file:

1. **An outer driver script** (`ralph.sh`) — this is **not** an agent at all. It belongs in `genesis/` or `scripts/`, not `.claude/agents/`.
2. **A single agent prompt** (`prompt.md` / `CLAUDE.md`) — this *could* be expressed as a `.claude/agents/ralph-iterator.md`, but it's a one-shot self-driving prompt, not a Task-tool-callable specialist. Calling it through Claude Code's subagent dispatch (`Task(subagent_type="ralph-iterator")`) doesn't make sense — it expects to *be* the top-level conversation.
3. **Two skills** (`skills/prd`, `skills/ralph`) — these are skills, not agents. They go in `~/.claude/skills/` or via `/plugin install ralph-skills@ralph-marketplace`. They have no native home in `.claude/agents/`.

**The §7 architecture cannot host Ralph as a peer to `frontend.md` / `backend.md`.** Ralph operates *one level above* the agent layer — it's a session driver. Trying to force it into `.claude/agents/` would either lose its essence (the fresh-context loop) or break the §7 contract that everything in that folder is a Task-callable subagent.

### 11.3 Where Ralph **does** fit — three positioning options

#### Option A — Sprint executor for a single feature branch (recommended)

Ralph becomes a **fourth path** beside the existing two paths in [`CLAUDE.md` "Git 브랜치 전략" section](/work/secusy/CLAUDE.md):

| Path | Operator | Use case |
|---|---|---|
| Path A | Human maintainer | Direct main commits |
| Path B | Agent team (frontend/backend/qa…) | Plane-ticket-driven feat branches |
| **Path C (new)** | **Ralph driver** | **PRD-driven autonomous burn-down of a multi-story feature inside one sprint** |

Concretely:
- Ralph's `prd.json` ≈ secusy's Plane Cycle (a list of stories to burn down)
- Ralph's `progress.txt` ≈ secusy's Mattermost #scrum-secusy thread + `agent-handoff` skill
- Ralph's `<promise>COMPLETE</promise>` ≈ pm's `/sprint-end` trigger

Mapping to file system:

```
genesis/
├── ralph.sh                     ← copied from snarktank/ralph (MIT vendored)
├── ralph-prompt.md              ← extends Ralph's CLAUDE.md with secusy quality gates
└── install-curated-agents.sh    ← unchanged from §8

.claude/
├── agents/                      ← unchanged 54-file curated set from §7
└── skills/
    ├── prd/                     ← from snarktank/ralph (or via /plugin install)
    └── ralph/                   ← from snarktank/ralph
```

Ralph's prompt then *calls* secusy's existing agents per iteration:
- Each Ralph iteration picks one story, then dispatches to `frontend.wshobson.md` / `backend-architect.wshobson.md` / etc. via Task tool
- After implementation: dispatch to `code-reviewer.ecc.md` + `qa.md` for the In-Review gate
- Ralph's loop = "outer scrum master that keeps the iteration alive across context windows"; secusy agents = "individual contributors per story"

#### Option B — Skills only (lightest touch, ship today)

Skip `ralph.sh` and the loop entirely. Just install the two skills (`/prd`, `/ralph`) globally:

```bash
# In Claude Code
/plugin marketplace add snarktank/ralph
/plugin install ralph-skills@ralph-marketplace
```

Now anywhere in the secusy project, a human or pm agent can run `/prd "feature description"` to draft a PRD, then `/ralph tasks/prd-feature.md` to turn it into the secusy backlog (adapt the schema converter to emit Plane intake tickets instead of `prd.json`).

This is a **drop-in upgrade for `pm.md`'s requirement-intake step** without taking on Ralph's loop runtime.

#### Option C — Embed Ralph's *idea* into pm.md (no vendoring)

Don't import Ralph at all. Steal the four most valuable concepts and bake them into existing secusy agents:

| Ralph concept | Secusy equivalent today | Action |
|---|---|---|
| Fresh-context iterations | Each Plane-ticket-driven Task() spawn | Already done |
| `progress.txt` Codebase Patterns header | `MEMORY.md` + `CLAUDE.md` | Already done |
| `<promise>COMPLETE</promise>` stop signal | pm's `/sprint-end` | Already done |
| `prd.json` priority + passes flags | Plane state machine (Backlog→Todo→Done) | Already done |
| AGENTS.md auto-update by every iteration | "When you discover reusable patterns, append to nearest CLAUDE.md / docs/" rule | **Missing — add to pm.md or create a memory-curator agent** |

The one genuinely novel idea Ralph brings is the **enforce-pattern-extraction** rule: "every iteration MUST update the nearest `AGENTS.md` / `CLAUDE.md` if it discovered a reusable pattern". Adding this as a hook (PostToolUse on Edit) or as a step-9 in `frontend.md` / `backend.md` would capture Ralph's distinctive signal without vendoring its loop.

### 11.4 Recommended positioning for secusy

**Adopt Options B + C; defer Option A.**

Reasoning:
- Option A's loop **conflicts with secusy's existing assignee-based Plane lifecycle**. Ralph's "one fresh instance picks the highest priority unfinished story" is exactly what `pm`'s sprint-dispatch already does — except secusy does it across *multiple parallel agents* (frontend, backend, qa) instead of one sequential loop. Ralph's serial model is *less* powerful than what secusy already runs. Forcing it in would actually slow down sprint throughput.
- Option B is essentially free — two skills installed, used at requirements-gathering time only. It strengthens `pm.md` without touching the agent runtime.
- Option C captures Ralph's one missing idea (forced pattern-extraction) as a small rule addition to existing agents.

### 11.5 If Option A is chosen later

Should secusy decide to add Ralph as an autonomous burn-down path (e.g., for off-hours or unattended weekend runs), the additions to §7 would be:

```
.claude/skills/
├── prd/                                   ← from /plugin install ralph-skills@ralph-marketplace
└── ralph/                                 ← same

genesis/
├── ralph.sh                               ← vendored from snarktank/ralph (MIT, attribution required)
├── ralph-prompt.secusy.md                 ← secusy's wrapper around Ralph's CLAUDE.md
│                                            (calls Plane API instead of editing prd.json,
│                                             posts to Mattermost instead of progress.txt)
└── install-curated-agents.sh              ← unchanged
```

§9 attribution rules would extend: any vendored Ralph file gets the same `Source-Repo: snarktank/ralph` header. License is MIT — fully compatible with the §10 strategy. No outbound license changes needed.

A *new* `.claude/agents/ralph-driver.md` would NOT be created — Ralph drives at the *session* level, above the subagent layer.

### 11.6 Verdict — one-line summary for the doc reader

Ralph is **NOT an agent** — it's a 50-line bash loop + a prompt template + two skills. The pattern it embodies (PRD → fresh-context iterations → `progress.txt` learnings) is **already implemented in secusy at a higher fidelity** by the Plane-Cycle + agent-ownership architecture. Adopt Ralph's two **skills** (`/prd`, `/ralph`) and its **mandatory inline knowledge capture rule** (§12 — Option B + C). Skip the loop driver (Option A) — it's a regression versus secusy's parallel agent model. License: MIT, no attribution drama if any of it is vendored.

---

## 12. Ideas worth borrowing from Ralph (reference cheat sheet)

> Companion to §11. §11 is "should we adopt Ralph?" — answer was *partially*. This section is the **concrete checklist of ideas to lift from Ralph** if/when we choose to adopt them, with each idea boiled down to (a) what Ralph does, (b) the closest standard SE term, (c) how to express it in secusy's existing structure. Source for every quote: [snarktank/ralph](https://github.com/snarktank/ralph) (MIT) — `prompt.md`, `CLAUDE.md`, `ralph.sh`, `README.md`.

### 12.1 Mandatory inline knowledge capture (the one truly novel idea)

**What Ralph does** ([CLAUDE.md "Update AGENTS.md Files" section](https://github.com/snarktank/ralph/blob/main/CLAUDE.md)):

> Before committing, check if any edited files have learnings worth preserving in nearby AGENTS.md files:
> 1. Identify directories with edited files
> 2. Check for existing AGENTS.md
> 3. Add valuable learnings — if you discovered something future developers/agents should know: API patterns or conventions specific to that module · Gotchas or non-obvious requirements · Dependencies between files · Testing approaches for that area · Configuration or environment requirements

And ([CLAUDE.md "Consolidate Patterns" section](https://github.com/snarktank/ralph/blob/main/CLAUDE.md)):

> If you discover a **reusable pattern** that future iterations should know, add it to the `## Codebase Patterns` section at the TOP of progress.txt (create it if it doesn't exist).

**Closest standard SE term**: *living documentation* (Cyrille Martraire) enforced as a *commit gate*. Adjacent academic term: *knowledge externalization* (Nonaka & Takeuchi SECI model — making tacit knowledge explicit).

**Why it's powerful**: most "documentation drift" doesn't happen because devs don't *want* to document — it happens because there's no forcing function. Ralph turns "update the docs if you discovered a pattern" from *aspiration* into *commit-blocking step*.

**Filter rule (also from Ralph CLAUDE.md, equally important)**:

> **Do NOT add:** Story-specific implementation details · Temporary debugging notes · Information already in progress.txt
> Only update CLAUDE.md if you have **genuinely reusable knowledge** that would help future work in that directory.

So the rule is bidirectional: *forced* to capture genuine patterns, *forbidden* from polluting with story-specific noise.

**How to graft into secusy**:

Add a step-7.5 to `frontend.md`, `backend.md`, `qa.md`, `security.md`, `devops.md`, `designer.md` orchestrators (right before "PR 생성"):

```markdown
## 7.5 커밋 직전 — 패턴 캡처 게이트 (Ralph-style)

자기 변경분을 살펴보고, 다음 중 하나라도 발견했으면 **커밋 전에** 가장 가까운 디렉터리의
`CLAUDE.md` 또는 `docs/<feature>.md` 에 한 줄 추가한다:

- 이 모듈만의 API/네이밍/설정 컨벤션
- 다른 파일과의 비명시적 의존 ("X 바꾸면 Y도 같이 바꿔야 함")
- 비자명한 gotcha (Plane state machine, hook 동작 순서, env var 종속 등)
- 이 영역 특유의 테스트 셋업 절차

다음은 절대 추가하지 않는다:
- 이번 티켓 한정의 구현 디테일
- 임시 디버깅 노트
- 이미 `progress.txt` / Mattermost 스레드에 있는 정보

판단 기준 한 줄: **"내가 다음 sprint에 이 디렉터리 처음 본 다른 agent라면 이 정보가 시간을
아껴줄까?"** 답이 yes 일 때만 커밋. (Ralph CLAUDE.md `Update AGENTS.md Files` §)
```

**Optional 강제 hook** (PostToolUse on Edit, 실험적):

```jsonc
// .claude/settings.json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Edit|Write",
        "command": "node .claude/hooks/pattern-capture-reminder.js",
        "description": "If the edit touches a directory with no recent CLAUDE.md update, prompt the agent to consider updating it before commit"
      }
    ]
  }
}
```

훅이 차단(`exit 2`)하지는 않고, agent에게 stderr로 한 줄 알림만 띄우는 부드러운 강제. Ralph의 prompt 안 instruction → secusy의 hook으로 매체 변환.

### 12.2 Append-only progress log with consolidated patterns header

**What Ralph does** ([prompt.md](https://github.com/snarktank/ralph/blob/main/prompt.md)):

```
## Codebase Patterns                       ← 항상 최상단, 일반화된 학습만
- Use `sql<number>` template for aggregations
- Always use `IF NOT EXISTS` for migrations
- Export types from actions.ts for UI components

## [Date/Time] - [Story ID]                ← append-only timeline
- What was implemented
- Files changed
- **Learnings for future iterations:** ...
---
```

**파일 구조의 핵심**: 두 영역이 분리되어 있다.
- **상단** = 일반화·재사용 가능 (consolidated)
- **하단** = 시간순 append-only (chronological)

**Closest standard SE term**: *append-only event log* + *materialized projection* — 이벤트소싱 패턴의 단순한 적용. 또는 *engineering decision log* (ADR의 가벼운 형태).

**How to graft into secusy**:

secusy는 이미 등가물을 갖고 있다 — Mattermost #scrum-secusy 스레드 + Plane 이슈 코멘트가 *chronological log*, `MEMORY.md` + 디렉터리별 `CLAUDE.md` 가 *consolidated projection*. 따라서 새로 만들 필요 없음. 다만 **두 영역의 분리 규칙**을 더 명시적으로 강제하는 게 가치가 있다:

| Ralph location | Secusy 등가물 | 어디에 올라가야 하는 내용 |
|---|---|---|
| `## Codebase Patterns` (top of progress.txt) | `MEMORY.md` + `<dir>/CLAUDE.md` + `docs/design-system.md` | 재사용 가능한 패턴, 컨벤션, 결정 |
| `## [Date/Time] - [Story ID]` (timeline) | Mattermost 이슈 스레드 + Plane 이슈 코멘트 | 이번 작업 한정 진행 상황 |

위 표는 §12.1 step-7.5 보강을 위해 `pm.md` 또는 `CLAUDE.md` 본문에 그대로 넣을 수 있다.

### 12.3 Right-sized stories rule

**What Ralph does** ([README.md "Small Tasks" section](https://github.com/snarktank/ralph/blob/main/README.md)):

> Each PRD item should be small enough to complete in one context window. If a task is too big, the LLM runs out of context before finishing and produces poor code.
>
> **Right-sized stories**: Add a database column and migration · Add a UI component to an existing page · Update a server action with new logic · Add a filter dropdown to a list
>
> **Too big (split these)**: "Build the entire dashboard" · "Add authentication" · "Refactor the API"

**Closest standard SE term**: *INVEST criteria* (eXtreme Programming) — 특히 *S = Small*. 또는 *vertical slice* + *story splitting* (Mike Cohn).

**Why it matters here**: secusy의 estimate 옵션은 이미 `3분 / 5분 / 10분 / 15분 / 30분 / 45분`인데, **상한 45분**이 LLM context 한계와 우연히 잘 맞는다 — Ralph의 "한 context window 안에 끝나야 함" 룰의 secusy 버전이다. 명시적으로 적어두면 estimate 결정이 더 일관돼진다.

**How to graft into secusy**:

`pm.md` 의 "Estimate" 섹션 또는 `CLAUDE.md` 의 "티켓 관리 원칙" 에 한 줄 추가:

```markdown
### Estimate 가이드 (Ralph "right-sized stories" 원칙)
- 한 티켓 = 한 agent의 한 context window 안에 끝나야 함 (45분 상한 = ~한 세션)
- 끝나지 않을 가능성이 보이면 **분할**: 30분 + 30분 두 티켓이 60분 한 티켓보다 항상 낫다
- Ralph 가이드의 "too big" 예시: "전체 대시보드 구축", "인증 추가", "API 리팩토링" → 이런 사이즈는 Story가 아니라 Epic(=Module)
```

### 12.4 Stop signal as explicit completion contract

**What Ralph does** ([prompt.md](https://github.com/snarktank/ralph/blob/main/prompt.md)):

> If ALL stories are complete and passing, reply with:
> `<promise>COMPLETE</promise>`
> If there are still stories with `passes: false`, end your response normally.

`ralph.sh` 가 stdout에서 그 토큰을 grep해서 루프 종료. **machine-readable completion contract** — LLM이 자유 텍스트로 "다 했어요!"라고 말해도 무시되고, 약속된 정확한 토큰만 신뢰.

**Closest standard SE term**: *explicit termination protocol* / *sentinel value*. 또는 분산 시스템의 *commit token* / *ack message*.

**How to graft into secusy**:

secusy는 이미 더 강한 등가물을 갖고 있다 — Plane 상태 머신 (`In Review → Done`) + qa 멘션 (`@qa.{project} merge·Done 처리 가능`). 텍스트 토큰보다 훨씬 안정적이라 그대로 가져올 가치는 없다. **다만** Mattermost 스레드 자동 파싱이 필요한 경우(예: pm 봇이 sprint-status를 자동 집계)에는 정확한 토큰 컨벤션이 도움이 된다. 후보:

```
[SCZ-N] ✅ Done — <한 줄 요약>      ← 이미 사용 중 (.claude/rules/agent-ownership.md §3)
```

이미 있으니 변경 없음. 단지 Ralph가 같은 패턴을 쓴다는 점이 검증.

### 12.5 Branch-scoped run archiving

**What Ralph does** ([ralph.sh](https://github.com/snarktank/ralph/blob/main/ralph.sh) lines 38-58):

```bash
# Archive previous run if branch changed
if [ "$CURRENT_BRANCH" != "$LAST_BRANCH" ]; then
  ARCHIVE_FOLDER="$ARCHIVE_DIR/$DATE-$FOLDER_NAME"
  mkdir -p "$ARCHIVE_FOLDER"
  cp "$PRD_FILE" "$ARCHIVE_FOLDER/"
  cp "$PROGRESS_FILE" "$ARCHIVE_FOLDER/"
  # Reset progress file for new run
fi
```

작업 중인 feature 브랜치가 바뀌면 직전 run의 prd/progress를 자동으로 `archive/YYYY-MM-DD-feature/` 로 옮기고 새 progress.txt를 시작.

**Closest standard SE term**: *snapshot isolation* between runs / *generation-scoped state*.

**How to graft into secusy**:

이미 등가물 존재 — Plane Cycle 종료 시 `/sprint-end` 가 Mattermost에 회고 요약 게시 + Cycle archive. secusy에는 추가할 게 없다. **단**, sprint 단위가 아니라 **개별 PR/feat 브랜치 단위로 학습을 보존**하고 싶다면 Ralph 패턴을 흉내낼 수 있다:

```
docs/sprint-archives/
└── 2026-W18-SCZ-23-design-tokens/
    ├── plane-snapshot.json   ← 그 cycle 시작 시점 이슈 상태
    ├── mattermost-thread.md  ← 해당 이슈 스레드 export
    └── learnings.md          ← consolidated patterns from this run
```

`/sprint-end` skill 에 옵션 단계로 추가 가능. 우선순위는 낮음.

### 12.6 한눈에 보는 Borrow / Skip 표

| Ralph 구성요소 | secusy에 가져올까? | 이유 |
|---|---|---|
| `/prd` skill (PRD 생성) | **Borrow** (§11 Option B) | pm의 요구사항 수집 단계를 강화. 비용 0 |
| `/ralph` skill (PRD→JSON 변환) | **Borrow** (스키마 적응) | Plane intake 자동 생성으로 변환 시 가치 |
| §12.1 mandatory knowledge capture rule | **Borrow** (필수) | secusy에 빠진 유일한 가치, 도입 비용 작음 |
| §12.2 append-only + consolidated 분리 규칙 | **Borrow as documentation** | 이미 구현되어 있지만 명문화 가치 |
| §12.3 right-sized stories | **Borrow as guidance line** | estimate 가이드라인에 한 줄 추가 |
| §12.4 stop signal token | **Skip** | secusy는 Plane state machine으로 이미 더 강하게 함 |
| §12.5 branch-scoped archiving | **Skip (선택적)** | `/sprint-end` 로 충분, 추가 가치 한계적 |
| `ralph.sh` 루프 driver | **Skip (§11 Option A 거부)** | 병렬 agent 모델 대비 회귀 |
| `prompt.md` / `CLAUDE.md` 단일 prompt | **Skip** | secusy의 분산 agent 모델과 충돌 |
| `prd.json` schema | **Skip** | Plane이 이미 단일 진실 소스 |
| `progress.txt` 파일 | **Skip** | Mattermost #scrum-secusy + Plane comments로 대체 |

### 12.7 도입 시 첫 액션 3개 (구체적)

위 분석에 기반해 secusy에 실제로 적용하려면 다음 3개를 하면 된다 (모두 작은 PR 한 번씩):

1. **§12.1 step-7.5 추가**: `frontend.md`, `backend.md`, `qa.md`, `security.md`, `devops.md`, `designer.md`, `pm.md` 의 본문에 "커밋 직전 — 패턴 캡처 게이트" 섹션을 같은 텍스트로 추가. 이 7개 orchestrator 가 모두 이 규칙을 따르게 만드는 게 핵심.
2. **§12.3 한 줄 추가**: `CLAUDE.md` 의 "Estimate (분 단위 직접 값)" 섹션에 "한 티켓 = 한 agent context window 분량" 가이드라인 한 줄 추가.
3. **§11.4 Option B 실행**: `/plugin marketplace add snarktank/ralph` + `/plugin install ralph-skills@ralph-marketplace` 를 `genesis/setup-plugins.md` 에 등록. pm agent가 intake 정제할 때 `/prd` 사용 가능해짐.

이 3개로 Ralph 가치의 ~95% 를 0.5일 분량 작업으로 흡수한다. 나머지 5%(loop driver, prd.json 스키마)는 secusy 구조와 충돌하므로 의도적으로 버린다.
