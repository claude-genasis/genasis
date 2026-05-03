//! Driver dispatch for the read and write paths.

use std::path::Path;

use crate::adapters::{atlas, drizzle_kit, duckdb, mysql, postgres, sqlite};
use crate::guard::check_readonly;
use genasis_core::error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Driver {
    Postgres,
    Mysql,
    Sqlite,
    Duckdb,
}

impl Driver {
    pub fn parse(s: &str) -> Result<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "postgres" | "postgresql" | "pg" => Ok(Self::Postgres),
            "mysql" | "mariadb" => Ok(Self::Mysql),
            "sqlite" | "sqlite3" => Ok(Self::Sqlite),
            "duckdb" => Ok(Self::Duckdb),
            other => Err(Error::Db(format!("unknown db driver: {other}"))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationTool {
    Atlas,
    DrizzleKit,
    RawRunner,
}

impl MigrationTool {
    pub fn parse(s: &str) -> Result<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "atlas" => Ok(Self::Atlas),
            "drizzle-kit" | "drizzle_kit" | "drizzle" => Ok(Self::DrizzleKit),
            "raw" | "raw-runner" => Ok(Self::RawRunner),
            other => Err(Error::Db(format!("unknown migration_tool: {other}"))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueryOutput {
    pub stdout: String,
    pub stderr: String,
}

/// Run a SQL string in read-only mode against the configured driver.
pub async fn query_readonly(driver: Driver, url: &str, sql: &str) -> Result<QueryOutput> {
    check_readonly(sql)?;
    match driver {
        Driver::Postgres => postgres::query_readonly(url, sql).await,
        Driver::Mysql => mysql::query_readonly(url, sql).await,
        Driver::Sqlite => sqlite::query_readonly(url, sql).await,
        Driver::Duckdb => duckdb::query_readonly(url, sql).await,
    }
}

/// Apply migrations.
pub async fn migrate(tool: MigrationTool, project_root: &Path, env: Option<&str>) -> Result<String> {
    match tool {
        MigrationTool::Atlas => atlas::apply(project_root, env).await,
        MigrationTool::DrizzleKit => drizzle_kit::apply(project_root).await,
        MigrationTool::RawRunner => crate::adapters::raw_runner::apply(project_root).await,
    }
}

/// Diff (plan) without applying.
pub async fn diff(tool: MigrationTool, project_root: &Path) -> Result<String> {
    match tool {
        MigrationTool::Atlas => atlas::diff(project_root).await,
        MigrationTool::DrizzleKit => drizzle_kit::diff(project_root).await,
        MigrationTool::RawRunner => crate::adapters::raw_runner::diff(project_root).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn driver_parsing() {
        assert_eq!(Driver::parse("postgres").unwrap(), Driver::Postgres);
        assert_eq!(Driver::parse("PG").unwrap(), Driver::Postgres);
        assert_eq!(Driver::parse("duckdb").unwrap(), Driver::Duckdb);
        assert!(Driver::parse("oracle").is_err());
    }

    #[test]
    fn migration_tool_parsing() {
        assert_eq!(MigrationTool::parse("atlas").unwrap(), MigrationTool::Atlas);
        assert_eq!(
            MigrationTool::parse("drizzle-kit").unwrap(),
            MigrationTool::DrizzleKit
        );
        assert_eq!(MigrationTool::parse("raw").unwrap(), MigrationTool::RawRunner);
    }
}
