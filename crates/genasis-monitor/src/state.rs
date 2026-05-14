//! Shared state for the monitor TUI.
//!
//! Updated by collectors on their respective intervals;
//! read by the render loop every 250ms.

use std::collections::HashMap;

use crate::collector::jsonl::UsageSnapshot;
use crate::collector::plane::{AgentIssue, SprintSnapshot};
use crate::collector::sessions::ClaudeSession;

#[derive(Debug, Default, Clone)]
pub struct AppState {
    // Sprint (from Plane API, 30s poll)
    pub sprint: SprintSnapshot,

    // Token usage (from JSONL scan, 60-120s TTL)
    pub usage: UsageSnapshot,

    // Limits (configurable via env)
    pub limit_5h_tokens: u64,
    pub limit_week_all_tokens: u64,
    pub limit_week_sonnet_tokens: u64,
    pub limit_overage_usd: f64,

    // Claude sessions (from /proc, 1s poll)
    pub sessions: Vec<ClaudeSession>,

    // Agent roles → issue assignments (from Plane API)
    pub agent_issues: Vec<AgentIssue>,
    pub agent_role_uuids: HashMap<String, String>, // role → plane UUID

    // Token savings & MCP (incremented by hooks / JSONL scan)
    pub rtk_saved_tokens: u64,
    pub mcp_calls: u64,
    pub mcp_cache_hits: u64,
    pub anthropic_cache_hit_pct: f64,

    // Network counters (incremented by CLI hooks)
    pub plane_calls: u64,
    pub mm_calls: u64,
    pub gh_calls: u64,
    pub network_bytes: u64,

    // Deploy
    pub deploy: DeployState,

    // Design
    pub design: DesignWidgetState,

    // Dev server ports per role
    pub role_ports: HashMap<String, u16>,
    pub port_status: HashMap<String, bool>,

    // Log tail
    pub log_tail: Vec<String>,

    // UI state
    pub focus: WidgetFocus,

    // Plan info
    pub plan_name: String,
    pub plan_tier: String,

    // Catalog version
    pub agents_version: String,

    // Data freshness
    pub last_plane_poll: u64,
    pub last_jsonl_scan: u64,
    pub last_session_scan: u64,

    // Trial-flavor wiring (D-025) — populated at startup from
    // `genasis.toml`. When `trial_mode` is true the run-loop polls the
    // trial-app's sim endpoints instead of a real Plane API.
    pub trial_mode: bool,
    pub trial_url: String,
    pub team_token: String,
    pub project_slug: String,
    pub scrum_channel: String,
    /// Per-team demo-app metadata reported by `/api/trial/team-app/status`.
    /// Surfaced in the Sprint widget header so the operator can see
    /// which showcase the agents have published.
    pub trial_app_kind: String,
    pub trial_app_features: Vec<String>,
    /// D-058: 사용자가 잘못된 디렉터리에서 `genasis monitor` 를 실행해
    /// `genasis.toml` 을 못 찾았을 때 띄울 가이드 (banner / log_tail).
    pub config_hint: Option<String>,
}

/// Snapshot of the active design system.
#[derive(Debug, Default, Clone)]
pub struct DesignWidgetState {
    pub mode: String,
    pub slug: String,
    pub applied_at: String,
    pub override_count: u32,
    pub preview_url: String,
    pub gallery_url: String,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum WidgetFocus {
    #[default]
    Sprint,
    Tokens,
    Agents,
    Deploy,
    Sessions,
    Log,
    Design,
}

#[derive(Debug, Default, Clone)]
pub struct DeployState {
    pub dev_url: Option<String>,
    pub prod_url: Option<String>,
    pub dev_up: bool,
    pub prod_up: bool,
    pub dev_refreshed: bool,
    pub prod_refreshed: bool,
    pub last_build_sha: Option<String>,
    pub last_build_ts: Option<u64>,
}
