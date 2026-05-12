//! Sprint widget — shows current Cycle and issue counts.

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use genasis_i18n::tr;

use crate::state::AppState;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let dday = state
        .sprint
        .d_day
        .map(|d| format!("D-{d}"))
        .unwrap_or_else(|| "—".into());

    // D-025: When the monitor is wired to a trial-app, surface the
    // current showcase (`app_kind` + `app_features`) in the header so
    // the operator can confirm at a glance that PM routing landed.
    let header = if state.trial_mode {
        let app = if state.trial_app_kind.is_empty() {
            "(no showcase yet)".to_string()
        } else if state.trial_app_features.is_empty() {
            format!("[{}]", state.trial_app_kind)
        } else {
            format!(
                "[{} · {}]",
                state.trial_app_kind,
                state.trial_app_features.join(",")
            )
        };
        format!("Cycle: {} {app}", state.sprint.name)
    } else {
        format!("Cycle: {}", state.sprint.name)
    };

    let body = format!(
        "{header}\nD-day: {}\nTodo:{}  In:{}  Review:{}  Done:{}",
        dday,
        state.sprint.todo,
        state.sprint.in_progress,
        state.sprint.in_review,
        state.sprint.done
    );
    let title = format!(" {} (1) ", tr("monitor.widget.sprint"));
    let p = Paragraph::new(body)
        .block(Block::default().borders(Borders::ALL).title(title))
        .style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(p, area);
}
