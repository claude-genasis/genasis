//! Step 2 widget: language selection.

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

use crate::wizard::state::WizardState;

pub fn render(frame: &mut Frame, area: Rect, state: &WizardState) {
    let options = [("English", "en"), ("한국어", "ko")];

    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, (label, code))| {
            let arrow = if i == state.lang.cursor { "▸" } else { " " };
            let style = if i == state.lang.cursor {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Span::styled(format!("  {arrow} [{code}] {label}"), style))
        })
        .collect();

    let mut all = vec![
        ListItem::new(""),
        ListItem::new("  Choose agent instruction language:"),
        ListItem::new("  에이전트 지시 언어를 선택하세요:"),
        ListItem::new(""),
    ];
    all.extend(items);
    all.push(ListItem::new(""));
    all.push(ListItem::new(Span::styled(
        "  ⚠ Both languages in one context is not supported.",
        Style::default().fg(Color::Yellow),
    )));
    all.push(ListItem::new(Span::styled(
        "    See: docs/impact-of-multilang-prompts.md",
        Style::default().fg(Color::DarkGray),
    )));

    let list = List::new(all).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" ② Language Selection "),
    );
    frame.render_widget(list, area);
}
