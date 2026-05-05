> 한국어: [`../ko/ADR/ADR-012-debug-history-feedback-loop.md`](../ko/ADR/ADR-012-debug-history-feedback-loop.md)

# ADR-012: Debug History — Field Drift as Feedback Loop for Self-Improvement

## Status

Proposed (2026-05-05).

## Context

Genasis is a **meta-tool**: it generates and manages agentic team
configurations inside real-world projects. Users inevitably modify these
generated files to:
- Fix bugs in overlay templates (wrong lifecycle commands, missing env vars)
- Adapt to project-specific workflows (custom sprint cadence, non-standard
  Plane labels, extra Mattermost channels)
- Extend agent instructions beyond what Genasis templates provide
- Work around genasis limitations or edge cases

These modifications represent **the most valuable signal for improving
Genasis** — they tell us exactly where the tool falls short and what real
teams actually need. Currently this signal is lost: users fix things
locally and never report upstream.

### Problem statement

How can Genasis systematically collect field modifications to its own
generated output, without compromising user source code security, and feed
those modifications back into Genasis development with minimal (ideally
zero) developer effort?

### Design principles

1. **Always-on, zero-config collection** — drift tracking must happen by
   default with no setup required.
2. **Security-first** — user source code, secrets, and project identity
   must never leave the local machine unless explicitly opted in.
3. **Minimal developer effort** — submitting feedback should be one
   command; using feedback for genasis improvement should work with
   existing Claude Code workflows.
4. **Self-contained for Claude Code** — the debug history must be
   structured so that Claude Code can read it directly when working on
   genasis, without needing external tools or human interpretation.

## Alternatives

| Alternative | Decision | Reason |
|---|---|---|
| (a) GitHub Issues template (manual bug reports) | Rejected | Requires developer effort; most drift is never reported; no structured format for Claude Code to parse |
| (b) Telemetry service (phone-home) | Rejected | Privacy concerns; requires hosted infra; users distrust telemetry |
| (c) **Local manifest + diff + opt-in submit** | **Accepted** | Zero-infra, security-by-default, structured for machine consumption |
| (d) Git hook in user project that auto-PRs | Rejected | Too invasive; couples genasis to user's git workflow |
| (e) Overlay checksum warning only (no collection) | Rejected | Detects drift but discards the content — wastes the signal |

## Decision

### Architecture: Manifest-Drift-Submit Pipeline

```
┌─────────────────────────────────────────────────────────┐
│  USER PROJECT (always local, always automatic)          │
│                                                         │
│  genasis attach / init                                  │
│    └─► .claude/genasis/.manifest.json                   │
│         (SHA-256 of every managed file at install time) │
│                                                         │
│  genasis doctor / any CLI invocation                    │
│    └─► drift detection (compare live files vs manifest) │
│    └─► .claude/genasis/.drift-log/<timestamp>.jsonl     │
│         (structured diff records, append-only)          │
│                                                         │
│  genasis debug collect                                  │
│    └─► ~/.genasis/debug-history/<project-hash>/         │
│         └─► <timestamp>.patch.json                      │
│              (anonymised, source-stripped, overlay-only) │
└─────────────────────────────────────────────────────────┘
          │
          │  genasis debug submit (OPT-IN, interactive confirm)
          ▼
┌─────────────────────────────────────────────────────────┐
│  GENASIS REPO                                           │
│                                                         │
│  debug-history/                                         │
│    ├── index.jsonl      (append-only patch registry)    │
│    ├── patches/                                         │
│    │   ├── 2026-05-05_a1b2c3d4.patch.json              │
│    │   ├── 2026-05-06_e5f6g7h8.patch.json              │
│    │   └── ...                                          │
│    └── analysis/                                        │
│        ├── clusters.md  (auto-generated pattern groups) │
│        └── proposed-fixes.md (Claude Code proposals)    │
│                                                         │
│  .claude/skills/debug-review.md                         │
│    (skill that reads debug-history/ and proposes fixes) │
└─────────────────────────────────────────────────────────┘
```

### 1. Manifest Generation (at `attach` / `init` time)

When genasis creates or injects overlay files, it records a manifest:

```json
{
  "genasis_version": "0.2.0",
  "agents_catalog_version": "1.3.0",
  "attached_at": "2026-05-05T14:30:00Z",
  "lang": "ko",
  "files": {
    ".claude/genasis/skills/plane-ops.md": {
      "sha256": "a1b2c3d4...",
      "template_source": "agents/skills/plane-ops.md.tera",
      "size_bytes": 2048
    },
    ".claude/agents/frontend.md": {
      "fence_sha256": "e5f6g7h8...",
      "fence_start_line": 5,
      "fence_end_line": 22
    }
  }
}
```

Key points:
- Only tracks files genasis manages (overlay scope)
- For agent files, only tracks the marker-fenced section (not user content)
- Stored at `.claude/genasis/.manifest.json`

### 2. Drift Detection (passive, every CLI invocation)

On every `genasis` command (including `doctor`, `monitor`, etc.), the CLI
silently compares current file state against the manifest. If drift is
detected:

```jsonl
{"ts":"2026-05-06T09:15:00Z","file":".claude/genasis/skills/plane-ops.md","type":"content_modified","old_hash":"a1b2c3d4","new_hash":"f9e8d7c6","diff_lines":4}
{"ts":"2026-05-06T09:15:00Z","file":".claude/genasis/hooks/session-start.sh","type":"deleted","old_hash":"11223344"}
{"ts":"2026-05-07T11:00:00Z","file":".claude/genasis/skills/custom-deploy.md","type":"added","new_hash":"55667788"}
```

This is appended to `.claude/genasis/.drift-log/current.jsonl` (local only,
never committed by user). Cost: one SHA-256 per managed file per CLI call
(< 1ms for typical 20-file overlay).

### 3. Debug Collect (explicit, local aggregation)

```bash
genasis debug collect
```

This command:
1. Reads `.drift-log/current.jsonl`
2. For each modified file, generates a **unified diff** of overlay-scoped
   content only
3. Strips any content that matches security exclusion patterns:
   - Lines containing `TOKEN`, `SECRET`, `KEY`, `PASSWORD`, `CREDENTIAL`
     (case-insensitive) → replaced with `[REDACTED]`
   - Any absolute paths → replaced with `<PROJECT_ROOT>/...`
   - `.env` variable values → replaced with `[ENV_VALUE]`
4. Generates a `patch.json`:

```json
{
  "schema_version": 1,
  "project_hash": "one-way-hash-of-project-path",
  "genasis_version": "0.2.0",
  "agents_catalog_version": "1.3.0",
  "os": "linux-x86_64",
  "lang": "ko",
  "drift_period_days": 2,
  "patches": [
    {
      "file": "skills/plane-ops.md",
      "template_source": "agents/skills/plane-ops.md.tera",
      "change_type": "content_modified",
      "diff": "@@ -15,3 +15,5 @@\n-기존 Plane 상태: Open → In Progress → In Review → Done\n+기존 Plane 상태: Open → In Progress → In Review → QA → Done\n+## QA 단계 추가\n+- QA 담당자가 확인 후 Done 전이 허용",
      "likely_reason": "workflow_extension"
    }
  ],
  "user_comment": null
}
```

5. Saves to `~/.genasis/debug-history/<project-hash>/<timestamp>.patch.json`

### 4. Debug Submit (opt-in, explicit, with preview)

```bash
genasis debug submit [--all | --latest | --file <path>]
```

Flow:
1. Shows the exact JSON payload that will be submitted
2. Prompts: "이 내용을 genasis 개선에 제출하시겠습니까? (y/N)"
3. User can optionally add a comment explaining the change
4. Submits via one of:
   - **GitHub Issue** (auto-created with `debug-history` label) — works
     without genasis repo write access
   - **PR to `debug-history/patches/`** — if user has fork/write access

### 5. Self-Improvement Machinery (in genasis repo)

#### 5a. `/debug-review` skill

A Claude Code skill at `.claude/skills/debug-review.md` that:
- Reads all patches in `debug-history/patches/`
- Clusters them by affected template/file
- Identifies recurring patterns (e.g., "12 users added a QA stage to
  plane-ops lifecycle")
- Proposes template changes and drafts PRs
- Updates `debug-history/analysis/clusters.md`

#### 5b. `GENASIS.md` self-reference

The `GENASIS.md` protocol contract (injected into user projects) will
include a note:

```markdown
## Debug History

이 overlay는 자동으로 변경 사항을 추적합니다.
- 수정된 내용은 로컬에만 저장됩니다 (외부 전송 없음)
- `genasis debug collect` — 변경 요약 생성
- `genasis debug submit` — genasis 개선에 기여 (선택)
```

#### 5c. Analysis automation

When working on genasis in Claude Code, the agent should:
1. Check `debug-history/patches/` for relevant signals before modifying
   templates
2. Reference specific patch IDs when proposing changes ("this addresses
   drift pattern seen in patches a1b2, c3d4, e5f6")
3. After fixing a template, tag the related patches as `resolved` in
   `debug-history/index.jsonl`

### 6. Security Model

| Layer | Protection |
|---|---|
| Collection scope | Only `.claude/genasis/` and marker-fenced sections — never `src/`, `lib/`, `app/`, tests, etc. |
| Secret stripping | Regex-based redaction of tokens/keys/passwords before any export |
| Path anonymisation | Absolute paths replaced; project identified only by irreversible hash |
| Opt-in submission | Nothing leaves the machine without explicit `debug submit` + confirmation |
| No binary/blob | Only text diffs of markdown/shell/toml files — no compiled artifacts |
| Payload preview | Full JSON shown to user before submission — no hidden fields |
| Rate limiting | Max 1 submission per project per day (prevent accidental spam) |

### 7. Enabling Claude Code to use debug-history without extra tooling

The key insight: **debug-history patches are just structured JSON files in
the repo.** Claude Code already knows how to read files. No special MCP
server, no external API, no custom skill infrastructure needed beyond a
simple skill prompt.

The `/debug-review` skill prompt:

```markdown
Read all files in debug-history/patches/ and debug-history/index.jsonl.
For each patch:
1. Identify which template/overlay the change targets
2. Determine if the change is a bug fix, workflow extension, or
   project-specific adaptation
3. For bug fixes and workflow extensions that appear in ≥2 patches:
   propose a template change
4. Draft the change as an Edit to the relevant .tera template
5. Update debug-history/analysis/clusters.md with findings
```

This requires **zero new tools** — just file reads and edits that Claude
Code already does.

### 8. Contribution Governance — Data-Only PR Model

External contributors (fork users and collaborators) interact with
debug-history through a strict separation of concerns:

```
┌────────────────────────────────────────────────────────────────┐
│  CONTRIBUTORS (humans)                                         │
│                                                                │
│  ALLOWED:                                                      │
│    • PR to debug-history/patches/ (submit patch.json only)     │
│    • Add user_comment to their patch explaining context        │
│    • Open Issues tagged [debug-history] with context           │
│                                                                │
│  NOT ALLOWED:                                                  │
│    • Modify templates (.tera) based on debug data              │
│    • Modify overlay source files based on debug data           │
│    • Modify analysis/ or clusters.md                           │
│    • Directly act on debug patches to change genasis code      │
└────────────────────────────────────────────────────────────────┘
          │
          │  patch.json accumulates
          ▼
┌────────────────────────────────────────────────────────────────┐
│  MAINTAINER (automated development via Claude Code)            │
│                                                                │
│  1. /debug-review reads accumulated patches                    │
│  2. Claude Code clusters, analyses, proposes fixes             │
│  3. Maintainer reviews auto-generated PRs                      │
│  4. Merged fixes ship in next genasis release                  │
│  5. Resolved patches tagged in index.jsonl                     │
└────────────────────────────────────────────────────────────────┘
```

#### Why this separation is critical

| Risk if contributors modify code directly | Mitigation via data-only model |
|---|---|
| Malicious template injection (e.g., injecting commands into overlay hooks that run on all genasis users) | Contributors never touch executable templates — they only submit inert JSON describing what changed |
| Inconsistent fix quality (contributor fixes their case but breaks others) | Maintainer's Claude Code sees ALL patches, can validate a fix doesn't contradict other users' drift |
| Review burden explosion (reviewing template logic PRs from many contributors) | Reviewing a `patch.json` is trivial — it's structured data with a known schema |
| Supply chain attack surface | debug-history/patches/ is pure data; CI can validate schema without executing anything |

#### CI enforcement

```yaml
# .github/workflows/debug-history-pr.yml
# Validates PRs that touch debug-history/
- Only files matching debug-history/patches/*.patch.json are allowed
- JSON schema validation (schema_version, required fields)
- No executable content (reject if diff contains shebang, backticks in non-diff context, etc.)
- Auto-label: [debug-history]
- Auto-assign: maintainer
# NO API keys, NO secrets — pure data validation only
```

#### Maintainer's automated development flow

The maintainer (repo owner) uses **local Claude Code** (not CI, not API)
with the `/debug-review` skill to process accumulated patches:

1. **Trigger**: maintainer opens Claude Code locally in the genasis repo
2. **Input**: all unresolved patches in `debug-history/patches/`
3. **Process**: `/debug-review` skill reads patches, proposes template Edits
4. **Review**: maintainer reviews changes in-session, accepts or iterates
5. **Commit**: maintainer commits and pushes the fix
6. **Close loop**: merged fixes → tag patches as resolved → next release

**No API keys used. No cloud automation.** The maintainer's Claude Code
Pro subscription is the only resource needed. This ensures:
- **Zero risk from contributors** — they can only add data
- **Zero API cost** — Claude Code Pro covers all analysis
- **Full maintainer control** — every change reviewed in-session
- **Maximum signal extraction** — all patches are machine-analysed
- **Audit trail** — every fix links back to the patches that motivated it

## CLI Surface Addition

```
genasis debug
├── status              현재 프로젝트의 drift 요약 (몇 개 파일 변경, 마지막 collect 시점)
├── collect             drift → anonymised patch.json 생성
├── submit              opt-in 제출 (GitHub Issue or PR)
├── log                 .drift-log 내용 열람
└── reset               manifest를 현재 상태로 갱신 (drift 히스토리 초기화)
```

## Consequences

### Positive
- Genasis gets structured, machine-readable field feedback with zero
  developer effort (beyond the initial `debug submit`)
- Template improvements are grounded in real usage patterns, not guesses
- Claude Code can autonomously propose improvements by reading patch files
- Users who submit get their fixes reflected in future genasis versions
- Security is default-safe (local only until explicit opt-in)

### Negative
- Manifest adds ~2KB to `.claude/genasis/` per project
- SHA comparison on every CLI call adds ~1ms overhead (negligible)
- `debug-history/` in the genasis repo will grow over time — needs
  periodic archival (propose: archive patches older than 6 months into
  `debug-history/archive/YYYY-MM/`)

### Risks
- Users may submit patches containing project-specific terminology that
  could reveal business context → mitigated by preview + redaction
- Patch volume could overwhelm the `/debug-review` skill → mitigated by
  clustering and frequency thresholds (only propose changes for patterns
  seen ≥2 times)

## Implementation Plan

| Phase | Milestone | Scope |
|---|---|---|
| P1 | M15 | Manifest generation at attach/init + drift detection on every CLI call |
| P2 | M15 | `genasis debug status/collect/log/reset` commands |
| P3 | M16 | `genasis debug submit` + GitHub Issue creation |
| P4 | M16 | `/debug-review` skill + `debug-history/` repo structure |
| P5 | M17 | Analysis automation + `clusters.md` generation |
