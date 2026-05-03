//! SQLite — `sqlite3` CLI wrapper. Read-only is enforced via
//! `PRAGMA query_only = 1` before every query.

use crate::kernel::QueryOutput;
use genasis_core::error::Result;

use super::run_cli;

pub async fn query_readonly(url: &str, sql: &str) -> Result<QueryOutput> {
    let wrapped = format!("PRAGMA query_only = 1;\n{sql}");
    run_cli("sqlite3", &[url, &wrapped]).await
}
