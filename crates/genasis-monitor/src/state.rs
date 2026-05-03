//! Shared state for the monitor TUI.

use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct AppState {
    pub sprint_name: String,
    pub d_day: Option<i64>,
    pub todo: u32,
    pub in_progress: u32,
    pub in_review: u32,
    pub done: u32,
    pub rtk_saved_tokens: u64,
    pub mcp_calls: u64,
    pub mcp_cache_hits: u64,
    pub anthropic_cache_hit_pct: f32,
    pub network_bytes: u64,
    pub plane_calls: u64,
    pub mm_calls: u64,
    pub gh_calls: u64,
    pub agents: Vec<AgentActivity>,
    pub deploy: DeployState,
    pub log_tail: Vec<String>,
    pub focus: WidgetFocus,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum WidgetFocus {
    #[default]
    Sprint,
    Tokens,
    Agents,
    Network,
    Deploy,
    Log,
}

#[derive(Debug, Default, Clone)]
pub struct AgentActivity {
    pub role: String,
    pub last_active_secs_ago: u64,
    pub current_issue: Option<String>,
    pub status: AgentStatus,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus {
    #[default]
    Idle,
    Working,
    InReview,
}

#[derive(Debug, Default, Clone)]
pub struct DeployState {
    pub dev_url: Option<String>,
    pub prod_url: Option<String>,
    pub dev_up: bool,
    pub prod_up: bool,
    pub dev_refreshed: bool,
    pub prod_refreshed: bool,
    pub last_build_ts: Option<i64>,
    pub last_build_sha: Option<String>,
    pub manifest_hash: HashMap<String, String>,
}
