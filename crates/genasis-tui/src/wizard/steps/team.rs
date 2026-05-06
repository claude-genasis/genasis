//! Step 3: Team bootstrap (init) or agent detection (attach).

use crossterm::event::KeyCode;

use crate::wizard::state::{AsyncResult, WizardState};

pub fn handle_key(state: &mut WizardState, code: KeyCode) {
    match code {
        KeyCode::Enter if state.team.done => {
            let n = state.team.agents_found.len();
            state.advance(format!("{n} agents"));
        }
        KeyCode::Up => {
            if state.team.selected > 0 {
                state.team.selected -= 1;
            }
        }
        KeyCode::Down => {
            if state.team.selected + 1 < state.team.agents_found.len() {
                state.team.selected += 1;
            }
        }
        _ => {}
    }
}

pub fn handle_async(state: &mut WizardState, result: AsyncResult) {
    match result {
        AsyncResult::TeamScanComplete(agents) => {
            state.team.agents_found = agents;
            state.team.scanning = false;
            state.team.done = true;
        }
        AsyncResult::TeamBootstrapProgress(created, total) => {
            state.team.agents_created = created;
            state.team.agents_total = total;
        }
        AsyncResult::TeamBootstrapDone(agents) => {
            state.team.agents_found = agents;
            state.team.applying = false;
            state.team.done = true;
        }
        _ => {}
    }
}
