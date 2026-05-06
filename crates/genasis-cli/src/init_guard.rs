//! Init safety guard — prevents accidental `genasis init` in wrong directories.
//!
//! Classifies the current working directory into safety levels and either
//! proceeds, asks for confirmation, or blocks with guidance.
//!
//! Safety classification:
//!
//! | Level | Situation | Action |
//! |-------|-----------|--------|
//! | SAFE  | Project with existing agentic setup, user confirmed | Proceed |
//! | ASK   | Ambiguous — needs user confirmation | Prompt + record decision |
//! | BLOCK | Dangerous (system dir, home root, /tmp) | Block with explanation |
//!
//! Decision records are stored in `.genasis-init-decision.json` so that
//! subsequent runs don't re-ask the same question.

use std::path::{Path, PathBuf};

/// Result of directory safety analysis.
#[derive(Debug, Clone)]
pub struct SafetyCheck {
    pub level: SafetyLevel,
    pub reason: String,
    pub suggestion: Option<String>,
    pub context: DirContext,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafetyLevel {
    /// Safe to proceed without asking.
    Safe,
    /// Ambiguous — ask user for confirmation.
    Ask,
    /// Dangerous — block and explain why.
    Block,
}

/// What we found in the directory.
#[derive(Debug, Clone, Default)]
pub struct DirContext {
    pub is_empty: bool,
    pub is_home_dir: bool,
    pub is_system_dir: bool,
    pub is_tmp: bool,
    pub is_root_fs: bool,
    pub has_git: bool,
    pub has_claude_dir: bool,
    pub has_claude_md: bool,
    pub has_agents: bool,
    pub agent_count: usize,
    pub has_genasis_toml: bool,
    pub has_package_json: bool,
    pub has_cargo_toml: bool,
    pub has_src_dir: bool,
    pub has_previous_init: bool,
    pub previous_init_decision: Option<String>,
}

/// Analyze a directory and determine if it's safe to run `genasis init`.
pub fn check(dir: &Path) -> SafetyCheck {
    let ctx = analyze_dir(dir);

    // ── BLOCK situations ──────────────────────────────────────────

    if ctx.is_root_fs {
        return SafetyCheck {
            level: SafetyLevel::Block,
            reason: "Cannot init in filesystem root (/).".into(),
            suggestion: Some("cd to your project directory first.".into()),
            context: ctx,
        };
    }

    if ctx.is_system_dir {
        return SafetyCheck {
            level: SafetyLevel::Block,
            reason: format!("System directory detected: {}", dir.display()),
            suggestion: Some("genasis should be initialized in a project directory, not a system path.".into()),
            context: ctx,
        };
    }

    if ctx.is_tmp {
        return SafetyCheck {
            level: SafetyLevel::Block,
            reason: "Temporary directory (/tmp). Files here are not persistent.".into(),
            suggestion: Some("Create a proper project directory first.".into()),
            context: ctx,
        };
    }

    // ── Previously approved ───────────────────────────────────────

    if let Some(ref decision) = ctx.previous_init_decision {
        if decision == "approved" {
            return SafetyCheck {
                level: SafetyLevel::Safe,
                reason: "Previously approved for genasis init.".into(),
                suggestion: None,
                context: ctx,
            };
        }
    }

    // ── Re-init detection ─────────────────────────────────────────

    if ctx.has_previous_init {
        return SafetyCheck {
            level: SafetyLevel::Ask,
            reason: "genasis was previously initialized here. Re-init will overwrite existing settings (genasis.toml, .env.agents, GENASIS.md, overlay fences).".into(),
            suggestion: Some("Use `genasis upgrade` to update without losing settings, or confirm to re-init.".into()),
            context: ctx,
        };
    }

    // ── SAFE: existing agentic project ────────────────────────────

    if ctx.has_agents && ctx.has_claude_dir {
        return SafetyCheck {
            level: SafetyLevel::Safe,
            reason: format!(
                "Existing agentic project detected ({} agent files in .claude/agents/).",
                ctx.agent_count
            ),
            suggestion: None,
            context: ctx,
        };
    }

    // ── ASK situations ────────────────────────────────────────────

    if ctx.is_home_dir {
        return SafetyCheck {
            level: SafetyLevel::Ask,
            reason: "This is your home directory. genasis is usually initialized inside a specific project, not at home root.".into(),
            suggestion: Some("If this is intentional (global setup), confirm to proceed.".into()),
            context: ctx,
        };
    }

    if ctx.is_empty {
        return SafetyCheck {
            level: SafetyLevel::Ask,
            reason: "This directory is empty. genasis works best when initialized inside an existing project.".into(),
            suggestion: Some("If you're starting a new project here, confirm to proceed.".into()),
            context: ctx,
        };
    }

    if !ctx.has_claude_dir && !ctx.has_git && !ctx.has_package_json && !ctx.has_cargo_toml && !ctx.has_src_dir {
        return SafetyCheck {
            level: SafetyLevel::Ask,
            reason: "No project markers found (no .git, no .claude/, no package.json/Cargo.toml/src/). This may not be a project directory.".into(),
            suggestion: Some("If this is your project root, confirm to proceed.".into()),
            context: ctx,
        };
    }

    // Project markers present but no Claude setup
    if ctx.has_git && !ctx.has_claude_dir {
        return SafetyCheck {
            level: SafetyLevel::Safe,
            reason: "Git project detected. No existing Claude setup — clean init.".into(),
            suggestion: None,
            context: ctx,
        };
    }

    // Claude dir exists but no agents
    if ctx.has_claude_dir && !ctx.has_agents {
        return SafetyCheck {
            level: SafetyLevel::Safe,
            reason: "Claude Code setup found but no agents. Will create agent team.".into(),
            suggestion: None,
            context: ctx,
        };
    }

    // Fallback: ask
    SafetyCheck {
        level: SafetyLevel::Ask,
        reason: "Unrecognized directory layout.".into(),
        suggestion: Some("Confirm this is where you want to initialize genasis.".into()),
        context: ctx,
    }
}

/// Record the user's init decision so we don't re-ask next time.
pub fn record_decision(dir: &Path, approved: bool) {
    let record_path = dir.join(".genasis-init-decision.json");
    let content = serde_json::json!({
        "decision": if approved { "approved" } else { "rejected" },
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "directory": dir.to_string_lossy(),
    });
    // Best-effort write — don't fail if we can't record
    let _ = std::fs::write(&record_path, serde_json::to_string_pretty(&content).unwrap_or_default());
}

fn analyze_dir(dir: &Path) -> DirContext {
    let mut ctx = DirContext::default();

    let canonical = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
    let dir_str = canonical.to_string_lossy();

    // System/special directories
    ctx.is_root_fs = dir_str == "/";
    ctx.is_home_dir = dirs::home_dir()
        .map(|h| canonical == h)
        .unwrap_or(false);
    ctx.is_tmp = dir_str.starts_with("/tmp") || dir_str.starts_with("/var/tmp");
    ctx.is_system_dir = matches!(
        dir_str.as_ref(),
        "/usr" | "/usr/local" | "/etc" | "/var" | "/opt" | "/bin" | "/sbin"
            | "/usr/bin" | "/usr/sbin" | "/usr/lib"
    );

    // Empty check
    ctx.is_empty = std::fs::read_dir(dir)
        .map(|mut rd| rd.next().is_none())
        .unwrap_or(true);

    // Project markers
    ctx.has_git = dir.join(".git").exists();
    ctx.has_package_json = dir.join("package.json").exists();
    ctx.has_cargo_toml = dir.join("Cargo.toml").exists();
    ctx.has_src_dir = dir.join("src").is_dir();

    // Claude setup
    ctx.has_claude_dir = dir.join(".claude").is_dir();
    ctx.has_claude_md = dir.join("CLAUDE.md").exists()
        || dir.join(".claude").join("CLAUDE.md").exists();

    // Agents
    let agents_dir = dir.join(".claude").join("agents");
    if agents_dir.is_dir() {
        let count = std::fs::read_dir(&agents_dir)
            .map(|rd| {
                rd.filter_map(|e| e.ok())
                    .filter(|e| {
                        e.file_name()
                            .to_string_lossy()
                            .ends_with(".md")
                    })
                    .count()
            })
            .unwrap_or(0);
        ctx.has_agents = count > 0;
        ctx.agent_count = count;
    }

    // Previous genasis init
    ctx.has_genasis_toml = dir.join("genasis.toml").exists();
    ctx.has_previous_init = ctx.has_genasis_toml
        || dir.join(".claude").join("genasis").is_dir()
        || dir.join("GENASIS.md").exists();

    // Previous decision record
    let decision_path = dir.join(".genasis-init-decision.json");
    if decision_path.is_file() {
        if let Ok(content) = std::fs::read_to_string(&decision_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                ctx.previous_init_decision = json
                    .get("decision")
                    .and_then(|d| d.as_str())
                    .map(|s| s.to_string());
            }
        }
    }

    ctx
}

/// Format a user-facing safety check message.
pub fn format_check(check: &SafetyCheck, dir: &Path) -> String {
    let mut msg = String::new();
    msg.push_str(&format!("Directory: {}\n\n", dir.display()));

    match check.level {
        SafetyLevel::Block => {
            msg.push_str(&format!("❌ BLOCKED: {}\n", check.reason));
            if let Some(ref sug) = check.suggestion {
                msg.push_str(&format!("   → {sug}\n"));
            }
        }
        SafetyLevel::Ask => {
            msg.push_str(&format!("⚠  {}\n", check.reason));
            if let Some(ref sug) = check.suggestion {
                msg.push_str(&format!("   → {sug}\n"));
            }
            msg.push_str("\n   Proceed with genasis init here? [y/N] ");
        }
        SafetyLevel::Safe => {
            msg.push_str(&format!("✓  {}\n", check.reason));
        }
    }

    // Context summary
    let ctx = &check.context;
    let mut markers = Vec::new();
    if ctx.has_git { markers.push("git"); }
    if ctx.has_claude_dir { markers.push(".claude/"); }
    if ctx.has_agents { markers.push(&format!("{}agents", ctx.agent_count)); }
    if ctx.has_genasis_toml { markers.push("genasis.toml"); }
    if ctx.has_package_json { markers.push("package.json"); }
    if ctx.has_cargo_toml { markers.push("Cargo.toml"); }
    if !markers.is_empty() {
        msg.push_str(&format!("\n   Detected: {}\n", markers.join(" · ")));
    }

    msg
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn empty_dir_asks() {
        let d = tempdir().unwrap();
        let check = check(d.path());
        assert_eq!(check.level, SafetyLevel::Ask);
        assert!(check.reason.contains("empty"));
    }

    #[test]
    fn git_project_no_claude_is_safe() {
        let d = tempdir().unwrap();
        std::fs::create_dir(d.path().join(".git")).unwrap();
        let check = check(d.path());
        assert_eq!(check.level, SafetyLevel::Safe);
    }

    #[test]
    fn previous_init_warns() {
        let d = tempdir().unwrap();
        std::fs::write(d.path().join("genasis.toml"), "[project]\nname=\"test\"").unwrap();
        let check = check(d.path());
        assert_eq!(check.level, SafetyLevel::Ask);
        assert!(check.reason.contains("previously initialized"));
    }

    #[test]
    fn approved_decision_is_safe() {
        let d = tempdir().unwrap();
        let decision = serde_json::json!({"decision": "approved"});
        std::fs::write(
            d.path().join(".genasis-init-decision.json"),
            serde_json::to_string(&decision).unwrap(),
        ).unwrap();
        let check = check(d.path());
        assert_eq!(check.level, SafetyLevel::Safe);
    }

    #[test]
    fn root_is_blocked() {
        let check = check(Path::new("/"));
        assert_eq!(check.level, SafetyLevel::Block);
    }

    #[test]
    fn existing_agents_is_safe() {
        let d = tempdir().unwrap();
        let agents = d.path().join(".claude/agents");
        std::fs::create_dir_all(&agents).unwrap();
        std::fs::write(agents.join("pm.md"), "---\nname: pm\n---\n").unwrap();
        let check = check(d.path());
        assert_eq!(check.level, SafetyLevel::Safe);
    }

    #[test]
    fn no_project_markers_asks() {
        let d = tempdir().unwrap();
        std::fs::write(d.path().join("random.txt"), "hello").unwrap();
        let check = check(d.path());
        assert_eq!(check.level, SafetyLevel::Ask);
        assert!(check.reason.contains("No project markers"));
    }
}
