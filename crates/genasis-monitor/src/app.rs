//! Monitor app loop — Ratatui + crossterm event-driven render.

use std::io;
use std::time::Duration;

use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Terminal;

use genasis_core::error::Result;

use crate::state::{AppState, WidgetFocus};
use crate::widgets;

pub async fn run() -> Result<()> {
    let mut state = AppState::default();
    state.sprint_name = "(no sprint loaded)".into();

    enable_raw_mode().map_err(io_err)?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).map_err(io_err)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(io_err)?;

    let res = run_loop(&mut terminal, &mut state).await;

    disable_raw_mode().map_err(io_err)?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .map_err(io_err)?;
    terminal.show_cursor().map_err(io_err)?;
    res
}

async fn run_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    state: &mut AppState,
) -> Result<()> {
    loop {
        terminal
            .draw(|frame| {
                let area = frame.area();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(6),  // Sprint + Tokens row
                        Constraint::Length(8),  // Agents
                        Constraint::Length(7),  // Deploy + Network
                        Constraint::Min(3),     // Log tail
                    ])
                    .split(area);

                let top_row = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(chunks[0]);
                widgets::sprint::render(frame, top_row[0], state);
                widgets::tokens::render(frame, top_row[1], state);

                widgets::agents::render(frame, chunks[1], state);

                let mid_row = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                    .split(chunks[2]);
                widgets::deploy::render(frame, mid_row[0], state);
                widgets::network::render(frame, mid_row[1], state);

                widgets::log_tail::render(frame, chunks[3], state);
            })
            .map_err(io_err)?;

        if event::poll(Duration::from_millis(250)).map_err(io_err)? {
            if let Event::Key(key) = event::read().map_err(io_err)? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('1') => state.focus = WidgetFocus::Sprint,
                    KeyCode::Char('2') => state.focus = WidgetFocus::Tokens,
                    KeyCode::Char('3') => state.focus = WidgetFocus::Agents,
                    KeyCode::Char('4') => state.focus = WidgetFocus::Deploy,
                    KeyCode::Char('5') => state.focus = WidgetFocus::Network,
                    KeyCode::Char('6') => state.focus = WidgetFocus::Log,
                    KeyCode::Char('v') => {
                        state.deploy.dev_refreshed = false;
                        state.deploy.prod_refreshed = false;
                    }
                    _ => {}
                }
            }
        }
    }
}

fn io_err(e: io::Error) -> genasis_core::Error {
    genasis_core::Error::Io(e)
}
