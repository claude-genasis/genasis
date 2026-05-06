//! Top tab bar: shows 6 steps with status + summary.

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::wizard::state::WizardState;
use crate::wizard::step::WizardStep;

pub fn render(frame: &mut Frame, area: Rect, state: &WizardState) {
    let mut tabs = Vec::new();
    let mut summaries = Vec::new();

    for step in WizardStep::ALL {
        let idx = step.index();
        let meta = &state.steps[idx];
        let is_current = state.current_step == step;

        let glyph = meta.status.glyph();
        let label = step.label(state.mode);
        let num = step.number();

        let style = if is_current {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else if meta.status.is_complete() {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        tabs.push(Span::styled(format!(" {glyph}{num} {label} "), style));
        tabs.push(Span::raw("│"));

        // Summary line
        let sum = if meta.summary.is_empty() {
            "—".to_string()
        } else {
            meta.summary.clone()
        };
        summaries.push(Span::styled(
            format!(" {sum} "),
            Style::default().fg(Color::DarkGray),
        ));
        summaries.push(Span::raw("│"));
    }

    let content = vec![Line::from(tabs), Line::from(summaries)];

    let widget = Paragraph::new(content).block(
        Block::default()
            .borders(Borders::BOTTOM)
            .title(" genasis init "),
    );
    frame.render_widget(widget, area);
}
