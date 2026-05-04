# Golden fixture: `with-ko-locale`

Exercises `genasis attach --lang ko` end-to-end. The `input/` tree is a
minimal ECC team (one agent file with frontmatter + body but no Genasis
fence). The `expected/` tree shows the same agent file after attaching
the Korean overlay — fence body comes from
`crates/genasis-templates/templates/ko/agent-overlays/frontend.patch.md.tera`.

Update procedure when `templates/ko/` changes:

1. Run `cargo run -- attach --project tests/golden/with-ko-locale/input --lang ko --non-interactive --yes`.
2. `diff -ru tests/golden/with-ko-locale/input tests/golden/with-ko-locale/expected`.
3. Copy `input/` over `expected/` once the diff is what you expected.
