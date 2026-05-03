# ADR-004: DB read / write channel separation

## Status

Accepted (2026-05-03).

## Context

Agentic teams must be able to inspect their target app's database (to
explain a bug, gather context, write a migration), but unbounded write
access from an LLM session is unacceptably risky.

We considered three approaches:

1. **MCP server** that lets the agent issue arbitrary SQL.
2. **Read-only MCP server** with a curated subset.
3. **CLI dispatch + lex-time SQL guard** for reads, **dedicated migration
   tool** for writes.

Approach 1 fails the safety bar. Approach 2 requires per-driver MCP
implementations, several of which (DuckDB in particular) do not exist as
maintained OSS at our quality bar. Approach 3 reuses the canonical CLI
clients (`psql`, `mysql`, `sqlite3`, `duckdb`) that the user has already
installed and audited.

## Decision

**Reads** route through `genasis db query "<SQL>"`:

1. The lex guard in `genasis-db::guard` rejects any statement whose first
   significant token is in the forbidden set (see ADR-001 sibling
   discussion).
2. The dispatcher invokes the matching CLI in an explicitly read-only mode:
   - PostgreSQL: `BEGIN; SET TRANSACTION READ ONLY; …; ROLLBACK;`
   - MySQL: `SET SESSION TRANSACTION READ ONLY` prefix
   - SQLite: `PRAGMA query_only = 1` prefix
   - DuckDB: `-readonly` CLI flag

Both layers are second-line defences for each other.

**Writes** route through `genasis db migrate`:

- Default: **Atlas** (declarative HCL/SQL).
- Auto-detected: **Drizzle Kit** when `drizzle.config.ts` is present.
- Fallback: **raw SQL runner** for DuckDB and exotic engines.

The agent overlay protocol (per-role fences) names *both* commands
explicitly. There is no legitimate path from agent → unrestricted SQL.

## Consequences

**Easier**:
- Database safety becomes auditable: a single regex of "agent code that
  contains a write SQL keyword outside of a `migrate` invocation" is
  enough to flag violations.
- New drivers add only an adapter file; the kernel + guard + CLI surface
  stay constant.

**Harder**:
- Users who want to run an interactive REPL get a CLI shell instead of a
  conversational tool. Documented as the expected behaviour in
  `docs/PROVIDERS.md` and the `cmd_db` help text.

**Foreclosed**:
- An unrestricted "agent SQL console" is out of scope. Power users who
  insist can shell out to `psql` directly outside of Genasis — they
  implicitly own the consequences.

## References

- Implementation: `crates/genasis-db/`
- Lex guard: `crates/genasis-db/src/guard.rs`
- Blueprint: `blueprint.md` §6 (Schema Kernel & DB 운영)
