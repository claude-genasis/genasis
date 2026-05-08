# Golden fixture conventions

Golden fixtures pin **deterministic disk-state output** that is hard to
verify with a unit test (round-trip equality, multi-file overlay
results, language-specific overlay bodies). Anything that can be
expressed as a single function call against pure data — driver
detection, slug parsing, role inference — belongs in a unit test
inside the relevant crate, not here.

| Fixture | Scenario | Owning Rust test |
|---|---|---|
| `ecc-only/` | A repo with `.claude/agents/{frontend,backend,...}.md` matching ECC defaults; round-trip + idempotent attach. | `crates/genasis-overlay/tests/golden_ecc_only.rs` |
| `blank/` | Empty repo — `genasis bootstrap` (M14) scaffolds 10 base agents, then `attach` injects fences. Active since M14.4. | `crates/genasis-overlay/tests/golden_blank.rs` (`BLESS=1` to refresh) |
| `with-ko-locale/` | Exercises `genasis attach --lang ko` end-to-end — Korean overlay body lands inside the fence, `genasis.toml [i18n].active = "ko"` persisted. | covered via integration test in `crates/genasis-overlay` |

## Retired fixtures (M18 audit, 2026-05-08)

These directories existed as M0 stubs but never gained an `expected/`
snapshot because their scenarios are either better covered by unit
tests or do not yet correspond to an implemented code path.

| Fixture | Reason for retirement |
|---|---|
| `kw-plugins/` | Detector reads frontmatter `name:` only — knowledge-work-plugins repos hit the same code path as ECC. No code-level distinction to pin. |
| `legacy-bash-genesis/` | `cmd migrate-from-genesis` is intentionally docs-only for v0.1.0 (progress.md M11, deferred [s]). No code path to exercise. |
| `with-drizzle/` | Drizzle Kit detection is a single `detected()` call against `drizzle.config.{ts,js}`. Now covered by unit tests in `crates/genasis-db/src/adapters/drizzle_kit.rs::tests`. |
| `with-duckdb/` | DuckDB driver dispatch is a single `Driver::parse("duckdb")`. Already covered by unit tests in `crates/genasis-db/src/kernel.rs::tests`. |

## Adding a new fixture
1. Create `tests/golden/<name>/{input,expected}/`.
2. Drop in the minimum files needed to trigger the code path.
3. Run the relevant `genasis` subcommand against `input/` (or use
   `BLESS=1 cargo test`) and copy the result to `expected/`.
4. Add a row to the table above with the owning Rust test.
5. If the scenario can be expressed as a unit test against pure data,
   prefer that — golden fixtures are for **deterministic disk state**.
