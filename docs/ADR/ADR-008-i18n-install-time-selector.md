# ADR-008: i18n via install-time language selector + active-language singularity

## Status

Accepted (2026-05-04). Implements blueprint §19 (M12).

## Context

Genasis ships agent overlays, slash commands, skills, hooks, and a
contract document (`GENASIS.md`) into the user's `.claude/` tree. It
also ships its own CLI/TUI surfaces (`genasis init`, `genasis monitor`,
etc.) and `install.sh` interactive output. As an OSS project we must
support contributors and operators in multiple natural languages.

We surveyed three architectural options for handling Korean alongside
English:

1. **Both languages installed simultaneously** into agent context.
2. **Single-language installation** chosen at install time.
3. **English-only internal context, runtime translation proxy** (the
   `claude-ts` model).

The investigation report `docs/impact-of-multilang-prompts.md`
synthesized empirical evidence from:

- Anthropic's official multilingual support page (Korean at 96.7% of
  English on Sonnet 4.5).
- Two open Claude Code bugs where the model drifts language even with
  explicit single-language instructions
  ([anthropics/claude-code#46846](https://github.com/anthropics/claude-code/issues/46846),
  [#24941](https://github.com/anthropics/claude-code/issues/24941)).
- The "Understanding and Mitigating Language Confusion in LLMs" paper
  ([arXiv 2406.20052](https://arxiv.org/html/2406.20052v1)) which
  identifies Korean as among the most line-level-confusion-prone
  languages, and observes that *isolated* (start- or end-positioned)
  instructions outperform *integrated* ones by 15–20%.
- "Lost in the Mix" code-switching benchmark
  ([arXiv 2506.14012](https://arxiv.org/html/2506.14012v1)): non-English
  tokens embedded in English matrix consistently degrade comprehension.
- Anthropic prompt caching docs: byte-prefix matching means a bilingual
  prefix never cache-hits and pays write cost twice.
- The de-facto OSS template ecosystem norm (awesome-claude-code,
  aitmpl.com, awesome-claude-code-toolkit, Piebald-AI/claude-code-system-prompts)
  ships English-only.

## Decision

### Active-language singularity

User-installed Genasis context (`.claude/agents/*` overlay fences,
`.claude/genasis/{skills,commands,hooks}/`, `GENASIS.md`,
`@import`-chained reference) is **always exactly one language**. The
language is recorded in `genasis.toml [i18n] active`.

### Install-time selection

`genasis init` and `genasis attach` take `--lang en|ko` (extensible to
`ja`, `zh`, etc. when contributor PRs add the templates). Resolution
priority:

1. `--lang` flag (`en|ko`; `both` is rejected — see below).
2. Interactive language selection prompt when stdin is a TTY and no
   flag given. Prompt is bilingual at the header, lists the install
   targets (`.claude/agents/`, `.claude/genasis/{skills,commands,hooks}/`,
   `GENASIS.md`), warns about the drift risk of mixing languages, and
   defaults to the language inferred from `$LANG`.
3. Non-TTY (CI, pipeline) environments fall back to `$LANG` parsing
   (`ko_KR.*` → `ko`, otherwise `en`) and announce the choice on stdout.

`install.sh` (Bash, runs before binary download) implements the same
prompt as a `case` block — no external dependencies, identical text
and layout to the Rust prompt to keep user experience consistent.

### `--lang both` is rejected

A single, explicit error message cites
`docs/impact-of-multilang-prompts.md` and offers two recommended
alternatives:

1. Pick one now, switch later via `genasis lang switch <lang>`.
2. Install English active and ship the other language as on-disk
   reference docs (`docs/genasis-i18n-reference/<lang>/`) that Claude
   never `@import`s.

### Atomic locale swap — `genasis lang switch <lang>`

Replaces all GENASIS-fence bodies, the `.claude/genasis/` tree, and
`GENASIS.md` in a single git commit. The cache prefix rotates exactly
once; subsequent turns cache normally. This is strictly cheaper than
maintaining a bilingual prefix that never cache-hits.

### Runtime i18n — `rust-i18n`

The CLI/TUI runtime uses [`rust-i18n`](https://crates.io/crates/rust-i18n)
v3 with YAML resource files (`locales/en.yml`, `locales/ko.yml`). The
`t!()` macro is compile-time, has no per-call overhead, and adds
~50KB to the binary. Cascading fallback: missing key in `ko.yml`
falls back to `en.yml` with a warning logged.

### Documentation duality

The repository's own documentation tree is bilingual:

- Top-level `README.md`, `blueprint.md`, `progress.md` are English
  source-of-truth with `*.ko.md` mirrors.
- `docs/*.md` and `docs/ADR/*.md` mirror to `docs/ko/`.

This is **independent of agent context language** — the docs are for
humans reading GitHub, not for Claude. `lint-i18n` CI checks reject
Korean text in English source files (structural violation) and warn on
mirror drift (release-time hard-fail via separate `release-prep`
workflow + automated translation-completion PR).

## Alternatives considered

### A. Both languages installed simultaneously (rejected)

Rejected because:

1. **Instruction divergence (F2)**: two prose protocol contracts
   maintained by hand will drift; a Korean fence saying `--rebase`
   while the English fence still says `--squash` becomes a silent
   ownership/merge bug detected only when the model picks the wrong
   language at decode time.
2. **Empirical drift (F1)**: Claude Code drifts even with explicit
   single-language instructions; adding the wrong language to context
   guarantees worse outcomes.
3. **Cache cost (F3)**: bilingual prefix never matches and doubles
   cache writes.
4. **OSS norm**: the entire Claude Code template ecosystem chose
   single-language internal context; we are not the place to be
   contrarian on this.

### B. English-only + external translation proxy (rejected)

The `claude-ts` model translates user I/O at the edges and keeps
English as the only internal context. This works but pushes the
install-time and operator-facing language burden onto external tooling.
We want native Korean experience for the install flow itself, the
prompt that explains *what* gets installed, the doctor diagnostics,
and the monitor TUI.

### C. Crowdin / Weblate translation platform (deferred)

Reasonable when there are 5+ active locales. Premature now (en + ko
only). The `docs/i18n/CONTRIBUTE-LANG.md` guide leaves a clear path to
adopt this when scale arrives.

### D. fluent-rs runtime (rejected)

`fluent-rs` would give us proper plural/gender/case branching in
locale resources. Korean has no plural inflection and our message
catalog is ~50 keys — the expressiveness is overkill, and the binary
cost is ~200KB plus a 4-crate dependency tree. `rust-i18n` is
simpler, lighter, faster to build, and emits shorter macro call sites
(meaningful because Claude reads our source).

### E. PR-time hard-fail on mirror drift (rejected)

A strict drift gate on every PR forces every contributor to maintain
both languages on every typo fix. This raises the bar for English-only
contributors (the OSS-target audience) too high. Instead we warn on
PRs and hard-fail only at release time, with an automated
`[i18n] Translation completion for vX.Y.Z` PR that batches the
mirror-update work.

## Consequences

### Easier

- Predictable agent behavior — only one language in context, so the
  model has nothing to drift between.
- Clean prompt cache — single prefix rotation per `lang switch`,
  steady-state cache hits.
- Native install/operator experience for non-English users including
  the install.sh prompt.
- Low-friction OSS contribution path: contributors write in whichever
  language they're fluent in; the release-prep automation closes the
  mirror gap.
- Adding a new locale (`ja`, `zh`, …) is a known recipe:
  `templates/<lang>/`, `locales/<lang>.yml`, `docs/<lang>/`,
  `README.<lang>.md`, badge-row update.

### Harder

- Multilingual teams that want simultaneous Korean and English agent
  output must accept that the agent always speaks the team's chosen
  language; humans translate at the edges. Mitigated by `lang switch`
  being one commit and reversible.
- We must keep the en/ko `.yml` parity in CI (`scripts/i18n-extract-keys.sh`).
- Translation completion is a release-time obligation, not free.

### Foreclosed

- Per-contributor language switching at runtime.
- "Sprinkle Korean comments inside English templates for clarity" — the
  `lint-i18n` job rejects this structurally.

## References

- Investigation: [`docs/impact-of-multilang-prompts.md`](../impact-of-multilang-prompts.md)
- Plan: [`blueprint.md` §19](../../blueprint.md)
- Tracker: [`progress.md` M12](../../progress.md)
- Related ADRs: ADR-001 (overlay marker fence), ADR-002 (Rust single
  binary), ADR-005 (provider flavor system).
