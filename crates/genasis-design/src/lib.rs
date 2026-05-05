//! genasis-design — design-system.md hot-swap orchestrator.
//!
//! Two layered APIs:
//!
//! - [`swap::run`] / [`restore::run`] / [`verify::run`] / [`override_log`]
//!   — Phase D entries. External design provider integration (getdesign
//!   delegate or local `--from`), pointer body rendering, `.design-state.toml`
//!   lifecycle, pristine restore, sha256 verification, user-override §B
//!   accumulation.
//! - [`change_protocol::run`] — legacy M7 5-phase orchestrator that takes
//!   an externally-produced `design-system.md` body and emits per-area
//!   IMPROVEMENT issue plans. Kept for `genasis design swap <url> --body`.
//! - [`ticket_emitter::auto_plan`] — Phase D EPIC vs per-area dispatcher
//!   used by `cmd_design swap` after a successful swap to choose how to
//!   plan Plane issues.

pub mod change_protocol;
pub mod diff;
pub mod extractor;
pub mod mode;
pub mod override_log;
pub mod pointer;
pub mod restore;
pub mod swap;
pub mod ticket_emitter;
pub mod verify;

pub use change_protocol::{run as run_legacy_swap, SwapOutcome as LegacySwapOutcome, SwapPhase};
pub use diff::{categorize, changed_areas, ImpactArea};
pub use mode::{sha256_hex, Mode, State};
pub use override_log::{
    add as override_add, list as override_list, remove as override_remove, OverrideEntry,
};
pub use pointer::Locale;
pub use restore::{run as run_restore, RestoreOutcome};
pub use swap::{run as run_swap, Source as SwapSource, SwapInput, SwapOutcome};
pub use ticket_emitter::{
    auto_plan, Plan, PlanMode, PlannedEpic, PlannedIssue, DEFAULT_FULL_REWRITE_THRESHOLD,
};
pub use verify::{run as run_verify, VerifyOutcome};
