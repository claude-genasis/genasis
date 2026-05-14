//! Log-tail widget — recent agent activity lines.

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

use genasis_i18n::tr;

use crate::state::AppState;

/// D-086: every log line MUST render with a leading `HH:MM` so the
/// timestamp signal stays consistent even if a collector forgot to
/// prefix its push (config_hint banner, legacy sim chat lines, etc.).
/// If the line already starts with `HH:MM ` (5 chars + space) we leave
/// it alone; otherwise we slap the current local time on the front.
fn ensure_hm_prefix(line: &str) -> String {
    let bytes = line.as_bytes();
    let looks_prefixed = bytes.len() >= 6
        && bytes[2] == b':'
        && bytes[0].is_ascii_digit()
        && bytes[1].is_ascii_digit()
        && bytes[3].is_ascii_digit()
        && bytes[4].is_ascii_digit()
        && (bytes[5] == b' ' || bytes[5] == b'\t');
    if looks_prefixed {
        line.to_string()
    } else {
        let now = chrono::Local::now().format("%H:%M");
        format!("{now}  {line}")
    }
}

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
                .map(|l| ListItem::new(ensure_hm_prefix(l))),
        );
    }
    let title = format!(" {} (6) ", tr("monitor.widget.log_tail"));
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .style(Style::default().fg(Color::Gray));
    frame.render_widget(list, area);
}
