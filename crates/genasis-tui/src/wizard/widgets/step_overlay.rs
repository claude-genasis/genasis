//! Step 5 widget: overlay injection plan + diff preview.

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

use crate::wizard::state::WizardState;

pub fn render(frame: &mut Frame, area: Rect, state: &WizardState) {
    let title = " ⑤ Overlay Injection ";

    if state.overlay.planning {
        let items = vec![ListItem::new("  Planning overlay injection...")];
        let list = List::new(items).block(Block::default().borders(Borders::ALL).title(title));
        frame.render_widget(list, area);
        return;
    }

    if state.overlay.applying {
        let items = vec![ListItem::new(format!(
            "  Applying... {}/{}",
            state.overlay.files_injected, state.overlay.files_total
        ))];
        let list = List::new(items).block(Block::default().borders(Borders::ALL).title(title));
        frame.render_widget(list, area);
        return;
    }

    if state.overlay.show_diff && !state.overlay.diff_text.is_empty() {
        let p = Paragraph::new(state.overlay.diff_text.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" ⑤ Diff Preview [d] toggle "),
            )
            .scroll((state.overlay.scroll, 0))
            .wrap(Wrap { trim: false });
        frame.render_widget(p, area);
        return;
    }

    let status = if state.overlay.applied {
        format!(
            "✅ {} files injected · {} conflicts",
            state.overlay.files_injected, state.overlay.conflicts
        )
    } else {
        format!(
            "{} files to modify · {} conflicts",
            state.overlay.files_total, state.overlay.conflicts
        )
    };

    let action = if state.overlay.applied {
        "Press Enter to continue"
    } else {
        "[Enter] Apply  [d] Show diff  [q] Cancel"
    };

    let items = vec![
        ListItem::new(""),
        ListItem::new(format!("  {status}")),
        ListItem::new(""),
        ListItem::new(Span::styled(
            format!("  {action}"),
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(title));
    frame.render_widget(list, area);
}
