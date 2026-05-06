//! JSONL session file parser.
//!
//! Scans `~/.claude/sessions/` for `.jsonl` files and extracts:
//! - Token usage per 5-hour window and 7-day window
//! - Rate limit events (status, reset time)
//! - Context window usage for active sessions
//! - Cost tracking (USD)
//!
//! Reference: Python `agent_monitor.py` MonitorCollector._scan_jsonl_files()

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Token usage snapshot (aggregated from JSONL events).
#[derive(Debug, Clone, Default)]
pub struct UsageSnapshot {
    // 5-hour window
    pub five_h_input_tokens: u64,
    pub five_h_output_tokens: u64,
    pub five_h_cache_read: u64,
    pub five_h_cache_create: u64,
    pub five_h_cost_usd: f64,
    pub five_h_window_start: u64,

    // 7-day window (all models)
    pub week_input_tokens: u64,
    pub week_output_tokens: u64,
    pub week_cache_read: u64,
    pub week_cache_create: u64,

    // 7-day window (sonnet only)
    pub week_sonnet_input: u64,
    pub week_sonnet_output: u64,
    pub week_sonnet_cache_read: u64,
    pub week_sonnet_cache_create: u64,

    // Rate limit status
    pub five_h_status: String,   // "allowed" | "warning" | "limited"
    pub five_h_reset_epoch: u64,
    pub week_status: String,
    pub week_reset_epoch: u64,

    // Context window (latest active session)
    pub ctx_input: u64,
    pub ctx_output: u64,
    pub ctx_cache_read: u64,
    pub ctx_cache_create: u64,
    pub ctx_window_size: u64,
    pub ctx_model: String,

    // Plan info
    pub plan: String,      // "Max (20x)" etc
    pub tier: String,

    // Scan metadata
    pub scanned_at: u64,
    pub files_scanned: usize,
}

impl UsageSnapshot {
    /// Calculate 5h window usage percentage against a budget.
    pub fn five_h_pct(&self, budget: u64) -> f32 {
        if budget == 0 { return 0.0; }
        let used = self.five_h_input_tokens + self.five_h_output_tokens;
        (used as f64 / budget as f64 * 100.0) as f32
    }

    /// Calculate 7d all-model usage percentage.
    pub fn week_all_pct(&self, budget: u64) -> f32 {
        if budget == 0 { return 0.0; }
        let used = self.week_input_tokens + self.week_output_tokens;
        (used as f64 / budget as f64 * 100.0) as f32
    }

    /// Calculate 7d sonnet-only usage percentage.
    pub fn week_sonnet_pct(&self, budget: u64) -> f32 {
        if budget == 0 { return 0.0; }
        let used = self.week_sonnet_input + self.week_sonnet_output;
        (used as f64 / budget as f64 * 100.0) as f32
    }

    /// Context window usage percentage.
    pub fn ctx_pct(&self) -> f32 {
        if self.ctx_window_size == 0 { return 0.0; }
        let used = self.ctx_input + self.ctx_output + self.ctx_cache_read;
        (used as f64 / self.ctx_window_size as f64 * 100.0) as f32
    }

    /// Seconds remaining until 5h rate limit reset.
    pub fn five_h_reset_countdown(&self) -> i64 {
        let now = now_epoch();
        self.five_h_reset_epoch as i64 - now as i64
    }

    /// Seconds remaining until weekly reset.
    pub fn week_reset_countdown(&self) -> i64 {
        let now = now_epoch();
        self.week_reset_epoch as i64 - now as i64
    }
}

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Scan JSONL files in `~/.claude/sessions/` and aggregate usage data.
///
/// This is the expensive operation (~50-300ms depending on file count).
/// Caller should cache results and re-scan at `JSONL_SCAN_TTL` intervals.
pub fn scan_sessions_dir() -> UsageSnapshot {
    let sessions_dir = match dirs::home_dir() {
        Some(h) => h.join(".claude").join("sessions"),
        None => return UsageSnapshot::default(),
    };
    scan_dir(&sessions_dir)
}

/// Scan a specific directory for JSONL files (testable).
pub fn scan_dir(dir: &Path) -> UsageSnapshot {
    let mut snapshot = UsageSnapshot::default();
    snapshot.scanned_at = now_epoch();

    if !dir.is_dir() {
        return snapshot;
    }

    let now = now_epoch();
    let five_h_start = now.saturating_sub(5 * 3600);
    let week_start = now.saturating_sub(7 * 24 * 3600);
    snapshot.five_h_window_start = five_h_start;

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return snapshot,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("jsonl") {
            continue;
        }
        snapshot.files_scanned += 1;
        scan_single_file(&path, &mut snapshot, five_h_start, week_start);
    }

    snapshot
}

/// Parse a single JSONL file and accumulate into the snapshot.
fn scan_single_file(
    path: &Path,
    snapshot: &mut UsageSnapshot,
    five_h_start: u64,
    week_start: u64,
) {
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return,
    };
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        if line.is_empty() { continue; }

        let event: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let event_type = event.get("type").and_then(|t| t.as_str()).unwrap_or("");
        let timestamp = event.get("timestamp")
            .and_then(|t| t.as_u64())
            .or_else(|| event.get("timestampMs").and_then(|t| t.as_u64()).map(|ms| ms / 1000))
            .unwrap_or(0);

        match event_type {
            "usage" | "api_response" => {
                let usage = event.get("usage").unwrap_or(&event);
                let input = usage.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                let output = usage.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                let cache_read = usage.get("cache_read_input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                let cache_create = usage.get("cache_creation_input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                let model = usage.get("model").and_then(|v| v.as_str()).unwrap_or("");
                let is_sonnet = model.contains("sonnet");

                // 5h window
                if timestamp >= five_h_start {
                    snapshot.five_h_input_tokens += input;
                    snapshot.five_h_output_tokens += output;
                    snapshot.five_h_cache_read += cache_read;
                    snapshot.five_h_cache_create += cache_create;
                }

                // 7d window
                if timestamp >= week_start {
                    snapshot.week_input_tokens += input;
                    snapshot.week_output_tokens += output;
                    snapshot.week_cache_read += cache_read;
                    snapshot.week_cache_create += cache_create;

                    if is_sonnet {
                        snapshot.week_sonnet_input += input;
                        snapshot.week_sonnet_output += output;
                        snapshot.week_sonnet_cache_read += cache_read;
                        snapshot.week_sonnet_cache_create += cache_create;
                    }
                }

                // Context window (keep latest)
                snapshot.ctx_input = input;
                snapshot.ctx_output = output;
                snapshot.ctx_cache_read = cache_read;
                snapshot.ctx_cache_create = cache_create;
                if !model.is_empty() {
                    snapshot.ctx_model = model.to_string();
                }
            }
            "rate_limit_event" => {
                let status = event.get("status").and_then(|s| s.as_str()).unwrap_or("").to_string();
                let reset = event.get("resetsAt").and_then(|r| r.as_u64())
                    .or_else(|| event.get("expires_at").and_then(|r| r.as_u64()))
                    .unwrap_or(0);
                let window = event.get("window").and_then(|w| w.as_str()).unwrap_or("");

                match window {
                    "5h" | "five_hour" => {
                        snapshot.five_h_status = status;
                        snapshot.five_h_reset_epoch = reset;
                    }
                    "7d" | "weekly" => {
                        snapshot.week_status = status;
                        snapshot.week_reset_epoch = reset;
                    }
                    _ => {
                        // Default to 5h if not specified
                        if snapshot.five_h_reset_epoch == 0 {
                            snapshot.five_h_status = status;
                            snapshot.five_h_reset_epoch = reset;
                        }
                    }
                }
            }
            "context_window" => {
                let size = event.get("windowSize").and_then(|s| s.as_u64())
                    .or_else(|| event.get("context_window").and_then(|s| s.as_u64()))
                    .unwrap_or(0);
                if size > 0 {
                    snapshot.ctx_window_size = size;
                }
            }
            _ => {}
        }
    }
}

/// Read Claude credentials for plan info.
pub fn read_credentials() -> (String, String) {
    let path = match dirs::home_dir() {
        Some(h) => h.join(".claude").join(".credentials.json"),
        None => return ("unknown".into(), "-".into()),
    };
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return ("unknown".into(), "-".into()),
    };
    let creds: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return ("unknown".into(), "-".into()),
    };

    let plan = creds.get("plan")
        .or_else(|| creds.get("planName"))
        .and_then(|p| p.as_str())
        .unwrap_or("unknown")
        .to_string();
    let tier = creds.get("tier")
        .or_else(|| creds.get("serviceTier"))
        .and_then(|t| t.as_str())
        .unwrap_or("-")
        .to_string();

    (plan, tier)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn scan_empty_dir_returns_defaults() {
        let d = tempdir().unwrap();
        let snap = scan_dir(d.path());
        assert_eq!(snap.files_scanned, 0);
        assert_eq!(snap.five_h_input_tokens, 0);
    }

    #[test]
    fn scan_with_usage_events() {
        let d = tempdir().unwrap();
        let jsonl = d.path().join("test-session.jsonl");
        let now = now_epoch();
        let mut f = fs::File::create(&jsonl).unwrap();

        // Write a usage event within the 5h window
        writeln!(f, r#"{{"type":"usage","timestamp":{},"usage":{{"input_tokens":1000,"output_tokens":500,"cache_read_input_tokens":200,"model":"claude-sonnet-4-6"}}}}"#, now - 100).unwrap();
        // Write one outside the 5h window but inside 7d
        writeln!(f, r#"{{"type":"usage","timestamp":{},"usage":{{"input_tokens":2000,"output_tokens":800,"model":"claude-opus-4-6"}}}}"#, now - 6 * 3600).unwrap();

        let snap = scan_dir(d.path());
        assert_eq!(snap.files_scanned, 1);
        assert_eq!(snap.five_h_input_tokens, 1000);
        assert_eq!(snap.five_h_output_tokens, 500);
        assert_eq!(snap.week_input_tokens, 3000); // 1000 + 2000
        assert_eq!(snap.week_sonnet_input, 1000); // only the sonnet event
    }

    #[test]
    fn five_h_pct_calculates() {
        let mut snap = UsageSnapshot::default();
        snap.five_h_input_tokens = 1_000_000;
        snap.five_h_output_tokens = 500_000;
        let pct = snap.five_h_pct(7_000_000);
        assert!((pct - 21.4).abs() < 0.5);
    }
}
