//! Step 3 widget: team bootstrap / agent detection.

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem};
use ratatui::Frame;

use crate::wizard::state::{AgentEntryStatus, WizardMode, WizardState};

pub fn render(frame: &mut Frame, area: Rect, state: &WizardState) {
    let title = match state.mode {
        WizardMode::Init => " ③ Team Setup ",
        WizardMode::Attach => " ③ Agent Detection ",
    };

    if state.team.scanning || state.team.applying {
        let label = if state.team.scanning {
            "Scanning..."
        } else {
            "Bootstrapping..."
        };
        let ratio = if state.team.agents_total > 0 {
            state.team.agents_created as f64 / state.team.agents_total as f64
        } else {
            0.0
        };
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(title))
            .gauge_style(Style::default().fg(Color::Cyan))
            .ratio(ratio)
            .label(format!(
                "{label} {}/{}",
                state.team.agents_created, state.team.agents_total
            ));
        frame.render_widget(gauge, area);
        return;
    }

    let items: Vec<ListItem> = state
        .team
        .agents_found
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let glyph = match a.status {
                AgentEntryStatus::Created | AgentEntryStatus::Detected => "✅",
                AgentEntryStatus::Skipped => "⬜",
                AgentEntryStatus::Creating => "●",
                AgentEntryStatus::Pending => "○",
            };
            let kind = match a.status {
                AgentEntryStatus::Created => "created",
                AgentEntryStatus::Detected => "detected",
                AgentEntryStatus::Skipped => "skipped",
                AgentEntryStatus::Creating => "creating...",
                AgentEntryStatus::Pending => "pending",
            };
            let style = if i == state.team.selected {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Span::styled(
                format!("  {glyph} {:<20} {kind}", a.role),
                style,
            ))
        })
        .collect();

    let mut all = items;
    if state.team.done {
        all.push(ListItem::new(""));
        all.push(ListItem::new(Span::styled(
            "  Press Enter to continue",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let list = List::new(all).block(Block::default().borders(Borders::ALL).title(title));
    frame.render_widget(list, area);
}
