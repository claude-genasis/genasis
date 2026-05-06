//! Agents widget — last activity and current ticket per role.

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

use genasis_i18n::tr;

use crate::collector::plane::IssueState;
use crate::state::AppState;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let items: Vec<ListItem> = if state.agent_issues.is_empty() {
        vec![ListItem::new(
            "(no agent activity collected yet — wire SessionStart hook)",
        )]
    } else {
        state
            .agent_issues
            .iter()
            .map(|a| {
                let dot = match a.state {
                    IssueState::InProgress => "●",
                    IssueState::InReview => "◐",
                    IssueState::Todo | IssueState::Done => "◌",
                };
                ListItem::new(format!(
                    "{dot} {:<10} {}  {}",
                    a.role, a.issue_id, a.issue_title,
                ))
            })
            .collect()
    };

    let title = format!(" {} (3) ", tr("monitor.widget.agents"));
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .style(Style::default().fg(Color::White));
    frame.render_widget(list, area);
}
