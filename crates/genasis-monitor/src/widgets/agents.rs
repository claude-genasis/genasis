//! Agents widget — last activity and current ticket per role.

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

use crate::state::{AgentStatus, AppState};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let items: Vec<ListItem> = if state.agents.is_empty() {
        vec![ListItem::new(
            "(no agent activity collected yet — wire SessionStart hook)",
        )]
    } else {
        state
            .agents
            .iter()
            .map(|a| {
                let dot = match a.status {
                    AgentStatus::Working => "●",
                    AgentStatus::InReview => "◐",
                    AgentStatus::Idle => "◌",
                };
                ListItem::new(format!(
                    "{dot} {:<10} last={}s ago  {}",
                    a.role,
                    a.last_active_secs_ago,
                    a.current_issue.as_deref().unwrap_or("idle")
                ))
            })
            .collect()
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Agents (3) "))
        .style(Style::default().fg(Color::White));
    frame.render_widget(list, area);
}
