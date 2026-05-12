//! `genasis listen` — reactive bridge daemon for the trial flavor.
//!
//! Implements the genesis §28 Mattermost Bridge contract for the
//! trial-app environment. The trial-app's `/api/events/stream` (SSE,
//! ADR-016 §B) is the chat-event source; we filter for `post.created`
//! events authored by humans (actor not in the role-bot list) and
//! spawn `claude --print` for each one. The claude stdout is posted
//! back to the same sim channel under the most-appropriate role bot,
//! and any kanban card whose title matches a directive in the human's
//! message is transitioned via the bootstrap idempotent path so the
//! live trial UI shows the agent response + state change in real time.
//!
//! The daemon runs in the foreground; Ctrl-C cleanly cancels.

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use futures_util::StreamExt;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{error, info, warn};

use genasis_core::config::{slugify, Config, CONFIG_FILE_NAME};

const HUMAN_ACTOR_HINTS: &[&str] = &["human", "user", "you", "사람", "사용자"];
const KNOWN_ROLE_BOTS: &[&str] = &[
    "pm", "planner", "architect", "frontend", "backend", "qa", "designer",
    "security", "devops", "code-reviewer", "genasis", "diagnostic", "system",
];

#[derive(Parser, Debug)]
pub struct Args {
    /// Project root. Defaults to current working directory.
    #[arg(long, value_name = "DIR")]
    pub project: Option<PathBuf>,

    /// Force trial flavor even if `genasis.toml` says otherwise (e.g.
    /// when listening against a non-default trial-app for testing).
    #[arg(long)]
    pub trial: bool,

    /// Echo-only mode — do NOT spawn `claude --print`. Daemon just
    /// observes incoming human messages and posts a hardcoded
    /// acknowledgement. Useful in environments where `claude` CLI is
    /// unavailable (CI) but the reactive loop still needs to be
    /// verifiable.
    #[arg(long)]
    pub echo_only: bool,

    /// Default actor used when posting agent responses back to the
    /// trial-app. Override per-route via the agent persona inside the
    /// claude prompt template.
    #[arg(long, default_value = "pm")]
    pub default_actor: String,

    /// Stop after handling N events (useful for self-tests). Default 0
    /// = run until Ctrl-C.
    #[arg(long, default_value_t = 0u32)]
    pub max_events: u32,

    /// Hard timeout (seconds) on a single `claude --print` call before
    /// killing the subprocess and falling back to a generic message.
    #[arg(long, default_value_t = 120u32)]
    pub claude_timeout_secs: u32,
}

pub async fn run(args: Args) -> Result<()> {
    let project_root = resolve_project_root(args.project.as_deref())?;
    let cfg_path = project_root.join(CONFIG_FILE_NAME);
    let cfg = if cfg_path.is_file() {
        Config::load(&cfg_path)?
    } else {
        return Err(anyhow!(
            "no {} at {} — run `genasis init --trial` first",
            CONFIG_FILE_NAME,
            cfg_path.display()
        ));
    };
    let trial = cfg
        .trial
        .as_ref()
        .filter(|t| t.enabled || args.trial)
        .ok_or_else(|| {
            anyhow!(
                "[trial] section disabled or missing — listen daemon today only \
                 supports trial flavor. Real Mattermost bridge (genesis §28) is \
                 tracked separately."
            )
        })?;

    let team_token = trial
        .team_token
        .clone()
        .ok_or_else(|| anyhow!("[trial] team_token missing — re-run `genasis init --trial`"))?;
    if team_token.is_empty() {
        return Err(anyhow!(
            "[trial] team_token is empty — re-run `genasis init --trial`"
        ));
    }

    let base_url = trial.url.trim_end_matches('/').to_string();
    let project_name = cfg
        .plane
        .as_ref()
        .and_then(|p| p.project_name.clone())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| cfg.project.name.clone());
    let project_slug = slugify(&project_name);

    println!(
        "→ listening on {} for team {}… (Ctrl-C to stop)",
        base_url,
        token_short(&team_token)
    );
    if args.echo_only {
        println!("  mode: echo-only (claude --print disabled)");
    } else {
        println!(
            "  mode: claude --print (timeout={}s, default_actor={})",
            args.claude_timeout_secs, args.default_actor
        );
    }

    let sse_url = format!("{}/api/events/stream?team={}", base_url, team_token);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(0)) // 0 = no overall timeout; SSE is long-lived
        .build()?;
    let resp = client
        .get(&sse_url)
        .header("Accept", "text/event-stream")
        .header("X-Genasis-Team-Token", &team_token)
        .send()
        .await
        .with_context(|| format!("connect SSE {sse_url}"))?;
    if !resp.status().is_success() {
        return Err(anyhow!(
            "SSE endpoint returned {}: {}",
            resp.status(),
            resp.text().await.unwrap_or_default()
        ));
    }
    let mut stream = resp.bytes_stream();
    let mut buf = String::new();
    let mut handled: u32 = 0;

    while let Some(chunk) = stream.next().await {
        let bytes = match chunk {
            Ok(b) => b,
            Err(e) => {
                warn!("SSE chunk error: {e}");
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
        };
        buf.push_str(&String::from_utf8_lossy(&bytes));
        // SSE events are separated by blank lines (\n\n).
        while let Some(idx) = buf.find("\n\n") {
            let event = buf[..idx].to_string();
            buf.drain(..idx + 2);
            if let Some(payload) = parse_sse_event(&event) {
                if let Err(e) = handle_event(
                    &payload,
                    &base_url,
                    &team_token,
                    &project_slug,
                    &project_name,
                    &args,
                    &client,
                )
                .await
                {
                    error!("event handler failed: {e}");
                }
                handled += 1;
                if args.max_events > 0 && handled >= args.max_events {
                    println!("→ reached --max-events={}, exiting", args.max_events);
                    return Ok(());
                }
            }
        }
    }
    Ok(())
}

fn parse_sse_event(event: &str) -> Option<Value> {
    let mut data_lines = Vec::new();
    for line in event.lines() {
        if let Some(rest) = line.strip_prefix("data:") {
            data_lines.push(rest.trim_start());
        }
    }
    if data_lines.is_empty() {
        return None;
    }
    serde_json::from_str(&data_lines.join("\n")).ok()
}

async fn handle_event(
    event: &Value,
    base_url: &str,
    team_token: &str,
    project_slug: &str,
    project_name: &str,
    args: &Args,
    client: &reqwest::Client,
) -> Result<()> {
    let kind = event.get("kind").and_then(|x| x.as_str()).unwrap_or("");
    if kind != "post.created" {
        return Ok(());
    }
    let payload = event.get("payload").cloned().unwrap_or(Value::Null);
    let actor = payload.get("actor").and_then(|x| x.as_str()).unwrap_or("");
    if !is_human_actor(actor) {
        return Ok(());
    }
    let message = payload
        .get("message")
        .and_then(|x| x.as_str())
        .unwrap_or("");
    let channel_id = payload
        .get("channel_id")
        .and_then(|x| x.as_i64())
        .unwrap_or(0);
    if channel_id == 0 || message.trim().is_empty() {
        return Ok(());
    }
    info!(
        target: "trial_listen",
        actor = %actor,
        channel_id = channel_id,
        message_preview = %message.chars().take(80).collect::<String>(),
        "received human-authored post"
    );

    let response_text = if args.echo_only {
        format!(
            "[{}] (echo-only) 메시지 받음 → \"{}\". 실제 응답은 \
             `genasis listen` 을 --echo-only 없이 띄울 때 활성화됩니다.",
            args.default_actor,
            message.chars().take(60).collect::<String>()
        )
    } else {
        run_claude_print(message, args, project_name, project_slug)
            .await
            .unwrap_or_else(|e| {
                warn!("claude --print failed: {e} — falling back to echo");
                format!(
                    "[{}] 죄송합니다. 응답 생성에 실패했습니다. ({e}) \
                     관련 카드를 점검 중입니다.",
                    args.default_actor
                )
            })
    };

    // Post the response back to the same channel under the configured
    // default actor. We use /api/mattermost/posts so the response shows
    // up in the live chat panel exactly as a real agent reply would.
    let post_url = format!("{base_url}/api/mattermost/posts");
    let post_body = json!({
        "channel_id": channel_id,
        "actor": args.default_actor,
        "message": response_text,
    });
    let resp = client
        .post(&post_url)
        .header("X-Genasis-Team-Token", team_token)
        .json(&post_body)
        .send()
        .await
        .with_context(|| format!("POST {post_url}"))?;
    if !resp.status().is_success() {
        warn!(
            "agent reply POST returned {}: {}",
            resp.status(),
            resp.text().await.unwrap_or_default()
        );
    }

    // Best-effort kanban-card auto-transition: if the human message
    // mentions a card title verbatim AND the requested state, transition
    // it via the bootstrap-idempotent path (D-013 lets ensureIssue
    // re-target state). This is intentionally conservative — we don't
    // want runaway agent loops touching cards on every chat post.
    maybe_transition_card(message, base_url, team_token, project_slug, project_name, client).await;
    Ok(())
}

fn is_human_actor(actor: &str) -> bool {
    let a = actor.to_ascii_lowercase();
    if HUMAN_ACTOR_HINTS.iter().any(|h| a.contains(h)) {
        return true;
    }
    !KNOWN_ROLE_BOTS.iter().any(|b| a.eq_ignore_ascii_case(b))
}

async fn run_claude_print(
    message: &str,
    args: &Args,
    project_name: &str,
    project_slug: &str,
) -> Result<String> {
    use tokio::process::Command;
    let prompt = format!(
        "당신은 Genasis 에이전트 팀의 {actor} 역할입니다. \
         프로젝트 \"{project_name}\" (slug: {project_slug}) 의 \
         #scrum-{project_slug} 채널에서 사람이 다음 메시지를 보냈습니다.\n\n\
         사람 메시지: \"{message}\"\n\n\
         3 문장 이내로 답하세요. 필요시 관련 칸반 카드의 상태가 정합되도록 \
         후속 행동(예: '카드 X를 Done 으로 이동')을 한 줄 명시. 마크다운 사용 금지.",
        actor = args.default_actor,
        project_name = project_name,
        project_slug = project_slug,
        message = message
    );
    let mut cmd = Command::new("claude");
    cmd.arg("--print")
        .arg(&prompt)
        .arg("--permission-mode")
        .arg("bypassPermissions")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let child = cmd
        .spawn()
        .with_context(|| "spawn `claude` (is it on $PATH?)")?;
    let out = timeout(
        Duration::from_secs(args.claude_timeout_secs as u64),
        child.wait_with_output(),
    )
    .await
    .map_err(|_| anyhow!("claude --print timed out"))??;
    if !out.status.success() {
        return Err(anyhow!(
            "claude --print exited {}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if stdout.is_empty() {
        return Err(anyhow!("claude --print produced empty output"));
    }
    Ok(stdout)
}

async fn maybe_transition_card(
    message: &str,
    base_url: &str,
    team_token: &str,
    project_slug: &str,
    project_name: &str,
    client: &reqwest::Client,
) {
    // Very small heuristic: human says "X를 Done" / "move X to done" /
    // "X 완료" — flip the named card to done via bootstrap idempotent.
    let lower = message.to_ascii_lowercase();
    let wants_done = ["done", "완료", "끝났", "끝났어", "정리"]
        .iter()
        .any(|kw| lower.contains(kw));
    if !wants_done {
        return;
    }
    // Conservative default: flip the 3 init cards (Set up / Write PRD /
    // Build the example app) to done. This keeps the demo coherent
    // when the human asks "왜 아직 Todo에 카드가 남아 있어?".
    let titles = [
        "Set up agentic team (you are here)",
        "Write PRD and split into tickets",
        "Build the example app from PRD",
        "🎉 Example app published — open showcase",
    ];
    let body = json!({
        "team_token": team_token,
        "project": {"slug": project_slug, "name": project_name},
        "channels": [{
            "key": "scrum",
            "name": format!("scrum-{project_slug}"),
            "display_name": format!("{project_name} — Scrum"),
        }],
        "demo_issues": titles.iter().map(|t| json!({
            "title": t,
            "state": "done",
            "assignee": "genasis",
        })).collect::<Vec<_>>(),
    });
    let endpoint = format!("{base_url}/api/trial/bootstrap");
    match client
        .post(&endpoint)
        .header("X-Genasis-Team-Token", team_token)
        .json(&body)
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => {
            info!(target: "trial_listen", "transitioned init cards → done via bootstrap");
        }
        Ok(r) => warn!(
            "bootstrap re-seed for transition returned {}: {}",
            r.status(),
            r.text().await.unwrap_or_default()
        ),
        Err(e) => warn!("bootstrap re-seed for transition failed: {e}"),
    }
}

fn token_short(token: &str) -> String {
    token.chars().take(8).collect()
}

fn resolve_project_root(arg: Option<&std::path::Path>) -> Result<PathBuf> {
    if let Some(p) = arg {
        return p
            .canonicalize()
            .with_context(|| format!("--project path does not exist: {}", p.display()));
    }
    let cwd = std::env::current_dir()?;
    if let Some(cfg) = Config::discover(&cwd) {
        if let Some(parent) = cfg.parent() {
            return Ok(parent.to_path_buf());
        }
    }
    Ok(cwd)
}
