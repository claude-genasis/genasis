//! OAuth-backed usage fetcher.
//!
//! D-131: the JSONL aggregator in `jsonl.rs` is a heuristic — it sums up
//! local token totals and divides by a static (or tier-derived) budget.
//! That's *close* but never matches the percentages Anthropic actually
//! reports on `claude.ai/settings/usage`, because the server-side
//! limiter applies its own weighting (cache rates, multi-window
//! interaction, plan-specific carve-outs). The user's screenshot of
//! that settings page showed `9% / 12% / 2% / Claude Design 0%` while
//! the monitor still rendered `0% / 0% / 0% / 0%`.
//!
//! The Claude Code CLI itself hits `GET /api/oauth/usage` with the
//! user's OAuth bearer (`~/.claude/.credentials.json` →
//! `claudeAiOauth.accessToken`) and parses the same shape we capture
//! here. By reusing that endpoint we surface the **actual** numbers
//! the server reports — the JSONL aggregator stays in place as a
//! fallback for the moment the token is missing / expired / disabled.
//!
//! No new credential surface area: the OAuth token already exists for
//! Claude Code itself; we read it and forward it. We do NOT touch
//! `ANTHROPIC_API_KEY` (per the user's standing instruction in
//! `feedback_no_claude_api.md`) — this is a different mechanism: a
//! user-scoped OAuth bearer for stats, not an API key for inference.
//!
//! Disable knob: set `MONITOR_DISABLE_OAUTH_USAGE=1` to skip the call
//! entirely and stick to JSONL-only mode.

use std::fs;
use std::time::Duration;

use serde::Deserialize;

const USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";
const USER_AGENT: &str = concat!("genasis-monitor/", env!("CARGO_PKG_VERSION"));
/// Beta header the Claude CLI sends with every usage request. Without
/// it the endpoint occasionally returns a stripped response shape; with
/// it the response matches what `claude /usage` would have seen.
const BETA_HEADER: &str = "claude-code-20250219";

/// Per-window slot in `/api/oauth/usage`. `utilization` is a
/// 0.0–100.0 percentage of the user's actual plan limit on the server
/// side (already cache-weighted, already plan-aware), and `resets_at`
/// is an ISO 8601 timestamp.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct UsageWindow {
    #[serde(default)]
    pub utilization: Option<f64>,
    #[serde(default)]
    pub resets_at: Option<String>,
}

/// `/api/oauth/usage` extra-usage (overage credits) slot. The Anthropic
/// settings page shows this as "사용 크레딧" / "Usage Credits": when
/// the plan limit is hit, the user can opt into spending pre-purchased
/// credits up to `monthly_limit` (cents → USD) before being blocked.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ExtraUsage {
    #[serde(default)]
    pub is_enabled: bool,
    /// Monthly cap in **minor currency units** (USD cents) — `20000`
    /// means a $200 monthly cap. Display layer divides by 100.
    #[serde(default)]
    pub monthly_limit: Option<f64>,
    /// Credits already consumed this billing period (same minor units
    /// as `monthly_limit`).
    #[serde(default)]
    pub used_credits: Option<f64>,
    #[serde(default)]
    pub utilization: Option<f64>,
    #[serde(default)]
    pub currency: Option<String>,
}

/// Full `/api/oauth/usage` response. All windows are optional because
/// the server omits fields that don't apply to the user's plan
/// (e.g. `seven_day_opus` is `null` on plans that don't separate Opus
/// from the all-models bucket).
#[derive(Debug, Clone, Deserialize, Default)]
pub struct OAuthUsage {
    #[serde(default)]
    pub five_hour: Option<UsageWindow>,
    #[serde(default)]
    pub seven_day: Option<UsageWindow>,
    #[serde(default)]
    pub seven_day_opus: Option<UsageWindow>,
    #[serde(default)]
    pub seven_day_sonnet: Option<UsageWindow>,
    /// "Claude Design" track surfaced on the settings page as a
    /// separate weekly bucket — server name is `seven_day_omelette`.
    #[serde(default)]
    pub seven_day_omelette: Option<UsageWindow>,
    #[serde(default)]
    pub extra_usage: Option<ExtraUsage>,
}

/// Fetch the live usage snapshot. Returns `Ok(None)` when the feature
/// is disabled or the credential file is missing — both are normal,
/// non-error states (the caller falls back to the JSONL aggregator).
/// Returns `Err` only for genuine failure (network, 4xx/5xx with a
/// valid token).
pub async fn fetch() -> Result<Option<OAuthUsage>, String> {
    if std::env::var("MONITOR_DISABLE_OAUTH_USAGE").ok().as_deref() == Some("1") {
        return Ok(None);
    }

    let Some(token) = read_access_token() else {
        return Ok(None);
    };

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .user_agent(USER_AGENT)
        .build()
    {
        Ok(c) => c,
        Err(e) => return Err(format!("client build: {e}")),
    };

    let resp = match client
        .get(USAGE_URL)
        .bearer_auth(token)
        .header("Content-Type", "application/json")
        .header("anthropic-beta", BETA_HEADER)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return Err(format!("request: {e}")),
    };

    let status = resp.status();
    if !status.is_success() {
        // 401 means the cached token expired without being refreshed;
        // Claude Code itself will rotate it on the next interactive
        // run. We surface a soft Ok(None) for that case so the monitor
        // doesn't spam the log with auth errors every poll.
        if status.as_u16() == 401 {
            return Ok(None);
        }
        return Err(format!("HTTP {status}"));
    }

    let body: OAuthUsage = match resp.json().await {
        Ok(b) => b,
        Err(e) => return Err(format!("decode: {e}")),
    };
    Ok(Some(body))
}

/// Read `~/.claude/.credentials.json` and return the OAuth bearer
/// token, or `None` when the file / field is missing or the token has
/// already expired (Claude Code refreshes its own copy on the next
/// run — best to skip rather than send a known-stale bearer).
fn read_access_token() -> Option<String> {
    let path = dirs::home_dir()?.join(".claude").join(".credentials.json");
    let content = fs::read_to_string(&path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    let oauth = v.get("claudeAiOauth")?;
    let token = oauth.get("accessToken")?.as_str()?.to_string();
    // `expiresAt` is epoch milliseconds. Treat tokens within the next
    // 60 s of expiry as already expired — Claude Code refreshes 5 min
    // ahead, but a 60 s margin avoids serving a token that'll 401
    // before the response lands.
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_millis() as u64;
    if let Some(exp) = oauth.get("expiresAt").and_then(|x| x.as_u64()) {
        if exp <= now_ms.saturating_add(60_000) {
            return None;
        }
    }
    Some(token)
}

/// Parse `resets_at` ISO 8601 into epoch seconds; returns 0 on parse
/// failure so the widget treats it as "no reset known".
pub fn parse_resets_at(s: &str) -> u64 {
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.timestamp().max(0) as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Real captured response shape — same as what the dev host got
    /// back from `/api/oauth/usage` while debugging D-131. Locks the
    /// parser against silent schema drift.
    const FIXTURE: &str = r#"{
        "five_hour": {"utilization": 10.0, "resets_at": "2026-05-19T15:20:00.590453+00:00"},
        "seven_day": {"utilization": 12.0, "resets_at": "2026-05-22T22:00:00+00:00"},
        "seven_day_oauth_apps": null,
        "seven_day_opus": null,
        "seven_day_sonnet": {"utilization": 2.0, "resets_at": "2026-05-22T22:00:00+00:00"},
        "seven_day_cowork": null,
        "seven_day_omelette": {"utilization": 0.0, "resets_at": null},
        "tangelo": null,
        "extra_usage": {
            "is_enabled": true,
            "monthly_limit": 20000,
            "used_credits": 0.0,
            "utilization": null,
            "currency": "USD",
            "disabled_reason": null
        }
    }"#;

    #[test]
    fn parses_live_fixture() {
        let u: OAuthUsage = serde_json::from_str(FIXTURE).expect("decode");
        assert_eq!(u.five_hour.as_ref().unwrap().utilization, Some(10.0));
        assert_eq!(u.seven_day.as_ref().unwrap().utilization, Some(12.0));
        assert!(u.seven_day_opus.is_none()); // null in fixture → None
        assert_eq!(u.seven_day_sonnet.as_ref().unwrap().utilization, Some(2.0));
        let extra = u.extra_usage.unwrap();
        assert!(extra.is_enabled);
        assert_eq!(extra.monthly_limit, Some(20000.0));
        assert_eq!(extra.used_credits, Some(0.0));
    }

    #[test]
    fn parses_resets_at() {
        let ts = parse_resets_at("2026-05-19T15:20:00+00:00");
        assert!(ts > 1_700_000_000);
        assert_eq!(parse_resets_at("garbage"), 0);
    }

    #[test]
    fn empty_response_ok() {
        let u: OAuthUsage = serde_json::from_str("{}").expect("empty decode");
        assert!(u.five_hour.is_none());
        assert!(u.extra_usage.is_none());
    }
}
