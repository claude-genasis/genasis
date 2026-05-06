//! Step 6: Done summary, smoke test, rollback.

use crossterm::event::KeyCode;

use crate::wizard::state::{AsyncResult, SmokeTestStatus, WizardState};

pub fn handle_key(state: &mut WizardState, code: KeyCode) {
    match code {
        KeyCode::Tab => {
            state.done.button_focus = (state.done.button_focus + 1) % 4;
        }
        KeyCode::Enter => match state.done.button_focus {
            0 => {
                // Run smoke test
                state.done.smoke_status = SmokeTestStatus::Running;
                // TODO: spawn smoke test
            }
            1 => {
                // Skip — just finish
                state.should_quit = true;
            }
            2 if state.done.rollback_available => {
                // Rollback
                // TODO: spawn rollback
            }
            3 => {
                // Open monitor
                // TODO: spawn genasis monitor process
                state.should_quit = true;
            }
            _ => {}
        },
        _ => {}
    }
}

pub fn handle_async(state: &mut WizardState, result: AsyncResult) {
    match result {
        AsyncResult::SmokeTestProgress(line) => {
            state.done.smoke_output.push(line);
        }
        AsyncResult::SmokeTestDone(ok) => {
            state.done.smoke_status = if ok {
                SmokeTestStatus::Passed
            } else {
                SmokeTestStatus::Failed
            };
            state.done.rollback_available = true;
        }
        AsyncResult::RollbackDone(_) => {
            state.done.rollback_available = false;
        }
        _ => {}
    }
}
