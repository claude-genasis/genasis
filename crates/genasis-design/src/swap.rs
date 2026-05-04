//! `genasis design swap` — set or replace the active external design.
//!
//! Two entry shapes share one pipeline:
//!
//! - `Source::Slug { slug, add_command }` — runs the configured shell
//!   template (default: `npx getdesign@latest add {slug} --force --out
//!   {out}`) so the chosen brand's `DESIGN.md` lands at
//!   `<external_dir>/DESIGN.md`.
//! - `Source::File(path)` — copies a local spec file to the same target
//!   (no network, no npm dependency).
//!
//! Both paths share the steps that follow:
//!  1. ensure `<external_dir>/` exists.
//!  2. on first transition Pristine→External, copy
//!     `docs/design-system.md` to `<external_dir>/pristine.bak`.
//!  3. fetch (slug) or copy (file) the new `DESIGN.md`.
//!  4. compute sha256, render the pointer body, write design-system.md.
//!  5. update `.design-state.toml`.
//!
//! User-override §B accumulation across swaps is preserved by reading the
//! current pointer body's §B.2 block before re-rendering and re-injecting
//! it into the new pointer (M-D2). M-D1 just emits the empty skeleton —
//! `override_count` stays at 0 because no override commands exist yet.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use genasis_core::error::{Error, Result};
use genasis_core::fs::{atomic_write, read_to_string_optional};

use crate::mode::{iso8601_now, sha256_hex, Mode, State};
use crate::pointer::{self, Locale};

#[derive(Debug, Clone)]
pub enum Source {
    /// Fetch via the configured shell template (default: `npx getdesign`).
    Slug {
        slug: String,
        add_command: String,
    },
    /// Copy a local spec file. Source string becomes `file:<absolute-path>`.
    File(PathBuf),
}

#[derive(Debug, Clone)]
pub struct SwapInput {
    pub project_root: PathBuf,
    pub external_dir: String,
    pub gallery_index_url: String,
    pub gallery_url_template: String,
    pub disable_telemetry: bool,
    pub locale: Locale,
    pub source: Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapOutcome {
    pub previous_state: State,
    pub new_state: State,
    pub design_md_path: PathBuf,
    pub pointer_path: PathBuf,
    pub pristine_backup_path: Option<PathBuf>,
    pub design_md_size: u64,
    pub fetch_command: Option<String>,
}

pub fn run(input: SwapInput) -> Result<SwapOutcome> {
    let previous_state = State::load(&input.project_root)?;

    let project_root = input.project_root.clone();
    let external_dir_abs = project_root.join(&input.external_dir);
    std::fs::create_dir_all(&external_dir_abs)?;

    let pointer_path = project_root.join("docs").join("design-system.md");
    let design_md_path = external_dir_abs.join("DESIGN.md");

    // Step 2 — Pristine→External: snapshot the existing design-system.md so
    // `restore` can put it back. We don't use the generic snapshot helper
    // because the backup needs a stable, well-known name (pristine.bak).
    let pristine_backup_path = if previous_state.mode == Mode::Pristine {
        let bak = external_dir_abs.join("pristine.bak");
        if let Some(body) = read_to_string_optional(&pointer_path)? {
            atomic_write(&bak, body.as_bytes())?;
            Some(bak)
        } else {
            // No prior body — backup is a zero-byte marker so restore knows
            // there *was* a pristine state (vs. fresh project).
            atomic_write(&bak, b"")?;
            Some(bak)
        }
    } else {
        None
    };

    // Step 3 — fetch or copy the new DESIGN.md.
    let (source_label, fetch_command, slug) = match &input.source {
        Source::Slug { slug, add_command } => {
            let cmd = render_add_command(add_command, slug, &design_md_path);
            run_shell(&cmd, input.disable_telemetry)?;
            if !design_md_path.is_file() {
                return Err(Error::Config(format!(
                    "add_command did not produce {} — check `{}`",
                    design_md_path.display(),
                    cmd
                )));
            }
            (format!("getdesign/{slug}"), Some(cmd), slug.clone())
        }
        Source::File(path) => {
            let abs = if path.is_absolute() {
                path.clone()
            } else {
                std::env::current_dir()?.join(path)
            };
            let body = read_to_string_optional(&abs)?.ok_or_else(|| {
                Error::Config(format!("--from path not found: {}", abs.display()))
            })?;
            atomic_write(&design_md_path, body.as_bytes())?;
            let label = format!("file:{}", abs.display());
            // For `--from`, the slug is the file stem so monitor / preview
            // URLs have something to render. Empty if the file has no stem.
            let slug = abs
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("local")
                .to_string();
            (label, None, slug)
        }
    };

    // Step 4 — compute hash, render pointer, persist.
    let design_body = std::fs::read_to_string(&design_md_path)?;
    let template_hash = sha256_hex(&design_body);
    let gallery_preview = render_preview_url(&input.gallery_url_template, &slug);

    let new_state = State {
        mode: Mode::External,
        slug: slug.clone(),
        source: source_label,
        source_command: fetch_command.clone().unwrap_or_default(),
        template_hash: template_hash.clone(),
        applied_at: iso8601_now(),
        previous_slug: if previous_state.mode == Mode::External {
            previous_state.slug.clone()
        } else {
            String::new()
        },
        gallery_preview,
        gallery_index: input.gallery_index_url.clone(),
        override_count: 0,
    };

    let pointer_body = pointer::render(&new_state, &input.external_dir, input.locale);
    atomic_write(&pointer_path, pointer_body.as_bytes())?;
    new_state.save(&project_root)?;

    let design_md_size = std::fs::metadata(&design_md_path).map(|m| m.len()).unwrap_or(0);

    Ok(SwapOutcome {
        previous_state,
        new_state,
        design_md_path,
        pointer_path,
        pristine_backup_path,
        design_md_size,
        fetch_command,
    })
}

fn render_add_command(template: &str, slug: &str, out: &Path) -> String {
    template
        .replace("{slug}", slug)
        .replace("{out}", &out.display().to_string())
}

fn render_preview_url(template: &str, slug: &str) -> String {
    template.replace("{slug}", slug)
}

fn run_shell(cmd: &str, disable_telemetry: bool) -> Result<()> {
    let mut command = Command::new("sh");
    command.arg("-c").arg(cmd);
    if disable_telemetry {
        command.env("GETDESIGN_DISABLE_TELEMETRY", "1");
    }
    let status = command
        .status()
        .map_err(|e| Error::Config(format!("spawn `{cmd}` failed: {e}")))?;
    if !status.success() {
        return Err(Error::Config(format!(
            "`{cmd}` exited with status {status}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    /// `--from` path — uses pure filesystem, no npx, so it round-trips in
    /// any environment.
    #[test]
    fn from_local_file_pristine_to_external() {
        let dir = tempdir().unwrap();
        let docs = dir.path().join("docs");
        std::fs::create_dir_all(&docs).unwrap();
        std::fs::write(docs.join("design-system.md"), "# pristine body\n").unwrap();

        let local = dir.path().join("brand-spec.md");
        std::fs::write(&local, "# external body\ncolor: red\n").unwrap();

        let out = run(SwapInput {
            project_root: dir.path().to_path_buf(),
            external_dir: "docs/design-system".into(),
            gallery_index_url: "https://getdesign.md/".into(),
            gallery_url_template: "https://getdesign.md/{slug}/design-md".into(),
            disable_telemetry: true,
            locale: Locale::En,
            source: Source::File(local),
        })
        .unwrap();

        // pristine backup created
        assert!(out.pristine_backup_path.unwrap().is_file());
        // DESIGN.md written
        let design_md = std::fs::read_to_string(&out.design_md_path).unwrap();
        assert!(design_md.contains("external body"));
        // pointer body now in design-system.md
        let ptr = std::fs::read_to_string(&out.pointer_path).unwrap();
        assert!(ptr.contains("External Reference"));
        // state is external
        let s = State::load(dir.path()).unwrap();
        assert_eq!(s.mode, Mode::External);
        assert_eq!(s.slug, "brand-spec");
        assert!(s.source.starts_with("file:"));
        assert_eq!(s.template_hash.len(), 64);
    }

    #[test]
    fn second_swap_records_previous_slug() {
        let dir = tempdir().unwrap();
        let docs = dir.path().join("docs");
        std::fs::create_dir_all(&docs).unwrap();
        std::fs::write(docs.join("design-system.md"), "# pristine\n").unwrap();

        // first swap
        let a = dir.path().join("a.md");
        std::fs::write(&a, "# A\n").unwrap();
        run(SwapInput {
            project_root: dir.path().to_path_buf(),
            external_dir: "docs/design-system".into(),
            gallery_index_url: "https://getdesign.md/".into(),
            gallery_url_template: "https://getdesign.md/{slug}/design-md".into(),
            disable_telemetry: true,
            locale: Locale::En,
            source: Source::File(a),
        })
        .unwrap();

        // second swap
        let b = dir.path().join("b.md");
        std::fs::write(&b, "# B\n").unwrap();
        let out = run(SwapInput {
            project_root: dir.path().to_path_buf(),
            external_dir: "docs/design-system".into(),
            gallery_index_url: "https://getdesign.md/".into(),
            gallery_url_template: "https://getdesign.md/{slug}/design-md".into(),
            disable_telemetry: true,
            locale: Locale::En,
            source: Source::File(b),
        })
        .unwrap();

        assert_eq!(out.new_state.previous_slug, "a");
        assert_eq!(out.new_state.slug, "b");
    }
}
