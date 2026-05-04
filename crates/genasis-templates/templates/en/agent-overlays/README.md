# Agent overlay templates

Each role gets one `<role>.patch.md.tera` here. The merger writes the rendered
output **between marker fences** in the user's own `.claude/agents/<role>.md`,
leaving everything outside the fence untouched. M6.

Roles: `pm`, `planner`, `architect`, `frontend`, `backend`, `qa`, `designer`, `security`, `devops`, `code-reviewer`.
