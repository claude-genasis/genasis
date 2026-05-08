//! `genasis debug {status,log,collect,reset}` — local drift detection
//! and patch capture for the Phase F feedback loop (ADR-012).
//!
//! - `status` re-hashes every file recorded in the manifest and prints
//!   a summary of what has drifted since the last `attach`/`bootstrap`.
//! - `log`    surfaces the contents of `.claude/genasis/.drift-log/`.
//! - `collect` produces an anonymised, secret-stripped patch.json
//!   under `~/.genasis/debug-history/<project-hash>/<ts>.patch.json`.
//! - `reset` rewrites the manifest from the live state, clearing the
//!   "drift" baseline so the user can start a clean tracking session.
//!
//! ADR-012 §6: drift detection is default-ON for every CLI invocation;
//! `GENASIS_DEBUG_DRIFT=0` opts out. The local detection runs in
//! `app_preamble` (wired from main.rs in M15.2 follow-up); this module
//! only owns the user-facing subcommand surface.

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

use genasis_core::manifest::{compare, hash_file, DriftKind, FileEntry, Manifest};

#[derive(Parser, Debug)]
pub struct Args {
    /// Project root. Defaults to the current working directory.
    #[arg(long, value_name = "DIR", global = true)]
    pub project: Option<PathBuf>,

    #[command(subcommand)]
    pub op: DebugOp,
}

#[derive(Subcommand, Debug)]
pub enum DebugOp {
    /// Print a one-line drift summary plus per-file breakdown.
    Status,
    /// Show the rolling drift log under `.claude/genasis/.drift-log/`.
    Log,
    /// Anonymise + stripe secrets and emit a `patch.json` to
    /// `~/.genasis/debug-history/<project-hash>/<ts>.patch.json`.
    Collect {
        /// Print the anonymised payload to stdout instead of writing
        /// to ~/.genasis. Useful for CI inspection.
        #[arg(long)]
        stdout: bool,
    },
    /// Refresh the manifest to the current state. Clears all "drift"
    /// for this project — the next `status` reads as pristine.
    Reset,
}

pub fn run(args: Args) -> Result<()> {
    let project_root = match args.project {
        Some(p) => p
            .canonicalize()
            .map_err(|e| anyhow::anyhow!("--project does not exist: {} ({e})", p.display()))?,
        None => std::env::current_dir()?,
    };
    match args.op {
        DebugOp::Status => status(&project_root),
        DebugOp::Log => log_dump(&project_root),
        DebugOp::Collect { stdout } => collect(&project_root, stdout),
        DebugOp::Reset => reset(&project_root),
    }
}

fn status(project_root: &std::path::Path) -> Result<()> {
    let manifest = match Manifest::load(project_root)? {
        Some(m) => m,
        None => {
            println!("no manifest at .claude/genasis/.manifest.json — run `genasis attach` first");
            return Ok(());
        }
    };
    let drift = compare(&manifest, project_root)?;
    if drift.is_empty() {
        println!(
            "drift: 0 files (manifest tracks {} managed file(s))",
            manifest.files.len()
        );
        return Ok(());
    }
    println!(
        "drift: {} file(s) (manifest tracks {} managed file(s))",
        drift.len(),
        manifest.files.len()
    );
    for d in &drift {
        let kind = match d.kind {
            DriftKind::Modified => "modified",
            DriftKind::Removed => "removed ",
            DriftKind::Added => "added   ",
        };
        println!("  {kind}  {}", d.file);
    }
    Ok(())
}

fn log_dump(project_root: &std::path::Path) -> Result<()> {
    let log_path = project_root
        .join(".claude/genasis/.drift-log/current.jsonl");
    if !log_path.is_file() {
        println!("no drift-log at {}", log_path.display());
        return Ok(());
    }
    let body = std::fs::read_to_string(&log_path)?;
    print!("{body}");
    Ok(())
}

fn collect(project_root: &std::path::Path, to_stdout: bool) -> Result<()> {
    let manifest = Manifest::load(project_root)?
        .ok_or_else(|| anyhow::anyhow!("no manifest yet — run `genasis attach` first"))?;
    let drift = compare(&manifest, project_root)?;

    let project_hash = hash_project_identity(project_root);
    let now = chrono::Utc::now().to_rfc3339();

    let mut payload = serde_json::Map::new();
    payload.insert(
        "schema_version".into(),
        serde_json::Value::String("1".into()),
    );
    payload.insert(
        "project_hash".into(),
        serde_json::Value::String(project_hash.clone()),
    );
    payload.insert("collected_at".into(), serde_json::Value::String(now.clone()));
    payload.insert(
        "genasis_version".into(),
        serde_json::Value::String(manifest.genasis_version.clone()),
    );
    payload.insert(
        "lang".into(),
        serde_json::Value::String(manifest.lang.clone()),
    );

    let mut entries = Vec::new();
    for d in &drift {
        let mut entry = serde_json::Map::new();
        entry.insert("file".into(), serde_json::Value::String(d.file.clone()));
        entry.insert(
            "kind".into(),
            serde_json::Value::String(format!("{:?}", d.kind).to_lowercase()),
        );
        if let Some(rec) = &d.recorded_hash {
            entry.insert("recorded_sha256".into(), serde_json::Value::String(rec.clone()));
        }
        if let Some(act) = &d.actual_hash {
            entry.insert("actual_sha256".into(), serde_json::Value::String(act.clone()));
            // Inline a strip-clean diff body — secrets removed.
            let abs = project_root.join(&d.file);
            if let Ok(body) = std::fs::read_to_string(&abs) {
                entry.insert(
                    "modified_body_excerpt".into(),
                    serde_json::Value::String(strip_secrets(&body)),
                );
            }
        }
        entries.push(serde_json::Value::Object(entry));
    }
    payload.insert("entries".into(), serde_json::Value::Array(entries));

    let json = serde_json::to_string_pretty(&serde_json::Value::Object(payload))?;

    if to_stdout {
        println!("{json}");
        return Ok(());
    }
    let dir = patch_dir(&project_hash)?;
    std::fs::create_dir_all(&dir)?;
    let target = dir.join(format!(
        "{}.patch.json",
        now.replace(':', "-").replace('.', "-")
    ));
    std::fs::write(&target, &json)?;
    println!("wrote {}", target.display());
    Ok(())
}

fn reset(project_root: &std::path::Path) -> Result<()> {
    let mut manifest = Manifest::load(project_root)?
        .ok_or_else(|| anyhow::anyhow!("no manifest yet — run `genasis attach` first"))?;

    let keys: Vec<String> = manifest.files.keys().cloned().collect();
    for rel in keys {
        let abs = project_root.join(&rel);
        match hash_file(&abs)? {
            Some(sha) => {
                let entry = manifest.files.entry(rel.clone()).or_default();
                entry.sha256 = sha;
            }
            None => {
                manifest.files.remove(&rel);
            }
        }
        let _ = FileEntry::default(); // silence unused import warnings
    }
    manifest.attached_at = chrono::Utc::now().to_rfc3339();
    manifest.save(project_root)?;
    let drift = compare(&manifest, project_root)?;
    println!(
        "manifest reset — {} managed file(s) tracked, {} drift",
        manifest.files.len(),
        drift.len()
    );
    Ok(())
}

fn patch_dir(project_hash: &str) -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("cannot determine home dir"))?;
    Ok(home.join(".genasis/debug-history").join(project_hash))
}

fn hash_project_identity(project_root: &std::path::Path) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(project_root.as_os_str().as_encoded_bytes());
    let digest = hasher.finalize();
    let mut s = String::with_capacity(16);
    for b in digest.iter().take(8) {
        use std::fmt::Write as _;
        let _ = write!(&mut s, "{b:02x}");
    }
    s
}

/// Remove obvious secret-shaped lines (TOKEN/SECRET/KEY/PASSWORD/CREDENTIAL).
fn strip_secrets(body: &str) -> String {
    let needle = regex::Regex::new(r"(?i)(TOKEN|SECRET|KEY|PASSWORD|CREDENTIAL)").unwrap();
    body.lines()
        .map(|line| {
            if needle.is_match(line) {
                "<redacted-secret-line>".to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_secrets_redacts_marked_lines() {
        let raw = "ok-line\nTOKEN=abc\nanother=ok\nMM_ADMIN_TOKEN=xx\nfoo: bar\nAPI_KEY=zz";
        let stripped = strip_secrets(raw);
        assert!(!stripped.contains("abc"));
        assert!(!stripped.contains("xx"));
        assert!(!stripped.contains("zz"));
        assert!(stripped.contains("ok-line"));
        assert!(stripped.contains("another=ok"));
        assert!(stripped.contains("foo: bar"));
    }

    #[test]
    fn project_hash_is_stable_per_path() {
        let h1 = hash_project_identity(std::path::Path::new("/work/a"));
        let h2 = hash_project_identity(std::path::Path::new("/work/a"));
        let h3 = hash_project_identity(std::path::Path::new("/work/b"));
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
        assert_eq!(h1.len(), 16);
    }
}
