# Unit tests

Unit tests live **inside their crate**, per the standard Cargo layout:

| Coverage | Location |
|---|---|
| Marker fence parser / serialiser / idempotency | `crates/genasis-core/tests/marker_idempotent.rs` |
| `.env.agents` round-tripping | `crates/genasis-core/tests/env_round_trip.rs` |
| Role inference | `crates/genasis-overlay/tests/role_inference.rs` |
| SQL read-only guard | `crates/genasis-db/tests/sql_guard.rs` |

Run from the workspace root:

```bash
cargo test --workspace
```

The directory at `tests/unit/` is reserved for cross-crate integration
tests that exercise multiple crates simultaneously (planned in M2+).
