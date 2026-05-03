# Golden fixture conventions

| Fixture | Scenario |
|---|---|
| `ecc-only/` | A repo with `.claude/agents/{planner,architect,frontend,backend,qa,...}.md` matching ECC defaults; no Plane/MM yet. |
| `kw-plugins/` | A repo using Anthropic knowledge-work-plugins instead of ECC. |
| `blank/` | Empty repo — `genasis init` should produce an end-to-end attached team. |
| `legacy-bash-genesis/` | Mimics a project bootstrapped via the original `create-agentic-team.sh` bash script. Tests `migrate-from-genesis`. |
| `with-drizzle/` | Has `drizzle-orm` in `package.json` and `drizzle.config.ts`. `genasis db migrate` should pick `drizzle-kit`. |
| `with-duckdb/` | Has `genasis.toml [db] driver = "duckdb"`. `genasis db migrate` should fall back to the raw SQL runner. |

## Adding a new fixture
1. Create `tests/golden/<name>/{input,expected}/`.
2. Drop in the minimum files needed to trigger the code path.
3. Run `genasis attach` against `input/` and copy the result to `expected/`.
4. Add a row to the table above.
