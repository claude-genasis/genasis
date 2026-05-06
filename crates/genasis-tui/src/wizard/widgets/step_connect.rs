//! Step 4 widget: Plane + Mattermost connection.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

use crate::wizard::state::{ConnStatus, WizardState};

pub fn render(frame: &mut Frame, area: Rect, state: &WizardState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_plane(frame, chunks[0], state);
    render_mm(frame, chunks[1], state);
}

fn status_glyph(s: ConnStatus) -> (&'static str, Color) {
    match s {
        ConnStatus::Untested => ("○", Color::DarkGray),
        ConnStatus::Testing => ("●", Color::Yellow),
        ConnStatus::Ok => ("✅", Color::Green),
        ConnStatus::Failed => ("❌", Color::Red),
        ConnStatus::Skipped => ("⬜", Color::DarkGray),
    }
}

fn render_plane(frame: &mut Frame, area: Rect, state: &WizardState) {
    let (glyph, color) = status_glyph(state.connect.plane_status);
    let focus_style = |idx: usize| {
        if state.connect.focus == idx {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        }
    };

    let items = vec![
        ListItem::new(Span::styled(
            format!("  URL:       {}", state.connect.plane_url),
            focus_style(0),
        )),
        ListItem::new(Span::styled(
            format!("  Workspace: {}", state.connect.plane_workspace),
            focus_style(1),
        )),
        ListItem::new(Span::styled(
            format!("  Status:    {glyph} {:?}", state.connect.plane_status),
            Style::default().fg(color),
        )),
    ];

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(" ④ Plane "));
    frame.render_widget(list, area);
}

fn render_mm(frame: &mut Frame, area: Rect, state: &WizardState) {
    let (glyph, color) = status_glyph(state.connect.mm_status);

    let items = vec![
        ListItem::new(Span::styled(
            format!("  URL:       {}", state.connect.mm_url),
            Style::default(),
        )),
        ListItem::new(Span::styled(
            format!("  Status:    {glyph} {:?}", state.connect.mm_status),
            Style::default().fg(color),
        )),
        ListItem::new(""),
        ListItem::new(Span::styled(
            "  [Tab] cycle  [Enter] probe  [s] skip",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" ④ Mattermost "),
    );
    frame.render_widget(list, area);
}
