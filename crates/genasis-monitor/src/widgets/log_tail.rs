//! Log-tail widget — recent agent activity lines.

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

use genasis_i18n::tr;

use crate::state::AppState;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    // D-058 + D-072: config_hint 를 첫 줄 alert 라인으로 surface. 사용자가
    // 잘못된 dir 에서 실행해서 cfg 못 찾았거나, walk-down 으로 자동 발견됐을
    // 때 둘 다 surface. log_tail 안에도 같은 문구가 push 되어 있어 중복이
    // 가능하지만, log_tail 이 많이 차면 hint 가 밀려나가는 걸 막기 위해 alert
    // 라인은 항상 widget 상단에 sticky 로 둠.
    let mut items: Vec<ListItem> = Vec::new();
    if let Some(hint) = &state.config_hint {
        let style = if hint.starts_with("⚠") {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Cyan)
        };
        items.push(ListItem::new(hint.clone()).style(style));
    }
    if state.log_tail.is_empty() && items.is_empty() {
        items.push(ListItem::new("(no log lines yet)"));
    } else {
        let cap = area.height.saturating_sub(2) as usize;
        let remaining = cap.saturating_sub(items.len());
        items.extend(
            state
                .log_tail
                .iter()
                .rev()
                .take(remaining)
                .map(|l| ListItem::new(l.clone())),
        );
    }
    let title = format!(" {} (6) ", tr("monitor.widget.log_tail"));
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .style(Style::default().fg(Color::Gray));
    frame.render_widget(list, area);
}
