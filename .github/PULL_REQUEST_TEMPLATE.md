<!--
Thanks for contributing to Genasis!

Please make sure your PR title follows Conventional Commits
(feat / fix / docs / test / refactor / chore / i18n).
-->

## Summary

<!-- 1-3 sentences. What changed and why. -->

## Type of change

- [ ] feat  — new functionality
- [ ] fix   — bug fix
- [ ] docs  — documentation only
- [ ] test  — tests only
- [ ] refactor — internal restructuring, no behavior change
- [ ] i18n  — locale / translation work

## i18n checklist (only if your change touches user-facing strings or docs)

- [ ] If you added a `t!()` key, you added it to BOTH `en.yml` and `ko.yml`.
- [ ] If you edited an English document, you flagged or updated the
      Korean mirror under `*.ko.md` / `docs/ko/`.
- [ ] You ran `scripts/check-i18n-drift.sh --warn` locally.
- [ ] You ran `scripts/i18n-extract-keys.sh` locally.

## Test plan

- [ ] `cargo test --workspace`
- [ ] Tested manually: ...
