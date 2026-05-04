//! Scan a project root for agent files and classify each one.
//!
//! Layout convention (Claude Code): agent definitions live at
//! `<project>/.claude/agents/*.md`. We do not recurse into sub-folders —
//! Claude Code itself does not.

use std::path::{Path, PathBuf};

use genasis_core::error::Result;
use genasis_core::frontmatter;
use genasis_core::fs::read_to_string_optional;
use genasis_core::marker;

use crate::role_inference::{infer_from_name, Classified};

/// One agent file the detector found.
#[derive(Debug, Clone)]
pub struct DetectedAgent {
    pub path: PathBuf,
    /// Agent slug, taken from the frontmatter `name:` field. Empty if missing.
    pub name: String,
    pub classification: Classified,
    /// Full file body — kept so the merger can splice in a fence.
    pub raw: String,
    /// True iff the file already has a Genasis fence.
    pub has_existing_fence: bool,
}

#[derive(Debug, Clone, Default)]
pub struct DetectionReport {
    pub agents: Vec<DetectedAgent>,
    /// Files we tried to read but couldn't parse (path, reason).
    pub skipped: Vec<(PathBuf, String)>,
}

/// Walk `<project_root>/.claude/agents/*.md` and classify every match.
pub fn scan(project_root: &Path) -> Result<DetectionReport> {
    let agents_dir = project_root.join(".claude").join("agents");
    let mut report = DetectionReport::default();

    let entries = match std::fs::read_dir(&agents_dir) {
        Ok(rd) => rd,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(report),
        Err(e) => return Err(e.into()),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') || name.starts_with('_') {
                continue;
            }
        }
        match classify_one(&path) {
            Ok(Some(agent)) => report.agents.push(agent),
            Ok(None) => {}
            Err(e) => report.skipped.push((path, e.to_string())),
        }
    }

    report
        .agents
        .sort_by(|a, b| a.path.file_name().cmp(&b.path.file_name()));
    Ok(report)
}

fn classify_one(path: &Path) -> Result<Option<DetectedAgent>> {
    let raw = match read_to_string_optional(path)? {
        Some(s) => s,
        None => return Ok(None),
    };

    let split = frontmatter::split(&raw);
    let name = split
        .frontmatter
        .as_ref()
        .and_then(|fm| frontmatter::read_scalar(fm.raw, "name"))
        .unwrap_or("")
        .to_string();

    let classification = if name.is_empty() {
        Classified::Custom(
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string(),
        )
    } else {
        infer_from_name(&name)
    };

    let has_existing_fence = marker::find(&raw)?.is_some();

    Ok(Some(DetectedAgent {
        path: path.to_path_buf(),
        name,
        classification,
        raw,
        has_existing_fence,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write_agent(root: &Path, file: &str, body: &str) {
        let dir = root.join(".claude/agents");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(file), body).unwrap();
    }

    #[test]
    fn empty_project_yields_empty_report() {
        let d = tempdir().unwrap();
        let r = scan(d.path()).unwrap();
        assert!(r.agents.is_empty());
        assert!(r.skipped.is_empty());
    }

    #[test]
    fn classifies_canonical_roles() {
        let d = tempdir().unwrap();
        write_agent(
            d.path(),
            "frontend.md",
            "---\nname: frontend\n---\n# Frontend Agent\n",
        );
        write_agent(
            d.path(),
            "qa.md",
            "---\nname: e2e-runner\n---\n# QA Agent\n",
        );
        write_agent(
            d.path(),
            "loop-operator.md",
            "---\nname: loop-operator\n---\n# Custom Agent\n",
        );

        let r = scan(d.path()).unwrap();
        assert_eq!(r.agents.len(), 3);

        let frontend = r
            .agents
            .iter()
            .find(|a| a.name == "frontend")
            .expect("frontend present");
        assert!(matches!(frontend.classification, Classified::Known(_)));
        assert!(!frontend.has_existing_fence);

        let custom = r
            .agents
            .iter()
            .find(|a| a.name == "loop-operator")
            .expect("custom agent present");
        match &custom.classification {
            Classified::Custom(s) => assert_eq!(s, "loop-operator"),
            _ => panic!("expected custom classification"),
        }
    }

    #[test]
    fn detects_existing_fence() {
        let d = tempdir().unwrap();
        let body = "---\nname: backend\n---\n\n<!-- GENASIS:BEGIN role=backend version=1.0 hash=00000000 -->\nbody\n<!-- GENASIS:END -->\n# Backend\n";
        write_agent(d.path(), "backend.md", body);
        let r = scan(d.path()).unwrap();
        assert!(r.agents[0].has_existing_fence);
    }

    #[test]
    fn ignores_hidden_underscored_and_non_md_files() {
        let d = tempdir().unwrap();
        // hidden / underscored agent files are skipped by `scan`
        let dir = d.path().join(".claude/agents");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(".hidden.md"), "---\nname: hidden\n---\n").unwrap();
        fs::write(dir.join("_removed.md"), "---\nname: removed\n---\n").unwrap();
        fs::write(dir.join("agents.json"), "{}").unwrap();
        fs::write(dir.join("frontend.md"), "---\nname: frontend\n---\n").unwrap();

        let r = scan(d.path()).unwrap();
        assert_eq!(r.agents.len(), 1);
        assert_eq!(r.agents[0].name, "frontend");
    }

    #[test]
    fn malformed_files_go_to_skipped_not_panicked() {
        let d = tempdir().unwrap();
        // Two BEGIN markers — find() returns Err.
        let body = "---\nname: frontend\n---\n<!-- GENASIS:BEGIN role=x version=1.0 hash=aa -->\nbody\n<!-- GENASIS:END -->\n<!-- GENASIS:BEGIN role=y version=1.0 hash=bb -->\nbody2\n<!-- GENASIS:END -->\n";
        write_agent(d.path(), "frontend.md", body);
        let r = scan(d.path()).unwrap();
        assert_eq!(r.skipped.len(), 1);
    }
}
