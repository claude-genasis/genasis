//! Monitor app loop — Ratatui + crossterm event-driven render.
//!
//! Collectors run on independent timers:
//! - Session detection: every 1s
//! - JSONL scan: every 60s (TTL-based cache)
//! - Plane API poll: every 30s (async, separate from render)
//! - Port probe: every 5s
//! - Render: every 250ms

use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Terminal;

use genasis_core::error::Result;

use crate::collector;
use crate::state::{AppState, WidgetFocus};
use crate::widgets;

const RENDER_TICK: Duration = Duration::from_millis(250);
const SESSION_TICK: Duration = Duration::from_secs(1);
const JSONL_TICK: Duration = Duration::from_secs(60);
const PORT_TICK: Duration = Duration::from_secs(5);

pub async fn run() -> Result<()> {
    let mut state = AppState::default();

    // Load configuration from env
    state.limit_5h_tokens = env_u64("MONITOR_5H_TOKEN_LIMIT", 7_000_000);
    state.limit_week_all_tokens = env_u64("MONITOR_WEEK_ALL_TOKEN_LIMIT", 50_000_000);
    state.limit_week_sonnet_tokens = env_u64("MONITOR_WEEK_SONNET_TOKEN_LIMIT", 30_000_000);
    state.limit_overage_usd = env_f64("MONITOR_OVERAGE_LIMIT_USD", 200.0);

    // Load plan info from credentials
    let (plan, tier) = collector::jsonl::read_credentials();
    state.plan_name = plan;
    state.plan_tier = tier;

    // Load design state
    state.design = load_design_state();

    // Initial data collection
    collect_sessions(&mut state);
    collect_jsonl(&mut state);
    collect_ports(&mut state);

    // Terminal setup. Mouse capture is intentionally NOT enabled so the
    // host terminal retains native text selection (drag, double-click,
    // triple-click). The monitor consumes only keyboard events; if a
    // future widget needs click handling, gate EnableMouseCapture
    // behind an opt-in flag rather than turning it on globally.
    enable_raw_mode().map_err(io_err)?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(io_err)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(io_err)?;

    let res = run_loop(&mut terminal, &mut state).await;

    // Cleanup
    disable_raw_mode().map_err(io_err)?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(io_err)?;
    terminal.show_cursor().map_err(io_err)?;
    res
}

async fn run_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    state: &mut AppState,
) -> Result<()> {
    let mut last_session = Instant::now();
    let mut last_jsonl = Instant::now();
    let mut last_port = Instant::now();

    loop {
        // Collect data on schedule
        if last_session.elapsed() >= SESSION_TICK {
            collect_sessions(state);
            last_session = Instant::now();
        }
        if last_jsonl.elapsed() >= JSONL_TICK {
            collect_jsonl(state);
            last_jsonl = Instant::now();
        }
        if last_port.elapsed() >= PORT_TICK {
            collect_ports(state);
            last_port = Instant::now();
        }

        // Render
        terminal
            .draw(|frame| {
                let area = frame.size();

                let main = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(6), // Sprint + Tokens
                        Constraint::Length(8), // Agents
                        Constraint::Length(5), // Deploy + Network + Design
                        Constraint::Length(8), // Sessions + Usage bars
                        Constraint::Min(3),    // Log tail
                    ])
                    .split(area);

                // Row 0: Sprint (left) + Tokens (right)
                let top_row = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(main[0]);
                widgets::sprint::render(frame, top_row[0], state);
                widgets::tokens::render(frame, top_row[1], state);

                // Row 1: Agents
                widgets::agents::render(frame, main[1], state);

                // Row 2: Deploy + Network + Design
                let mid_row = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(35),
                        Constraint::Percentage(30),
                        Constraint::Percentage(35),
                    ])
                    .split(main[2]);
                widgets::deploy::render(frame, mid_row[0], state);
                widgets::network::render(frame, mid_row[1], state);
                widgets::design::render(frame, mid_row[2], state);

                // Row 3: Sessions (65%) + Usage bars (35%)
                widgets::sessions::render(frame, main[3], state);

                // Row 4: Log tail
                widgets::log_tail::render(frame, main[4], state);
            })
            .map_err(io_err)?;

        // Handle keyboard events (250ms poll timeout)
        if event::poll(RENDER_TICK).map_err(io_err)? {
            if let Event::Key(key) = event::read().map_err(io_err)? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('r') => {
                        collect_sessions(state);
                        collect_jsonl(state);
                        collect_ports(state);
                    }
                    KeyCode::Char('1') => state.focus = WidgetFocus::Sprint,
                    KeyCode::Char('2') => state.focus = WidgetFocus::Tokens,
                    KeyCode::Char('3') => state.focus = WidgetFocus::Agents,
                    KeyCode::Char('4') => state.focus = WidgetFocus::Deploy,
                    KeyCode::Char('5') => state.focus = WidgetFocus::Sessions,
                    KeyCode::Char('6') => state.focus = WidgetFocus::Log,
                    KeyCode::Char('7') => state.focus = WidgetFocus::Design,
                    KeyCode::Tab => {
                        state.focus = match state.focus {
                            WidgetFocus::Sprint => WidgetFocus::Tokens,
                            WidgetFocus::Tokens => WidgetFocus::Agents,
                            WidgetFocus::Agents => WidgetFocus::Deploy,
                            WidgetFocus::Deploy => WidgetFocus::Sessions,
                            WidgetFocus::Sessions => WidgetFocus::Log,
                            WidgetFocus::Log => WidgetFocus::Design,
                            WidgetFocus::Design => WidgetFocus::Sprint,
                        };
                    }
                    _ => {}
                }
            }
        }
    }
}

fn collect_sessions(state: &mut AppState) {
    let project_root = std::env::current_dir().unwrap_or_default();
    let project_name = project_root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");
    let wt_prefix = format!("/tmp/{}-", project_name);

    state.sessions = collector::sessions::detect_sessions(&project_root, &wt_prefix);

    for session in &mut state.sessions {
        if state.usage.ctx_window_size > 0 {
            session.context_pct = Some(state.usage.ctx_pct());
        }
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    state.last_session_scan = now;
}

fn collect_jsonl(state: &mut AppState) {
    state.usage = collector::jsonl::scan_sessions_dir();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    state.last_jsonl_scan = now;
}

fn collect_ports(state: &mut AppState) {
    if !state.role_ports.is_empty() {
        state.port_status = collector::ports::probe_ports(&state.role_ports);
    }
}

fn load_design_state() -> crate::state::DesignWidgetState {
    let path = std::path::Path::new("docs/.design-state.toml");
    if !path.exists() {
        return crate::state::DesignWidgetState::default();
    }
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return crate::state::DesignWidgetState::default(),
    };
    let get = |key: &str| -> String {
        content
            .lines()
            .find(|l| l.starts_with(key))
            .and_then(|l| l.split('=').nth(1))
            .map(|v| v.trim().trim_matches('"').to_string())
            .unwrap_or_default()
    };
    crate::state::DesignWidgetState {
        mode: get("mode"),
        slug: get("slug"),
        applied_at: get("applied_at"),
        override_count: get("override_count").parse().unwrap_or(0),
        preview_url: get("gallery_preview"),
        gallery_url: String::new(),
    }
}

fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn env_f64(key: &str, default: f64) -> f64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn io_err(e: io::Error) -> genasis_core::error::Error {
    genasis_core::error::Error::Io(e)
}
