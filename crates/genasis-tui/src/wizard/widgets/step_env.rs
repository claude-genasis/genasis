//! Step 1 widget: environment prerequisite checklist.

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

use crate::wizard::state::WizardState;

pub fn render(frame: &mut Frame, area: Rect, state: &WizardState) {
    let title = " ① Environment Check ";

    if state.env.scanning {
        let items = vec![ListItem::new("  Scanning prerequisites...")];
        let list = List::new(items).block(Block::default().borders(Borders::ALL).title(title));
        frame.render_widget(list, area);
        return;
    }

    let items: Vec<ListItem> = state
        .env
        .checks
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let glyph = if c.found {
                "✅"
            } else if c.required {
                "❌"
            } else {
                "⬜"
            };
            let req = if c.required { "" } else { " (optional)" };
            let ver = if c.version.is_empty() {
                String::new()
            } else {
                format!("  {}", c.version)
            };
            let style = if i == state.env.selected {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Span::styled(
                format!("  {glyph} {:<12}{ver}{req}", c.tool),
                style,
            ))
        })
        .collect();

    let ok = state.env.checks.iter().filter(|c| c.found).count();
    let total = state.env.checks.len();
    let footer = format!("  {ok}/{total} tools available  —  Press Enter to continue");
    let mut all_items = items;
    all_items.push(ListItem::new(""));
    all_items.push(ListItem::new(Span::styled(
        footer,
        Style::default().fg(Color::DarkGray),
    )));

    let list = List::new(all_items).block(Block::default().borders(Borders::ALL).title(title));
    frame.render_widget(list, area);
}
