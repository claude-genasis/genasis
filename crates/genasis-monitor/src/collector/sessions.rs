//! Claude Code session detection.
//!
//! Scans running processes to find active Claude Code sessions,
//! then matches them to project paths and agent roles.
//!
//! D-097 (근본 fix): 기존 `is_claude_process` 는 `cmd.contains("code")`
//! substring 검사로 vscode-installed claude (`vscode-server-...`
//! 또는 `claude-code-2.1.x` path) 만 catch 하고 daemon 이 spawn 한
//! `/home/<user>/.npm-global/bin/claude -p --mcp-config ...` 는 놓쳤다.
//! 그래서 사용자의 SESSIONS 가 vscode shell 만 5개 보이고 실제
//! genasis daemon 이 띄운 claude 는 한 번도 안 보였다. 새 detection
//! 은 argv[0] 의 basename 이 "claude" 인지 확인 + cmdline 의 `-p` +
//! `--mcp-config` 패턴으로 daemon-spawn 여부까지 분류.
//!
//! Reference: `/work/secusy/scripts/agent_monitor.py` MonitorCollector.

use std::path::Path;
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

        // Filter to project-related sessions (D-073: relaxed — empty
        // project_root means "show everything", so the user always sees
        // running claude sessions even before cfg discovery picks a
        // sandbox).
        if !is_project_path_relaxed(&cwd, project_root, worktree_prefix) {
            continue;
        }

        // D-097: prefer launch-kind based labeling so the user sees
        // "daemon" for genasis-spawned claude vs "session" for their
        // own IDE-attached resume, instead of every row being "master".
        let role = infer_role_from_path(&cwd, project_root, worktree_prefix)
            .or_else(|| Some(classify_launch_kind(&cmd_str).to_string()));

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
///
/// D-097: argv[0] 의 basename 이 정확히 "claude" (또는 "claude-code")
/// 인 경우만 catch. 기존 코드의 `cmd.contains("code")` 는
/// `/home/<u>/.vscode-server-insiders/...` path 안의 "code" 만
/// catch 해서 daemon 이 spawn 한 `/home/<u>/.npm-global/bin/claude
/// -p --mcp-config ...` 를 놓쳤다.
fn is_claude_process(cmd: &str) -> bool {
    let first = cmd.split_whitespace().next().unwrap_or("");
    let basename = Path::new(first)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    basename == "claude" || basename == "claude-code"
}

/// D-097: Distinguish how the claude session was launched so the
/// SESSIONS widget can show a useful role label.
///
/// - `daemon`: spawned by `genasis listen` (carries `--mcp-config`
///   and `--append-system-prompt` flags)
/// - `interactive`: regular user `claude` invocation
fn classify_launch_kind(cmd: &str) -> &'static str {
    // The daemon always passes both -p (--print) and --mcp-config in
    // a single argv list. Either signal alone is too weak.
    if cmd.contains("--mcp-config") && (cmd.contains(" -p ") || cmd.contains("--print")) {
        "daemon"
    } else if cmd.contains("--session") || cmd.contains("--resume") {
        "session"
    } else {
        "interactive"
    }
}

/// D-073: project_root 가 빈 경로면 모든 claude 프로세스를 보여줌 (사용자가
/// monitor 를 어디서 띄웠든 "지금 머신에서 도는 claude 세션" 을 확인할 수
/// 있게). 그렇지 않으면 기존 strict path match 유지.
fn is_project_path_relaxed(cwd: &str, project_root: &Path, worktree_prefix: &str) -> bool {
    if project_root.as_os_str().is_empty() {
        return true;
    }
    is_project_path(cwd, project_root, worktree_prefix)
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
///
/// D-097: when `project_root` is empty (relaxed mode) we return None
/// so the caller can fall back to launch-kind based labeling. The old
/// code returned `Some("master")` for every absolute cwd in that
/// branch (because `cwd.starts_with("/")` always holds), making the
/// SESSIONS widget identical 5×"master" regardless of what each
/// claude process actually was.
fn infer_role_from_path(cwd: &str, project_root: &Path, worktree_prefix: &str) -> Option<String> {
    if project_root.as_os_str().is_empty() {
        return None;
    }
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
        // npm-global install
        assert!(is_claude_process(
            "/home/bravo/.npm-global/bin/claude -p --input-format stream-json --mcp-config {...}"
        ));
        // vscode native-binary install — D-097: previously the only thing
        // the old function caught, falsely positioning vscode-spawned
        // claudes as the only sessions worth showing.
        assert!(is_claude_process(
            "/home/bravo/.vscode-server-insiders/extensions/anthropic.claude-code-2.1.141-linux-x64/resources/native-binary/claude --output-format stream-json --verbose"
        ));
        // bare invocation
        assert!(is_claude_process(
            "claude --permission-mode bypassPermissions"
        ));
        // claude-code symlink name
        assert!(is_claude_process(
            "/home/user/.local/bin/claude-code --session xyz"
        ));
        // NOT claude — node wrapper, vim editing a .md file, npm
        assert!(!is_claude_process("node /usr/bin/npm install"));
        assert!(!is_claude_process("vim claude.md"));
        assert!(!is_claude_process("vim /home/.../claude/notes.md"));
        // path containing "claude" in a subdir but argv[0] isn't claude
        assert!(!is_claude_process("/home/me/.claude/scripts/helper.sh"));
    }

    #[test]
    fn classify_launch_kind_works() {
        assert_eq!(
            classify_launch_kind(
                "/home/bravo/.npm-global/bin/claude -p --input-format stream-json --mcp-config {...} --append-system-prompt foo"
            ),
            "daemon"
        );
        assert_eq!(
            classify_launch_kind(
                "/home/bravo/.vscode.../claude --output-format stream-json --resume b3f0ae"
            ),
            "session"
        );
        assert_eq!(
            classify_launch_kind("claude --permission-mode bypassPermissions"),
            "interactive"
        );
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
    fn infer_role_returns_none_when_project_root_empty() {
        // D-097: relaxed mode (empty project_root) must return None
        // so callers fall back to launch-kind labeling. The old code
        // accidentally returned Some("master") for every absolute cwd
        // because `cwd.starts_with("/")` always holds.
        let root = Path::new("");
        assert_eq!(infer_role_from_path("/work/foo", root, ""), None);
        assert_eq!(infer_role_from_path("/home/bravo", root, ""), None);
        assert_eq!(infer_role_from_path("/", root, ""), None);
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
