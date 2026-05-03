use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use genasis_core::config::{Config, CONFIG_FILE_NAME};
use genasis_db::{kernel, Driver, MigrationTool};

#[derive(Parser, Debug)]
pub struct Args {
    /// Project root. Defaults to the current working directory.
    #[arg(long, value_name = "DIR", global = true)]
    pub project: Option<PathBuf>,

    #[command(subcommand)]
    pub op: DbOp,
}

#[derive(Subcommand, Debug)]
pub enum DbOp {
    /// Run a read-only SQL query (DDL/DML rejected by the SQL guard).
    Query { sql: String },
    /// Show the current schema (driver-specific dump).
    Schema,
    /// Run pending migrations via the configured tool.
    Migrate {
        #[arg(long, default_value = "dev")]
        env: String,
    },
    /// Show the migration plan without applying.
    Diff,
    /// Show migration application history (driver-specific).
    Status,
    /// Verify driver / migration tool / connectivity.
    Doctor,
}

pub async fn run(args: Args) -> Result<()> {
    let project_root = resolve_project_root(args.project.as_deref())?;
    let cfg_path = project_root.join(CONFIG_FILE_NAME);
    let cfg = if cfg_path.is_file() {
        Config::load(&cfg_path)?
    } else {
        anyhow::bail!("genasis.toml missing at {}", cfg_path.display());
    };
    let db_cfg = cfg.db.as_ref().context("[db] section missing")?;
    let driver = Driver::parse(&db_cfg.driver)?;
    let tool = MigrationTool::parse(&db_cfg.migration_tool)?;

    match args.op {
        DbOp::Query { sql } => {
            let out = kernel::query_readonly(driver, &db_cfg.url, &sql).await?;
            print!("{}", out.stdout);
            if !out.stderr.is_empty() {
                eprint!("{}", out.stderr);
            }
        }
        DbOp::Schema => {
            // The "schema" command is a curated query per driver.
            let sql = match driver {
                Driver::Postgres => "\\d+",
                Driver::Mysql => "SHOW TABLES",
                Driver::Sqlite => ".schema",
                Driver::Duckdb => "DESCRIBE",
            };
            let out = kernel::query_readonly(driver, &db_cfg.url, sql).await?;
            print!("{}", out.stdout);
        }
        DbOp::Migrate { env } => {
            let report = kernel::migrate(tool, &project_root, Some(&env)).await?;
            print!("{report}");
        }
        DbOp::Diff => {
            let plan = kernel::diff(tool, &project_root).await?;
            print!("{plan}");
        }
        DbOp::Status => {
            println!("driver: {:?}", driver);
            println!("migration_tool: {:?}", tool);
            println!("url: {}", redact_url(&db_cfg.url));
        }
        DbOp::Doctor => {
            println!("driver: {:?}", driver);
            println!("migration_tool: {:?}", tool);
            println!("project_root: {}", project_root.display());
            check_tool(&driver_cli(driver));
            if matches!(tool, MigrationTool::Atlas) {
                check_tool("atlas");
            }
        }
    }
    Ok(())
}

fn resolve_project_root(arg: Option<&std::path::Path>) -> Result<PathBuf> {
    if let Some(p) = arg {
        return p
            .canonicalize()
            .with_context(|| format!("--project path does not exist: {}", p.display()));
    }
    let cwd = std::env::current_dir()?;
    if let Some(cfg) = Config::discover(&cwd) {
        if let Some(parent) = cfg.parent() {
            return Ok(parent.to_path_buf());
        }
    }
    Ok(cwd)
}

fn driver_cli(d: Driver) -> &'static str {
    match d {
        Driver::Postgres => "psql",
        Driver::Mysql => "mysql",
        Driver::Sqlite => "sqlite3",
        Driver::Duckdb => "duckdb",
    }
}

fn check_tool(name: &str) {
    match which::which(name) {
        Ok(p) => println!("  {name}: {} ✓", p.display()),
        Err(_) => println!("  {name}: not found ✗ (install — see install.sh guidance)"),
    }
}

fn redact_url(u: &str) -> String {
    if let Some((before, after)) = u.split_once("://") {
        if let Some(at) = after.find('@') {
            return format!("{before}://[redacted]@{}", &after[at + 1..]);
        }
    }
    u.to_string()
}
