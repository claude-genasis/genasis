//! `genasis design override` — accumulate, list, and remove user overrides
//! that win over the external `DESIGN.md`.
//!
//! The active design's `docs/design-system.md` (in external mode) holds a
//! §B.2 sentinel:
//!
//! ```text
//! ### B.2 Accumulated overrides (chronological)
//!
//! <!-- genasis design override add appends here. Do not edit by hand. -->
//! ```
//!
//! `add` appends an entry with id `override-<count+1>` immediately after
//! the sentinel. `remove <id>` deletes the matching block. `list` parses
//! the section and returns each entry's id, timestamp, and body.
//!
//! Conflict detection (§B.1 of the skill) is not done here — that is the
//! agent's job, with this module called only after the user has approved
//! the override. We bump `.design-state.toml.override_count` in `add` and
//! decrement in `remove` so the monitor and `status` stay accurate.

use std::path::Path;

use serde::{Deserialize, Serialize};

use genasis_core::error::{Error, Result};
use genasis_core::fs::atomic_write;

use crate::mode::{iso8601_now, Mode, State};

const SENTINEL: &str =
    "<!-- genasis design override add appends here. Do not edit by hand. -->";
const KO_SENTINEL: &str =
    "<!-- genasis design override add 가 자동 append. 직접 편집 금지. -->";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideEntry {
    pub id: String,
    pub applied_at: String,
    pub body: String,
}

pub fn add(project_root: &Path, body: &str) -> Result<OverrideEntry> {
    let mut state = State::load(project_root)?;
    if state.mode == Mode::Pristine {
        return Err(Error::Config(
            "overrides only apply in external mode — current mode is pristine".to_string(),
        ));
    }
    let next_id = format!("override-{}", state.override_count + 1);
    let ts = iso8601_now();
    let entry = OverrideEntry {
        id: next_id.clone(),
        applied_at: ts.clone(),
        body: body.trim().to_string(),
    };
    let pointer_path = project_root.join("docs").join("design-system.md");
    let pointer = std::fs::read_to_string(&pointer_path).map_err(|e| {
        Error::Config(format!(
            "read pointer {}: {e}",
            pointer_path.display()
        ))
    })?;
    let new_pointer = insert_entry(&pointer, &entry)?;
    atomic_write(&pointer_path, new_pointer.as_bytes())?;

    state.override_count += 1;
    state.save(project_root)?;
    Ok(entry)
}

pub fn list(project_root: &Path) -> Result<Vec<OverrideEntry>> {
    let pointer_path = project_root.join("docs").join("design-system.md");
    let pointer = match std::fs::read_to_string(&pointer_path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };
    Ok(parse_entries(&pointer))
}

pub fn remove(project_root: &Path, id: &str) -> Result<bool> {
    let mut state = State::load(project_root)?;
    let pointer_path = project_root.join("docs").join("design-system.md");
    let pointer = std::fs::read_to_string(&pointer_path)?;
    let (new_pointer, removed) = strip_entry(&pointer, id);
    if !removed {
        return Ok(false);
    }
    atomic_write(&pointer_path, new_pointer.as_bytes())?;
    if state.override_count > 0 {
        state.override_count -= 1;
    }
    state.save(project_root)?;
    Ok(true)
}

fn insert_entry(pointer: &str, entry: &OverrideEntry) -> Result<String> {
    let block = format!(
        "\n#### {} @ {}\n\n{}\n",
        entry.id, entry.applied_at, entry.body
    );
    if let Some(idx) = pointer.find(SENTINEL) {
        let cut = idx + SENTINEL.len();
        let mut out = String::with_capacity(pointer.len() + block.len());
        out.push_str(&pointer[..cut]);
        out.push_str(&block);
        out.push_str(&pointer[cut..]);
        return Ok(out);
    }
    if let Some(idx) = pointer.find(KO_SENTINEL) {
        let cut = idx + KO_SENTINEL.len();
        let mut out = String::with_capacity(pointer.len() + block.len());
        out.push_str(&pointer[..cut]);
        out.push_str(&block);
        out.push_str(&pointer[cut..]);
        return Ok(out);
    }
    Err(Error::Config(
        "pointer body has no §B.2 sentinel — was it edited by hand or never written by genasis?"
            .to_string(),
    ))
}

fn parse_entries(pointer: &str) -> Vec<OverrideEntry> {
    let mut out = Vec::new();
    let mut current: Option<(String, String, String)> = None;
    for line in pointer.lines() {
        if let Some(rest) = line.strip_prefix("#### ") {
            // flush previous
            if let Some((id, ts, body)) = current.take() {
                out.push(OverrideEntry {
                    id,
                    applied_at: ts,
                    body: body.trim_end().to_string(),
                });
            }
            // parse "override-N @ <ts>"
            let mut parts = rest.splitn(2, " @ ");
            let id = parts.next().unwrap_or("").trim().to_string();
            let ts = parts.next().unwrap_or("").trim().to_string();
            current = Some((id, ts, String::new()));
        } else if let Some((_, _, ref mut body)) = current.as_mut() {
            // Stop accumulating if we hit the next §C section header.
            if line.starts_with("## §C") || line.starts_with("## §A") || line.starts_with("## §B") {
                break;
            }
            body.push_str(line);
            body.push('\n');
        }
    }
    if let Some((id, ts, body)) = current {
        out.push(OverrideEntry {
            id,
            applied_at: ts,
            body: body.trim_end().to_string(),
        });
    }
    out.into_iter().filter(|e| !e.id.is_empty()).collect()
}

fn strip_entry(pointer: &str, target_id: &str) -> (String, bool) {
    let mut out = String::with_capacity(pointer.len());
    let mut in_target = false;
    let mut removed = false;
    for line in pointer.lines() {
        if let Some(rest) = line.strip_prefix("#### ") {
            let id = rest.split(" @ ").next().unwrap_or("").trim();
            if id == target_id {
                in_target = true;
                removed = true;
                continue;
            } else {
                in_target = false;
            }
        } else if line.starts_with("## §") {
            // Section break — leave any stripped state and copy through.
            in_target = false;
        }
        if !in_target {
            out.push_str(line);
            out.push('\n');
        }
    }
    (out, removed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pointer::Locale;
    use crate::swap::{self, Source, SwapInput};
    use tempfile::tempdir;

    fn setup_external(dir: &Path) {
        let docs = dir.join("docs");
        std::fs::create_dir_all(&docs).unwrap();
        std::fs::write(docs.join("design-system.md"), "# pristine\n").unwrap();
        let local = dir.join("spec.md");
        std::fs::write(&local, "# external\n").unwrap();
        swap::run(SwapInput {
            project_root: dir.to_path_buf(),
            external_dir: "docs/design-system".into(),
            gallery_index_url: "https://getdesign.md/".into(),
            gallery_url_template: "https://getdesign.md/{slug}/design-md".into(),
            disable_telemetry: true,
            locale: Locale::En,
            source: Source::File(local),
        })
        .unwrap();
    }

    #[test]
    fn add_in_pristine_errors() {
        let dir = tempdir().unwrap();
        let err = add(dir.path(), "primary should be red").unwrap_err();
        assert!(err.to_string().contains("pristine"));
    }

    #[test]
    fn add_then_list_returns_entry() {
        let dir = tempdir().unwrap();
        setup_external(dir.path());
        let e = add(dir.path(), "primary should be red").unwrap();
        assert_eq!(e.id, "override-1");
        let entries = list(dir.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "override-1");
        assert!(entries[0].body.contains("primary should be red"));
        assert_eq!(State::load(dir.path()).unwrap().override_count, 1);
    }

    #[test]
    fn add_two_then_remove_first() {
        let dir = tempdir().unwrap();
        setup_external(dir.path());
        add(dir.path(), "first override").unwrap();
        add(dir.path(), "second override").unwrap();
        assert_eq!(State::load(dir.path()).unwrap().override_count, 2);

        let removed = remove(dir.path(), "override-1").unwrap();
        assert!(removed);
        let entries = list(dir.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "override-2");
        assert_eq!(State::load(dir.path()).unwrap().override_count, 1);
    }

    #[test]
    fn remove_unknown_returns_false() {
        let dir = tempdir().unwrap();
        setup_external(dir.path());
        assert!(!remove(dir.path(), "override-99").unwrap());
    }

    #[test]
    fn ko_pointer_sentinel_works() {
        let dir = tempdir().unwrap();
        let docs = dir.path().join("docs");
        std::fs::create_dir_all(&docs).unwrap();
        std::fs::write(docs.join("design-system.md"), "# pristine\n").unwrap();
        let local = dir.path().join("spec.md");
        std::fs::write(&local, "# external\n").unwrap();
        swap::run(SwapInput {
            project_root: dir.path().to_path_buf(),
            external_dir: "docs/design-system".into(),
            gallery_index_url: "https://getdesign.md/".into(),
            gallery_url_template: "https://getdesign.md/{slug}/design-md".into(),
            disable_telemetry: true,
            locale: Locale::Ko,
            source: Source::File(local),
        })
        .unwrap();
        let e = add(dir.path(), "primary 빨강").unwrap();
        assert_eq!(e.id, "override-1");
        let entries = list(dir.path()).unwrap();
        assert_eq!(entries.len(), 1);
    }
}
