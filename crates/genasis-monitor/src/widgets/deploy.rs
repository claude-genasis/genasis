//! Deploy widget — dev / prod URL LEDs + REFRESHED badge.

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use genasis_i18n::tr;

use crate::state::AppState;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let mut lines: Vec<Line> = Vec::new();

    push_url_line(
        &mut lines,
        "dev",
        &state.deploy.dev_url,
        state.deploy.dev_up,
        state.deploy.dev_refreshed,
    );
    push_url_line(
        &mut lines,
        "prod",
        &state.deploy.prod_url,
        state.deploy.prod_up,
        state.deploy.prod_refreshed,
    );

    if let Some(sha) = &state.deploy.last_build_sha {
        let when = state
            .deploy
            .last_build_ts
            .map(|t| format!("{t} (unix)"))
            .unwrap_or_else(|| "—".into());
        lines.push(Line::from(format!("Last build: {sha} @ {when}")));
    }

    lines.push(Line::from(tr("monitor.key_hint")));

    let title = format!(" {} (4) ", tr("monitor.widget.deploy"));
    let p = Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(title));
    frame.render_widget(p, area);
}

fn push_url_line(
    lines: &mut Vec<Line>,
    label: &str,
    url: &Option<String>,
    up: bool,
    refreshed: bool,
) {
    let led = if up { "●" } else { "○" };
    let url_s = url.clone().unwrap_or_else(|| "(unset)".to_string());
    let mut spans = vec![
        Span::styled(
            led,
            Style::default().fg(if up { Color::Green } else { Color::Red }),
        ),
        Span::raw(format!(" {:<4} {}", label, url_s)),
    ];
    if refreshed {
        spans.push(Span::styled(
            "  🔄 REFRESHED",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    }
    lines.push(Line::from(spans));
}
