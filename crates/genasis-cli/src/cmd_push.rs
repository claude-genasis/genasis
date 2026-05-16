//! `genasis push` — ADR-020 showcase-push client.
//!
//! Packs the user's built static bundle into a gzipped tarball and
//! uploads it to the operator's trial-app at
//! `POST /api/trial/showcase-push?team=<token>`. The operator unpacks
//! and serves it from `https://mmplane-trial.realstory.blog/dev/<token>/`,
//! so the demo URL stays live even after the user's machine sleeps.
//!
//! Auto-detection: when `--dir` is omitted we look for `dist/`,
//! `build/`, or `out/` under `<project>/app/` (the layout the
//! frontend/devops agents leave behind) and pick the first one
//! containing an `index.html`.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::Parser;

use genasis_core::config::Config;

#[derive(Parser, Debug)]
pub struct Args {
    /// Project root. Defaults to the current working directory. We
    /// need this to read `genasis.toml` and discover the team token
    /// + trial URL.
    #[arg(long, value_name = "DIR")]
    pub project: Option<PathBuf>,

    /// Directory containing the built static bundle (must include
    /// `index.html` at the top). When omitted we auto-detect
    /// `app/dist`, `app/build`, `dist`, `build`, `out`.
    #[arg(long, value_name = "DIR")]
    pub dir: Option<PathBuf>,

    /// Skip auto-detect candidates and use this tar.gz directly. For
    /// CI / scripts that already produce a deploy artifact.
    #[arg(long, value_name = "PATH")]
    pub tarball: Option<PathBuf>,

    /// Print the resolved request body size + URL and exit without
    /// uploading.
    #[arg(long)]
    pub dry_run: bool,
}

pub async fn run(args: Args) -> Result<()> {
    let project_root = match args.project {
        Some(p) => p,
        None => std::env::current_dir().context("read cwd for --project default")?,
    };
    let cfg_path = Config::discover_or_descend(&project_root).ok_or_else(|| {
        anyhow::anyhow!(
            "no `genasis.toml` found near {} — run `genasis init --trial` \
             first or pass `--project <dir>` pointing at one",
            project_root.display()
        )
    })?;
    let cfg = Config::load(&cfg_path).context("load genasis.toml")?;
    let team_token = cfg
        .trial
        .as_ref()
        .and_then(|t| t.team_token.clone())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "{} has no [trial].team_token — `genasis push` only \
                 supports the trial flavor today",
                cfg_path.display()
            )
        })?;
    let trial_url = cfg
        .trial
        .as_ref()
        .map(|t| t.url.clone())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "https://mmplane-trial.realstory.blog".to_string());

    // Resolve the bundle. Three paths: --tarball wins, then --dir,
    // then auto-detection.
    let tarball_bytes: Vec<u8> = if let Some(p) = args.tarball.as_ref() {
        std::fs::read(p).with_context(|| format!("read tarball {}", p.display()))?
    } else {
        let bundle_dir = match args.dir {
            Some(d) => d,
            None => auto_detect_bundle(&project_root)?,
        };
        if !bundle_dir.is_dir() {
            bail!(
                "bundle directory {} does not exist or is not a directory",
                bundle_dir.display()
            );
        }
        if !bundle_dir.join("index.html").is_file() {
            eprintln!(
                "⚠ {} has no index.html — the trial-app will serve 404 on / \
                 until the build produces one.",
                bundle_dir.display()
            );
        }
        println!("→ packing {} into a gzipped tarball…", bundle_dir.display());
        pack_dir_tar_gz(&bundle_dir)?
    };

    let url = format!(
        "{}/api/trial/showcase-push?team={}",
        trial_url.trim_end_matches('/'),
        urlencode(&team_token)
    );

    println!("→ {} bytes → {}", tarball_bytes.len(), url);
    if args.dry_run {
        println!("  [dry-run] would POST but skipping (--dry-run).");
        return Ok(());
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .context("reqwest client")?;
    let resp = client
        .post(&url)
        .header("Content-Type", "application/gzip")
        .header("X-Genasis-Team-Token", &team_token)
        .body(tarball_bytes)
        .send()
        .await
        .with_context(|| format!("POST {url}"))?;
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        bail!("showcase-push failed: {status} — {body}");
    }
    println!("✓ pushed. Server response: {body}");
    let landing = format!(
        "{}/dev/{}/",
        trial_url.trim_end_matches('/'),
        urlencode(&team_token)
    );
    println!("\n  Live URL: {landing}");
    println!("  (open this in any browser — the user's local machine is no");
    println!("  longer required; the operator's trial-app now serves the");
    println!("  bundle directly from disk.)");
    Ok(())
}

fn auto_detect_bundle(project_root: &Path) -> Result<PathBuf> {
    let candidates = [
        "app/dist",
        "app/build",
        "app/out",
        "dist",
        "build",
        "out",
    ];
    for c in &candidates {
        let p = project_root.join(c);
        if p.join("index.html").is_file() {
            println!("→ auto-detected bundle: {}", p.display());
            return Ok(p);
        }
    }
    bail!(
        "could not auto-detect a built bundle under {}. Tried: app/dist, \
         app/build, app/out, dist, build, out. Pass `--dir <path>` to \
         pick one explicitly.",
        project_root.display()
    )
}

/// Stream-pack a directory into an in-memory gzipped tarball using
/// only the Rust std + a thin gzip writer. We avoid pulling tar-rs +
/// flate2 because the operator side is the only place actually
/// unpacking; on the client we just need a wire-format-compatible
/// archive of <= ~50 MB. Shell out to `tar -czf` which every install
/// already has.
fn pack_dir_tar_gz(dir: &Path) -> Result<Vec<u8>> {
    use std::process::{Command, Stdio};

    let abs = dir.canonicalize().with_context(|| {
        format!("canonicalize bundle dir {}", dir.display())
    })?;
    let parent = abs.parent().unwrap_or(&abs);
    let basename = abs
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("bundle path has no file name"))?;

    // We tar the directory contents (not the directory itself) so
    // the operator's unpack lands the user's `index.html` at the
    // root of the team's storage dir.
    let output = Command::new("tar")
        .arg("-czf")
        .arg("-")
        .arg("-C")
        .arg(&abs)
        .arg(".")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .with_context(|| {
            format!(
                "spawn `tar -czf - -C {} .` (bundle parent: {})",
                abs.display(),
                parent.display()
            )
        })?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        bail!(
            "tar failed for bundle {} ({:?}): {}",
            basename,
            output.status,
            err.trim()
        );
    }
    // Suppress unused-warning on basename for the success path —
    // we keep it for the error message above.
    let _ = basename;
    Ok(output.stdout)
}

fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                out.push('%');
                out.push_str(&format!("{:02X}", b));
            }
        }
    }
    out
}

