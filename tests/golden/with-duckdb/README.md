# Golden fixture: `with-duckdb`

> Status: M2 (input) / M2-M7 (expected). M0 placeholder.

## Scenario
TBD per fixture (see `progress.md` and `tests/golden/SHARED.md`).

## Layout
- `input/`  — a minimal mock repo representing the user's project before `genasis attach`.
- `expected/` — the deterministic output after `genasis attach` is run on `input/`.

The CI test in `tests/e2e/attach_detach.rs` copies `input/` to a temp dir,
runs the relevant `genasis` subcommand against it, and diffs against `expected/`.
