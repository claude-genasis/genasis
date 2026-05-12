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

**모든 설명은 한글로 한다** (사용자 요구사항 §13).

### 테스트 환경 (test bed)

- **테스트 베드 경로**: `/work/agenteams/team-ex/`
  - `CLAUDE.md` — 본 섹션의 SSOT 가 되는 사용자 지침 (= 외부 신규
    사용자 시점 프로토콜 13 개 조항). 본 문서는 그 내용과 일치해야 한다.
  - `PLAN.md` — 현재 사이클에서 처리할 항목 + 종료 후 사용자 액션 가이드
    (없으면 새로 생성).
  - `genasis-test-log.md` — 누적 테스트 로그.
  - `install.sh` — `curl` 로 받은 install.sh 본체 (테스트 산출물).
  - 그 외 사이클 산출물 (프로젝트 폴더, 스크린샷, Playwright 스크립트 등) 은
    각 사이클 시작 시 정리.

- **수정 대상 저장소**: `/work/genasis/` 자체. 별도 clone 하지 않고
  이 저장소 안에서 직접 수정 → commit → push.

- **테스트에 사용하는 바이너리**: GitHub Release `claude-genasis/genasis`
  의 musl-static 바이너리 (`install.sh` 가 받아오는 것).
  **Cargo 로 직접 빌드한 산출물을 테스트에 사용하지 않는다** — 사용자
  요구사항 §12 (clone 금지, 배포된 바이너리만 사용). 단, fix 검증 시
  로컬 release 빌드를 "프리뷰" 로 쓰는 것은 허용되지만, 사이클 종료
  시점의 시각화 / 사용자 확인 자료는 반드시 install.sh 가 받은 GitHub
  Release 바이너리로 재현되어야 한다.

### 루프 단계 (사용자 13 개 조항과 매핑)

1. **PLAN.md 확인** — `/work/agenteams/team-ex/PLAN.md` 가 존재하면
   그 안의 항목부터 처리한다. 없으면 단계 2 로 가서 새로 발견한다.

2. **테스트 베드 정리** (사용자 §12) — 사이클 시작 시점에
   `/work/agenteams/team-ex/` 하위 디렉터리는 **모두 삭제**하여
   외부 신규 사용자 환경을 재현한다. 루트의 `CLAUDE.md`, `PLAN.md`,
   `genasis-test-log.md` 만 남긴다.

3. **새 사용자 시뮬레이션** (사용자 §1–§3) — `/work/agenteams/team-ex/`
   에서 다음을 위에서 아래로 그대로 실행한다:
   - `https://github.com/claude-genasis/genasis` 의 README 를 받는 입장.
   - README §Quick Path 1–5 단계 전부 (install.sh → init --trial →
     example prd → init → monitor).
   - README §Step-by-Step Guide 의 모든 옵션 (Option A 트라이얼,
     Option B 자체 호스트).
   - 분기에서 본 저장소 내부 지식으로 잘라먹지 않는다. README 그대로.

4. **브라우저 검증 — Playwright 사용** (사용자 §8) — Mattermost,
   Plane, trial-app 웹 UI 의 의도 동작을 **반드시 Playwright** 로
   자동화하여 점검한다. 사용자가 눈으로 확인할 수 있도록 스크린샷을
   `/work/agenteams/team-ex/screenshots/` 에 저장한다.
   - 권장 환경: `/home/bravo/miniconda3/envs/llms/bin/python` (이미
     `playwright` 가용) + `/usr/bin/google-chrome` (시스템 크롬).
   - 브라우저 자동화 결과는 단순 PASS/FAIL 이 아니라 카드 / 메시지 /
     UI 요소 별 보이는지 여부를 JSON 으로 dump 해서 PLAN.md 에 인용
     가능하게 한다.
   - **검증 항목 1 — 카드 상태 정합성** (TR-1 정책): `genasis publish`
     호출 후 칸반 4 컬럼별 카드 수가 narrative 와 일치하는지 — Done
     ≥ 4 (init seeded 3 + publish seeded 1), InProgress = 0, Todo = 0.
     불일치하면 `ensureIssue` state-sync 또는 publish seed 가 깨진 것
     으로 간주.
   - **검증 항목 2 — Reactive loop** (TR-2 정책): 자가테스트가 sim
     채널에 `[human]` 메시지를 INSERT 한 뒤 60 초 안에 에이전트 응답
     (`actor != [human|user|...]`) 이 같은 채널에 도착해야 한다.
     도착 안 하면 `genasis listen` daemon 이 띄워져 있지 않거나
     `claude --print` 호출이 실패한 것 → 즉시 결함 등록.
   - **검증 항목 3 — 카드 transition 의도 정합성**: 사람이 "X 완료"
     라고 채팅하면 listen daemon 의 `maybe_transition_card` 가 관련
     카드를 Done 으로 옮겨야 한다. Playwright 가 transition 전후
     스냅샷을 캡처해서 비교.

5. **결함 기록 — PLAN.md** (사용자 §3, §5) — "README 대로 했는데
   의도한 결과가 안 나오는" 모든 케이스를 PLAN.md 에 항목으로 추가한다.
   각 결함은 다음을 포함:
   - ID (D-001, D-002, … 누적)
   - 심각도 (Critical / High / Medium / Low)
   - 재현 절차 (명령 그대로)
   - 기대 vs 실제 (실제는 응답 body / 스크린샷 / Playwright JSON 인용)
   - 추정 root cause
   - 해결 계획

6. **수정 — `/work/genasis/` 안에서** (사용자 §4–§6) — 코드 / 프롬프트 /
   템플릿을 고친다. 외부 clone 없이 본 저장소가 곧 수정 대상이다.
   - Bilingual mirror 정책 준수 (위 §Bilingual Mirror Policy).
   - `cargo fmt` + `cargo clippy` + 관련 단위 테스트 통과.
   - `progress.md` / `progress.ko.md` 양쪽에 사이클 로그 추가.
   - 패치 버전 bump (`Cargo.toml`) — 사용자가 명시한 버전을 우선,
     명시 없으면 0.0.1 단위 patch bump.

7. **Push & CI 모니터링** — 변경을 메인 브랜치에 push 한 뒤
   `gh run list --workflow=release.yml --limit 3` 으로 release workflow
   가 `ok` 가 될 때까지 기다린다. **무한 polling 금지** —
   `ScheduleWakeup` 으로 270–1800 초 간격을 두고 재진입한다.

8. **배포본 회수 & 재테스트** — release 가 publish 되면 단계 2 로 가서
   테스트 베드를 다시 정리한 뒤, `install.sh` 가 받아오는 바이너리가
   새 버전인지 `genasis --version` 으로 확인하고 단계 3 부터 반복.

9. **사용자 확인 자료 정리 — PLAN.md** (사용자 §10, §11) — 사이클 종료
   직전 PLAN.md 에 사용자가 결과를 직접 확인할 수 있는 자료를 모두
   적는다. 누락 금지 항목:
   - **링크** — trial-app Live URL, Plane 워크스페이스 URL,
     Mattermost team / 채널 URL, 스크린샷 경로, Playwright 스크립트
     경로.
   - **자격증명** — 위 링크들로 로그인 / 접속이 필요하다면 id / pw /
     token 을 함께 적는다. token-as-capability 인 경우 (예: trial-app)
     URL 자체가 자격증명이라는 점도 명시.
   - **사용자 다음 액션** — "이걸 열어서 X 를 확인하세요", "필요하면
     `genasis publish` 재호출", "운영자 재배포가 필요한 경우 명령" 등
     구체적 행동을 한글로 제시.

10. **종료 조건** (사용자 §7) — 다음 조건을 모두 만족할 때만 사이클 종료:
    - PLAN.md 의 open 결함 목록 비어있음.
    - README Quick Path / Step-by-Step 이 처음부터 끝까지 통과.
    - 위 §4 의 검증 항목 1·2·3 (카드 정합성 / reactive loop / transition
      의도) 모두 Playwright 결과로 PASS.
    - 그 외에는 무한 반복.

### 행동 원칙

- **README 가 진실** — 사용자는 README 만 본다. "README 에는 X 라고
  적혀 있는데 실제로는 Y 가 맞다" 라는 상황이 발견되면 코드를 README 에
  맞추거나 README 를 코드에 맞춘다 (둘 중 사용자 가치가 큰 쪽 선택).

- **clone 금지 / 바이너리만** (사용자 §12) — 새 사용자 시점 재현이
  무너지므로 테스트 행위 자체는 `git clone genasis` 로 시작하지 않는다.
  `install.sh` 가 GitHub Release 에서 받는 바이너리로 충분히 재현
  가능해야 하고, 그렇지 않다면 그것이 결함이다. (예: D-008 — Self-host
  Option B 가 `cd servers/` 부터 시작이라 install.sh 만 받은 사용자는
  진입 불가 → README 보강.)

- **Playwright 의무** (사용자 §8, §9) — 사람이 들어가서 확인할 수 있는
  근거 (스크린샷 + URL) 를 항상 남긴다. CLI 출력만으로 "동작했다" 라고
  결론짓지 않는다. SSE / 라이브 업데이트가 있는 UI 일수록 필수.

- **상위 시스템 (Plane / Mattermost) 변경 금지** — 자가테스트는 호스팅
  Plane / Mattermost 인스턴스에 데이터를 만든다. 그 외부 시스템의
  설정 변경 권한은 사용자에게 있고, 에이전트는 데이터 정리 (`genasis
  trial cleanup` 등) 만 한다.

- **CI 우회 금지** — `--no-verify`, i18n drift 미러 skip, clippy
  allow-by-default 추가 등은 **사용자가 명시 허락한 경우** 에만.

- **재현 불가능 결함은 보고만** — 한 번 보이고 사라진 결함은 PLAN.md 에
  "재현 불가 — 다음 사이클에서 재관찰" 로 기록만 하고 추측 수정 금지.

- **외부 디펜던시 핑계 금지** — "호스팅 인스턴스가 stale 이라 못 한다"
  류의 결론은 그 자체로 직접 진단 + 우회 경로 제공 의무로 이어진다.
  binary 측에서 `GENASIS_TRIAL_URL` 같은 escape hatch 를 만들거나 로컬
  docker 띄워서 end-to-end 증명하는 등 사용자가 외부 액션 없이도
  검증할 수 있게 한다.

### 신규 사이클 진입 명령

사용자가 `자가개발 및 테스트 착수` (또는 "self-test go") 라고 말하면
Claude Code 는 이 섹션의 단계 1 부터 즉시 시작한다. 별도 확인 질문 없이.

> 이 섹션은 외부 PLAN.md / CLAUDE.md 파일 (예: 다른 사용자 환경의
> `~/rnd/agenteams/team01/...`) 에 의존하지 않는다. 본 저장소 + 테스트
> 베드만으로 자족적으로 돌아가야 한다. `/work/agenteams/team-ex/CLAUDE.md`
> 가 사용자 SSOT 이고, 본 섹션은 그 내용을 항상 반영한다.
