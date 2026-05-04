//! Interactive language selection prompt for `genasis init` / `attach`.
//!
//! Mirrors the Bash prompt in `install.sh` so the user sees the same
//! layout regardless of entry path. See blueprint §19.3.3.

use std::io::{self, BufRead, IsTerminal, Write};

use anyhow::Result;
use genasis_i18n::{tr, tr_args, Lang};

/// Outcome of `--lang` / prompt resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Decision {
    pub lang: Lang,
    pub via: ChoiceSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChoiceSource {
    /// `--lang` flag bound the answer.
    Flag,
    /// User answered the interactive prompt.
    Prompt,
    /// stdin was not a TTY and `--non-interactive` (or env) forced a fallback to $LANG.
    LangEnvFallback,
    /// Default (no signal anywhere) — English.
    Default,
}

impl ChoiceSource {
    pub fn label(self) -> &'static str {
        match self {
            ChoiceSource::Flag => "flag",
            ChoiceSource::Prompt => "prompt",
            ChoiceSource::LangEnvFallback => "lang_env",
            ChoiceSource::Default => "default",
        }
    }
}

/// Sentinel returned when the caller passed `--lang both`. The caller
/// surfaces a localised error and exits with code 2.
#[derive(Debug, thiserror::Error)]
#[error("--lang both is not supported")]
pub struct BothRejected;

/// Resolve the language for the install action. `flag` is the value of
/// `--lang` if present, `non_interactive` skips the prompt when stdin
/// is a TTY, `assume_yes` auto-accepts the confirmation.
pub fn decide(
    flag: Option<&str>,
    non_interactive: bool,
    assume_yes: bool,
) -> Result<Decision, anyhow::Error> {
    if let Some(raw) = flag {
        if raw.eq_ignore_ascii_case("both") {
            print_both_rejection();
            return Err(BothRejected.into());
        }
        if let Some(lang) = Lang::parse(raw) {
            return Ok(Decision {
                lang,
                via: ChoiceSource::Flag,
            });
        }
        anyhow::bail!("unknown --lang value: {raw} (allowed: en, ko)");
    }

    let stdin_tty = io::stdin().is_terminal();
    if non_interactive || !stdin_tty {
        let suggested = suggested_from_env();
        let via = if std::env::var("LANG").is_ok() {
            ChoiceSource::LangEnvFallback
        } else {
            ChoiceSource::Default
        };
        eprintln!(
            "non-interactive: using --lang {} (override with --lang en|ko)",
            suggested.code()
        );
        return Ok(Decision {
            lang: suggested,
            via,
        });
    }

    let suggested = suggested_from_env();
    print_prompt(suggested);
    let answer = read_choice(suggested)?;
    if !assume_yes {
        confirm_or_abort(answer)?;
    }
    Ok(Decision {
        lang: answer,
        via: ChoiceSource::Prompt,
    })
}

fn suggested_from_env() -> Lang {
    if let Ok(raw) = std::env::var("LANG") {
        if let Some(lang) = Lang::parse(&raw) {
            return lang;
        }
    }
    Lang::En
}

fn print_prompt(suggested: Lang) {
    let mut out = io::stdout().lock();
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "┌─ {} / {} ─────────────────────────────",
        tr("lang.prompt.title_en"),
        tr("lang.prompt.title_ko")
    );
    let _ = writeln!(out, "│ {}", tr("lang.prompt.choose"));
    let _ = writeln!(out, "│");
    let _ = writeln!(out, "│ {}", tr("lang.prompt.targets_intro"));
    for key in [
        "lang.prompt.target_agents",
        "lang.prompt.target_skills",
        "lang.prompt.target_commands",
        "lang.prompt.target_hooks",
        "lang.prompt.target_contract",
    ] {
        let _ = writeln!(out, "│   • {}", tr(key));
    }
    let _ = writeln!(out, "│");
    let _ = writeln!(out, "│ ⚠ {}", tr("lang.prompt.drift_warning"));
    let _ = writeln!(out, "│");
    let lang_env = std::env::var("LANG").unwrap_or_else(|_| "(unset)".into());
    let _ = writeln!(
        out,
        "│ {}",
        tr_args(
            "lang.prompt.detected",
            &[
                ("lang_env", &lang_env),
                ("suggested", suggested.native_name())
            ]
        )
    );
    let _ = writeln!(out, "│");
    let suggested_marker = format!("    {}", tr("lang.prompt.suggested_marker"));
    let mark_en = if matches!(suggested, Lang::En) {
        suggested_marker.as_str()
    } else {
        ""
    };
    let mark_ko = if matches!(suggested, Lang::Ko) {
        suggested_marker.as_str()
    } else {
        ""
    };
    let _ = writeln!(out, "│   {}{}", tr("lang.prompt.option_en"), mark_en);
    let _ = writeln!(out, "│   {}{}", tr("lang.prompt.option_ko"), mark_ko);
    let _ = writeln!(out, "└────────────────────────────────────────────");
    let _ = out.flush();
}

fn read_choice(suggested: Lang) -> Result<Lang> {
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let prompt_key = match suggested {
        Lang::En => "lang.prompt.select_default_en",
        Lang::Ko => "lang.prompt.select_default_ko",
    };
    for _ in 0..3 {
        eprint!("{} ", tr(prompt_key));
        let _ = io::stderr().flush();
        let mut line = String::new();
        if handle.read_line(&mut line)? == 0 {
            return Ok(suggested);
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Ok(suggested);
        }
        match trimmed {
            "1" | "en" | "EN" | "English" | "english" => return Ok(Lang::En),
            "2" | "ko" | "KO" | "한국어" | "korean" | "Korean" => return Ok(Lang::Ko),
            _ => eprintln!("{}", tr("lang.prompt.invalid_choice")),
        }
    }
    anyhow::bail!("{}", tr("lang.prompt.too_many_attempts"));
}

fn confirm_or_abort(lang: Lang) -> Result<()> {
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    eprintln!(
        "\n✓ {}",
        tr_args(
            "lang.prompt.confirmation",
            &[
                ("lang_name", lang.native_name()),
                ("lang_code", lang.code()),
            ]
        )
    );
    eprint!("{} ", tr("lang.prompt.confirm_continue"));
    let _ = io::stderr().flush();
    let mut line = String::new();
    if handle.read_line(&mut line)? == 0 {
        return Ok(());
    }
    let trimmed = line.trim();
    if trimmed.is_empty()
        || trimmed.eq_ignore_ascii_case("y")
        || trimmed.eq_ignore_ascii_case("yes")
        || trimmed == "예"
    {
        return Ok(());
    }
    anyhow::bail!("{}", tr("lang.prompt.aborted"));
}

fn print_both_rejection() {
    let mut err = io::stderr().lock();
    let _ = writeln!(err);
    let _ = writeln!(err, "✘ {}", tr("lang.reject_both.line1"));
    let _ = writeln!(err);
    let _ = writeln!(err, "  {}", tr("lang.reject_both.line2"));
    let _ = writeln!(err, "  {}", tr("lang.reject_both.line3"));
    let _ = writeln!(err);
    let _ = writeln!(err, "  {}", tr("lang.reject_both.alternatives"));
    let _ = writeln!(err, "    1. {}", tr("lang.reject_both.alt1"));
    let _ = writeln!(err, "    2. {}", tr("lang.reject_both.alt2"));
    let _ = writeln!(err);
    let _ = writeln!(err, "  {}", tr("lang.reject_both.rerun"));
    let _ = err.flush();
}
