//! Step 2: Language selection.

use crossterm::event::KeyCode;

use crate::wizard::state::WizardState;

pub fn handle_key(state: &mut WizardState, code: KeyCode) {
    match code {
        KeyCode::Up | KeyCode::Char('1') => state.lang.cursor = 0,
        KeyCode::Down | KeyCode::Char('2') => state.lang.cursor = 1,
        KeyCode::Enter => {
            let lang = if state.lang.cursor == 0 { "en" } else { "ko" };
            state.advance(lang.to_string());
        }
        _ => {}
    }
}
