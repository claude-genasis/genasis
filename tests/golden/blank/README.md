# Golden fixture: `blank`

> Status: scaffolded as a stub since M0; activated under M14.4.

## Scenario

A repository with no `.claude/agents/` directory at all — i.e. a
green-field project. M14.4 fills `input/` with a minimal mock repo
(no agents, no genasis.toml beyond the `[project]` block) and
`expected/` with the result of:

```bash
genasis bootstrap --lang en --non-interactive
genasis attach    --lang en --non-interactive --yes
```

The expected output contains 10 base agent files (one per canonical
role) each with a Genasis marker fence already injected by the
chained attach.

See ADR-010 (base + patch ownership) and `progress.ko.md` §M14.

## Layout
- `input/`  — a minimal mock repo representing the user's project before `genasis attach`.
- `expected/` — the deterministic output after `genasis bootstrap` + `attach` is run on `input/`.

The CI test in `crates/genasis-overlay/tests/golden_blank.rs` (M14.4)
copies `input/` to a temp dir, runs the bootstrap+attach chain
against it, and diffs against `expected/`.
