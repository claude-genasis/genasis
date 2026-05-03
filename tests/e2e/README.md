# E2E tests

Workspace-level end-to-end tests that drive the `genasis` binary against the
golden fixture repos under `tests/golden/`.

| File | Scenario | Milestone |
|---|---|---|
| `attach_detach.rs` | `genasis attach` then `detach` returns to byte-identical input | M2 |
| `upgrade.rs` | Fence version bump rewrites only the fence | M2 |
| `db_query_guard.rs` | `genasis db query "DROP TABLE x"` exits non-zero | M5 |
| `design_swap.rs` | `genasis design swap <url>` updates `docs/design-system.md` | M7 |

Currently empty (M1). The tests are added at the milestone where the
relevant CLI command becomes implementation-complete.
