//! PostgreSQL — `psql` driver wrapper. Read-only is enforced via a
//! `BEGIN; SET TRANSACTION READ ONLY;` prefix and a final `ROLLBACK;`.

use crate::kernel::QueryOutput;
use genasis_core::error::Result;

use super::run_cli;

pub async fn query_readonly(url: &str, sql: &str) -> Result<QueryOutput> {
    let wrapped = format!(
        "BEGIN;\nSET TRANSACTION READ ONLY;\n{sql}\n;\nROLLBACK;\n"
    );
    run_cli(
        "psql",
        &[url, "--no-psqlrc", "-X", "-A", "-q", "-c", &wrapped],
    )
    .await
}
