//! Claude Code sessions + usage widget.
//!
//! Left 2/3: active session list (PID, role, ctx%, age, state)
//! Right 1/3: usage bar charts (5h, 7d all, 7d sonnet, cost)

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};
use ratatui::Frame;

use crate::collector::sessions::{format_age, format_countdown, SessionState};
use crate::state::AppState;

/// Render the sessions + usage panel.
pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(65),
            Constraint::Percentage(35),
        ])
        .split(area);

    render_sessions_list(frame, chunks[0], state);
    render_usage_bars(frame, chunks[1], state);
}

fn render_sessions_list(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default()
        .title(" SESSIONS ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if state.sessions.is_empty() {
        let msg = Paragraph::new("  No active Claude Code sessions detected.");
        frame.render_widget(msg, inner);
        return;
    }

    let mut lines: Vec<Line> = Vec::new();

    // Header
    lines.push(Line::from(vec![
        Span::styled(
            format!("{:<7} {:<12} {:>5} {:>6}  {}", "PID", "role", "ctx%", "age", "state"),
            Style::default().fg(Color::DarkGray),
        ),
    ]));

    for session in &state.sessions {
        let state_color = match session.state {
            SessionState::Active => Color::Green,
            SessionState::Idle => Color::DarkGray,
            SessionState::Error => Color::Red,
        };
        let state_icon = match session.state {
            SessionState::Active => "●",
            SessionState::Idle => "○",
            SessionState::Error => "✗",
        };
        let role = session.role.as_deref().unwrap_or("-");
        let ctx = session.context_pct
            .map(|p| format!("{:.0}%", p))
            .unwrap_or_else(|| "-".into());
        let age = format_age(session.age_secs);

        lines.push(Line::from(vec![
            Span::raw(format!("{:<7} ", session.pid)),
            Span::styled(format!("{:<12} ", role), Style::default().fg(Color::Cyan)),
            Span::raw(format!("{:>5} ", ctx)),
            Span::raw(format!("{:>6}  ", age)),
            Span::styled(
                format!("{} {}", state_icon, format!("{:?}", session.state).to_lowercase()),
                Style::default().fg(state_color),
            ),
        ]));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn render_usage_bars(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default()
        .title(" CLAUDE USAGE ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // 5h session
            Constraint::Length(2), // 7d all
            Constraint::Length(2), // 7d sonnet
            Constraint::Length(1), // cost
            Constraint::Length(1), // reset
            Constraint::Min(0),
        ])
        .split(inner);

    // 5h session usage
    let five_pct = state.usage.five_h_pct(state.limit_5h_tokens);
    let five_color = pct_color(five_pct);
    let five_label = format!("5h session  {:.0}%", five_pct);
    let five_gauge = Gauge::default()
        .gauge_style(Style::default().fg(five_color))
        .ratio((five_pct as f64 / 100.0).min(1.0))
        .label(five_label);
    frame.render_widget(five_gauge, chunks[0]);

    // 7d all models
    let week_pct = state.usage.week_all_pct(state.limit_week_all_tokens);
    let week_color = pct_color(week_pct);
    let week_label = format!("7d (all)    {:.0}%", week_pct);
    let week_gauge = Gauge::default()
        .gauge_style(Style::default().fg(week_color))
        .ratio((week_pct as f64 / 100.0).min(1.0))
        .label(week_label);
    frame.render_widget(week_gauge, chunks[1]);

    // 7d sonnet only
    let sonnet_pct = state.usage.week_sonnet_pct(state.limit_week_sonnet_tokens);
    let sonnet_label = format!("7d (Sonnet) {:.0}%", sonnet_pct);
    let sonnet_gauge = Gauge::default()
        .gauge_style(Style::default().fg(Color::Blue))
        .ratio((sonnet_pct as f64 / 100.0).min(1.0))
        .label(sonnet_label);
    frame.render_widget(sonnet_gauge, chunks[2]);

    // Cost
    let cost_text = format!(
        " ${:.2} / ${:.0}",
        state.usage.five_h_cost_usd,
        state.limit_overage_usd,
    );
    let cost_line = Paragraph::new(cost_text)
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(cost_line, chunks[3]);

    // Reset countdown
    let countdown = format_countdown(state.usage.five_h_reset_countdown());
    let reset_text = format!(" Reset: {}", countdown);
    let reset_line = Paragraph::new(reset_text)
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(reset_line, chunks[4]);
}

fn pct_color(pct: f32) -> Color {
    if pct >= 85.0 {
        Color::Red
    } else if pct >= 60.0 {
        Color::Yellow
    } else {
        Color::Green
    }
}
