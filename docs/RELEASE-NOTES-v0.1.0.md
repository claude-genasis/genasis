# Genasis v0.1.0

> First public release. The bolt-on agentic team layer for Plane +
> Mattermost is feature-complete against the README §CLI Reference
> surface and verified end-to-end.

## Highlights

- **Non-destructive overlay** of marker fences inside `.claude/agents/*.md`
  ([ADR-001](docs/ADR/ADR-001-overlay-marker-fence.md)). `genasis detach`
  removes everything cleanly.
- **Green-field bootstrap** — `genasis bootstrap` (or
  `genasis init --bootstrap`) scaffolds a 10-role default team and
  injects the protocol overlay in one shot
  ([ADR-010](docs/ADR/ADR-010-default-team-bootstrap.md), M14).
- **Trial mode** — `genasis init --trial` boots a hosted-equivalent
  workflow against the `trial-app` simulator so first-time users can
  exercise every command without installing Plane / Mattermost.
- **Hosted demo** — `trial-app/` (Next.js 15) ships at
  trial.realstory.blog with a scripted 8-step sprint, signup form,
  status page, and a fully wired live-bridge mode.
- **Dynamic agents catalog** — `genasis agents {list,install,installed,
  remove,browse}` against a versioned tarball published from
  `agents-pool/` ([ADR-011](docs/ADR/ADR-011-dynamic-agents-catalog.md)).
- **Design hot-swap** — `genasis design swap <slug|--from path>` with
  pristine/external mode + restore + override accumulation
  ([ADR-009](docs/ADR/ADR-009-design-catalog-delegation.md)).
- **Schema kernel** — `genasis db {query,migrate}` with read-only SQL
  guard and Atlas / Drizzle Kit / DuckDB raw-runner auto-detect
  ([ADR-004](docs/ADR/ADR-004-db-channel-separation.md)).
- **i18n active singularity** — install-time `--lang en|ko` selector;
  `genasis lang switch` rewrites the agent context atomically
  ([ADR-008](docs/ADR/ADR-008-i18n-install-time-selector.md)).
- **Debug history feedback loop** — `genasis debug {status,collect,
  submit}` lets users contribute anonymised field patches via PR-only
  flow; `/debug-review` skill clusters them and proposes template fixes
  ([ADR-012](docs/ADR/ADR-012-debug-history-feedback-loop.md)).

## Verified surface

Every command listed in `README.md §CLI Reference` is exercised by an
automated test:

- `tests/e2e/` (Rust integration via assert_cmd, default backend = trial
  flavor + mock catalog): **23 tests** across `lifecycle`, `agents`,
  `supporting`, `debug`.
- `trial-app/e2e/` (Playwright against the production build): **14
  tests** covering US-001..US-022 (1 skipped: admin webhook gated on
  `TRIAL_ADMIN_TOKEN` secret).
- `crates/*/tests/` + crate-internal unit tests: **199 tests** covering
  marker fence, env round-trip, role inference, SQL guard, frontmatter,
  i18n, manifest/drift, design swap, EPIC plan, bootstrap, etc.

Total: **222 Rust tests + 14 Playwright = 236 automated checks** at
v0.1.0 cut.

CI surface:
- `.github/workflows/ci.yml`: fmt + clippy + test + coverage on every PR.
- `.github/workflows/nightly-e2e.yml`: real Plane + Mattermost via
  `servers/docker-compose.yml`.
- `.github/workflows/release.yml`: cross-compiled release artefacts +
  i18n drift hard fail.
- `.github/workflows/debug-history-pr.yml`: schema validation +
  executable-content rejection on contributor PRs.
- `.github/workflows/debug-review.yml`: weekly clustering + draft PR.

## Breaking changes

None — this is the first public release.

## Known limitations

- `genasis migrate-from-genesis` is documentation-only for v0.1.0; full
  migration tooling lands post-release once we have real Genesis bash
  team operational data ([progress.md](progress.md) M11 [s]).
- `genasis agents status` invokes `reqwest::blocking` inside
  `#[tokio::main]` and panics at runtime shutdown. README does not list
  it; tracked as a v0.2.0 follow-up.
- `genasis debug submit`'s actual `gh pr create` invocation is gated
  behind `--dry-run` for v0.1.0; the canonical contract is asserted in
  E2E. Wired to the `agents-pool` repo plumbing in a v0.1.1 follow-up.

## Upgrade path

`v0.1.x → v0.2.x` will keep the marker fence v1.0 contract; users may
run `genasis upgrade --fence-version 1.0` at any point and expect a
no-op result.

## Acknowledgements

See [CREDITS.md](docs/CREDITS.md) for the full list of upstream projects
and contributor agents. Special thanks to the maintainers of ECC,
knowledge-work-plugins, claude-code-templates, awesome-design-md, and
the wshobson / VoltAgent / dl-ezo agent collections that seed the
catalog.
