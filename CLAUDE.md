# CLAUDE.md — Project Instructions for Genasis

> This file is loaded automatically by Claude Code. It contains project-wide
> conventions and constraints that all agents / sessions must follow.

---

## Core Philosophy — The North Star

Genasis exists to **make AI agents first-class team members alongside
humans** by embedding them into the collaboration tools humans already use
(Plane for issues, Mattermost for chat). Every feature, design decision,
and code change must be evaluated against this mission:

1. **Human-agent seamless collaboration**: Agents must participate in the
   same messengers, issue boards, and sprint ceremonies that human team
   members use — not in a separate "AI sandbox." The goal is that a human
   reviewing a Plane board or Mattermost channel cannot (and need not)
   distinguish whether a given update came from a human or an agent.

2. **Non-destructive adoption for existing teams**: Teams already running
   agentic workflows (ECC, knowledge-work-plugins, claude-code-templates,
   custom `.claude/agents/`) must be able to bolt Genasis on **without
   rewriting their existing agent definitions**. The overlay model (marker
   fences) exists precisely to honour this.

3. **Turnkey bootstrap for new teams**: Teams with zero agentic experience
   must get a fully functional agentic team (`genasis init`) that is
   immediately wired into Plane + Mattermost, ready to collaborate with
   humans from minute one.

4. **Agents operate through human-facing channels only**: Every agent
   action (status update, question, code review request, blocker
   escalation) flows through the same Plane tickets and Mattermost threads
   that humans read. No hidden side-channels.

### How to apply this philosophy

When proposing a new feature, refactor, or architectural change:
- Ask: "Does this bring agents closer to being natural team members in
  human collaboration tools?"
- If the answer is no, the proposal must include a justification for why
  the deviation serves the mission indirectly, or it should be rejected.
- If an alternative exists that better serves human-agent collaboration,
  propose it as a critical counter-suggestion — even if the original idea
  is technically elegant.
- Favour designs where a human PM, designer, or developer interacts with
  agents **through the same UX they already use** rather than through
  CLI-only or developer-only interfaces.

---

## Bilingual Mirror Policy

This project maintains **English ↔ Korean parallel documents**. When one
file in a mirror pair is modified, the corresponding file in the other
language **must be updated in the same commit** (or the immediately
following commit at most) to maintain structural and content parity.

### Mirror pairs (authoritative list)

| English (source of truth for code docs) | Korean (source of truth for planning) |
|---|---|
| `README.md` | `README.ko.md` |
| `blueprint.md` | `blueprint.ko.md` |
| `progress.md` | `progress.ko.md` |
| `CONTRIBUTING.md` | `CONTRIBUTING.ko.md` |
| `docs/ARCHITECTURE.md` | `docs/ko/ARCHITECTURE.md` |
| `docs/PROVIDERS.md` | `docs/ko/PROVIDERS.md` |
| `docs/MIGRATION-FROM-GENESIS.md` | `docs/ko/MIGRATION-FROM-GENESIS.md` |
| `docs/TOKEN-ECONOMICS.md` | `docs/ko/TOKEN-ECONOMICS.md` |
| `docs/MONITOR.md` | `docs/ko/MONITOR.md` |
| `docs/impact-of-multilang-prompts.md` | `docs/ko/impact-of-multilang-prompts.md` |
| `docs/TUTORIAL.md` | `docs/ko/TUTORIAL.md` |
| `docs/TESTING.md` | `docs/ko/TESTING.md` |
| `docs/MIGRATE-PG-CONSOLIDATION.md` | `docs/ko/MIGRATE-PG-CONSOLIDATION.md` |
| `docs/ADR/ADR-*.md` | `docs/ko/ADR/ADR-*.md` |

### Rules

1. **Structural parity**: Both files must have the same section
   headings, the same sub-step items (translated), and the same
   status markers (`[x]`, `[ ]`, `[s]`, etc.).
2. **Content parity**: Meaning must match. Verbatim word-for-word
   translation is not required, but no section may be present in one
   file and absent in the other.
3. **Single-commit rule**: If you edit `progress.ko.md`, you must
   also bring `progress.md` to parity before finishing. The same
   applies in the reverse direction.
4. **Cross-link header**: Every mirror file must start with a
   cross-link to its counterpart:
   - English: `> 한국어: [filename.ko.md](filename.ko.md)`
   - Korean: `> English: [filename.md](filename.md)`
5. **CI enforcement**: `scripts/check-i18n-drift.sh` warns on PRs and
   hard-fails on release-prep when drift is detected.
6. **New mirror files**: When creating a new English doc that warrants
   a Korean mirror (or vice versa), create both in the same commit
   and add the pair to this table.

### Scope

This policy applies to **all** `.md` documentation files that have a
declared mirror pair. It does NOT apply to:
- Code comments (English only)
- Commit messages (English only, Conventional Commits)
- Rust doc comments (English only)
- Template `.tera` files (already split by `templates/{en,ko}/`)
- `i18n/*.yml` locale bundles (managed by `lint-i18n` key parity)

## Conventions

- Rust: `cargo fmt` + `cargo clippy` before commit.
- Commits: Conventional Commits (`feat / fix / docs / chore / i18n`).
- New user-facing strings: `t!()` macro, both `en.yml` and `ko.yml`.
- ADRs: Korean SSOT in `docs/ko/ADR/`, English mirror in `docs/ADR/`.
- Progress tracking: `progress.ko.md` is the operational SSOT (full
  checklists); `progress.md` is its structural mirror in English.
  Both must stay in sync per the bilingual mirror policy above.

---

## Debug History — Field Feedback Loop

Genasis is a **meta-tool** that generates and manages agentic team
configurations inside real projects. Inevitably, users modify the
generated overlay files to fix bugs or adapt to project-specific needs.
These modifications are **invaluable signal** for improving Genasis itself.

### Concept: Drift-as-Feedback

```
User project (.claude/genasis/)
  │
  ├── Initial state (recorded at attach time as manifest)
  ├── Current state (live files)
  └── Drift = diff(manifest, current)
        │
        ▼
  genasis debug collect
        │  (strips source code, keeps only overlay-scoped diffs)
        ▼
  ~/.genasis/debug-history/<project-hash>/<timestamp>.patch
        │
        ▼
  genasis debug submit  (opt-in: pushes anonymised patches to genasis repo)
        │
        ▼
  genasis/debug-history/  (in genasis repo — curated field patches)
        ▼
  Claude Code reads debug-history/ when working on genasis to inform fixes
```

### Security constraints

- **NEVER** include user source code (`src/`, `lib/`, `app/`, etc.)
- **NEVER** include secrets (`.env`, tokens, credentials)
- **ONLY** diff files within `.claude/genasis/` and marker-fenced
  sections of `.claude/agents/*.md`
- Project identity is a one-way hash (not reversible to repo name/path)
- `debug submit` is always **opt-in** and shows the exact payload before
  sending

### How this enables genasis self-improvement

1. `debug-history/` patches in this repo serve as **regression seeds** —
   Claude Code can read them to understand what real users needed to fix.
2. A `/debug-review` skill (planned) will summarise accumulated patches,
   propose template/overlay improvements, and draft PRs automatically.
3. The manifest comparison runs **by default** (debug mode always on) so
   drift is silently tracked locally even if never submitted — zero
   developer effort to collect the data.

### Contribution governance (Data-Only PR Model)

- **Contributors** may ONLY submit `debug-history/patches/*.patch.json`
  files via PR. They must NOT modify templates, overlay source, or
  analysis files based on debug data.
- **Maintainer** processes accumulated patches via Claude Code automated
  development (`/debug-review` skill), reviews auto-generated PRs, and
  merges fixes.
- This separation ensures: zero supply-chain risk from contributors,
  consistent fix quality across all users, minimal review burden.
- See `docs/ADR/ADR-012-debug-history-feedback-loop.md` §8 for full
  rationale.

---

## 자가개발 및 테스트 (Self-Development & Testing Loop)

Genasis 는 사용자가 자신의 프로젝트에 agentic team 을 붙이는 도구이기
때문에, **외부 신규 사용자의 시점에서 README 가이드를 그대로 따라가는
실사용 테스트** 가 가장 강력한 회귀 검증 수단이다. Claude Code 세션은
다음 루프를 **스스로** 수행해서 발견된 결함을 이 저장소에 즉시 반영한다.

### 테스트 환경 (test bed)

- 테스트 베드 경로: `/work/agenteams/team-ex/`
  - `CLAUDE.local.md` — 7단계 테스트 프로토콜 (SSOT)
  - `genasis-test-log.md` — 누적 테스트 로그
  - `PLAN.md` — 현재 사이클에서 수정해야 할 항목 (없으면 새로 생성)
  - `quickpath-test/` , `selfhost-test/` — 실제 attach 결과물이 누적되는
    프로젝트 폴더 (각 사이클 시작 시 정리 가능)
- 작업 저장소(=수정 대상): `/work/genasis/` (clone 한 게 아니라 그 자체)
- 빌드 산출물: GitHub Release `claude-genasis/genasis` 의 musl-static
  바이너리 (`install.sh` 가 이걸 받는다)

### 루프 단계

1. **PLAN.md 확인** — `/work/agenteams/team-ex/PLAN.md` 가 존재하면
   그 안의 항목부터 처리한다. 없으면 단계 2 로 가서 새로 발견한다.
2. **새 사용자 시뮬레이션** — `/work/agenteams/team-ex/` 에서 README 의
   Quick Path 와 Step-by-Step Guide 를 **위에서 아래로** 그대로 실행한다.
   외부 사용자가 README 만 보고 따라하는 상황을 재현해야 하므로, 본
   저장소의 내부 지식으로 분기를 건너뛰지 않는다.
3. **결함 기록** — Mattermost / Plane 웹 UI 까지 확인해서 "README 대로
   했는데 의도한 결과가 안 나오는" 모든 케이스를 PLAN.md 에 항목으로
   추가한다 (재현 절차 + 기대 / 실제 + 영향도).
4. **수정 — `/work/genasis/`** 에서 코드 / 프롬프트 / 템플릿을 고친다.
   - Bilingual mirror 정책 준수.
   - `cargo fmt` + `cargo clippy` + 관련 단위 테스트 통과.
   - `progress.md` / `progress.ko.md` 에 사이클 로그 추가.
   - 패치 버전 bump (`Cargo.toml`) — 사용자가 명시한 버전을 우선,
     명시 없으면 0.0.1 단위 patch bump.
5. **Push & CI 모니터링** — 변경을 메인 브랜치에 push 한 뒤
   `gh run list --limit 5` / `gh run watch` 로 release workflow 가 `ok`
   가 될 때까지 기다린다. **무한 polling 금지** — `ScheduleWakeup` 으로
   적절한 간격을 두고 재진입한다.
6. **배포본 회수 & 재테스트** — release 가 publish 되면 테스트 베드의
   `install.sh` 가 받아오는 바이너리가 새 버전인지 `genasis --version`
   으로 확인한 뒤 단계 2 부터 다시.
7. **종료 조건** — PLAN.md 가 비어 있고 README Quick Path / Step-by-Step
   가 처음부터 끝까지 통과하면 사이클을 종료한다. 그 외에는 무한 반복.

### 행동 원칙

- **README 가 진실** — 사용자는 README 만 본다. "README 에는 X 라고
  적혀 있는데 실제로는 Y 가 맞다" 라는 상황이 발견되면 코드를 README 에
  맞추거나 README 를 코드에 맞춘다 (둘 중 사용자 가치가 큰 쪽 선택).
- **데이터 보존** — `/work/agenteams/team-ex/quickpath-test` 와
  `selfhost-test` 는 회귀 검증용 fixture 다. 함부로 `rm -rf` 하지 않고,
  필요하다면 각 사이클 시작 시 git stash 같은 방식으로 보존한다.
- **상위 시스템 (Plane / Mattermost) 변경 금지** — 자가테스트는 호스팅
  Plane / Mattermost 인스턴스에 데이터를 만든다. 그 외부 시스템의
  설정 변경 권한은 사용자에게 있고, 에이전트는 데이터 정리 (`genasis
  trial cleanup` 등) 만 한다.
- **CI 우회 금지** — `--no-verify`, i18n drift 미러 skip, clippy
  allow-by-default 추가 등은 **사용자가 명시 허락한 경우** 에만.
- **재현 불가능 결함은 보고만** — 한 번 보이고 사라진 결함은 PLAN.md 에
  "재현 불가 — 다음 사이클에서 재관찰" 로 기록만 하고 추측 수정 금지.

### 신규 사이클 진입 명령

사용자가 `자가개발 및 테스트 착수` (또는 "self-test go") 라고 말하면
Claude Code 는 이 섹션의 단계 1 부터 즉시 시작한다. 별도 확인 질문 없이.

> 이 섹션은 외부 PLAN.md / CLAUDE.md 파일 (예: 다른 사용자 환경의
> `~/rnd/agenteams/team01/...`) 에 의존하지 않는다. 본 저장소 + 테스트
> 베드만으로 자족적으로 돌아가야 한다.
