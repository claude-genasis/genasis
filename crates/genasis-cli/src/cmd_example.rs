//! `genasis example {prd|design|prd2}` — drop a sample document into the
//! current project so tutorials and onboarding flows have something
//! immediately actionable for the agentic team to chew on.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use clap::{Args as ClapArgs, Subcommand};

const TEMPLATE_PRD: &str = include_str!("../templates/examples/prd.md");
const TEMPLATE_DESIGN: &str = include_str!("../templates/examples/design-system.md");
const TEMPLATE_PRD2: &str = include_str!("../templates/examples/prd2.md");

#[derive(ClapArgs, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub kind: Kind,

    /// Project root. Defaults to the current working directory.
    #[arg(long, value_name = "DIR", global = true)]
    pub project: Option<PathBuf>,

    /// Overwrite an existing destination file.
    #[arg(long, global = true)]
    pub force: bool,
}

#[derive(Subcommand, Debug)]
pub enum Kind {
    /// Write a sample PRD.md to the project root.
    Prd,
    /// Write a sample design-system.md to the project root.
    Design,
    /// Write a sample PRD2.md (feature expansion) to the project root.
    Prd2,
}

pub fn run(args: Args) -> Result<()> {
    let root = if let Some(p) = args.project.as_deref() {
        if !p.exists() {
            fs::create_dir_all(p)
                .with_context(|| format!("create --project dir {}", p.display()))?;
        }
        p.canonicalize()
            .with_context(|| format!("canonicalize {}", p.display()))?
    } else {
        std::env::current_dir()?
    };

    let (filename, body) = match args.kind {
        Kind::Prd => ("PRD.md", TEMPLATE_PRD),
        Kind::Design => ("design-system.md", TEMPLATE_DESIGN),
        Kind::Prd2 => ("PRD2.md", TEMPLATE_PRD2),
    };

    let dest = root.join(filename);
    write_template(&dest, body, args.force)?;
    println!("→ wrote {}", dest.display());
    Ok(())
}

fn write_template(dest: &Path, body: &str, force: bool) -> Result<()> {
    if dest.exists() && !force {
        return Err(anyhow!(
            "{} already exists (use --force to overwrite)",
            dest.display()
        ));
    }
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create parent dir {}", parent.display()))?;
    }
    fs::write(dest, body).with_context(|| format!("write {}", dest.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn templates_have_content() {
        assert!(TEMPLATE_PRD.contains("# PRD"));
        assert!(TEMPLATE_DESIGN.contains("# Design System"));
        assert!(TEMPLATE_PRD2.contains("# PRD2"));
    }

    #[test]
    fn prd_writes_to_project_root() {
        let tmp = TempDir::new().unwrap();
        let args = Args {
            kind: Kind::Prd,
            project: Some(tmp.path().to_path_buf()),
            force: false,
        };
        run(args).unwrap();
        let body = fs::read_to_string(tmp.path().join("PRD.md")).unwrap();
        assert!(body.contains("# PRD"));
    }

    #[test]
    fn refuses_to_overwrite_without_force() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("PRD.md");
        fs::write(&path, "preexisting").unwrap();
        let args = Args {
            kind: Kind::Prd,
            project: Some(tmp.path().to_path_buf()),
            force: false,
        };
        let err = run(args).unwrap_err().to_string();
        assert!(err.contains("already exists"));
        assert_eq!(fs::read_to_string(&path).unwrap(), "preexisting");
    }

    #[test]
    fn force_overwrites() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("PRD.md");
        fs::write(&path, "preexisting").unwrap();
        let args = Args {
            kind: Kind::Prd,
            project: Some(tmp.path().to_path_buf()),
            force: true,
        };
        run(args).unwrap();
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.contains("# PRD"));
    }

    #[test]
    fn each_kind_writes_correct_filename() {
        let tmp = TempDir::new().unwrap();
        for (kind, filename) in [
            (Kind::Prd, "PRD.md"),
            (Kind::Design, "design-system.md"),
            (Kind::Prd2, "PRD2.md"),
        ] {
            let args = Args {
                kind,
                project: Some(tmp.path().to_path_buf()),
                force: false,
            };
            run(args).unwrap();
            assert!(tmp.path().join(filename).exists());
        }
    }
}
