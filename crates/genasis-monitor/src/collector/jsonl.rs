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
//!
//! D-130: Filled in the dead fields that made the CLAUDE USAGE widget look
//! broken for everyone — cost (`five_h_cost_usd` / `week_cost_usd`) was
//! declared but never written; reset epochs only came from `rate_limit_event`
//! JSONL events that current Claude Code (v2.1.x) doesn't emit; the Sonnet
//! line was permanently 0 % for Opus-heavy users; `read_credentials` looked
//! at top-level keys but the real plan info lives under
//! `claudeAiOauth.subscriptionType` / `.rateLimitTier`.
//!
//! Token costs are computed from a model→pricing table (per-million-token
//! rates published by Anthropic). Reset countdowns are derived from the
//! *oldest* assistant event still inside each sliding window
//! (`oldest_ts + window_len`). Plan reading falls back to legacy top-level
//! keys so older credential files keep working.

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Per-million-token pricing for a single model. All fields in USD per
/// 1 M tokens. `cache_read` is normally `input * 0.1` and `cache_create`
/// is `input * 1.25` per Anthropic's prompt-caching documentation, but
/// the table keeps them explicit so future rate changes are a single-line
/// edit.
#[derive(Debug, Clone, Copy)]
struct ModelPricing {
    input: f64,
    output: f64,
    cache_read: f64,
    cache_create: f64,
}

/// D-130: pricing table covering the current Claude 4.x lineup. Unknown
/// models fall through to `default_pricing()` (mid-tier estimate) so cost
/// is non-zero but slightly inaccurate — preferable to silently dropping
/// usage from cost totals.
fn pricing_for(model: &str) -> ModelPricing {
    let m = model.to_ascii_lowercase();
    if m.contains("opus") {
        ModelPricing {
            input: 15.0,
            output: 75.0,
            cache_read: 1.5,
            cache_create: 18.75,
        }
    } else if m.contains("haiku") {
        ModelPricing {
            input: 1.0,
            output: 5.0,
            cache_read: 0.1,
            cache_create: 1.25,
        }
    } else if m.contains("sonnet") {
        ModelPricing {
            input: 3.0,
            output: 15.0,
            cache_read: 0.3,
            cache_create: 3.75,
        }
    } else {
        // Default to Sonnet pricing — middle of the lineup, safer than
        // assuming Haiku (which would under-count).
        ModelPricing {
            input: 3.0,
            output: 15.0,
            cache_read: 0.3,
            cache_create: 3.75,
        }
    }
}

/// Coarse model family classifier — used to bucket weekly token totals
/// into Sonnet / Opus / Haiku / Other rows in the widget. Lowercase
/// substring match matches the long model IDs Anthropic returns
/// (`claude-opus-4-7`, `claude-sonnet-4-6`, …).
fn classify_model(model: &str) -> ModelFamily {
    let m = model.to_ascii_lowercase();
    if m.contains("opus") {
        ModelFamily::Opus
    } else if m.contains("sonnet") {
        ModelFamily::Sonnet
    } else if m.contains("haiku") {
        ModelFamily::Haiku
    } else {
        ModelFamily::Other
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelFamily {
    Opus,
    Sonnet,
    Haiku,
    Other,
}

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
    /// D-130: oldest assistant-event timestamp still inside the 5h window.
    /// Drives `five_h_reset_countdown()` since Claude Code no longer
    /// emits `rate_limit_event` records. 0 = no events in window.
    pub five_h_oldest_event_ts: u64,

    // 7-day window (all models)
    pub week_input_tokens: u64,
    pub week_output_tokens: u64,
    pub week_cache_read: u64,
    pub week_cache_create: u64,
    pub week_cost_usd: f64,
    /// D-130: oldest assistant-event timestamp still inside the 7d window.
    pub week_oldest_event_ts: u64,

    // 7-day window (sonnet only) — kept for back-compat callers, also
    // mirrored into per-family counters below.
    pub week_sonnet_input: u64,
    pub week_sonnet_output: u64,
    pub week_sonnet_cache_read: u64,
    pub week_sonnet_cache_create: u64,

    /// D-130: 7-day Opus track — Max plan accounts these separately from
    /// Sonnet, and Opus-heavy users were watching a permanently-0 % bar.
    pub week_opus_input: u64,
    pub week_opus_output: u64,
    pub week_opus_cache_read: u64,
    pub week_opus_cache_create: u64,

    /// D-130: 7-day Haiku + "other" buckets — surfaced as the residual
    /// when the user runs a model we don't classify.
    pub week_haiku_input: u64,
    pub week_haiku_output: u64,
    pub week_other_input: u64,
    pub week_other_output: u64,

    // Rate limit status (legacy — rate_limit_event JSONL events from
    // pre-2.1 Claude Code; kept so old transcripts still drive the widget).
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
    /// D-130: counted assistant events inside the 5h window. Used by the
    /// widget to detect "no activity yet" and surface a hint instead of
    /// rendering empty gauges.
    pub five_h_event_count: u64,

    // ────────────────────────────────────────────────────────────────
    // D-131: server-reported utilization from `/api/oauth/usage`.
    // When `Some`, these override the JSONL-derived percentages because
    // the server already applies plan-specific weighting (cache rates,
    // per-window carve-outs, plan limits) that we cannot reproduce
    // locally. JSONL totals stay populated so cost / token-count widgets
    // keep working — only the % gauges defer to the server values.
    // ────────────────────────────────────────────────────────────────
    pub oauth_five_h_pct: Option<f32>,
    pub oauth_seven_day_pct: Option<f32>,
    pub oauth_seven_day_opus_pct: Option<f32>,
    pub oauth_seven_day_sonnet_pct: Option<f32>,
    /// Anthropic's "Claude Design" weekly bucket (server name
    /// `seven_day_omelette`). Surfaced as a fifth gauge when non-null.
    pub oauth_seven_day_design_pct: Option<f32>,
    pub oauth_five_h_resets_at: Option<u64>,
    pub oauth_seven_day_resets_at: Option<u64>,
    /// Extra-usage / overage credits — monthly cap and current usage
    /// in USD cents (divide by 100 to display dollars). Mirrors the
    /// "사용 크레딧 / Usage Credits" section of the Anthropic settings
    /// page so the cost line on the widget shows the same number the
    /// user sees there.
    pub oauth_extra_used_credits_cents: Option<f64>,
    pub oauth_extra_monthly_limit_cents: Option<f64>,
    pub oauth_extra_pct: Option<f32>,
    /// Epoch seconds when the last successful OAuth fetch landed; 0
    /// when we've never had one. Used by the widget to fall back to
    /// JSONL gauges if the server data is stale.
    pub oauth_fetched_at: u64,
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

    /// D-130: 5h % with cache tokens folded in via Anthropic's published
    /// prompt-caching weights (cache_read counts at 0.1×, cache_create
    /// at 1.25× of input). Closer to what the server-side rate limiter
    /// actually sees.
    pub fn five_h_pct_weighted(&self, budget: u64) -> f32 {
        if budget == 0 {
            return 0.0;
        }
        let used = self.five_h_input_tokens as f64
            + self.five_h_output_tokens as f64
            + self.five_h_cache_create as f64 * 1.25
            + self.five_h_cache_read as f64 * 0.1;
        (used / budget as f64 * 100.0) as f32
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

    /// D-130: 7d Opus-only usage percentage. Max-plan users see this
    /// tracked separately from Sonnet on the server side.
    pub fn week_opus_pct(&self, budget: u64) -> f32 {
        if budget == 0 {
            return 0.0;
        }
        let used = self.week_opus_input + self.week_opus_output;
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
    ///
    /// D-130: prefer the legacy `rate_limit_event` epoch when present
    /// (older Claude Code versions still write them); otherwise derive
    /// from the oldest assistant event still inside the 5h window —
    /// `oldest_ts + 5h` is when that earliest call slides out of the
    /// window, freeing up that share of the budget. Returns 0 when there
    /// is no activity to anchor a window on.
    pub fn five_h_reset_countdown(&self) -> i64 {
        let now = now_epoch();
        if self.five_h_reset_epoch != 0 {
            return self.five_h_reset_epoch as i64 - now as i64;
        }
        if self.five_h_oldest_event_ts == 0 {
            return 0;
        }
        let reset = self.five_h_oldest_event_ts + 5 * 3600;
        reset as i64 - now as i64
    }

    /// Seconds remaining until weekly reset. Same fallback logic as 5h
    /// — derived from the oldest event in the 7-day window when no
    /// explicit `rate_limit_event` is recorded.
    pub fn week_reset_countdown(&self) -> i64 {
        let now = now_epoch();
        if self.week_reset_epoch != 0 {
            return self.week_reset_epoch as i64 - now as i64;
        }
        if self.week_oldest_event_ts == 0 {
            return 0;
        }
        let reset = self.week_oldest_event_ts + 7 * 24 * 3600;
        reset as i64 - now as i64
    }

    /// D-130: returns true when no assistant activity was seen in the 5h
    /// window. The widget renders an empty-state hint instead of three
    /// 0 % gauges that look broken.
    pub fn is_empty_5h(&self) -> bool {
        self.five_h_event_count == 0
    }

    /// D-131: copy the values from `/api/oauth/usage` onto the snapshot.
    /// Called by the run-loop after each successful fetch. The widget
    /// prefers these over the JSONL-derived percentages when present.
    pub fn apply_oauth_usage(&mut self, src: &crate::collector::oauth_usage::OAuthUsage) {
        use crate::collector::oauth_usage::parse_resets_at;
        self.oauth_five_h_pct = src
            .five_hour
            .as_ref()
            .and_then(|w| w.utilization)
            .map(|v| v as f32);
        self.oauth_seven_day_pct = src
            .seven_day
            .as_ref()
            .and_then(|w| w.utilization)
            .map(|v| v as f32);
        self.oauth_seven_day_opus_pct = src
            .seven_day_opus
            .as_ref()
            .and_then(|w| w.utilization)
            .map(|v| v as f32);
        self.oauth_seven_day_sonnet_pct = src
            .seven_day_sonnet
            .as_ref()
            .and_then(|w| w.utilization)
            .map(|v| v as f32);
        self.oauth_seven_day_design_pct = src
            .seven_day_omelette
            .as_ref()
            .and_then(|w| w.utilization)
            .map(|v| v as f32);
        self.oauth_five_h_resets_at = src
            .five_hour
            .as_ref()
            .and_then(|w| w.resets_at.as_deref())
            .map(parse_resets_at)
            .filter(|t| *t > 0);
        self.oauth_seven_day_resets_at = src
            .seven_day
            .as_ref()
            .and_then(|w| w.resets_at.as_deref())
            .map(parse_resets_at)
            .filter(|t| *t > 0);
        if let Some(extra) = &src.extra_usage {
            self.oauth_extra_used_credits_cents = extra.used_credits;
            self.oauth_extra_monthly_limit_cents = extra.monthly_limit;
            self.oauth_extra_pct = extra.utilization.map(|v| v as f32);
        }
        self.oauth_fetched_at = now_epoch();
    }

    /// D-131: seconds remaining until the server-reported 5 h window
    /// resets — preferred over the JSONL sliding-window estimate when
    /// available because the server's `resets_at` is authoritative.
    pub fn five_h_oauth_countdown(&self) -> Option<i64> {
        let ts = self.oauth_five_h_resets_at?;
        let now = now_epoch();
        Some(ts as i64 - now as i64)
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
                let family = classify_model(model);

                if input + output + cache_read + cache_create == 0 {
                    // assistant 메시지인데 usage 가 모두 0 — drop.
                    continue;
                }

                // D-130: cost = (tokens / 1M) × per-million rate.
                let pricing = pricing_for(model);
                let cost = (input as f64 * pricing.input
                    + output as f64 * pricing.output
                    + cache_read as f64 * pricing.cache_read
                    + cache_create as f64 * pricing.cache_create)
                    / 1_000_000.0;

                // 5h window
                if timestamp >= five_h_start {
                    snapshot.five_h_input_tokens += input;
                    snapshot.five_h_output_tokens += output;
                    snapshot.five_h_cache_read += cache_read;
                    snapshot.five_h_cache_create += cache_create;
                    snapshot.five_h_cost_usd += cost;
                    snapshot.five_h_event_count += 1;
                    // D-130: track oldest event in window for sliding-window
                    // reset estimate. Skip timestamp==0 (unparseable).
                    if timestamp > 0
                        && (snapshot.five_h_oldest_event_ts == 0
                            || timestamp < snapshot.five_h_oldest_event_ts)
                    {
                        snapshot.five_h_oldest_event_ts = timestamp;
                    }
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
                    snapshot.week_cost_usd += cost;
                    if timestamp > 0
                        && (snapshot.week_oldest_event_ts == 0
                            || timestamp < snapshot.week_oldest_event_ts)
                    {
                        snapshot.week_oldest_event_ts = timestamp;
                    }

                    match family {
                        ModelFamily::Sonnet => {
                            snapshot.week_sonnet_input += input;
                            snapshot.week_sonnet_output += output;
                            snapshot.week_sonnet_cache_read += cache_read;
                            snapshot.week_sonnet_cache_create += cache_create;
                        }
                        ModelFamily::Opus => {
                            snapshot.week_opus_input += input;
                            snapshot.week_opus_output += output;
                            snapshot.week_opus_cache_read += cache_read;
                            snapshot.week_opus_cache_create += cache_create;
                        }
                        ModelFamily::Haiku => {
                            snapshot.week_haiku_input += input;
                            snapshot.week_haiku_output += output;
                        }
                        ModelFamily::Other => {
                            snapshot.week_other_input += input;
                            snapshot.week_other_output += output;
                        }
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

/// Anthropic rate-limit tier → (5h tokens, weekly all-model tokens,
/// weekly opus tokens, weekly sonnet tokens). Values reflect Anthropic's
/// published Claude Code limits for Pro / Max / Team plans (D-130). Tier
/// strings come from `~/.claude/.credentials.json`'s
/// `claudeAiOauth.rateLimitTier` field.
///
/// Returns `None` for unknown tiers so the caller can fall back to env
/// defaults rather than silently using wrong numbers.
pub fn tier_to_limits(tier: &str) -> Option<(u64, u64, u64, u64)> {
    // Source: Anthropic's "Claude Code usage limits" page, approximations.
    // Conservative — when in doubt, undershoot so the widget warns earlier.
    match tier {
        // Pro plan
        "default_claude_pro" | "pro" => Some((220_000, 5_000_000, 2_000_000, 5_000_000)),
        // Max 5x
        "default_claude_max_5x" | "max_5x" => Some((1_100_000, 25_000_000, 10_000_000, 25_000_000)),
        // Max 20x
        "default_claude_max_20x" | "max_20x" => {
            Some((4_400_000, 100_000_000, 40_000_000, 100_000_000))
        }
        // Team
        "default_claude_team" | "team" => Some((220_000, 10_000_000, 4_000_000, 10_000_000)),
        // Enterprise — assume Max 20x equivalent until we have a real number
        "default_claude_enterprise" | "enterprise" => {
            Some((4_400_000, 100_000_000, 40_000_000, 100_000_000))
        }
        _ => None,
    }
}

/// Map an internal `rateLimitTier` string to the user-friendly label that
/// Anthropic uses ("Max (20x)", "Pro", etc).
pub fn plan_display_name(subscription: &str, tier: &str) -> String {
    // Prefer tier (more specific than subscription).
    match tier {
        "default_claude_pro" | "pro" => return "Pro".to_string(),
        "default_claude_max_5x" | "max_5x" => return "Max (5x)".to_string(),
        "default_claude_max_20x" | "max_20x" => return "Max (20x)".to_string(),
        "default_claude_team" | "team" => return "Team".to_string(),
        "default_claude_enterprise" | "enterprise" => return "Enterprise".to_string(),
        _ => {}
    }
    match subscription {
        "max" => "Max".to_string(),
        "pro" => "Pro".to_string(),
        "team" => "Team".to_string(),
        "enterprise" => "Enterprise".to_string(),
        s if !s.is_empty() => s.to_string(),
        _ => "unknown".to_string(),
    }
}

/// Read Claude credentials for plan info.
///
/// D-130: the real structure is `{"claudeAiOauth": {"subscriptionType":
/// "max", "rateLimitTier": "default_claude_max_20x", ...}}`. The old
/// implementation looked at top-level `plan` / `planName` / `tier` /
/// `serviceTier` and always returned ("unknown", "-"). Top-level keys
/// are still tried as a fallback so legacy credential files keep working.
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

    let oauth = creds.get("claudeAiOauth");
    let subscription = oauth
        .and_then(|o| o.get("subscriptionType"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let tier = oauth
        .and_then(|o| o.get("rateLimitTier"))
        .and_then(|v| v.as_str())
        .or_else(|| {
            creds
                .get("tier")
                .or_else(|| creds.get("serviceTier"))
                .and_then(|t| t.as_str())
        })
        .unwrap_or("");

    if !subscription.is_empty() || !tier.is_empty() {
        return (plan_display_name(subscription, tier), tier.to_string());
    }

    // Legacy fallback (pre-OAuth credentials shape).
    let plan = creds
        .get("plan")
        .or_else(|| creds.get("planName"))
        .and_then(|p| p.as_str())
        .unwrap_or("unknown")
        .to_string();
    let tier_legacy = creds
        .get("tier")
        .or_else(|| creds.get("serviceTier"))
        .and_then(|t| t.as_str())
        .unwrap_or("-")
        .to_string();

    (plan, tier_legacy)
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
        assert!(snap.is_empty_5h(), "empty scan must report empty 5h");
        assert_eq!(snap.five_h_reset_countdown(), 0);
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
        assert_eq!(snap.week_opus_input, 2000); // D-130: Opus tracked separately
    }

    #[test]
    fn five_h_pct_calculates() {
        let mut snap = UsageSnapshot::default();
        snap.five_h_input_tokens = 1_000_000;
        snap.five_h_output_tokens = 500_000;
        let pct = snap.five_h_pct(7_000_000);
        assert!((pct - 21.4).abs() < 0.5);
    }

    /// D-130: cost was a dead field — verify it's populated and roughly
    /// matches the Sonnet rate (3 + 15 = 18 USD per MTok of mixed traffic).
    #[test]
    fn scan_populates_cost() {
        let d = tempdir().unwrap();
        let jsonl = d.path().join("session.jsonl");
        let now = now_epoch();
        let mut f = fs::File::create(&jsonl).unwrap();
        // 1 M input + 1 M output Sonnet = $3 + $15 = $18.
        writeln!(
            f,
            r#"{{"type":"assistant","timestamp":"{}","message":{{"model":"claude-sonnet-4-6","usage":{{"input_tokens":1000000,"output_tokens":1000000}}}}}}"#,
            chrono::DateTime::from_timestamp(now as i64 - 60, 0)
                .unwrap()
                .to_rfc3339(),
        )
        .unwrap();

        let snap = scan_dir(d.path());
        assert!(
            (snap.five_h_cost_usd - 18.0).abs() < 0.01,
            "expected ~$18, got ${}",
            snap.five_h_cost_usd
        );
        assert!((snap.week_cost_usd - 18.0).abs() < 0.01);
    }

    /// D-130: Opus rates are 5× Sonnet — verify pricing table picks the
    /// right row.
    #[test]
    fn opus_cost_higher_than_sonnet() {
        let d = tempdir().unwrap();
        let jsonl = d.path().join("session.jsonl");
        let now = now_epoch();
        let mut f = fs::File::create(&jsonl).unwrap();
        writeln!(
            f,
            r#"{{"type":"assistant","timestamp":"{}","message":{{"model":"claude-opus-4-7","usage":{{"input_tokens":1000000,"output_tokens":1000000}}}}}}"#,
            chrono::DateTime::from_timestamp(now as i64 - 60, 0)
                .unwrap()
                .to_rfc3339(),
        )
        .unwrap();

        let snap = scan_dir(d.path());
        // Opus: $15 + $75 = $90.
        assert!(
            (snap.five_h_cost_usd - 90.0).abs() < 0.5,
            "expected ~$90, got ${}",
            snap.five_h_cost_usd
        );
        assert_eq!(snap.week_opus_input, 1_000_000);
        assert_eq!(snap.week_sonnet_input, 0);
    }

    /// D-130: reset countdown should derive from the oldest in-window
    /// event when no `rate_limit_event` is present.
    #[test]
    fn reset_countdown_from_oldest_event() {
        let d = tempdir().unwrap();
        let jsonl = d.path().join("session.jsonl");
        let now = now_epoch();
        let mut f = fs::File::create(&jsonl).unwrap();
        // Event 2h ago (still in 5h window).
        let old = now as i64 - 2 * 3600;
        writeln!(
            f,
            r#"{{"type":"assistant","timestamp":"{}","message":{{"model":"claude-sonnet-4-6","usage":{{"input_tokens":100,"output_tokens":50}}}}}}"#,
            chrono::DateTime::from_timestamp(old, 0).unwrap().to_rfc3339(),
        )
        .unwrap();

        let snap = scan_dir(d.path());
        let countdown = snap.five_h_reset_countdown();
        // Reset = oldest + 5h = now - 2h + 5h = now + 3h.
        let expected = 3 * 3600;
        assert!(
            (countdown - expected).abs() < 120,
            "expected ~{expected}s countdown, got {countdown}"
        );
    }

    #[test]
    fn tier_lookup() {
        assert!(tier_to_limits("default_claude_max_20x").is_some());
        assert!(tier_to_limits("default_claude_pro").is_some());
        assert!(tier_to_limits("unknown_tier").is_none());
    }

    #[test]
    fn plan_label_max_20x() {
        assert_eq!(
            plan_display_name("max", "default_claude_max_20x"),
            "Max (20x)"
        );
        assert_eq!(plan_display_name("pro", "default_claude_pro"), "Pro");
        assert_eq!(plan_display_name("", ""), "unknown");
    }

    /// D-130: cache-weighted % > raw % when cache tokens dominate.
    #[test]
    fn weighted_pct_includes_cache() {
        let mut snap = UsageSnapshot::default();
        snap.five_h_input_tokens = 100;
        snap.five_h_output_tokens = 100;
        snap.five_h_cache_read = 1_000_000;
        let raw = snap.five_h_pct(10_000_000);
        let weighted = snap.five_h_pct_weighted(10_000_000);
        assert!(
            weighted > raw,
            "weighted ({weighted}) should exceed raw ({raw}) when cache_read is large"
        );
    }
}
