//! Per-step logic: on_enter, handle_key, handle_async.

pub mod connect;
pub mod done;
pub mod env;
pub mod humans;
pub mod lang;
pub mod overlay;
pub mod team;

use super::state::{AsyncResult, WizardState};

/// Route an async result to the correct step handler.
pub fn dispatch_async(state: &mut WizardState, result: AsyncResult) {
    match &result {
        AsyncResult::EnvScanComplete(_) => env::handle_async(state, result),
        AsyncResult::TeamScanComplete(_)
        | AsyncResult::TeamBootstrapProgress(_, _)
        | AsyncResult::TeamBootstrapDone(_) => team::handle_async(state, result),
        AsyncResult::PlaneProbeResult(_, _) | AsyncResult::MmProbeResult(_, _) => {
            connect::handle_async(state, result)
        }
        AsyncResult::OverlayPlanReady(_, _, _) | AsyncResult::OverlayApplied(_) => {
            overlay::handle_async(state, result)
        }
        AsyncResult::HumansLoaded(_) | AsyncResult::HumansSyncDone(_, _) => {
            humans::handle_async(state, result)
        }
        AsyncResult::SmokeTestProgress(_)
        | AsyncResult::SmokeTestDone(_)
        | AsyncResult::RollbackDone(_) => done::handle_async(state, result),
    }
}
