//! Driver-specific adapters.

pub mod atlas;
pub mod drizzle_kit;
pub mod duckdb;
pub mod mysql;
pub mod postgres;
pub mod raw_runner;
pub mod sqlite;

use tokio::process::Command;

use genasis_core::error::{Error, Result};

use crate::kernel::QueryOutput;

/// Run an external CLI with arguments, capturing stdout/stderr.
pub(crate) async fn run_cli(program: &str, args: &[&str]) -> Result<QueryOutput> {
    let out = Command::new(program)
        .args(args)
        .output()
        .await
        .map_err(|e| Error::Db(format!("{program}: {e} — is it installed?")))?;
    if !out.status.success() {
        return Err(Error::Db(format!(
            "{program} exited {} — stderr: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        )));
    }
    Ok(QueryOutput {
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    })
}
