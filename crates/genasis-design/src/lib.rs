//! genasis-design — design-system.md hot-swap orchestrator.
//!
//! Public entry: [`change_protocol::run`] orchestrates the 5-phase swap.
//! Library helpers ([`extractor`], [`diff`], [`ticket_emitter`]) are exposed
//! for direct use by tests and the cmd_design CLI command.

pub mod change_protocol;
pub mod diff;
pub mod extractor;
pub mod ticket_emitter;

pub use change_protocol::{run as run_swap, SwapOutcome, SwapPhase};
pub use diff::{categorize, ImpactArea};
