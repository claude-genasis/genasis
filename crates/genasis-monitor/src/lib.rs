//! genasis-monitor — runtime Ratatui dashboard.
//!
//! Collectors gather data from Plane API, Claude sessions, JSONL logs,
//! and TCP port probes. Widgets render the data every 250ms.
//!
//! See agents-pool/prd/monitor-app.md for the full PRD.

pub mod actions;
pub mod app;
pub mod collector;
pub mod state;
pub mod widgets;
