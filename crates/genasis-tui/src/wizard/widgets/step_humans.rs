//! Step 5 widget: human roster CRUD.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::wizard::state::{HumansForm, WizardState};

pub fn render(frame: &mut Frame, area: Rect, state: &WizardState) {
    if let Some(form) = state.humans.form.as_ref() {
        render_form(frame, area, form);
        return;
    }
    render_list(frame, area, state);
}

fn render_list(frame: &mut Frame, area: Rect, state: &WizardState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),
            Constraint::Length(3),
            Constraint::Length(2),
        ])
        .split(area);

    let title = " ⑤ Human Roster (ADR-014) ";

    let items: Vec<ListItem> = if state.humans.entries.is_empty() {
        vec![ListItem::new(Span::styled(
            "  (no humans yet — press `a` to add one)",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        state
            .humans
            .entries
            .iter()
            .enumerate()
            .map(|(i, h)| {
                let style = if i == state.humans.selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                let prov = match h.provisioned.as_str() {
                    "mm+plane" => "✅ mm+plane",
                    "mm" => "🟡 mm",
                    "plane" => "🟡 plane",
                    _ => "⬜ no",
                };
                ListItem::new(Span::styled(
                    format!(
                        "  {:<22} {:<28} {:<14} {}",
                        truncate(&h.name, 22),
                        truncate(&h.email, 28),
                        truncate(&h.role, 14),
                        prov
                    ),
                    style,
                ))
            })
            .collect()
    };

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(title));
    frame.render_widget(list, chunks[0]);

    let hints = Paragraph::new(Line::from(vec![
        Span::styled(" a ", Style::default().fg(Color::Cyan)),
        Span::raw("add  "),
        Span::styled(" e ", Style::default().fg(Color::Cyan)),
        Span::raw("edit  "),
        Span::styled(" d ", Style::default().fg(Color::Cyan)),
        Span::raw("delete  "),
        Span::styled(" s ", Style::default().fg(Color::Cyan)),
        Span::raw("sync to MM+Plane  "),
        Span::styled(" Enter ", Style::default().fg(Color::Cyan)),
        Span::raw("continue"),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" actions "));
    frame.render_widget(hints, chunks[1]);

    let status_color = if state.humans.syncing {
        Color::Yellow
    } else if state.humans.status_line.starts_with('+') || state.humans.status_line.starts_with('~')
    {
        Color::Green
    } else if state.humans.status_line.starts_with('-')
        || state.humans.status_line.contains("failed")
    {
        Color::Red
    } else {
        Color::DarkGray
    };
    let status = Paragraph::new(Span::styled(
        format!(
            "  {}",
            if state.humans.status_line.is_empty() {
                "—".to_string()
            } else {
                state.humans.status_line.clone()
            }
        ),
        Style::default().fg(status_color),
    ));
    frame.render_widget(status, chunks[2]);
}

fn render_form(frame: &mut Frame, area: Rect, form: &HumansForm) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    let title = if form.editing_email.is_some() {
        " ⑤ Edit human "
    } else {
        " ⑤ Add human "
    };
    frame.render_widget(
        Block::default().borders(Borders::ALL).title(title),
        Rect::new(area.x, area.y, area.width, 1),
    );

    field(frame, chunks[0], "name", &form.name, form.focus == 0);
    field(frame, chunks[1], "email", &form.email, form.focus == 1);
    field(frame, chunks[2], "role", &form.role, form.focus == 2);
    field(
        frame,
        chunks[3],
        "mm_username (optional)",
        &form.mm_username,
        form.focus == 3,
    );
    field(
        frame,
        chunks[4],
        "locale (optional, en|ko)",
        &form.locale,
        form.focus == 4,
    );

    let hints = Paragraph::new(Line::from(vec![
        Span::styled(" Tab ", Style::default().fg(Color::Cyan)),
        Span::raw("next field  "),
        Span::styled(" Shift+Tab ", Style::default().fg(Color::Cyan)),
        Span::raw("prev  "),
        Span::styled(" Enter ", Style::default().fg(Color::Cyan)),
        Span::raw("save  "),
        Span::styled(" Esc ", Style::default().fg(Color::Cyan)),
        Span::raw("cancel"),
    ]));
    frame.render_widget(hints, chunks[5]);

    if !form.error.is_empty() {
        let err = Paragraph::new(Span::styled(
            format!("  ⚠ {}", form.error),
            Style::default().fg(Color::Red),
        ));
        frame.render_widget(err, chunks[6]);
    }
}

fn field(frame: &mut Frame, area: Rect, label: &str, value: &str, focused: bool) {
    let style = if focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let cursor = if focused { "▌" } else { "" };
    let body = Paragraph::new(Span::styled(format!("  {value}{cursor}"), style)).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" {label} ")),
    );
    frame.render_widget(body, area);
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(n.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}
