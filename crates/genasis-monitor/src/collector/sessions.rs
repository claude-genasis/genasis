//! Claude Code session detection.
//!
//! Scans running processes to find active Claude Code sessions,
//! then matches them to project paths and agent roles.
//!
//! Reference: `/work/secusy/scripts/agent_monitor.py` MonitorCollector.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use sysinfo::{ProcessRefreshKind, RefreshKind, System};

/// A detected Claude Code session.
#[derive(Debug, Clone)]
pub struct ClaudeSession {
    pub pid: u32,
    pub cwd: String,
    pub role: Option<String>,
    pub age_secs: u64,
    pub state: SessionState,
    pub context_pct: Option<f32>,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Active,
    Idle,
    Error,
}

impl Default for SessionState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Detect all running Claude Code processes and classify them.
pub fn detect_sessions(project_root: &Path, worktree_prefix: &str) -> Vec<ClaudeSession> {
    let mut sys = System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
    );
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let mut sessions = Vec::new();

    for (pid, process) in sys.processes() {
        let cmd = process.cmd();
        let cmd_str: String = cmd
            .iter()
            .map(|s| s.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(" ");

        // Look for Claude Code processes
        if !is_claude_process(&cmd_str) {
            continue;
        }

        let cwd = process
            .cwd()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        // Filter to project-related sessions
        if !is_project_path(&cwd, project_root, worktree_prefix) {
            continue;
        }

        let role = infer_role_from_path(&cwd, project_root, worktree_prefix);

        let start_time = process.start_time();
        let age = if start_time > 0 {
            now.saturating_sub(start_time)
        } else {
            0
        };

        let state = if process.status() == sysinfo::ProcessStatus::Zombie {
            SessionState::Error
        } else {
            SessionState::Active
        };

        sessions.push(ClaudeSession {
            pid: pid.as_u32(),
            cwd,
            role,
            age_secs: age,
            state,
            context_pct: None, // Filled by JSONL parser
            session_id: None,  // Filled by JSONL parser
        });
    }

    sessions.sort_by_key(|s| s.pid);
    sessions
}

/// Check if a process command line looks like a Claude Code session.
fn is_claude_process(cmd: &str) -> bool {
    cmd.contains("claude")
        && (cmd.contains("--session") || cmd.contains("code") || cmd.contains("claude-code"))
}

/// Check if a path belongs to this project (main dir or worktree).
fn is_project_path(cwd: &str, project_root: &Path, worktree_prefix: &str) -> bool {
    let root_str = project_root.to_string_lossy();
    cwd == root_str.as_ref()
        || cwd.starts_with(&format!("{}/", root_str))
        || (!worktree_prefix.is_empty() && cwd.starts_with(worktree_prefix))
}

/// Infer which agent role a session belongs to, based on its cwd.
///
/// Convention: worktrees are at `/tmp/{project}-{role}/` or
/// the main project root is the `master` / `pm` role.
fn infer_role_from_path(cwd: &str, project_root: &Path, worktree_prefix: &str) -> Option<String> {
    let root_str = project_root.to_string_lossy().to_string();
    if cwd == root_str || cwd.starts_with(&format!("{}/", root_str)) {
        return Some("master".into());
    }
    if !worktree_prefix.is_empty() && cwd.starts_with(worktree_prefix) {
        let suffix = &cwd[worktree_prefix.len()..];
        // suffix is like "frontend/" or "frontend"
        let role = suffix.trim_end_matches('/').split('/').next().unwrap_or("");
        if !role.is_empty() {
            return Some(role.to_string());
        }
    }
    None
}

/// Format seconds as human-readable age string.
pub fn format_age(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m", seconds / 60)
    } else if seconds < 86400 {
        format!("{}h", seconds / 3600)
    } else {
        format!("{}d", seconds / 86400)
    }
}

/// Format a countdown (seconds remaining) as human-readable.
pub fn format_countdown(remaining_secs: i64) -> String {
    if remaining_secs <= 0 {
        return "reset pending".into();
    }
    let hours = remaining_secs / 3600;
    let mins = (remaining_secs % 3600) / 60;
    if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_claude_process_matches() {
        assert!(is_claude_process(
            "node /usr/bin/claude --session abc123 code"
        ));
        assert!(is_claude_process(
            "/home/user/.local/bin/claude-code --session xyz"
        ));
        assert!(!is_claude_process("node /usr/bin/npm install"));
        assert!(!is_claude_process("vim claude.md"));
    }

    #[test]
    fn is_project_path_works() {
        let root = Path::new("/work/myproject");
        assert!(is_project_path("/work/myproject", root, "/tmp/myproject-"));
        assert!(is_project_path(
            "/work/myproject/src",
            root,
            "/tmp/myproject-"
        ));
        assert!(is_project_path(
            "/tmp/myproject-frontend",
            root,
            "/tmp/myproject-"
        ));
        assert!(!is_project_path("/work/other", root, "/tmp/myproject-"));
    }

    #[test]
    fn infer_role_from_worktree() {
        let root = Path::new("/work/myproject");
        assert_eq!(
            infer_role_from_path("/work/myproject", root, "/tmp/myproject-"),
            Some("master".into())
        );
        assert_eq!(
            infer_role_from_path("/tmp/myproject-frontend", root, "/tmp/myproject-"),
            Some("frontend".into())
        );
        assert_eq!(
            infer_role_from_path("/tmp/myproject-qa/tests", root, "/tmp/myproject-"),
            Some("qa".into())
        );
    }

    #[test]
    fn format_age_display() {
        assert_eq!(format_age(30), "30s");
        assert_eq!(format_age(120), "2m");
        assert_eq!(format_age(7200), "2h");
        assert_eq!(format_age(86400), "1d");
    }

    #[test]
    fn format_countdown_display() {
        assert_eq!(format_countdown(0), "reset pending");
        assert_eq!(format_countdown(-10), "reset pending");
        assert_eq!(format_countdown(3600 + 120), "1h 2m");
        assert_eq!(format_countdown(300), "5m");
    }
}
