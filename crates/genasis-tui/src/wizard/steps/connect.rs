//! Step 4: Plane + Mattermost connection.

use crossterm::event::KeyCode;

use crate::wizard::state::{AsyncResult, ConnStatus, WizardState};

pub fn handle_key(state: &mut WizardState, code: KeyCode) {
    match code {
        KeyCode::Enter => {
            let p = state.connect.plane_status;
            let m = state.connect.mm_status;
            if p == ConnStatus::Ok || p == ConnStatus::Skipped {
                if m == ConnStatus::Ok || m == ConnStatus::Skipped {
                    let ps = if p == ConnStatus::Ok {
                        "P✅"
                    } else {
                        "P skip"
                    };
                    let ms = if m == ConnStatus::Ok {
                        "M✅"
                    } else {
                        "M skip"
                    };
                    state.advance(format!("{ps} {ms}"));
                }
            }
        }
        KeyCode::Tab => {
            state.connect.focus = (state.connect.focus + 1) % 4;
        }
        KeyCode::Char('s') => {
            // Skip shortcuts
            if state.connect.plane_status == ConnStatus::Untested {
                state.connect.plane_status = ConnStatus::Skipped;
            }
            if state.connect.mm_status == ConnStatus::Untested {
                state.connect.mm_status = ConnStatus::Skipped;
            }
        }
        _ => {}
    }
}

pub fn handle_async(state: &mut WizardState, result: AsyncResult) {
    match result {
        AsyncResult::PlaneProbeResult(ok, msg) => {
            state.connect.plane_status = if ok {
                ConnStatus::Ok
            } else {
                ConnStatus::Failed
            };
            state.connect.probing = false;
            if !ok {
                state.connect.plane_url = msg;
            }
        }
        AsyncResult::MmProbeResult(ok, msg) => {
            state.connect.mm_status = if ok {
                ConnStatus::Ok
            } else {
                ConnStatus::Failed
            };
            state.connect.probing = false;
            if !ok {
                state.connect.mm_url = msg;
            }
        }
        _ => {}
    }
}
