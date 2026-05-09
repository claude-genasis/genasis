//! Terminal lifecycle and event loop for the init/attach wizard.
//! Mirrors `genasis-monitor/src/app.rs` patterns.

use std::io;
use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Terminal;

use genasis_core::error::Result;

use super::state::{WizardMode, WizardState};
use super::step::WizardStep;
use super::steps;
use super::widgets;

/// Main entry point. Sets up terminal, runs the wizard, restores terminal.
pub async fn run(mode: WizardMode, project_root: PathBuf, non_interactive: bool) -> Result<()> {
    if non_interactive {
        return run_text_mode(mode, project_root).await;
    }

    enable_raw_mode().map_err(io_err)?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(io_err)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(io_err)?;

    let mut state = WizardState::new(mode, project_root);

    // Kick off first step.
    steps::env::on_enter(&mut state);

    let result = run_loop(&mut terminal, &mut state).await;

    // Restore terminal.
    disable_raw_mode().map_err(io_err)?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(io_err)?;
    terminal.show_cursor().map_err(io_err)?;

    result
}

async fn run_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    state: &mut WizardState,
) -> Result<()> {
    loop {
        if state.should_quit {
            return Ok(());
        }

        // 1. Drain async results.
        while let Ok(result) = state.async_rx.try_recv() {
            steps::dispatch_async(state, result);
        }

        // 2. Draw.
        terminal
            .draw(|frame| {
                let area = frame.size();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3), // Tab bar
                        Constraint::Min(10),   // Content
                        Constraint::Length(1), // Key hints
                    ])
                    .split(area);

                widgets::tab_bar::render(frame, chunks[0], state);

                match state.current_step {
                    WizardStep::Env => widgets::step_env::render(frame, chunks[1], state),
                    WizardStep::Lang => widgets::step_lang::render(frame, chunks[1], state),
                    WizardStep::Team => widgets::step_team::render(frame, chunks[1], state),
                    WizardStep::Connect => widgets::step_connect::render(frame, chunks[1], state),
                    WizardStep::Humans => widgets::step_humans::render(frame, chunks[1], state),
                    WizardStep::Overlay => widgets::step_overlay::render(frame, chunks[1], state),
                    WizardStep::Done => widgets::step_done::render(frame, chunks[1], state),
                }

                widgets::key_hints::render(frame, chunks[2], state);
            })
            .map_err(io_err)?;

        // 3. Poll for input.
        if event::poll(Duration::from_millis(250)).map_err(io_err)? {
            if let Event::Key(key) = event::read().map_err(io_err)? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                handle_key(state, key.code);
            }
        }
    }
}

fn handle_key(state: &mut WizardState, code: KeyCode) {
    // The Humans step opens a modal CRUD form. While the form is open
    // we suppress the global Esc-quit / numeric jumps / arrow nav so
    // typing 'q', '1', etc. flows into the form fields.
    let humans_form_open = state.current_step == WizardStep::Humans && state.humans.form_open();

    if !humans_form_open {
        // Global keys.
        match code {
            KeyCode::Char('q') | KeyCode::Esc => {
                state.should_quit = true;
                return;
            }
            KeyCode::Left => {
                if let Some(prev) = state.current_step.prev() {
                    if state.steps[prev.index()].status.is_complete() {
                        state.current_step = prev;
                    }
                }
                return;
            }
            KeyCode::Right => {
                if let Some(next) = state.current_step.next() {
                    if state.steps[state.current_step.index()].status.is_complete() {
                        state.go_to(next);
                    }
                }
                return;
            }
            KeyCode::Char(c @ '1'..='7') => {
                let idx = (c as usize) - ('1' as usize);
                if let Some(target) = WizardStep::from_index(idx) {
                    state.go_to(target);
                }
                return;
            }
            _ => {}
        }
    }

    // Delegate to current step.
    match state.current_step {
        WizardStep::Env => steps::env::handle_key(state, code),
        WizardStep::Lang => steps::lang::handle_key(state, code),
        WizardStep::Team => steps::team::handle_key(state, code),
        WizardStep::Connect => steps::connect::handle_key(state, code),
        WizardStep::Humans => steps::humans::handle_key(state, code),
        WizardStep::Overlay => steps::overlay::handle_key(state, code),
        WizardStep::Done => steps::done::handle_key(state, code),
    }
}

/// Non-interactive text-only mode.
async fn run_text_mode(_mode: WizardMode, _project_root: PathBuf) -> Result<()> {
    // TODO: Phase 5 — sequential step execution with println output.
    println!("[genasis] non-interactive mode not yet implemented");
    Ok(())
}

fn io_err(e: io::Error) -> genasis_core::error::Error {
    genasis_core::error::Error::Io(e)
}
