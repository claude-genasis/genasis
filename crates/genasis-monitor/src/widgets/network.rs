//! Network widget — Plane / MM / GitHub call counters and bytes.

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use genasis_i18n::tr;

use crate::state::AppState;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let body = format!(
        "Plane: {}\nMM:    {}\nGH:    {}\nBytes: {}",
        state.plane_calls,
        state.mm_calls,
        state.gh_calls,
        human_bytes(state.network_bytes),
    );
    let title = format!(" {} (5) ", tr("monitor.widget.network"));
    let p = Paragraph::new(body)
        .block(Block::default().borders(Borders::ALL).title(title))
        .style(Style::default().fg(Color::Magenta));
    frame.render_widget(p, area);
}

fn human_bytes(b: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut v = b as f64;
    let mut idx = 0;
    while v >= 1024.0 && idx < UNITS.len() - 1 {
        v /= 1024.0;
        idx += 1;
    }
    if idx == 0 {
        format!("{} {}", b, UNITS[idx])
    } else {
        format!("{:.1} {}", v, UNITS[idx])
    }
}
