# Base agent templates (M14)

Each role gets one `<role>.md.tera` here. The bootstrap stage of
`genasis bootstrap` (or `genasis init --bootstrap`) renders these into
`.claude/agents/<role>.md` **only when the file is absent**. After
emit, the file is yours to edit freely — genasis will never rewrite it
again.

The marker fence injected later by `genasis attach` (using the
`agent-overlays/<role>.patch.md.tera` siblings) lives **inside** the
file but **outside** this base content. See ADR-001 (marker fence) and
ADR-010 (base + patch ownership boundary).

Roles: `pm`, `planner`, `architect`, `frontend`, `backend`, `qa`,
`designer`, `security`, `devops`, `code-reviewer`.

Template contract — every base file must have:

- YAML frontmatter with these 5 keys: `name`, `description`, `tools`,
  `model`, `color`.
- `name` matches the file stem so role-inference classifies it as
  `Known(_)` immediately.
- A 5–10 line role header (markdown) describing what the role owns.
  Keep it short — the protocol body lives in the patch overlay, not
  here.
