# Blank fixture project

This directory represents a green-field project — no `.claude/`, no
`genasis.toml`, no agents. Used by `crates/genasis-overlay/tests/golden_blank.rs`
to verify that `genasis bootstrap` followed by `genasis attach` produces
the deterministic output captured under `../expected/`.
