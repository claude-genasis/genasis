//! Data collectors for the monitor TUI.
//!
//! Each sub-module gathers data from a specific source and produces
//! a snapshot struct that the render loop reads.

pub mod jsonl;
pub mod plane;
pub mod ports;
pub mod sessions;
