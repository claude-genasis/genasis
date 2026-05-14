//! Monitor app loop — Ratatui + crossterm event-driven render.
//!
//! Collectors run on independent timers:
//! - Session detection: every 1s
//! - JSONL scan: every 60s (TTL-based cache)
//! - Plane API poll: every 30s (async, separate from render)
//! - Port probe: every 5s
//! - Render: every 250ms

use std::io;
use std::path::Path;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Terminal;

use genasis_core::config::{slugify, Config, DEFAULT_TEAM_TOKEN};
use genasis_core::error::Result;

use crate::collector;
use crate::state::{AppState, WidgetFocus};
use crate::widgets;

const RENDER_TICK: Duration = Duration::from_millis(250);
const SESSION_TICK: Duration = Duration::from_secs(1);
const JSONL_TICK: Duration = Duration::from_secs(60);
const PORT_TICK: Duration = Duration::from_secs(5);
const TRIAL_TICK: Duration = Duration::from_secs(5);
const LISTEN_LOG_TICK: Duration = Duration::from_secs(3);

pub async fn run(project_root: Option<std::path::PathBuf>) -> Result<()> {
    let mut state = AppState::default();

    // Load configuration from env
    state.limit_5h_tokens = env_u64("MONITOR_5H_TOKEN_LIMIT", 7_000_000);
    state.limit_week_all_tokens = env_u64("MONITOR_WEEK_ALL_TOKEN_LIMIT", 50_000_000);
    state.limit_week_sonnet_tokens = env_u64("MONITOR_WEEK_SONNET_TOKEN_LIMIT", 30_000_000);
    state.limit_overage_usd = env_f64("MONITOR_OVERAGE_LIMIT_USD", 200.0);

    // Load plan info from credentials
    let (plan, tier) = collector::jsonl::read_credentials();
    state.plan_name = plan;
    state.plan_tier = tier;

    // Load design state
    state.design = load_design_state();

    // D-025 + D-058: Load `genasis.toml`. project_root 가 명시적이면 그
    // 디렉터리부터 walk-up 검색 (--project flag), 없으면 cwd.
    load_trial_config(&mut state, project_root.as_deref());

    // Initial data collection
    collect_sessions(&mut state);
    collect_jsonl(&mut state);
    collect_ports(&mut state);
    if state.trial_mode {
        collect_trial(&mut state).await;
    }

    // Terminal setup. Mouse capture is intentionally NOT enabled so the
    // host terminal retains native text selection (drag, double-click,
    // triple-click). The monitor consumes only keyboard events; if a
    // future widget needs click handling, gate EnableMouseCapture
    // behind an opt-in flag rather than turning it on globally.
    enable_raw_mode().map_err(io_err)?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(io_err)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(io_err)?;

    let res = run_loop(&mut terminal, &mut state).await;

    // Cleanup
    disable_raw_mode().map_err(io_err)?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(io_err)?;
    terminal.show_cursor().map_err(io_err)?;
    res
}

async fn run_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    state: &mut AppState,
) -> Result<()> {
    let mut last_session = Instant::now();
    let mut last_jsonl = Instant::now();
    let mut last_port = Instant::now();
    let mut last_trial = Instant::now();
    let mut last_listen_log = Instant::now();

    loop {
        // Collect data on schedule
        if last_session.elapsed() >= SESSION_TICK {
            collect_sessions(state);
            last_session = Instant::now();
        }
        if last_jsonl.elapsed() >= JSONL_TICK {
            collect_jsonl(state);
            last_jsonl = Instant::now();
        }
        if last_port.elapsed() >= PORT_TICK {
            collect_ports(state);
            last_port = Instant::now();
        }
        if state.trial_mode && last_trial.elapsed() >= TRIAL_TICK {
            collect_trial(state).await;
            last_trial = Instant::now();
        }
        // D-065: tail .genasis/listen.log into log_tail widget.
        if last_listen_log.elapsed() >= LISTEN_LOG_TICK {
            if let Some(root) = state.project_root.clone() {
                collector::listen_log::poll(state, &root);
            }
            last_listen_log = Instant::now();
        }

        // Render
        terminal
            .draw(|frame| {
                let area = frame.size();

                let main = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(6), // Sprint + Tokens
                        Constraint::Length(8), // Agents
                        Constraint::Length(5), // Deploy + Network + Design
                        Constraint::Length(8), // Sessions + Usage bars
                        Constraint::Min(3),    // Log tail
                    ])
                    .split(area);

                // Row 0: Sprint (left) + Tokens (right)
                let top_row = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(main[0]);
                widgets::sprint::render(frame, top_row[0], state);
                widgets::tokens::render(frame, top_row[1], state);

                // Row 1: Agents
                widgets::agents::render(frame, main[1], state);

                // Row 2: Deploy + Network + Design
                let mid_row = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(35),
                        Constraint::Percentage(30),
                        Constraint::Percentage(35),
                    ])
                    .split(main[2]);
                widgets::deploy::render(frame, mid_row[0], state);
                widgets::network::render(frame, mid_row[1], state);
                widgets::design::render(frame, mid_row[2], state);

                // Row 3: Sessions (65%) + Usage bars (35%)
                widgets::sessions::render(frame, main[3], state);

                // Row 4: Log tail
                widgets::log_tail::render(frame, main[4], state);
            })
            .map_err(io_err)?;

        // Handle keyboard events (250ms poll timeout)
        if event::poll(RENDER_TICK).map_err(io_err)? {
            if let Event::Key(key) = event::read().map_err(io_err)? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('r') => {
                        collect_sessions(state);
                        collect_jsonl(state);
                        collect_ports(state);
                        if state.trial_mode {
                            collect_trial(state).await;
                            last_trial = Instant::now();
                        }
                    }
                    KeyCode::Char('1') => state.focus = WidgetFocus::Sprint,
                    KeyCode::Char('2') => state.focus = WidgetFocus::Tokens,
                    KeyCode::Char('3') => state.focus = WidgetFocus::Agents,
                    KeyCode::Char('4') => state.focus = WidgetFocus::Deploy,
                    KeyCode::Char('5') => state.focus = WidgetFocus::Sessions,
                    KeyCode::Char('6') => state.focus = WidgetFocus::Log,
                    KeyCode::Char('7') => state.focus = WidgetFocus::Design,
                    KeyCode::Tab => {
                        state.focus = match state.focus {
                            WidgetFocus::Sprint => WidgetFocus::Tokens,
                            WidgetFocus::Tokens => WidgetFocus::Agents,
                            WidgetFocus::Agents => WidgetFocus::Deploy,
                            WidgetFocus::Deploy => WidgetFocus::Sessions,
                            WidgetFocus::Sessions => WidgetFocus::Log,
                            WidgetFocus::Log => WidgetFocus::Design,
                            WidgetFocus::Design => WidgetFocus::Sprint,
                        };
                    }
                    _ => {}
                }
            }
        }
    }
}

fn collect_sessions(state: &mut AppState) {
    // D-084: operator wants to see "every claude on this box" — not
    // just the children of the discovered sandbox. Previously we passed
    // state.project_root, which made is_project_path_relaxed take the
    // strict-filter branch and drop vscode-server's master claude and
    // any other unrelated session. We pass an empty path so the relaxed
    // filter returns "show everything". project_root is still tracked
    // on AppState for the listen.log collector (D-065) which uses it
    // for the daemon log path, not for session filtering.
    let project_root = std::path::PathBuf::new();
    let wt_prefix = String::new();
    state.sessions = collector::sessions::detect_sessions(&project_root, &wt_prefix);

    for session in &mut state.sessions {
        if state.usage.ctx_window_size > 0 {
            session.context_pct = Some(state.usage.ctx_pct());
        }
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    state.last_session_scan = now;
}

fn collect_jsonl(state: &mut AppState) {
    state.usage = collector::jsonl::scan_sessions_dir();
    // D-073: Anthropic prompt-cache hit rate over the 5h window.
    // Hit = cache_read_input_tokens / (cache_read + cache_creation + input).
    // High percentages (typically 80-95%) signal healthy caching; near-zero
    // means caching is misconfigured or every call is a cold start.
    let usage = &state.usage;
    let denom = usage.five_h_cache_read + usage.five_h_cache_create + usage.five_h_input_tokens;
    state.anthropic_cache_hit_pct = if denom > 0 {
        (usage.five_h_cache_read as f64 / denom as f64) * 100.0
    } else {
        0.0
    };
    // D-082: MCP calls 5h + cache hits — populated from JSONL scan above.
    state.mcp_calls = usage.mcp_calls_5h;
    state.mcp_cache_hits = usage.mcp_cache_hits_5h;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    state.last_jsonl_scan = now;
}

fn collect_ports(state: &mut AppState) {
    if !state.role_ports.is_empty() {
        state.port_status = collector::ports::probe_ports(&state.role_ports);
    }
}

/// D-025: Read `genasis.toml` and populate trial-mode fields on
/// `AppState`. Best-effort — when no config exists or trial is not
/// configured, leaves `trial_mode = false` so the run-loop skips the
/// trial collector entirely.
fn load_trial_config(state: &mut AppState, project_root: Option<&Path>) {
    let start = match project_root {
        Some(p) => p.to_path_buf(),
        None => match std::env::current_dir() {
            Ok(p) => p,
            Err(_) => return,
        },
    };
    let cfg_path = match Config::discover_or_descend(&start) {
        Some(p) => {
            // D-072: walk-down 으로 발견된 경우 사용자에게 명시 — testbed
            // root 에서 monitor 를 띄웠을 때 자동으로 자식 sandbox 를 잡았다는
            // 사실을 banner 로 알림. 동작은 정상이지만 사용자가 "왜 이 디렉토리
            // 가 잡혔지?" 라고 헷갈리지 않게.
            let parent_of_cfg = p.parent().unwrap_or(&p);
            if parent_of_cfg != start {
                let hint = format!(
                    "ℹ Auto-discovered sandbox at {} (walked down from {}). Pass `--project <dir>` to pin a different one.",
                    parent_of_cfg.display(),
                    start.display()
                );
                state.log_tail.push(hint.clone());
                state.config_hint = Some(hint);
            }
            p
        }
        None => {
            // D-058 + D-072: 사용자에게 명확한 hint — silently 빈 widget 으로
            // 끝나지 않도록 log_tail 에 한 줄 남기고 state.config_hint 에도
            // 보관. log_tail widget 이 첫 줄로 surface (alert tinted).
            let hint = format!(
                "⚠ genasis.toml not found near {} (walked up and one level down). Run `genasis monitor` inside your project sandbox, or pass `--project <dir>`.",
                start.display()
            );
            state.log_tail.push(hint.clone());
            state.config_hint = Some(hint);
            return;
        }
    };
    let mut cfg = match Config::load(&cfg_path) {
        Ok(c) => c,
        Err(_) => return,
    };
    cfg.derive_naming_defaults();

    let plane_trial = cfg
        .plane
        .as_ref()
        .map(|p| p.flavor == "trial")
        .unwrap_or(false);
    let mm_trial = cfg
        .mattermost
        .as_ref()
        .map(|m| m.flavor == "trial")
        .unwrap_or(false);
    let trial_enabled = cfg.trial.as_ref().map(|t| t.enabled).unwrap_or(false);
    if !(trial_enabled && (plane_trial || mm_trial)) {
        return;
    }

    let trial_url = cfg
        .trial
        .as_ref()
        .map(|t| t.url.clone())
        .unwrap_or_default();
    let team_token = cfg.effective_team_token().to_string();
    // sim_issues 는 slugified project_slug 로 인덱싱. project_id 가 비어
    // 있는 trial 경로는 `project_name` 또는 `project.name` 을 slugify.
    let project_name_raw = cfg
        .plane
        .as_ref()
        .and_then(|p| p.project_name.clone())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| cfg.project.name.clone());
    let project_slug = cfg
        .plane
        .as_ref()
        .and_then(|p| p.project_id.clone())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| slugify(&project_name_raw));
    let scrum_channel = cfg
        .mattermost_channel("scrum")
        .map(|c| c.name.clone())
        .unwrap_or_default();

    if trial_url.is_empty() || project_slug.is_empty() || team_token == DEFAULT_TEAM_TOKEN {
        // Refuse to poll without a per-team token — would otherwise
        // surface another team's data on this monitor.
        return;
    }

    state.trial_mode = true;
    state.trial_url = trial_url;
    state.team_token = team_token;
    state.project_slug = project_slug;
    state.scrum_channel = scrum_channel;
    // D-065: remember project_root so collector::listen_log can tail
    // `<project_root>/.genasis/listen.log`.
    if let Some(parent) = cfg_path.parent() {
        state.project_root = Some(parent.to_path_buf());
    }
}

/// D-082: snapshot counts of `sim_issues` (Plane equivalent) and
/// `sim_posts` (Mattermost equivalent) seen on the trial-app since the
/// monitor started. We persist the previous totals on AppState so the
/// "Plane / MM calls" Network widget counts the delta since startup
/// — that matches the operator's mental model ("how active has the
/// team been since I opened the monitor").
fn refresh_trial_network_counters(state: &mut AppState, issues_total: u64, posts_total: u64) {
    if state.trial_baseline_issues == 0 && state.trial_baseline_posts == 0 {
        state.trial_baseline_issues = issues_total;
        state.trial_baseline_posts = posts_total;
    }
    state.plane_calls = issues_total.saturating_sub(state.trial_baseline_issues);
    state.mm_calls = posts_total.saturating_sub(state.trial_baseline_posts);
    state.network_bytes = (issues_total + posts_total) * 256; // rough display heuristic
}

/// D-025: Hit the trial-app sim endpoints once and update state.
/// Best-effort — on transport error, leaves stale data in place and
/// pushes the error onto `log_tail` so the operator can see it.
async fn collect_trial(state: &mut AppState) {
    let snap = collector::trial::poll_trial(
        &state.trial_url,
        &state.team_token,
        &state.project_slug,
        &state.scrum_channel,
    )
    .await;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    match snap {
        Ok(s) => {
            // D-082: derive Network widget counts from sim totals BEFORE
            // overwriting state.sprint. plane = total issues seen,
            // mm = total posts seen (delta from baseline).
            let issues_total =
                (s.sprint.todo + s.sprint.in_progress + s.sprint.in_review + s.sprint.done) as u64;
            let posts_total = s.posts_total as u64;
            refresh_trial_network_counters(state, issues_total, posts_total);
            state.sprint = s.sprint;
            state.agent_issues = s.agent_issues;
            // D-084: DO NOT overwrite state.log_tail wholesale here —
            // listen_log collector pushes daemon events with HH:MM
            // prefix into log_tail, and our previous `state.log_tail =
            // s.log_tail` clobbered them every 5 s with sim_posts lines
            // that have no timestamp. Now we merge: trial chat lines
            // get the same HH:MM prefix (current local clock) and are
            // appended, then we cap at 200 keeping newest entries.
            let now_hm = chrono::Local::now().format("%H:%M").to_string();
            for line in &s.log_tail {
                if !state.log_tail.iter().any(|existing| existing.ends_with(line)) {
                    state.log_tail.push(format!("{now_hm}  {line}"));
                }
            }
            if state.log_tail.len() > 200 {
                let overflow = state.log_tail.len() - 200;
                state.log_tail.drain(..overflow);
            }
            state.trial_app_kind = s.app_kind;
            state.trial_app_features = s.app_features;
            state.last_plane_poll = now;
        }
        Err(e) => {
            let now_hm = chrono::Local::now().format("%H:%M").to_string();
            state.log_tail.push(format!("{now_hm}  (trial poll error: {e})"));
            if state.log_tail.len() > 200 {
                let overflow = state.log_tail.len() - 200;
                state.log_tail.drain(..overflow);
            }
        }
    }
}

fn load_design_state() -> crate::state::DesignWidgetState {
    let path = std::path::Path::new("docs/.design-state.toml");
    if !path.exists() {
        return crate::state::DesignWidgetState::default();
    }
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return crate::state::DesignWidgetState::default(),
    };
    let get = |key: &str| -> String {
        content
            .lines()
            .find(|l| l.starts_with(key))
            .and_then(|l| l.split('=').nth(1))
            .map(|v| v.trim().trim_matches('"').to_string())
            .unwrap_or_default()
    };
    crate::state::DesignWidgetState {
        mode: get("mode"),
        slug: get("slug"),
        applied_at: get("applied_at"),
        override_count: get("override_count").parse().unwrap_or(0),
        preview_url: get("gallery_preview"),
        gallery_url: String::new(),
    }
}

fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn env_f64(key: &str, default: f64) -> f64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn io_err(e: io::Error) -> genasis_core::error::Error {
    genasis_core::error::Error::Io(e)
}
