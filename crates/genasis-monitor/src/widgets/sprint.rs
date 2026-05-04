//! Sprint widget — shows current Cycle and issue counts.

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use genasis_i18n::tr;

use crate::state::AppState;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let dday = state
        .d_day
        .map(|d| format!("D-{d}"))
        .unwrap_or_else(|| "—".into());
    let body = format!(
        "Cycle: {}\nD-day: {}\nTodo:{}  In:{}  Review:{}  Done:{}",
        state.sprint_name, dday, state.todo, state.in_progress, state.in_review, state.done
    );
    let title = format!(" {} (1) ", tr("monitor.widget.sprint"));
    let p = Paragraph::new(body)
        .block(Block::default().borders(Borders::ALL).title(title))
        .style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(p, area);
}
