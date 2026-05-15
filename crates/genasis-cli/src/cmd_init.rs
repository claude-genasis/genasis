use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use genasis_core::config::{
    random_team_token, slugify, Config, CONFIG_FILE_NAME, DEFAULT_TEAM_TOKEN,
};
use genasis_i18n::{tr, tr_args};
use genasis_providers::{mattermost, plane};

#[derive(Parser, Debug)]
pub struct Args {
    /// Project root. Defaults to the current working directory.
    #[arg(long, value_name = "DIR")]
    pub project: Option<PathBuf>,

    /// Stop after pinging Plane / Mattermost — do not provision anything.
    #[arg(long)]
    pub probe_only: bool,

    /// Bootstrap in trial mode: skip real Plane/MM provisioning, write a
    /// minimal `genasis.toml` with `[trial]` enabled, and offer to open
    /// the operator-hosted trial demo at https://mmplane-trial.realstory.blog.
    #[arg(long)]
    pub trial: bool,

    /// Human-readable project name to write into `[project].name` and
    /// derive the trial-mode `team_token`-scoped Plane/MM identifiers
    /// from (ADR-016). When omitted in `--trial` mode, derived from the
    /// project directory name (or "Trial Demo" when that fails).
    /// Ignored outside `--trial` mode.
    #[arg(long, value_name = "NAME")]
    pub name: Option<String>,

    /// Alias for `genasis bootstrap` — scaffold canonical 10-role base
    /// agent files when `.claude/agents/` is empty, then auto-chain into
    /// `attach`. ADR-010 §3 decision (b)+(d).
    #[arg(long)]
    pub bootstrap: bool,

    /// Comma-separated subset of roles for `--bootstrap`. Forwarded to
    /// `cmd_bootstrap`.
    #[arg(long, value_name = "LIST")]
    pub roles: Option<String>,

    /// Skip the auto-publish + auto-daemon-start tail of
    /// `init --trial` / `init` (alpha.36 UX simplification). Useful
    /// when you want the bootstrap output but plan to run `publish`
    /// / `listen start` yourself, or for CI / scripting.
    #[arg(long)]
    pub no_auto_start: bool,
}

pub async fn run(args: Args) -> Result<()> {
    run_with_globals(args, None, false, false).await
}

pub async fn run_with_globals(
    args: Args,
    lang_flag: Option<String>,
    non_interactive: bool,
    assume_yes: bool,
) -> Result<()> {
    if args.bootstrap {
        let bootstrap_args = crate::cmd_bootstrap::Args {
            project: args.project.clone(),
            roles: args.roles.clone(),
            no_attach_after: false,
            dry_run: false,
        };
        return crate::cmd_bootstrap::run(bootstrap_args, lang_flag, non_interactive, assume_yes)
            .await;
    }
    if args.trial {
        return run_trial(args, lang_flag, non_interactive, assume_yes).await;
    }
    let project_root = resolve_project_root(args.project.as_deref())?;
    let cfg_path = project_root.join(CONFIG_FILE_NAME);
    let mut cfg = if cfg_path.is_file() {
        Config::load(&cfg_path)?
    } else {
        anyhow::bail!(
            "no genasis.toml at {} — copy templates/genasis.toml.tera and fill it in",
            cfg_path.display()
        );
    };
    // ADR-016: legacy configs (pre-channel-list) get a synthesised
    // single "scrum" channel + a project_name fallback so the channel
    // lookup below never silently runs on empty data.
    cfg.derive_naming_defaults();

    let plane_cfg = cfg.plane.as_ref().context("[plane] section missing")?;
    let mm_cfg = cfg
        .mattermost
        .as_ref()
        .context("[mattermost] section missing")?;

    let plane_flavor = plane::FlavorChoice::parse(&plane_cfg.flavor)?;
    let mm_flavor = mattermost::FlavorChoice::parse(&mm_cfg.flavor)?;

    // Trial flavor pulls URL/secret from [trial]; env vars are only
    // required for real backends. This lets `genasis init` (without
    // `--trial`) still work on a project whose config is already wired
    // for trial mode — useful for re-running provisioning steps.
    let plane_token = if plane_flavor == plane::FlavorChoice::Trial {
        String::new()
    } else {
        std::env::var("PLANE_API_KEY")
            .or_else(|_| std::env::var("PLANE_TOKEN_PM"))
            .context("PLANE_API_KEY (or PLANE_TOKEN_PM) not set in environment")?
    };
    let mm_token = if mm_flavor == mattermost::FlavorChoice::Trial {
        String::new()
    } else {
        std::env::var("MM_ADMIN_TOKEN").context("MM_ADMIN_TOKEN not set in environment")?
    };

    println!(
        "{}",
        tr_args("init.resolving_plane", &[("flavor", &plane_cfg.flavor)])
    );
    let plane_client = plane::build(
        plane_flavor,
        &plane_cfg.url,
        &plane_cfg.workspace_slug,
        &plane_token,
        cfg.trial.as_ref(),
    )
    .await?;
    let plane_health = plane_client.health().await?;
    println!("  plane health: {}", short_json(&plane_health));

    println!(
        "{}",
        tr_args("init.resolving_mm", &[("flavor", &mm_cfg.flavor)])
    );
    let mm_client =
        mattermost::build(mm_flavor, &mm_cfg.url, &mm_token, cfg.trial.as_ref()).await?;
    let mm_ping = mm_client.ping().await?;
    println!("  mattermost ping: {}", short_json(&mm_ping));

    if args.probe_only {
        println!("\n{}", tr("init.probe_only_skip"));
        return Ok(());
    }

    println!(
        "\n{}",
        tr_args("init.ensure_project", &[("name", &cfg.project.name)])
    );
    let project_id = plane_client
        .ensure_project(&cfg.project.name, &slug_to_identifier(&cfg.project.name))
        .await?;
    println!("  plane project_id = {project_id}");

    let labels = [
        ("BUG", "#FF0000"),
        ("FEATURE", "#22C55E"),
        ("IMPROVEMENT", "#3B82F6"),
        ("QUESTION", "#EAB308"),
    ];
    for (name, color) in labels {
        let l = plane_client.ensure_label(&project_id, name, color).await?;
        println!("  plane label {name} → {}", l.id);
    }

    // ADR-016 §1: read the structured channel list from
    // [mattermost].channels rather than string-formatting from
    // project.name. derive_naming_defaults ensures a "scrum" entry
    // exists even on legacy configs.
    let scrum = cfg
        .mattermost_channel("scrum")
        .context("[mattermost].channels is missing a key=\"scrum\" entry")?;
    println!(
        "\n{}",
        tr_args("init.ensure_channel", &[("channel", &scrum.name)])
    );
    // v0.5.2 (Issue #9): if MM_TEAM_ID isn't set, try to auto-resolve
    // it from `[mattermost].team_name` via Mattermost's REST endpoint
    // GET /api/v4/teams/name/{name} (returns the team's id). This
    // closes a gap where the README + help text only document
    // MM_ADMIN_TOKEN, leaving users to discover MM_TEAM_ID by trial
    // and error. Falls through to the old "skipped" message when
    // we can't resolve (network failure, team doesn't exist yet,
    // trial flavor with no admin token).
    let mut team_id = std::env::var("MM_TEAM_ID").unwrap_or_default();
    if team_id.is_empty() && !mm_token.is_empty() {
        if let Some(resolved) =
            lookup_mm_team_id_by_name(&mm_cfg.url, &mm_cfg.team_name, &mm_token).await
        {
            println!("  resolved MM_TEAM_ID = {resolved} (from team_name)");
            team_id = resolved;
        }
    }
    if !team_id.is_empty() {
        let ch = mm_client
            .ensure_channel(&team_id, &scrum.name, &scrum.display_name)
            .await?;
        println!("  mm channel = {} ({})", ch.name, ch.id);
    } else {
        println!("  {}", tr("init.mm_team_id_missing"));
    }

    // ADR-014: provision human roster (idempotent). Failures are surfaced
    // as warnings so init still finishes — the user can re-run
    // `genasis humans sync` after fixing credentials.
    if !cfg.humans.is_empty() {
        println!(
            "\n→ provisioning {} human(s) from [[humans]]…",
            cfg.humans.len()
        );
        let humans_args = crate::cmd_humans::Args {
            project: Some(project_root.clone()),
            action: crate::cmd_humans::Action::Sync {
                mm_only: false,
                plane_only: false,
                dry_run: false,
            },
        };
        if let Err(e) = crate::cmd_humans::run(humans_args).await {
            eprintln!("  ⚠ humans sync failed: {e} — re-run `genasis humans sync` later");
        }
    }

    println!("\n{}", tr("init.next_step"));

    // alpha.36: try to auto-start the daemon for the real flavor as
    // well. The real-flavor `listen` path is less well-trodden than
    // the trial one (Mattermost is WebSocket rather than SSE), so
    // any failure here surfaces a clear "what to run manually"
    // message rather than aborting init.
    if !args.no_auto_start {
        println!();
        print!("→ Auto: starting reactive daemon (background)… ");
        match crate::cmd_listen::start_daemon(&project_root, false) {
            Ok(()) => {
                println!("✓");
                println!();
                println!("  Daemon running. Mattermost channel + Plane project");
                println!("  are wired up. Stop with `genasis stop`, follow logs");
                println!("  with `genasis logs -f`.");
            }
            Err(e) => {
                println!("⚠ skipped");
                eprintln!(
                    "  Reason: {e}\n  Real-flavor daemon is still maturing — \
                     run `genasis listen start` manually to retry. The \
                     Plane/MM provisioning above completed successfully and \
                     does not need re-running."
                );
            }
        }
    }
    Ok(())
}

/// v0.5.2 Issue #9 helper: GET `/api/v4/teams/name/<name>` on the
/// Mattermost server and return the resolved team id. Returns `None`
/// on any failure (network / not-found / auth) so the caller falls
/// through to the legacy "skipped" message instead of aborting init.
async fn lookup_mm_team_id_by_name(
    mm_base_url: &str,
    team_name: &str,
    admin_token: &str,
) -> Option<String> {
    let url = format!(
        "{}/api/v4/teams/name/{}",
        mm_base_url.trim_end_matches('/'),
        team_name
    );
    let resp = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .ok()?
        .get(&url)
        .bearer_auth(admin_token)
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let v: serde_json::Value = resp.json().await.ok()?;
    v.get("id").and_then(|x| x.as_str()).map(String::from)
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

fn slug_to_identifier(name: &str) -> String {
    let upper: String = name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect();
    if upper.len() >= 3 {
        upper.chars().take(4).collect()
    } else {
        format!("{upper:0<3}")
    }
}

fn short_json(v: &serde_json::Value) -> String {
    let s = v.to_string();
    if s.len() > 200 {
        format!("{}…", &s[..200])
    } else {
        s
    }
}

/// Render the trial-mode `genasis.toml` body from the user's chosen
/// project name and a freshly minted `team_token` (ADR-016 §2). The
/// project name flows into `[project].name`, the slug into
/// `workspace_slug` / `team_name` / channel slugs, and the token into
/// `[trial].team_token` so the trial-app can scope this user's
/// sandbox without colliding with other concurrent demos on the
/// shared hosted instance.
fn render_trial_config(project_name: &str, project_slug: &str, team_token: &str) -> String {
    format!(
        r#"# Genasis trial-mode config — generated by `genasis init --trial`.
# The trial-app at {trial_url} stands in as a Plane + Mattermost
# simulator so the agentic workflow can be exercised without
# installing either tool.
#
# Routing rules
# -------------
# When [plane].flavor or [mattermost].flavor is "trial", the per-provider
# `url` field below is IGNORED at runtime. The [trial] section at the
# bottom is the single source of truth for the trial-app endpoint and
# shared secret (ADR-013). Per ADR-016, [trial].team_token additionally
# scopes every sim row to your sandbox so concurrent users on the
# shared hosted instance do not overwrite each other.

[project]
name = "{project_name}"

[plane]
# Ignored when flavor = "trial" (uses [trial].url instead).
url = "{trial_url}"
workspace_slug = "{project_slug}"
flavor = "trial"
project_name = "{project_name}"

[mattermost]
# Ignored when flavor = "trial" (uses [trial].url instead).
url = "{trial_url}"
team_name = "{project_slug}"
flavor = "trial"
channels = [
  {{ key = "scrum", name = "scrum-{project_slug}", display_name = "{project_name} — Scrum" }},
]

[trial]
# When `enabled = true` AND a provider's flavor is "trial", calls are
# forwarded to `url` with `shared_secret` in the X-Genasis-Trial-Secret
# header. Disable trial routing by flipping `enabled = false` AND
# changing the per-provider flavors away from "trial".
enabled = true
url = "{trial_url}"
# Empty = browser-tab same-origin enforcement only. Set this if you're
# making server-to-server calls and the operator gave you a secret.
shared_secret = ""
# Per-team isolation key (ADR-016). Random hex written by
# `genasis init --trial`. The trial-app scopes every sim row by this
# token so concurrent users do not overwrite each other on the shared
# hosted instance. Keep it secret — anyone with this value can read
# and write your sandbox.
team_token = "{team_token}"
"#,
        trial_url = trial_app_url(),
        project_name = project_name,
        project_slug = project_slug,
        team_token = team_token,
    )
}

/// URL of the operator-hosted public trial-app. Hardcoded because the
/// trial flow is meant to be zero-setup — users run `genasis init
/// --trial` and the binary points them at this site directly.
///
/// v0.5.10 (D-009 follow-up): The operator-hosted instance can lag
/// behind `agents-pool` main (during this cycle we shipped
/// `demo_issues` / `welcome_message` support to the bootstrap route,
/// but the hosted Caddy + docker container hasn't been redeployed —
/// `/api/plane/issues` + `/api/mattermost/posts` still 401 with the
/// pre-D-001 secret-required contract, and the new bootstrap fields
/// are silently dropped by the old `z` schema). Users who hit this
/// can self-host the trial-app in a single
/// `docker run mmplane-trial-app` and point the binary at it by
/// exporting `GENASIS_TRIAL_URL=http://localhost:<port>` BEFORE
/// running `genasis init --trial`. The value flows through to the
/// landing URL, the bootstrap POST, the per-team open URL printed in
/// the summary box, and into `genasis.toml [trial].url`.
const DEFAULT_TRIAL_APP_URL: &str = "https://mmplane-trial.realstory.blog";

fn trial_app_url() -> String {
    std::env::var("GENASIS_TRIAL_URL")
        .ok()
        .filter(|s| !s.is_empty())
        .map(|s| s.trim_end_matches('/').to_string())
        .unwrap_or_else(|| DEFAULT_TRIAL_APP_URL.to_string())
}

/// Best-effort name suggestion when `--name` is omitted: humanise the
/// project directory's basename ("marketing-squad" → "Marketing
/// Squad"). Falls back to "Trial Demo" if the directory has no usable
/// name (e.g. `/`).
fn suggest_name_from_dir(project_root: &std::path::Path) -> String {
    let raw = project_root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    if raw.is_empty() {
        return "Trial Demo".to_string();
    }
    let mut out = String::with_capacity(raw.len());
    let mut new_word = true;
    for c in raw.chars() {
        if c == '-' || c == '_' || c == ' ' {
            out.push(' ');
            new_word = true;
        } else if new_word {
            out.extend(c.to_uppercase());
            new_word = false;
        } else {
            out.push(c);
        }
    }
    out
}

/// POST a bootstrap call to the trial-app so the user's freshly
/// generated `team_token` already has its project + scrum channel
/// seeded by the time they open `/?team=<token>`. Failure is
/// non-fatal — the trial-app will lazily create rows on first use,
/// but the kanban header / chat sidebar may briefly show stale
/// defaults until then.
async fn try_bootstrap_trial_app(
    base_url: &str,
    team_token: &str,
    project_slug: &str,
    project_name: &str,
) -> Result<()> {
    let endpoint = format!("{base_url}/api/trial/bootstrap");
    // v0.5.9 D-009: include `demo_issues` + `welcome_message` so the
    // Live Trial UI shows immediate visual proof of activity. Without
    // these, users opening the post-init URL see an empty kanban + empty
    // chat thread and reasonably wonder whether `genasis init --trial`
    // did anything. The trial-app side is idempotent on
    // `(team_token, project_slug, title)` / `(actor, message)` so
    // re-running bootstrap never duplicates.
    let payload = serde_json::json!({
        "team_token": team_token,
        "project": {
            "slug": project_slug,
            "name": project_name,
        },
        "channels": [
            {
                "key": "scrum",
                "name": format!("scrum-{project_slug}"),
                "display_name": format!("{project_name} — Scrum"),
            }
        ],
        "demo_issues": [
            {
                "title": "Set up agentic team (you are here)",
                "state": "done",
                "assignee": "genasis",
            },
            {
                "title": "Write PRD and split into tickets",
                "state": "inprogress",
                "assignee": "pm",
            },
            {
                "title": "Build the example app from PRD",
                "state": "todo",
                "assignee": null,
            },
        ],
        "welcome_message": {
            "actor": "genasis",
            "text": format!(
                "👋 {project_name} 팀이 시작됐어요. 우측 칸반에 데모 카드가 보이고, \
                 이 채팅 (#scrum-{project_slug}) 은 에이전트 활동을 실시간으로 \
                 흘려보냅니다. 다음 단계: `genasis example prd` → `genasis init` → \
                 Claude Code 세션 띄워서 작업 시키기."
            ),
        },
    });
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()?;
    let res = client
        .post(&endpoint)
        .json(&payload)
        .header("X-Genasis-Team-Token", team_token)
        .send()
        .await?;
    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        anyhow::bail!("trial-app bootstrap returned {status}: {body}");
    }
    // v0.5.12 D-011 트랙 3b: 호스팅 trial-app 이 stale 인지 응답으로 즉시 감지.
    // ec7f149 이상이라면 응답 body 에 `demo_issues` 와 `welcome_message` 키가
    // echo 됨 (빈 배열이라도 키 자체는 존재). 둘 다 없으면 호스팅이 D-009
    // 이전 빌드라는 뜻 — 카드/메시지가 sim DB 에 안 들어갈 것이므로
    // 사용자에게 즉시 경고하여 README §Known limitations 의 우회 경로로
    // 안내한다.
    let bootstrap_resp: serde_json::Value = res.json().await.unwrap_or_default();
    let host_supports_demo = bootstrap_resp.get("demo_issues").is_some();
    let host_supports_welcome = bootstrap_resp.get("welcome_message").is_some();
    if !(host_supports_demo && host_supports_welcome) {
        eprintln!(
            "  ⚠ trial-app at {base_url} appears to be running a pre-D-009 build \
             (no `demo_issues`/`welcome_message` keys in bootstrap response). \
             The live trial kanban + chat will stay empty until the operator \
             redeploys. README §\"Known limitations\" documents the workaround \
             (export GENASIS_TRIAL_URL=<your-instance>)."
        );
    }

    // v0.5.3 issue 나: bootstrap POST returning 200 doesn't prove
    // the team row actually landed (the deployed trial-app may be
    // running an older version that accepts the request but
    // doesn't persist it, or the schema may have drifted). Verify
    // by GET'ing `/api/trial/team-app/status?team=<token>` and
    // checking `team_exists`. A real mismatch surfaces here so the
    // user knows their browser tab will fall into the "unknown
    // token" path, instead of finding out empirically after
    // pasting the token into the TokenBar.
    let verify_url = format!("{base_url}/api/trial/team-app/status?team={team_token}");
    let v = client
        .get(&verify_url)
        .header("X-Genasis-Team-Token", team_token)
        .send()
        .await?;
    if !v.status().is_success() {
        anyhow::bail!(
            "trial-app post-bootstrap verify returned {} (bootstrap may not have persisted)",
            v.status()
        );
    }
    let body: serde_json::Value = v.json().await.unwrap_or_default();
    let team_exists = body
        .get("team_exists")
        .and_then(|x| x.as_bool())
        .unwrap_or(false);
    if !team_exists {
        anyhow::bail!(
            "trial-app accepted the bootstrap POST but the team row was not persisted \
             (deployed trial-app may be older than the bootstrap-route contract this \
             binary expects). Try again later or ask the operator to redeploy."
        );
    }

    Ok(())
}

async fn run_trial(
    args: Args,
    lang_flag: Option<String>,
    non_interactive: bool,
    assume_yes: bool,
) -> Result<()> {
    use std::fs;
    use std::process::{Command, Stdio};

    let project_root = if let Some(p) = args.project.as_deref() {
        if !p.exists() {
            fs::create_dir_all(p)
                .with_context(|| format!("create --project dir {}", p.display()))?;
        }
        p.canonicalize()
            .with_context(|| format!("canonicalize {}", p.display()))?
    } else {
        std::env::current_dir()?
    };

    println!(
        "→ Initializing trial-mode project at {}",
        project_root.display()
    );

    let cfg_path = project_root.join(CONFIG_FILE_NAME);
    let cfg_existed = cfg_path.exists();

    let assume_yes = std::env::var("GENASIS_TRIAL_AUTOLAUNCH").ok().as_deref() == Some("1");

    let mut team_token: Option<String> = None;
    if !cfg_existed {
        let project_name = if let Some(n) = args.name.as_deref() {
            n.trim().to_string()
        } else if assume_yes || args.probe_only {
            suggest_name_from_dir(&project_root)
        } else {
            let suggested = suggest_name_from_dir(&project_root);
            dialoguer::Input::new()
                .with_prompt("Project name (shown in trial-app kanban + chat)")
                .default(suggested)
                .interact_text()
                .unwrap_or_else(|_| suggest_name_from_dir(&project_root))
        };
        let project_slug = slugify(&project_name);
        let token = random_team_token();
        let body = render_trial_config(&project_name, &project_slug, &token);
        fs::write(&cfg_path, body).with_context(|| format!("write {}", cfg_path.display()))?;
        println!(
            "  wrote {} (project={project_name}, team_token={short})",
            cfg_path.display(),
            short = &token[..8.min(token.len())],
        );
        team_token = Some(token);
    } else {
        println!("  {} already exists — leaving as-is", cfg_path.display());
        // Re-read existing config so we still bootstrap the trial-app
        // with the right team_token if the user has run init --trial
        // before. Skipped silently if the existing config has no
        // [trial] section (the user is in real mode).
        if let Ok(existing) = Config::load(&cfg_path) {
            if let Some(t) = existing.trial.as_ref() {
                if let Some(tok) = t.team_token.clone() {
                    if !tok.is_empty() && tok != DEFAULT_TEAM_TOKEN {
                        team_token = Some(tok);
                    }
                }
            }
        }
    }

    fs::create_dir_all(project_root.join(".claude/agents")).ok();
    fs::create_dir_all(project_root.join(".genasis")).ok();

    // Scaffold the 10 base agent .md files + apply overlay fences in
    // one shot. ADR-010 + CLAUDE.md §"Turnkey bootstrap for new
    // teams": `genasis init --trial` is supposed to be "minute-one
    // ready" — a fully populated `.claude/agents/` is the visible
    // proof that the team exists. The previous implementation only
    // mkdir'd the directory, leaving the user to discover `genasis
    // bootstrap` on their own; users reported the empty folder as a
    // bug, which it was.
    //
    // `cmd_bootstrap::run` chains into `cmd_attach::pub_run`
    // automatically (`no_attach_after: false`), so this single call
    // produces both base files AND marker-fence overlay content.
    //
    // Failures are surfaced as a warning rather than aborting init —
    // `try_bootstrap_trial_app` and the summary box still need to
    // run so the user gets their team_token regardless of catalog
    // hiccups. `--probe-only` skips this entirely; tests that just
    // want the toml round-trip don't pay the catalog-fetch cost.
    if !args.probe_only {
        println!("\n→ Scaffolding base agents into .claude/agents/…");
        let bootstrap_args = crate::cmd_bootstrap::Args {
            project: Some(project_root.clone()),
            roles: None,
            no_attach_after: false,
            dry_run: false,
        };
        if let Err(e) = crate::cmd_bootstrap::run(
            bootstrap_args,
            lang_flag.clone(),
            non_interactive,
            assume_yes,
        )
        .await
        {
            eprintln!(
                "  ⚠ agent bootstrap failed: {e}\n  re-run `genasis bootstrap` once the issue is fixed. Trial init will continue with the rest of the setup."
            );
        }
    }

    if let Some(tok) = team_token.as_deref() {
        // Re-derive name + slug from the freshly written config so the
        // bootstrap call is consistent with what was actually persisted.
        if let Ok(written) = Config::load(&cfg_path) {
            let name = written.project.name.clone();
            let slug = slugify(&name);
            let trial_url = trial_app_url();
            match try_bootstrap_trial_app(&trial_url, tok, &slug, &name).await {
                Ok(()) => println!("  trial-app bootstrap ok ({slug})"),
                Err(e) => {
                    eprintln!("  ⚠ trial-app bootstrap failed: {e} — will lazily seed on first use")
                }
            }
        }
    }

    let trial_url = trial_app_url();
    if args.probe_only {
        println!("\n--probe-only set — skipping browser open");
        println!("  Trial app: {trial_url}");
        if let Some(tok) = team_token.as_deref() {
            println!("  Per-team URL: {trial_url}/?tab=live&team={tok}");
        }
        return Ok(());
    }

    let landing = match team_token.as_deref() {
        Some(tok) => format!("{trial_url}/?tab=live&team={tok}"),
        None => trial_url.clone(),
    };

    let open_browser = if assume_yes {
        true
    } else {
        match dialoguer::Confirm::new()
            .with_prompt(format!("Open the trial app at {landing} in a browser?"))
            .default(true)
            .interact()
        {
            Ok(b) => b,
            Err(_) => false,
        }
    };

    if open_browser {
        if let Some(opener) = pick_browser_opener() {
            let _ = Command::new(opener)
                .arg(&landing)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
        }
    }

    // Always print the explicit-token-gating summary at the end so
    // the user can copy the team_token even if the browser-open path
    // was skipped (probe_only / GENASIS_TRIAL_AUTOLAUNCH=0 / headless
    // CI / xdg-open not found). The trial-app's TokenBar requires
    // this value before any of the Live Trial UI activates
    // (ADR-017 §6).
    if let Some(tok) = team_token.as_deref() {
        let project_label = Config::load(&cfg_path)
            .ok()
            .map(|c| c.project.name)
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| "(unnamed)".to_string());
        let bar = "━".repeat(64);
        println!("\n{bar}");
        println!("  Trial team ready · {project_label}");
        println!("{bar}");
        println!();
        println!("  Team token (paste into the Live Trial top input):");
        println!();
        println!("    {tok}");
        println!();
        println!("  Open directly with the token pre-filled:");
        println!();
        println!("    {landing}");
        println!();
        println!("  Save this token. It's the only key that ties YOUR agents'");
        println!("  kanban cards + chat messages to YOUR Live Trial view —");
        println!("  without it the trial-app keeps the whole panel disabled");
        println!("  (multi-tenant partition, ADR-017 §6).");
        println!("{bar}");
    } else {
        println!("\nTrial app: {landing}");
    }
    println!(
        "\nTwo tabs:\n  Live trial      — kanban + chat that mirrors agent calls live (requires team token)\n  Borrow real env — request a real Plane + Mattermost project from the operator"
    );
    println!(
        "\nThe trial app is operator-hosted — no local install needed. To self-host instead, see\nhttps://github.com/claude-genasis/agents-pool/tree/main/trial-app (private repo)\nand point [trial].url in genasis.toml at your own deployment."
    );

    // UX simplification (alpha.36): users were stuck running `init`
    // → `publish` → `listen start` three times. `init --trial` now
    // wraps the publish + daemon-start tail so the Quick Path is
    // one command. `--no-auto-start` bypasses for power users.
    if !args.no_auto_start {
        auto_publish_and_start_daemon(&project_root).await;
    }
    Ok(())
}

/// Run `genasis publish` + `genasis listen start --trial` in
/// sequence, swallowing recoverable errors so the user still gets
/// the trial-team-ready banner above even if one of the auto-steps
/// fails. Failure messages get reported inline with an explicit
/// "what to run yourself" hint.
async fn auto_publish_and_start_daemon(project_root: &std::path::Path) {
    println!();
    print!("→ Auto: publishing trial app… ");
    let publish_args = crate::cmd_trial::PublishArgs {
        dry_run: false,
        project: Some(project_root.to_path_buf()),
    };
    match crate::cmd_trial::run_publish_with_project(publish_args).await {
        Ok(()) => println!("✓"),
        Err(e) => {
            println!("✗");
            eprintln!(
                "  ⚠ publish failed: {e}\n    Run `genasis publish` manually to retry."
            );
        }
    }

    print!("→ Auto: starting reactive daemon (background)… ");
    match crate::cmd_listen::start_daemon(project_root, true) {
        Ok(()) => {
            println!("✓");
            println!();
            println!("  Daemon is running in the background. Open the Live Trial");
            println!("  URL above and type into the chat panel — PM / frontend /");
            println!("  devops agents will reply within a minute.");
            println!();
            println!("  Stop with:  genasis stop");
            println!("  Logs:       genasis logs -f");
            println!("  Status:     genasis status");
        }
        Err(e) => {
            println!("✗");
            eprintln!(
                "  ⚠ daemon failed to start: {e}\n    Run `genasis listen start --trial` manually to retry."
            );
        }
    }
}

fn pick_browser_opener() -> Option<&'static str> {
    if cfg!(target_os = "macos") {
        Some("open")
    } else if cfg!(target_os = "windows") {
        None
    } else if which::which("xdg-open").is_ok() {
        Some("xdg-open")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn run_trial_writes_config_and_skips_launch_in_probe_only() {
        let tmp = TempDir::new().unwrap();
        let args = Args {
            project: Some(tmp.path().to_path_buf()),
            probe_only: true,
            trial: true,
            name: Some("Marketing Squad".into()),
            bootstrap: false,
            roles: None,
        };
        run_trial(args, None, true, true)
            .await
            .expect("trial probe_only succeeds");
        let cfg = std::fs::read_to_string(tmp.path().join(CONFIG_FILE_NAME)).unwrap();
        assert!(cfg.contains("[trial]"));
        assert!(cfg.contains("enabled = true"));
        assert!(cfg.contains("flavor = \"trial\""));
        assert!(
            cfg.contains("name = \"Marketing Squad\""),
            "expected user's project name in config:\n{cfg}"
        );
        assert!(
            cfg.contains("workspace_slug = \"marketing-squad\""),
            "expected slugified workspace in config:\n{cfg}"
        );
        assert!(
            cfg.contains("name = \"scrum-marketing-squad\""),
            "expected scrum channel in config:\n{cfg}"
        );
        assert!(
            cfg.contains("team_token = \""),
            "expected team_token in config:\n{cfg}"
        );
    }

    #[tokio::test]
    async fn run_trial_does_not_overwrite_existing_config() {
        let tmp = TempDir::new().unwrap();
        let cfg_path = tmp.path().join(CONFIG_FILE_NAME);
        std::fs::write(&cfg_path, "# existing config\n").unwrap();
        let args = Args {
            project: Some(tmp.path().to_path_buf()),
            probe_only: true,
            trial: true,
            name: Some("Anything".into()),
            bootstrap: false,
            roles: None,
        };
        run_trial(args, None, true, true).await.unwrap();
        let cfg = std::fs::read_to_string(&cfg_path).unwrap();
        assert_eq!(cfg, "# existing config\n");
    }

    #[tokio::test]
    async fn run_trial_derives_name_from_directory_when_flag_omitted() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("brand-new-team");
        std::fs::create_dir_all(&dir).unwrap();
        let args = Args {
            project: Some(dir.clone()),
            probe_only: true,
            trial: true,
            name: None,
            bootstrap: false,
            roles: None,
        };
        run_trial(args, None, true, true)
            .await
            .expect("derive-from-dir succeeds");
        let cfg = std::fs::read_to_string(dir.join(CONFIG_FILE_NAME)).unwrap();
        assert!(
            cfg.contains("Brand New Team"),
            "expected humanised dirname:\n{cfg}"
        );
    }
}
