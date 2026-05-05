//! Design widget — shows the active design (pristine vs external) plus
//! preview / gallery links. M-D3.
//!
//! Key bindings (handled in app.rs):
//!  - `7`     focus this widget
//!  - Enter   open `preview_url` (or `gallery_url` in pristine mode) in
//!            the OS default browser via `open` / `xdg-open`.

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use genasis_i18n::tr;

use crate::state::{AppState, WidgetFocus};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let mut lines: Vec<Line> = Vec::new();
    let d = &state.design;
    if d.mode == "external" {
        lines.push(Line::from(vec![
            Span::raw("Mode: "),
            Span::styled("external", Style::default().add_modifier(Modifier::BOLD)),
        ]));
        let head = format!(
            "🎨 {}    applied {}",
            if d.slug.is_empty() { "(unset)" } else { &d.slug },
            if d.applied_at.is_empty() {
                "(unset)"
            } else {
                &d.applied_at
            },
        );
        lines.push(Line::from(head));
        lines.push(Line::from(format!("Overrides: {}", d.override_count)));
        if !d.preview_url.is_empty() {
            lines.push(Line::from(format!("Preview ▸ {}", d.preview_url)));
        }
        if !d.gallery_url.is_empty() {
            lines.push(Line::from(format!("Gallery ▸ {}", d.gallery_url)));
        }
    } else {
        lines.push(Line::from(vec![
            Span::raw("Mode: "),
            Span::styled("pristine", Style::default().add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(
            "Source: docs/design-system.md (local)".to_string(),
        ));
        if !d.gallery_url.is_empty() {
            lines.push(Line::from(format!("Gallery ▸ {}", d.gallery_url)));
        }
    }
    if state.focus == WidgetFocus::Design {
        lines.push(Line::from(
            "[Enter] open preview/gallery in browser".to_string(),
        ));
    }
    let title = format!(" {} (7) ", tr("monitor.widget.design"));
    let p = Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(title));
    frame.render_widget(p, area);
}
