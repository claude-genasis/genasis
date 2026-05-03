# Golden fixture: `ecc-only`

> Status: M2 (input authored) — full snapshot lands in M6 once every role has
> a Tera template.

A small mock project containing three agents:

- `frontend.md` (canonical role — has a Genasis overlay template at M2)
- `backend.md` (canonical role — overlay template arrives in M6)
- `loop-operator.md` (custom — `attach` should classify as `Custom` and skip)

## What the M2 test asserts

`crates/genasis-overlay/tests/golden_ecc_only.rs::round_trip` copies
`input/` into a temp directory, runs the in-process `plan_attach` followed
by `apply`, then `plan_detach` followed by `apply`. It verifies:

1. After attach, `frontend.md` contains a Genasis marker fence.
2. After detach, every file is byte-identical to its `input/` counterpart.

There is no static `expected/` snapshot yet — once M6 ships templates for
all 10 roles, attach output becomes deterministic and we'll snapshot it.
