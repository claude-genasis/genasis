//! MySQL / MariaDB — `mysql` CLI wrapper. Read-only is enforced via a
//! `SET SESSION TRANSACTION READ ONLY` prefix.
//!
//! `url` is expected to contain credentials acceptable to `--defaults-file`
//! or be a `mysql://user:pass@host/db` style URL parsed by the CLI itself.

use crate::kernel::QueryOutput;
use genasis_core::error::Result;

use super::run_cli;

pub async fn query_readonly(url: &str, sql: &str) -> Result<QueryOutput> {
    let wrapped = format!("SET SESSION TRANSACTION READ ONLY;\n{sql}");
    // Defer URL parsing to the CLI; users with ~/.my.cnf can pass an empty url.
    if url.is_empty() {
        run_cli("mysql", &["-N", "-e", &wrapped]).await
    } else {
        run_cli("mysql", &[url, "-N", "-e", &wrapped]).await
    }
}
