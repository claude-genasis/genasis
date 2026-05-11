> 한국어: [`../ko/ADR/ADR-017-trial-app-showcase-model.md`](../ko/ADR/ADR-017-trial-app-showcase-model.md)

# ADR-017: Trial-App Showcase Model — Example PRD as the Demo Path

## Status

Proposed (2026-05-11). Supersedes the scripted-demo half of ADR-013's
trial-app contract; layers a new "showcase" capability on top of
ADR-016's per-team multi-tenancy.

## Context

After ADR-016 the trial-app reliably scopes each user's sandbox by
`team_token`. But the user-facing experience still has a credibility
problem the multi-tenancy work could not fix on its own:

1. **The scripted "Try it" tab is indistinguishable from real work.**
   Today the trial-app exposes three tabs — `Try it` (scripted
   demo), `Live trial` (real `genasis init --trial` sandbox), and
   `Apply` (sign-up form). The scripted demo animates the *same*
   kanban + chat widgets that the live mode uses, so a first-time
   visitor cannot tell which one is "what the agents really do" vs.
   "a canned animation". The two tabs compete for attention and
   neither is convincing on its own.

2. **`genasis example prd` emits a generic task-tracker PRD.** The
   reference PRD that ships with the CLI describes a vague "task
   status" feature. Agents asked to implement it produce a generic
   kanban — which is visually identical to the trial-app itself, so
   the user has no way to tell "this is the agentic team's work"
   apart from "this is the trial-app's built-in widgets". The demo
   eats its own dogfood.

3. **There is no end-state in the trial flow.** Even after agents
   finish a sprint, nothing in the trial-app changes. The user has
   no "the team built this" moment — the sandbox stays a sandbox
   forever.

4. **The hosted Plane/Mattermost path is mis-linked.** Both READMEs
   point at `trial.realstory.blog` (without `mmplane-`), which has
   been incorrect since ADR-016. The Apply tab also reads "Apply" /
   "신청하기", framing it as a generic application rather than what
   it actually is: borrowing real Plane + Mattermost on the
   operator's server.

## Decision

Four coordinated changes that together convert the trial-app from a
"demo + sandbox + signup" triptych into a single end-to-end
showcase narrative.

### 1. Remove the scripted demo. Live trial is the only demo.

- The `Try it` tab and `DemoBoard.tsx` component are deleted.
- `lib/i18n.ts` `demo.*` keys are deleted.
- `app/page.tsx` tab resolver no longer accepts `tab=demo`.
- `e2e/demo.spec.ts` is deleted; coverage for the per-team flow is
  the only Playwright surface.
- The default landing tab becomes `live` (was `demo`).

The user always lands on what their team is actually doing.

### 2. `genasis example prd` produces a concrete, distinctive app

The reference PRD ships in two languages — `prd.en.md` and
`prd.ko.md` under `crates/genasis-cli/templates/examples/`. The CLI
picks one based on `[i18n].active` resolved from `genasis.toml`
(falls back to `en`), so a user who ran `genasis init` in Korean
gets a Korean PRD; English projects get English.

The app described is **"나는 Claude Code 전문가 / I Am a Claude
Code Expert"** — a self-assessment quiz:

- Mobile-phone-bordered single-page app (visually distinct from
  the trial-app's kanban/chat).
- Start button → 5 questions drawn at random from a bank covering
  beginner / intermediate / advanced Claude Code knowledge.
- Score determines a level (초급 / 중급 / 고급 → Beginner /
  Intermediate / Advanced).
- Restart pulls a fresh 5-question sample from the bank.

Agents reading the PRD have a clear, bounded deliverable that is
visually unmistakable from the trial-app's own widgets.

### 3. Trial-app embeds the reference quiz; per-team status gates it

- The reference quiz implementation lives inside the trial-app as a
  React component (`app/components/QuizApp.tsx` + question bank at
  `lib/quiz-bank.ts`). Treating it as static asset avoids the
  rabbit-hole of uploading arbitrary user code to a hosted service.
- A new sliding panel — `ShowcasePanel` — anchors to the left edge
  of the live-trial view. A button on the LiveBoard header toggles
  it. Click-outside / Esc closes.
- The sim schema migrates from `user_version = 2` to `3`:
  `sim_teams` gains an `app_status TEXT` column (`NULL` |
  `'building'` | `'complete'`). The panel toggle is enabled only
  when the team's `app_status = 'complete'`. The default-tenancy
  fallback (`DEFAULT_TEAM_TOKEN`) is always treated as `'complete'`
  so anonymous browser visitors who never ran `genasis init` still
  see the showcase — the value-prop animation.

Result: every team that runs through the trial has a clear "we
built this" payoff. The quiz becomes the visible artefact that
proves agents did something.

### 4. Explicit completion signal: `genasis trial publish`

When agents finish the PRD, the user (or an agent shell hook)
invokes:

```bash
genasis trial publish
```

The command reads `[trial].team_token` from `genasis.toml`, POSTs
`{ team_token, status: "complete" }` to
`POST /api/trial/team-app/status` on the trial-app, and prints the
URL the user should open to see the showcase
(`<trial_url>/?tab=live&team=<token>` with the panel now enabled).

The signal is explicit rather than inferred from Plane ticket state
because: (a) the trial-app sim is fed by the agents themselves, so
"all tickets done" doesn't carry independent confirmation, and
(b) operators may want to publish a partial milestone before every
issue closes.

### 6. Explicit team-token gating on Live Trial (TokenBar)

Field feedback after ADR-017 §3 shipped: anonymous visitors silently
landed in the `DEFAULT_TEAM_TOKEN` sandbox, which made the
multi-partition story confusing — users couldn't tell which view
"belonged" to them until they opened the per-team URL the CLI
printed. And if a user pasted that URL on a different machine, the
URL `?team=<token>` worked but a subsequent navigation that dropped
the query string silently fell back to the shared sandbox.

This ADR amendment removes the default-fallback rendering and
introduces an explicit token-gate at the top of the Live Trial view.
The full behaviour:

- A new client component **`TeamTokenBar`** sits at the top of the
  Live tab. It's the single owner of token persistence.
- Token resolution order (server-side, in `page.tsx`):
  1. `?team=<token>` URL query — wins, lets per-team landing URLs
     work cross-machine.
  2. `genasis-trial-team` cookie — set client-side by TokenBar
     after a successful `Connect`. Lasts 1 year (tokens are stable
     per `genasis init --trial` run).
  3. Empty — render the LiveBoard in `disabled` mode: kanban,
     chat, and ShowcasePanel all dimmed with `pointer-events: none`
     plus a `live.disabled.overlay` banner. The TokenBar is the
     only interactive surface.
- TokenBar validates pasted tokens via
  `GET /api/trial/team-app/status?team=<token>` (extended in this
  amendment to return `team_exists` + `project_name`). A token that
  no `sim_teams` row matches surfaces as `live.tokenbar.error.unknown`,
  pointing the user back to `genasis.toml [trial].team_token` or a
  re-run of `genasis init --trial`.
- The `DEFAULT_TEAM_TOKEN` sandbox is no longer auto-displayed; it
  remains reachable for the public shared demo only via explicit
  `?team=default` URL (rarely useful in practice — kept for
  bookmark compatibility, not promoted).

CLI cooperation: `genasis init --trial` now ends with a hard-to-miss
ASCII-bar summary that prints (a) the project name, (b) the
`team_token`, (c) the landing URL with the token pre-filled. The
"copy this into Live Trial's top input" message is the same
language as the `live.tokenbar.idle.*` strings, so a user who pastes
the URL and a user who pastes only the token see consistent
guidance.

### 5. Apply tab rename: "Borrow real env" / "실환경 빌리기"

The tab and form headings shift from "Apply" / "신청하기" to
**"Borrow real env"** / **"실환경 빌리기"**. The form's purpose
is unchanged — submit a request to the operator and receive
credentials for a real Plane + Mattermost project on
`mmplane-trial.realstory.blog`'s shared infrastructure. The new
label says what you actually get: a borrowed real environment,
not a hosted "application".

`README.md` / `README.ko.md` "Trial Server" link is corrected from
`trial.realstory.blog` to
`https://mmplane-trial.realstory.blog/?tab=signup` so the click
takes the reader directly to the request form. All other lingering
`trial.realstory.blog` references in docs / PRD / i18n strings are
swept to `mmplane-trial.realstory.blog` in the same commit.

## Consequences

**Easier**:
- One screen tells the whole story: agents build, kanban + chat
  show the work happening, sliding panel reveals the completed app.
  No more "which tab is real?" confusion.
- The reference PRD is now visually distinct from the trial-app,
  so the agentic team's output is recognisable as the team's work.
- The Korean / English split in `genasis example prd` matches the
  existing `[i18n].active` convention used by every other generated
  file (agent prompts, GENASIS.md, slash commands), removing a
  surprising exception.
- The `Apply` → `Borrow real env` rename clarifies what users
  actually receive when they submit the form.

**Harder**:
- One more sim schema migration (V2 → V3). Same idempotent
  pattern as ADR-016's V1 → V2 — adds a single nullable column.
- The trial-app now carries an embedded quiz implementation. If
  the canonical "Claude Code expertise" question set drifts, the
  trial-app must be redeployed to stay in sync with what agents
  would actually produce. Acceptable: questions change rarely;
  monthly catalog refresh is fine.

**Foreclosed**:
- Free-form arbitrary user-app uploads. The decision is to ship a
  fixed reference implementation, not to accept agent-built code.
  Users who want their actual agent output hosted somewhere are
  served by the `Borrow real env` path (ADR-017 §5).
- The scripted-demo path is gone for good. ADR-018 may revive it
  if user testing shows the live mode is too overwhelming as a
  first impression, but the current evidence (the credibility
  problem in §Context.1) points the other way.

## Verification

- Unit tests:
  - `crates/genasis-core/src/config.rs`: existing i18n round-trip
    still passes (no schema change).
  - `crates/genasis-cli/src/cmd_example.rs`: new test —
    `prd_emits_korean_when_active_lang_ko` /
    `prd_emits_english_when_active_lang_en`.
  - `crates/genasis-cli/src/cmd_trial_publish.rs`: new test —
    POST body shape, env-var override for the target URL.
- Trial-app:
  - Migration test: V2 fixture → V3 yields `app_status` column on
    `sim_teams` with all existing rows = `NULL`.
  - Status route: POST with valid token sets `app_status =
    'complete'`; GET returns current status; idempotent on
    repeated POST.
  - Quiz: bank size ≥ 15, level mapping deterministic for a fixed
    score (0–1 → 초급, 2–3 → 중급, 4–5 → 고급).
- Playwright e2e:
  - `demo.spec.ts` deleted.
  - New `showcase.spec.ts`: panel toggle behaviour, click-outside
    close, gated state for `'building'` status.

## References

- ADR-013 (trial-bridge config wiring) — the scripted demo
  introduced there is removed by this ADR §1.
- ADR-014 (human roster) — the `Borrow real env` form continues
  to populate `[[humans]]` for the borrowed project.
- ADR-016 (identifier alignment + multi-tenancy) — this ADR
  layers on top; team_token is still the tenancy key, now also
  scoping `sim_teams.app_status`.
- Implementation:
  `crates/genasis-cli/src/{cmd_example,cmd_trial_publish}.rs`,
  `crates/genasis-cli/templates/examples/prd.{en,ko}.md`,
  `agents-pool/trial-app/{db,app/components,app/api/trial/team-app,lib/quiz-bank,e2e}/*`,
  `README.{md,ko.md}`.
