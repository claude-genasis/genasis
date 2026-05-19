//! Claude Code sessions + usage widget.
//!
//! Left 2/3: active session list (PID, role, ctx%, age, state)
//! Right 1/3: usage bar charts (5h, 7d all, 7d Sonnet/Opus, cost, reset)
//!
//! D-130: extended layout to surface Opus + Sonnet as separate rows
//! (Anthropic Max plans track them independently) and to render an
//! empty-state hint when no assistant activity exists in the 5h window
//! — the pre-D-130 layout silently rendered all-zero gauges in that
//! case, which looked broken on first-time / clean-env installs.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};
use ratatui::Frame;

use crate::collector::sessions::{format_age, format_countdown, SessionState};
use crate::state::AppState;

/// Render the sessions + usage panel.
pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(area);

    render_sessions_list(frame, chunks[0], state);
    render_usage_bars(frame, chunks[1], state);
}

fn render_sessions_list(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default()
        .title(" SESSIONS ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if state.sessions.is_empty() {
        let msg = Paragraph::new("  No active Claude Code sessions detected.");
        frame.render_widget(msg, inner);
        return;
    }

    let mut lines: Vec<Line> = Vec::new();

    // Header
    lines.push(Line::from(vec![Span::styled(
        format!(
            "{:<7} {:<12} {:>5} {:>6}  {}",
            "PID", "role", "ctx%", "age", "state"
        ),
        Style::default().fg(Color::DarkGray),
    )]));

    for session in &state.sessions {
        let state_color = match session.state {
            SessionState::Active => Color::Green,
            SessionState::Idle => Color::DarkGray,
            SessionState::Error => Color::Red,
        };
        let state_icon = match session.state {
            SessionState::Active => "●",
            SessionState::Idle => "○",
            SessionState::Error => "✗",
        };
        let role = session.role.as_deref().unwrap_or("-");
        let ctx = session
            .context_pct
            .map(|p| format!("{:.0}%", p))
            .unwrap_or_else(|| "-".into());
        let age = format_age(session.age_secs);

        lines.push(Line::from(vec![
            Span::raw(format!("{:<7} ", session.pid)),
            Span::styled(format!("{:<12} ", role), Style::default().fg(Color::Cyan)),
            Span::raw(format!("{:>5} ", ctx)),
            Span::raw(format!("{:>6}  ", age)),
            Span::styled(
                format!(
                    "{} {}",
                    state_icon,
                    format!("{:?}", session.state).to_lowercase()
                ),
                Style::default().fg(state_color),
            ),
        ]));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn render_usage_bars(frame: &mut Frame, area: Rect, state: &AppState) {
    let title = if state.plan_name.is_empty() || state.plan_name == "unknown" {
        " CLAUDE USAGE ".to_string()
    } else {
        format!(" CLAUDE USAGE — {} ", state.plan_name)
    };
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // D-130: empty-state hint — only show it when BOTH the JSONL window
    // is empty AND the OAuth fetch has not landed authoritative numbers.
    // D-131: server-reported utilization can be non-zero even when the
    // local JSONL has nothing in the 5h window (e.g. fresh test bed with
    // no projects/ entries but the user has used Claude elsewhere), so
    // skip the hint when we have live data.
    if state.usage.is_empty_5h() && state.usage.oauth_fetched_at == 0 {
        let lines = vec![
            Line::from(vec![Span::styled(
                "  No Claude activity in last 5h.",
                Style::default().fg(Color::DarkGray),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "  Run `claude` in any project to populate.",
                Style::default().fg(Color::DarkGray),
            )]),
        ];
        let hint = Paragraph::new(lines);
        frame.render_widget(hint, inner);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // 5h session
            Constraint::Length(1), // 7d all
            Constraint::Length(1), // 7d Opus / Design
            Constraint::Length(1), // 7d Sonnet
            Constraint::Length(1), // cost
            Constraint::Length(1), // reset
            Constraint::Min(0),
        ])
        .split(inner);

    // D-131: source-of-truth selector — prefer server values when an
    // OAuth fetch has landed. JSONL estimates remain the fallback so a
    // disabled / offline path still shows a sensible bar. `oauth_*` is
    // a percentage 0–100; JSONL helpers also return 0–100, so the
    // formatting branch is identical.
    let from_oauth = state.usage.oauth_fetched_at > 0;
    let src_tag = if from_oauth { "live" } else { "local" };

    // 5h session usage
    let five_pct = state
        .usage
        .oauth_five_h_pct
        .unwrap_or_else(|| state.usage.five_h_pct(state.limit_5h_tokens));
    let five_color = pct_color(five_pct);
    let five_label = format!("5h session  {:.0}%", five_pct);
    let five_gauge = Gauge::default()
        .gauge_style(Style::default().fg(five_color))
        .ratio((five_pct as f64 / 100.0).min(1.0))
        .label(five_label);
    frame.render_widget(five_gauge, chunks[0]);

    // 7d all models
    let week_pct = state
        .usage
        .oauth_seven_day_pct
        .unwrap_or_else(|| state.usage.week_all_pct(state.limit_week_all_tokens));
    let week_color = pct_color(week_pct);
    let week_label = format!("7d (all)    {:.0}%", week_pct);
    let week_gauge = Gauge::default()
        .gauge_style(Style::default().fg(week_color))
        .ratio((week_pct as f64 / 100.0).min(1.0))
        .label(week_label);
    frame.render_widget(week_gauge, chunks[1]);

    // D-131: when the OAuth response declares a `seven_day_opus` track
    // we show that; otherwise we fall back to "Claude Design"
    // (`seven_day_omelette`) which Max plans surface in place of a
    // separate Opus bar. Final fallback: the JSONL Opus aggregate from
    // D-130 (still useful when no OAuth fetch has landed).
    let (third_label, third_pct, third_color) =
        if let Some(pct) = state.usage.oauth_seven_day_opus_pct {
            ("7d (Opus)  ", pct, Color::Magenta)
        } else if let Some(pct) = state.usage.oauth_seven_day_design_pct {
            ("7d (Design)", pct, Color::Cyan)
        } else {
            let opus_budget = if state.limit_week_opus_tokens > 0 {
                state.limit_week_opus_tokens
            } else {
                state.limit_week_all_tokens
            };
            (
                "7d (Opus)  ",
                state.usage.week_opus_pct(opus_budget),
                Color::Magenta,
            )
        };
    let third_gauge = Gauge::default()
        .gauge_style(Style::default().fg(third_color))
        .ratio((third_pct as f64 / 100.0).min(1.0))
        .label(format!("{third_label} {:.0}%", third_pct));
    frame.render_widget(third_gauge, chunks[2]);

    // 7d Sonnet
    let sonnet_pct = state
        .usage
        .oauth_seven_day_sonnet_pct
        .unwrap_or_else(|| state.usage.week_sonnet_pct(state.limit_week_sonnet_tokens));
    let sonnet_label = format!("7d (Sonnet) {:.0}%", sonnet_pct);
    let sonnet_gauge = Gauge::default()
        .gauge_style(Style::default().fg(Color::Blue))
        .ratio((sonnet_pct as f64 / 100.0).min(1.0))
        .label(sonnet_label);
    frame.render_widget(sonnet_gauge, chunks[3]);

    // D-131: cost line. When the OAuth response carries
    // `extra_usage.used_credits` we show that (matches the "사용 크레딧"
    // number on the Anthropic settings page exactly); otherwise fall
    // back to the JSONL-derived estimate from D-130. `monthly_limit` is
    // also in cents from the server, so display divided by 100.
    let cost_text = if let (Some(used_cents), Some(cap_cents)) = (
        state.usage.oauth_extra_used_credits_cents,
        state.usage.oauth_extra_monthly_limit_cents,
    ) {
        format!(
            " Credits ${:.2} / ${:.0}  ·  est 5h ${:.2}",
            used_cents / 100.0,
            cap_cents / 100.0,
            state.usage.five_h_cost_usd,
        )
    } else {
        format!(
            " 5h ${:.2}  ·  7d ${:.2}  ·  cap ${:.0}",
            state.usage.five_h_cost_usd, state.usage.week_cost_usd, state.limit_overage_usd,
        )
    };
    let cost_line = Paragraph::new(cost_text).style(Style::default().fg(Color::Yellow));
    frame.render_widget(cost_line, chunks[4]);

    // D-131: countdown — prefer the server's `resets_at` when present;
    // fall back to the JSONL sliding-window estimate from D-130.
    let countdown_secs = state
        .usage
        .five_h_oauth_countdown()
        .unwrap_or_else(|| state.usage.five_h_reset_countdown());
    let countdown = if countdown_secs <= 0 {
        "—".to_string()
    } else {
        format_countdown(countdown_secs)
    };
    // D-131: tag tells the user whether the gauges came from the live
    // server endpoint or the local JSONL estimate. "live" matches the
    // numbers on the Anthropic settings page; "local" is the heuristic.
    let reset_text = format!(" Reset (5h): {}  ·  source: {}", countdown, src_tag);
    let reset_line = Paragraph::new(reset_text).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(reset_line, chunks[5]);
}

fn pct_color(pct: f32) -> Color {
    if pct >= 85.0 {
        Color::Red
    } else if pct >= 60.0 {
        Color::Yellow
    } else {
        Color::Green
    }
}
