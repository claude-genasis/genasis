//! JSONL session file parser.
//!
//! D-073: Updated to current Claude Code layout. The previous code looked
//! at `~/.claude/sessions/*.jsonl` (which today contains per-PID *JSON*
//! status files, not JSONL transcripts) and parsed flat `{"type":"usage",
//! "input_tokens":...}` events. The real format is:
//!
//!   ~/.claude/projects/<encoded-cwd>/<session-uuid>.jsonl
//!     ├── {"type":"assistant","message":{"model":"...","usage":{...}},
//!     │    "timestamp":"2026-05-14T14:29:13.052Z", ...}
//!     ├── {"type":"user", ...}
//!     └── {"type":"queue-operation", ...}
//!
//! So the scan needs to (a) recurse one level into per-project sub-dirs,
//! (b) extract tokens from `message.usage.*` of `type=assistant` events,
//! (c) parse ISO 8601 timestamps, and (d) read the model from
//! `message.model` (not from `usage.model`, which doesn't exist).

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
    pub five_h_status: String, // "allowed" | "warning" | "limited"
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
    pub plan: String, // "Max (20x)" etc
    pub tier: String,

    // Scan metadata
    pub scanned_at: u64,
    pub files_scanned: usize,
    /// D-082: tool_use events counted across the 5h window — feeds the
    /// Tokens widget's "MCP calls" counter (which is really every tool
    /// call the assistant made, MCP or built-in).
    pub mcp_calls_5h: u64,
    /// D-082: when an `assistant` event has cache_read_input_tokens > 0
    /// we count it as a cache hit. Ratio surfaces as `cache hit %`.
    pub mcp_cache_hits_5h: u64,
}

impl UsageSnapshot {
    /// Calculate 5h window usage percentage against a budget.
    pub fn five_h_pct(&self, budget: u64) -> f32 {
        if budget == 0 {
            return 0.0;
        }
        let used = self.five_h_input_tokens + self.five_h_output_tokens;
        (used as f64 / budget as f64 * 100.0) as f32
    }

    /// Calculate 7d all-model usage percentage.
    pub fn week_all_pct(&self, budget: u64) -> f32 {
        if budget == 0 {
            return 0.0;
        }
        let used = self.week_input_tokens + self.week_output_tokens;
        (used as f64 / budget as f64 * 100.0) as f32
    }

    /// Calculate 7d sonnet-only usage percentage.
    pub fn week_sonnet_pct(&self, budget: u64) -> f32 {
        if budget == 0 {
            return 0.0;
        }
        let used = self.week_sonnet_input + self.week_sonnet_output;
        (used as f64 / budget as f64 * 100.0) as f32
    }

    /// Context window usage percentage.
    pub fn ctx_pct(&self) -> f32 {
        if self.ctx_window_size == 0 {
            return 0.0;
        }
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
    let projects_dir = match dirs::home_dir() {
        Some(h) => h.join(".claude").join("projects"),
        None => return UsageSnapshot::default(),
    };
    scan_dir(&projects_dir)
}

/// Scan `~/.claude/projects/` (recurse one level) for `.jsonl` files.
/// D-073: real claude code layout is two levels deep —
/// `projects/<encoded-cwd>/<session-uuid>.jsonl`.
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

    let mut jsonl_files: Vec<PathBuf> = Vec::new();
    if let Ok(top) = fs::read_dir(dir) {
        for entry in top.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(inner) = fs::read_dir(&path) {
                    for e in inner.flatten() {
                        let p = e.path();
                        if p.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                            jsonl_files.push(p);
                        }
                    }
                }
            } else if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                // Tolerate flat layout too (legacy).
                jsonl_files.push(path);
            }
        }
    }

    // Newest first so the "latest active session" context-window snapshot
    // is the most recent one.
    jsonl_files.sort_by_cached_key(|p| {
        fs::metadata(p)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| std::cmp::Reverse(d.as_secs()))
            .unwrap_or(std::cmp::Reverse(0))
    });

    for path in &jsonl_files {
        snapshot.files_scanned += 1;
        scan_single_file(path, &mut snapshot, five_h_start, week_start);
    }

    snapshot
}

/// Parse an ISO 8601 timestamp like `2026-05-14T14:29:13.052Z` into an
/// epoch seconds u64. Returns 0 on unparseable input — caller treats that
/// as "outside any window" so unparseable events don't pollute counters.
fn parse_iso8601_to_epoch(s: &str) -> u64 {
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.timestamp() as u64)
        .unwrap_or(0)
}

/// Parse a single JSONL file and accumulate into the snapshot.
fn scan_single_file(path: &Path, snapshot: &mut UsageSnapshot, five_h_start: u64, week_start: u64) {
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
        if line.is_empty() {
            continue;
        }

        let event: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let event_type = event.get("type").and_then(|t| t.as_str()).unwrap_or("");
        // D-073: claude code 의 JSONL timestamp 는 ISO 8601 string 이지
        // u64 epoch 가 아님. 기존 코드 시도하던 u64 fallback 은 거의 항상
        // 0 으로 떨어져서 모든 event 가 5h/7d window 의 from 보다 작아 0
        // 토큰으로 집계됐다.
        let timestamp = event
            .get("timestamp")
            .and_then(|t| t.as_str())
            .map(parse_iso8601_to_epoch)
            .or_else(|| event.get("timestamp").and_then(|t| t.as_u64()))
            .or_else(|| {
                event
                    .get("timestampMs")
                    .and_then(|t| t.as_u64())
                    .map(|ms| ms / 1000)
            })
            .unwrap_or(0);

        match event_type {
            // D-073: 진짜 claude code JSONL 의 token 정보는 `assistant`
            // event 의 `message.usage` 안에 있다. 모델은 `message.model`.
            // 옛 `usage`/`api_response` 형식도 fallback 으로 유지.
            "assistant" | "usage" | "api_response" => {
                let (usage_obj, model) = if event_type == "assistant" {
                    let msg = event.get("message");
                    let u = msg.and_then(|m| m.get("usage"));
                    let m = msg
                        .and_then(|m| m.get("model"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    (u.unwrap_or(&event), m)
                } else {
                    let u = event.get("usage").unwrap_or(&event);
                    let m = u
                        .get("model")
                        .and_then(|v| v.as_str())
                        .or_else(|| event.get("model").and_then(|v| v.as_str()))
                        .unwrap_or("");
                    (u, m)
                };
                let input = usage_obj
                    .get("input_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let output = usage_obj
                    .get("output_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let cache_read = usage_obj
                    .get("cache_read_input_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let cache_create = usage_obj
                    .get("cache_creation_input_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let is_sonnet = model.contains("sonnet");

                if input + output + cache_read + cache_create == 0 {
                    // assistant 메시지인데 usage 가 모두 0 — drop.
                    continue;
                }

                // 5h window
                if timestamp >= five_h_start {
                    snapshot.five_h_input_tokens += input;
                    snapshot.five_h_output_tokens += output;
                    snapshot.five_h_cache_read += cache_read;
                    snapshot.five_h_cache_create += cache_create;
                    // D-082: 매 assistant turn 을 "MCP call" 카운트로
                    // 간주 — 진짜 MCP 호출만이 아니라 Bash / Edit / Read
                    // 같은 built-in tool 도 포함하지만 사용자가 보기
                    // 원하는 신호 ("팀이 얼마나 일했나") 와 부합.
                    snapshot.mcp_calls_5h += 1;
                    if cache_read > 0 {
                        snapshot.mcp_cache_hits_5h += 1;
                    }
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

                // Context window (keep latest seen — files are scanned
                // newest-first so the first assistant event wins).
                if snapshot.ctx_input + snapshot.ctx_output == 0 {
                    snapshot.ctx_input = input;
                    snapshot.ctx_output = output;
                    snapshot.ctx_cache_read = cache_read;
                    snapshot.ctx_cache_create = cache_create;
                    if !model.is_empty() {
                        snapshot.ctx_model = model.to_string();
                    }
                }
            }
            "rate_limit_event" => {
                let status = event
                    .get("status")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string();
                let reset = event
                    .get("resetsAt")
                    .and_then(|r| r.as_u64())
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
                let size = event
                    .get("windowSize")
                    .and_then(|s| s.as_u64())
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

    let plan = creds
        .get("plan")
        .or_else(|| creds.get("planName"))
        .and_then(|p| p.as_str())
        .unwrap_or("unknown")
        .to_string();
    let tier = creds
        .get("tier")
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
