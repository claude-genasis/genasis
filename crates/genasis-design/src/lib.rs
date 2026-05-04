//! genasis-design — design-system.md hot-swap orchestrator.
//!
//! Two layered APIs:
//!
//! - [`swap::run`] / [`restore::run`] — Phase D entry. External design
//!   provider integration (getdesign delegate or local `--from`), pointer
//!   body rendering, `.design-state.toml` lifecycle, pristine restore.
//! - [`change_protocol::run`] — legacy M7 5-phase orchestrator that takes
//!   an externally-produced `design-system.md` body and emits per-area
//!   IMPROVEMENT issue plans. Kept for `genasis design swap <url> --body`
//!   and consumed by M-D2 to plan EPIC + child issues post-swap.

pub mod change_protocol;
pub mod diff;
pub mod extractor;
pub mod mode;
pub mod pointer;
pub mod restore;
pub mod swap;
pub mod ticket_emitter;

pub use change_protocol::{run as run_legacy_swap, SwapOutcome as LegacySwapOutcome, SwapPhase};
pub use diff::{categorize, ImpactArea};
pub use mode::{sha256_hex, Mode, State};
pub use pointer::Locale;
pub use restore::{run as run_restore, RestoreOutcome};
pub use swap::{run as run_swap, Source as SwapSource, SwapInput, SwapOutcome};
