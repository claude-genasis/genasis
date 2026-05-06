//! Step 6 widget: final summary + smoke test + rollback.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::wizard::state::{SmokeTestStatus, WizardState};

pub fn render(frame: &mut Frame, area: Rect, state: &WizardState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(10)])
        .split(area);

    render_summary(frame, chunks[0], state);
    render_smoke_test(frame, chunks[1], state);
}

fn render_summary(frame: &mut Frame, area: Rect, state: &WizardState) {
    let mut items = vec![
        ListItem::new(""),
        ListItem::new(Span::styled(
            "  ✅ Genasis Setup Complete!",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        ListItem::new(""),
    ];

    // Show tab summaries as the configuration summary.
    for step in crate::wizard::step::WizardStep::ALL.iter().take(5) {
        let meta = &state.steps[step.index()];
        let label = step.label(state.mode);
        items.push(ListItem::new(format!(
            "  {} {:<12} {}",
            meta.status.glyph(),
            label,
            meta.summary
        )));
    }

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(" ⑥ Summary "));
    frame.render_widget(list, area);
}

fn render_smoke_test(frame: &mut Frame, area: Rect, state: &WizardState) {
    let btn_style = |idx: usize| {
        if state.done.button_focus == idx {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        }
    };

    let status_line = match state.done.smoke_status {
        SmokeTestStatus::NotRun => "  Run a smoke test to verify the team works end-to-end?",
        SmokeTestStatus::Running => "  ● Running smoke test...",
        SmokeTestStatus::Passed => "  ✅ Smoke test passed!",
        SmokeTestStatus::Failed => "  ❌ Smoke test failed.",
    };

    let mut items = vec![ListItem::new(status_line), ListItem::new("")];

    // Show smoke output if any.
    for line in state.done.smoke_output.iter().rev().take(3).rev() {
        items.push(ListItem::new(format!("    {line}")));
    }

    items.push(ListItem::new(""));
    items.push(ListItem::new(Span::styled(
        "  [▶ Run Smoke Test]",
        btn_style(0),
    )));
    items.push(ListItem::new(Span::styled("  [Skip]", btn_style(1))));

    if state.done.rollback_available {
        items.push(ListItem::new(Span::styled("  [Rollback]", btn_style(2))));
    }

    items.push(ListItem::new(Span::styled(
        "  [Open Monitor]",
        btn_style(3),
    )));

    let block = Block::default().borders(Borders::ALL).title(" Smoke Test ");
    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}
