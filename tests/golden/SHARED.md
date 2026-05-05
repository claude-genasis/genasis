# Golden fixture conventions

| Fixture | Scenario |
|---|---|
| `ecc-only/` | A repo with `.claude/agents/{planner,architect,frontend,backend,qa,...}.md` matching ECC defaults; no Plane/MM yet. |
| `kw-plugins/` | A repo using Anthropic knowledge-work-plugins instead of ECC. |
| `blank/` | Empty repo — `genasis bootstrap` (M14) scaffolds 10 base agents, then `attach` injects fences. Stub until M14.4. |
| `legacy-bash-genesis/` | Mimics a project bootstrapped via the original `create-agentic-team.sh` bash script. Tests `migrate-from-genesis`. |
| `with-drizzle/` | Has `drizzle-orm` in `package.json` and `drizzle.config.ts`. `genasis db migrate` should pick `drizzle-kit`. |
| `with-duckdb/` | Has `genasis.toml [db] driver = "duckdb"`. `genasis db migrate` should fall back to the raw SQL runner. |
| `with-ko-locale/` | Exercises `genasis attach --lang ko` end-to-end. Verifies the Korean templates/ko/agent-overlays/frontend.patch.md.tera body lands inside the agent fence and that genasis.toml `[i18n].active = "ko"` is persisted. |

## Adding a new fixture
1. Create `tests/golden/<name>/{input,expected}/`.
2. Drop in the minimum files needed to trigger the code path.
3. Run `genasis attach` against `input/` and copy the result to `expected/`.
4. Add a row to the table above.
