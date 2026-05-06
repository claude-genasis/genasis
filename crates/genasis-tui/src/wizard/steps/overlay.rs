//! Step 5: Overlay injection plan + apply.

use crossterm::event::KeyCode;

use crate::wizard::state::{AsyncResult, WizardState};

pub fn handle_key(state: &mut WizardState, code: KeyCode) {
    match code {
        KeyCode::Enter if state.overlay.applied => {
            state.advance(format!("{} injected", state.overlay.files_injected));
        }
        KeyCode::Char('d') => {
            state.overlay.show_diff = !state.overlay.show_diff;
        }
        KeyCode::Up => {
            state.overlay.scroll = state.overlay.scroll.saturating_sub(1);
        }
        KeyCode::Down => {
            state.overlay.scroll = state.overlay.scroll.saturating_add(1);
        }
        _ => {}
    }
}

pub fn handle_async(state: &mut WizardState, result: AsyncResult) {
    match result {
        AsyncResult::OverlayPlanReady(total, conflicts, diff) => {
            state.overlay.files_total = total;
            state.overlay.conflicts = conflicts;
            state.overlay.diff_text = diff;
            state.overlay.planning = false;
        }
        AsyncResult::OverlayApplied(count) => {
            state.overlay.files_injected = count;
            state.overlay.applying = false;
            state.overlay.applied = true;
        }
        _ => {}
    }
}
