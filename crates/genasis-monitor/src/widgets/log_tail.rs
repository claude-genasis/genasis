//! Log-tail widget — recent agent activity lines.

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

use genasis_i18n::tr;

use crate::state::AppState;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let items: Vec<ListItem> = if state.log_tail.is_empty() {
        vec![ListItem::new("(no log lines yet)")]
    } else {
        state
            .log_tail
            .iter()
            .rev()
            .take(area.height.saturating_sub(2) as usize)
            .map(|l| ListItem::new(l.clone()))
            .collect()
    };
    let title = format!(" {} (6) ", tr("monitor.widget.log_tail"));
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .style(Style::default().fg(Color::Gray));
    frame.render_widget(list, area);
}
