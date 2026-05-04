# Adding a new language to Genasis

> 한국어: [../ko/i18n/CONTRIBUTE-LANG.md](../ko/i18n/CONTRIBUTE-LANG.md)

Genasis ships with English (`en`) and Korean (`ko`) out of the box. Adding
another language (e.g. Japanese `ja`) is a contribution that touches four
parallel surfaces. Submit them together in a single PR titled
`[i18n] Add <Language> support`.

## Surfaces

1. **Tera template subtree** — `crates/genasis-templates/templates/<lang>/`
   - Mirror the layout of `templates/en/` exactly. 39 `.tera` files:
     1 GENASIS.md, 1 genasis.toml, 1 env.agents, 1 mcp.json,
     1 design-system, 10 agent-overlays, 16 commands, 6 skills, 6 hooks.
   - Preserve env vars (`${PLANE_TOKEN_*}`, `${MM_TOKEN_*}`), paths, code
     blocks, URLs, and Tera tags (`{{ var }}`, `{% if %}`).
   - Update `crates/genasis-templates/src/lib.rs` `SUPPORTED_LANGS`.
2. **Runtime i18n bundle** — `crates/genasis-i18n/locales/<lang>.yml`
   - Mirror every key in `en.yml`. Missing keys fall back to English at
     runtime, but `lint-i18n` warns and `release-prep` hard-fails.
   - Update `crates/genasis-i18n/src/lib.rs` `Lang` enum + `parse()`.
3. **Documentation tree** — `docs/<lang>/`
   - Mirror `ARCHITECTURE.md`, `PROVIDERS.md`, `MIGRATION-FROM-GENESIS.md`,
     `TOKEN-ECONOMICS.md`, `MONITOR.md`, `impact-of-multilang-prompts.md`,
     and `ADR/ADR-001`–`ADR-008`.
4. **README** — `README.<lang>.md`
   - Mirror the 18-section structure of `README.md`.
   - Add the language to the badge row in **all** existing READMEs:
     `README.md`, `README.ko.md`, and any other `README.<lang>.md`.
   - Add the language to the bottom-navigation section in all READMEs.
   - Open a PR to GitHub repo Topics asking the maintainer to add the
     language as a tag (`japanese`, `日本語`, etc.).

## Verifying

Run before opening the PR:

```bash
scripts/check-i18n-drift.sh --check-mirror-not-empty
scripts/i18n-extract-keys.sh
cargo test -p genasis-i18n
cargo test -p genasis-templates
```

The CI `lint-i18n` job will re-run these on the PR.

## Translation principles

- **Code blocks, env vars, CLI commands, URLs are NEVER translated.**
- **Plane / Mattermost lifecycle terms** (`Todo`, `In Progress`, `In Review`,
  `Done`, `PR`, `merge`, `squash`) stay as English loanwords because they
  match the actual Mattermost / Plane UI.
- **`@mention` syntax** stays English (`@qa.{{ project_name }}`).
- **Markdown headings** translate; the level (`##` / `###`) stays.

## Activating the language at runtime

Once merged, users select your language with:

```bash
genasis init --lang <lang>
genasis lang switch <lang>
```

`install.sh` parses `$LANG` for fallback (e.g. `ja_JP.UTF-8` → `ja`); add a
clause to the Bash `suggest_lang()` function for your locale.
