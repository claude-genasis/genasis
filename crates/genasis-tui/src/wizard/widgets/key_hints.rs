//! Bottom key hint bar.

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::wizard::state::WizardState;

pub fn render(frame: &mut Frame, area: Rect, _state: &WizardState) {
    let hints = Line::from(vec![
        Span::styled(" ←/→ ", Style::default().fg(Color::Cyan)),
        Span::raw("Navigate "),
        Span::styled(" Enter ", Style::default().fg(Color::Cyan)),
        Span::raw("Confirm "),
        Span::styled(" 1-7 ", Style::default().fg(Color::Cyan)),
        Span::raw("Go to step "),
        Span::styled(" q ", Style::default().fg(Color::Cyan)),
        Span::raw("Quit "),
        Span::styled(" Shift+drag ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "select text (in tmux)",
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    frame.render_widget(Paragraph::new(hints), area);
}
