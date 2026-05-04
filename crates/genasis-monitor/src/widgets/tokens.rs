//! Tokens widget — RTK savings + MCP / cache stats.

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use genasis_i18n::tr;

use crate::state::AppState;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let body = format!(
        "RTK saved: {} tokens\nMCP calls: {}, cache hit {:.0}%\nAnthropic cache HIT: {:.0}%",
        format_int(state.rtk_saved_tokens),
        format_int(state.mcp_calls),
        cache_pct(state.mcp_calls, state.mcp_cache_hits),
        state.anthropic_cache_hit_pct,
    );
    let title = format!(" {} (2) ", tr("monitor.widget.tokens"));
    let p = Paragraph::new(body)
        .block(Block::default().borders(Borders::ALL).title(title))
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(p, area);
}

fn format_int(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}

fn cache_pct(calls: u64, hits: u64) -> f64 {
    if calls == 0 {
        0.0
    } else {
        (hits as f64) / (calls as f64) * 100.0
    }
}
