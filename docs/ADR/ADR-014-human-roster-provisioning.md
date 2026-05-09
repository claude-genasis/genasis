> 한국어: [`../ko/ADR/ADR-014-human-roster-provisioning.md`](../ko/ADR/ADR-014-human-roster-provisioning.md)

# ADR-014: Human roster provisioning — humans as first-class team members

## Status

Proposed (2026-05-10).

## Context

Genasis's North Star (CLAUDE.md §Core Philosophy) says: "make AI agents
first-class team members alongside humans inside the collaboration tools
humans already use (Plane, Mattermost)." But up to v0.1, only **agent
bot accounts** were auto-provisioned; **human accounts** had to be
created by hand. So humans bootstrapped the tools manually, and the
agentic team had no idea who its humans were — every non-bot message
was assumed to be "probably a human" by heuristic.

### Concrete defects

1. **Onboarding asymmetry**: a fresh team running `genasis init` gets
   ten agent bots in Mattermost / Plane immediately, but the actual
   human team members must sign up separately. This violates the
   "turnkey bootstrap" mission.

2. **No requirement-source trust**: agents cannot tell who the
   stakeholders are. External guests, QA interns, even off-channel
   jokes from a colleague are processed as "probably a human request,"
   so any of them can route work. Wrong-sender authority is a
   classic ops-incident vector.

3. **Identity audit gap**: there is no mechanism for agents to verify
   the originator of a Mattermost message. If a bot is impersonated or
   a channel leaks, there is no first-line guardrail.

## Decision

Add `[[humans]]` as a first-class array in `genasis.toml`, and apply
the same level of auto-provisioning + runtime awareness that already
exists for agents.

### 1. Data model (genasis-core)

```toml
[[humans]]
name        = "Bravo"
email       = "gnoopy@gmail.com"
role        = "stakeholder"   # stakeholder | pm-human | reviewer | ...
mm_username = ""              # empty → derive from local-part of email
locale      = "ko"            # for outbound system messages
```

Two files split the concerns:
- `genasis.toml` `[[humans]]` is the **human-edited SSOT** — clean,
  committable.
- `.genasis/humans.lock.toml` holds **provisioning side-effects**
  (Mattermost user_id, Plane user_id, temporary password) — gitignored.

### 2. Mattermost provisioning

New trait method `MattermostProvider::ensure_human_user(spec, team_id)`.
Upstream impl:

1. Idempotent probe `GET /api/v4/users/email/{email}`. If it exists,
   return with `temp_password = None`.
2. Otherwise `POST /api/v4/users` (admin-create) with a freshly
   generated 24-char password (1 lower, 1 upper, 1 digit, 1 symbol —
   passes the strictest Mattermost password policy).
3. Force change-on-first-login (best-effort `PUT
   /users/{id}/password`).
4. If `team_id` is given, add to team (idempotent on "already exists").
5. Return `HumanUserRef { user_id, username, email, temp_password,
   must_change_password }`. The temp password is recorded in
   `.genasis/humans.lock.toml`; once the user logs in and rotates it,
   the field is cleared.

**Trade-off (admin-create vs invite-email)**: invite-email is more
secure, but Mattermost SMTP is rarely configured in self-hosted
deployments. The turnkey mission wins, so admin-create is the default.
A `[mattermost] human_provision_mode = "invite"` flag is reserved for
v2 to surface invite-email when SMTP is available.

### 3. Plane provisioning

Extend `provision-plane-users.mjs` (`ProvisionInput`) with a `humans:
HumanRequest[]` field. Humans are added as workspace `Member`s and no
PAT is issued (humans authenticate via the Plane UI).

The script is currently a stub, so the `humans` output echoes
placeholder IDs. When the real UI port lands, populating that field
auto-flows real user_ids into the Rust-side `humans.lock.toml`.

### 4. Runtime — Requirement intake protocol

Two new sections in `agents/GENASIS.md.tera`:

#### `## Human Roster`
Tabulates each provisioned human (name, email/MM username, role).
Every agent sees this in its imported context.

#### `### Requirement intake protocol`
Classify the originator of a new top-level message in
`#scrum-{project}` into three buckets:

1. **In the roster** → the message is a **binding stakeholder
   requirement**:
   - PM acknowledges within 5 minutes via `🟢 접수: <one-line>`.
   - Plane issue created or linked (verbatim quote).
   - Routed to the right role(s) via `assignees`.
   - Priority by role label: `stakeholder > pm-human > reviewer >
     other`.

2. **Not in the roster** → labeled `QUESTION`, PM verifies identity
   before any agent acts.

3. **Bot** (`from_bot=true` or `*-bot` username) → existing
   agent-to-agent protocol, intake skipped.

PM and Planner overlays (en/ko) mirror the same protocol in shorter
form so behaviour survives a context trim of GENASIS.md.

### 5. UX — TUI wizard CRUD + CLI

- The `genasis init` / `genasis attach` wizard gains a fifth step,
  **Humans** (Env → Lang → Team → Connect → **Humans** → Overlay →
  Done).
- Keys: `a` add, `e` edit, `d` delete, `s` sync (Mattermost + Plane),
  Enter advance.
- Same operations from the CLI: `genasis humans add | edit | remove |
  list | sync` (CI/script-friendly).
- Re-running the wizard reloads `[[humans]]` and `humans.lock.toml`,
  so the current state is editable in place — "rerun is the editor."

## Consequences

### Positives
- Mission alignment: human/agent symmetry restored — `genasis init`
  surfaces both populations in the tools at once.
- Higher requirement-source trust: agents can answer "is this sender
  one of our stakeholders?". First-line guardrail against bot
  impersonation.
- Backwards-compatible: unregistered senders are still treated as
  human (PM verifies, then routes). No regressions for projects with
  no `[[humans]]`.

### Costs
- Temp passwords sit in `humans.lock.toml` until first login —
  gitignore is mandatory.
- In SMTP-enabled environments, invite-email would be more
  appropriate; deferred to v2.
- Plane Playwright is still a stub, so real Plane user_ids land
  alongside the UI-port milestone.

### Follow-up milestones
- M20.1: schema + tests
- M20.2: MM `ensure_human_user`
- M20.3: Plane humans field
- M20.4: cmd_humans CLI + cmd_init wiring
- M20.5: TUI Humans step
- M20.6: GENASIS.md / overlay prompt updates
- M20.7: ADR + progress + bilingual mirror (this ADR)
- v2 follow-up: invite-email mode, Plane UI port, OAuth/SSO

## Alternatives rejected

- **Treat every non-bot as human (status quo)**: violates mission and
  leaves spoofing wide open.
- **Strict mode — only registered humans receive responses**: blocks
  legitimate ad-hoc inbounds (external guests, QA interns). Reserved
  as an opt-in v2 flag.
- **Invite-email by default**: too many self-hosted Mattermost
  installs ship without SMTP; would break turnkey bootstrap. Available
  via flag in v2.
