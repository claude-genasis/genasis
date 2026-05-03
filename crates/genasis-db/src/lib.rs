//! genasis-db — schema kernel and read-only SQL guard.
//!
//! Read path: `genasis db query` lex-checks SQL → dispatches to driver CLI in
//! enforced read-only mode (PG `SET TRANSACTION READ ONLY`, MySQL ro user,
//! SQLite `PRAGMA query_only=1`, DuckDB `-readonly`).
//!
//! Write path: `genasis db migrate` delegates to Atlas (default) or
//! drizzle-kit (auto-detected) or a raw SQL runner (DuckDB).

pub mod adapters;
pub mod guard;
pub mod kernel;

pub use genasis_core::{Error, Result};
pub use kernel::{Driver, MigrationTool, QueryOutput};
