//! Step 1: Environment prerequisite checks.

use crossterm::event::KeyCode;

use crate::wizard::state::{AsyncResult, PrereqCheck, StepStatus, WizardState};

/// Called when this step becomes active.
pub fn on_enter(state: &mut WizardState) {
    state.env.scanning = true;
    let tx = state.async_tx.clone();
    tokio::spawn(async move {
        let checks = scan_prerequisites();
        let _ = tx.send(AsyncResult::EnvScanComplete(checks));
    });
}

pub fn handle_key(state: &mut WizardState, code: KeyCode) {
    match code {
        KeyCode::Enter => {
            if !state.env.scanning {
                let ok_count = state.env.checks.iter().filter(|c| c.found).count();
                let total = state.env.checks.len();
                let required_ok = state
                    .env
                    .checks
                    .iter()
                    .filter(|c| c.required)
                    .all(|c| c.found);
                if required_ok {
                    state.advance(format!("{ok_count}/{total} ok"));
                } else {
                    state.fail_current("required tools missing".into());
                }
            }
        }
        KeyCode::Up => {
            if state.env.selected > 0 {
                state.env.selected -= 1;
            }
        }
        KeyCode::Down => {
            if state.env.selected + 1 < state.env.checks.len() {
                state.env.selected += 1;
            }
        }
        _ => {}
    }
}

pub fn handle_async(state: &mut WizardState, result: AsyncResult) {
    if let AsyncResult::EnvScanComplete(checks) = result {
        state.env.checks = checks;
        state.env.scanning = false;
    }
}

fn scan_prerequisites() -> Vec<PrereqCheck> {
    let tools: &[(&str, bool)] = &[
        ("git", true),
        ("curl", true),
        ("tar", true),
        ("bash", true),
        ("node", false),
        ("gh", false),
        ("claude", false),
        ("npx", false),
    ];
    tools
        .iter()
        .map(|(name, required)| {
            let found = which::which(name).is_ok();
            PrereqCheck {
                tool: name.to_string(),
                required: *required,
                found,
                version: String::new(),
            }
        })
        .collect()
}
