//! DuckDB — `duckdb` CLI wrapper. Run with `-readonly` for the read path.

use crate::kernel::QueryOutput;
use genasis_core::error::Result;

use super::run_cli;

pub async fn query_readonly(url: &str, sql: &str) -> Result<QueryOutput> {
    run_cli("duckdb", &["-readonly", "-c", sql, url]).await
}
