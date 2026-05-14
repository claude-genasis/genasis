//! D-065 (partial): tail `.genasis/listen.log` so the operator sees what
//! the daemon is doing in the monitor's Log + Agents widgets.
//!
//! The full hook system (SessionStart hook + RTK savings + per-agent
//! activity stream) is deferred to D-066. For now this collector simply
//! reads the last N lines of the daemon's log file and pushes them to
//! `state.log_tail`, so users running `genasis monitor --project <dir>`
//! against a live daemon see real activity instead of a static
//! "no log lines yet".

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::state::AppState;

/// How many trailing lines of listen.log to surface.
const TAIL_LINES: usize = 80;

/// Position cursor we remember between ticks so the same line isn't
/// re-emitted forever. Stored on `AppState` so we can resume across
/// renders without reopening from offset 0 every time.
pub fn poll(state: &mut AppState, project_root: &Path) {
    let log = project_root.join(".genasis").join("listen.log");
    if !log.is_file() {
        return;
    }
    let metadata = match std::fs::metadata(&log) {
        Ok(m) => m,
        Err(_) => return,
    };
    let size = metadata.len();
    // log rotation / truncation safety — if file shrank, restart from 0.
    if size < state.listen_log_offset {
        state.listen_log_offset = 0;
    }
    if size == state.listen_log_offset {
        return; // no new content
    }

    // First read of the file? Seek so we only pick up the last TAIL_LINES
    // worth of content (otherwise a long-running daemon would dump
    // thousands of lines into the widget on first tick).
    if state.listen_log_offset == 0 && size > 16_000 {
        // Roughly the last 16KB usually covers TAIL_LINES of activity.
        state.listen_log_offset = size.saturating_sub(16_000);
    }

    let f = match File::open(&log) {
        Ok(f) => f,
        Err(_) => return,
    };
    let mut reader = BufReader::new(f);
    use std::io::Seek;
    if reader
        .seek(std::io::SeekFrom::Start(state.listen_log_offset))
        .is_err()
    {
        return;
    }

    let mut new_lines: Vec<String> = Vec::new();
    for line in reader.lines().map_while(Result::ok) {
        let stripped = strip_ansi(&line);
        if stripped.trim().is_empty() {
            continue;
        }
        new_lines.push(stripped);
    }
    state.listen_log_offset = size;

    // Push to log_tail; trim to last TAIL_LINES so the widget area
    // stays bounded.
    state.log_tail.extend(new_lines);
    if state.log_tail.len() > TAIL_LINES {
        let overflow = state.log_tail.len() - TAIL_LINES;
        state.log_tail.drain(..overflow);
    }
}

/// Strip ANSI escape sequences (the daemon writes `tracing` colour
/// codes that would otherwise render as garbage in the TUI).
fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 0x1B && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            i += 2;
            while i < bytes.len() && !(bytes[i] as char).is_alphabetic() {
                i += 1;
            }
            if i < bytes.len() {
                i += 1;
            }
            continue;
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}
