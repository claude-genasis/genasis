//! genasis-monitor — runtime Ratatui dashboard.
//!
//! Widgets: sprint counts, agent activity, RTK token savings, MCP/cache stats,
//! network bytes, deploy LED with REFRESHED badge, log tail.
//!
//! Actions: build, deploy, rollback, mark visited.
//!
//! M9 fills these in.

pub mod actions;
pub mod app;
pub mod state;
pub mod widgets;
