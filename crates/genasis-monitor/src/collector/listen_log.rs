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
        new_lines.push(reformat_with_local_time(&stripped));
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

/// D-081: turn the daemon's verbose ISO-8601 timestamp prefix (e.g.
/// `2026-05-15T01:41:28.718972Z  INFO …`) into a compact 24h `HH:MM`
/// prefix in the operator's local timezone, so the Log widget reads
/// like `01:41  INFO …`. Lines without an ISO prefix are kept verbatim.
fn reformat_with_local_time(line: &str) -> String {
    let trimmed = line.trim_start();
    // Look for the typical `YYYY-MM-DDTHH:MM:SS.fffZ` prefix.
    if trimmed.len() < 20 || trimmed.as_bytes().get(10) != Some(&b'T') {
        return line.to_string();
    }
    // Find the end of the timestamp (Z, +HH:MM, or whitespace).
    let mut end = 19; // YYYY-MM-DDTHH:MM:SS
    let bytes = trimmed.as_bytes();
    if bytes.get(end) == Some(&b'.') {
        end += 1;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
    }
    if bytes.get(end) == Some(&b'Z') {
        end += 1;
    } else if bytes.get(end) == Some(&b'+') || bytes.get(end) == Some(&b'-') {
        end += 6; // ±HH:MM
    }
    let ts_str = &trimmed[..end.min(trimmed.len())];
    let rest = trimmed[end.min(trimmed.len())..].trim_start();
    let local_hm = chrono::DateTime::parse_from_rfc3339(ts_str)
        .ok()
        .map(|dt| dt.with_timezone(&chrono::Local).format("%H:%M").to_string());
    match local_hm {
        Some(hm) => format!("{hm}  {rest}"),
        None => line.to_string(),
    }
}

/// Strip ANSI escape sequences (the daemon writes `tracing` colour
/// codes that would otherwise render as garbage in the TUI).
///
/// D-103: iterate by `char`, not by `byte`. The previous implementation
/// did `out.push(bytes[i] as char)` which casts one UTF-8 byte to a
/// Latin-1 codepoint, splitting every Korean glyph (3 bytes) into
/// three nonsense Latin-1 chars. End result was widget rows like
/// "ëŠ 01분ï[pm]" instead of "08:01  [pm]" — exactly the mojibake the
/// user reported in their monitor screenshot. Iterating by `char` is
/// both UTF-8 safe and lets the alphabetic-check below work on the
/// real terminator character rather than a single byte.
fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1B' && chars.peek() == Some(&'[') {
            chars.next(); // consume '['
                          // Consume parameter bytes until we hit the terminator
                          // (any alphabetic char). Non-alphabetic Korean chars are
                          // safe to pass through here because ANSI param bytes are
                          // always ASCII 0x30..=0x3F per ECMA-48.
            while let Some(&p) = chars.peek() {
                if p.is_alphabetic() {
                    chars.next(); // consume terminator
                    break;
                }
                chars.next();
            }
            continue;
        }
        out.push(c);
    }
    out
}
