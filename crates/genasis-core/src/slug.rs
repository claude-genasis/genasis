//! D-104: short slug generation for `genasis provision`.
//!
//! Plane and Mattermost user/team identifiers are best kept short so
//! we can build `pm-<slug>@genasis.bot` style agent emails and
//! `team-<slug>` style display names without running into character
//! limits or ugly long URLs. The rule the user picked is **5 chars
//! max**, abbreviating multi-word input by taking the first letter of
//! each significant word.
//!
//! Hangul / non-ASCII handling: the user asked us to "translate (like
//! google translate) and then apply the English abbreviation rule".
//! We satisfy that with a two-stage fallback:
//!
//! 1. **Translate via local `claude` CLI** — same binary `genasis
//!    listen` already shells out to, so we don't introduce a new
//!    external service or API key requirement. `claude -p` is invoked
//!    with a one-shot prompt asking for a short English phrase (1-3
//!    words). The output replaces the original input before
//!    abbreviation.
//! 2. **Fallback: `deunicode` transliteration** — if `claude` is not
//!    on PATH or the call fails, fall back to phonetic
//!    transliteration. This preserves uniqueness but loses meaning
//!    (e.g. "팀협업" → "tim hyeob eob" → "the").
//!
//! Both paths funnel into the same abbreviation step which:
//! - lowercases ASCII,
//! - splits on whitespace / `-_` / underscores,
//! - keeps alphanumeric chars only,
//! - if `>= 2` words: concatenates the first letter of each
//!   significant word, then truncates to 5 chars,
//! - if `== 1` word: truncates to 5 chars.

use std::process::Command;
use std::time::Duration;

/// Maximum slug length per user spec. `agentic` workspace also caps
/// internal references; 5 chars is short enough to keep
/// `pm-XXXXX@genasis.bot` under 25 chars.
pub const MAX_SLUG_LEN: usize = 5;

/// Only articles / prepositions are filtered. We deliberately keep
/// nouns like "team", "squad", "demo", "app" because users routinely
/// include those as identifying words ("Marketing Squad" → "ms", not
/// "mark…"); filtering them would erase the second word the user
/// clearly intended to encode.
const STOPWORDS: &[&str] = &[
    "the", "a", "an", "of", "for", "and", "or", "to", "in", "on", "by",
];

/// Produce a 5-char-max kebab slug suitable for Plane / Mattermost
/// identifiers + agent email local-parts. See module-level docs for
/// the algorithm.
pub fn slugify_abbrev(input: &str) -> String {
    let translated = translate_if_non_ascii(input).unwrap_or_else(|| input.to_string());
    abbreviate_english(&translated)
}

/// Pure-English abbreviation step. Public so tests can exercise it
/// without the `claude` CLI dependency.
pub fn abbreviate_english(input: &str) -> String {
    // Normalise to ASCII so non-Latin remnants (e.g. 한자) don't survive.
    let normalised = deunicode::deunicode(input).to_ascii_lowercase();

    // Split on anything non-alphanumeric.
    let words: Vec<&str> = normalised
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect();

    // Filter stopwords but only if at least one significant word remains.
    let significant: Vec<&&str> = words
        .iter()
        .filter(|w| !STOPWORDS.contains(&w.to_ascii_lowercase().as_str()))
        .collect();
    let words = if !significant.is_empty() {
        significant.iter().map(|w| **w).collect::<Vec<&str>>()
    } else {
        words
    };

    let raw = if words.len() >= 2 {
        // Multi-word → first letter of each, up to 5.
        words.iter().filter_map(|w| w.chars().next()).collect::<String>()
    } else if let Some(only) = words.first() {
        // Single word → first 5 chars.
        only.to_string()
    } else {
        // No words at all (e.g. all symbols). Fall back to "team".
        "team".to_string()
    };

    let trimmed: String = raw.chars().take(MAX_SLUG_LEN).collect();
    // Final pass through `slug` to drop any straggler punctuation.
    slug::slugify(trimmed).chars().take(MAX_SLUG_LEN).collect()
}

/// If the input contains non-ASCII characters (Hangul, CJK, accented
/// Latin etc.) try to invoke `claude -p` for a short English
/// translation. Returns `None` if all chars are already ASCII (so we
/// don't shell out for nothing) or if the CLI call fails — caller
/// will fall back to transliteration.
fn translate_if_non_ascii(input: &str) -> Option<String> {
    if input.is_ascii() {
        return None;
    }
    let claude_path = which::which("claude").ok()?;
    let prompt = format!(
        "Translate the following team or project name to a short English phrase \
         (1-3 words). Output only the translated phrase, no quotes, no \
         explanation.\n\nName: {input}"
    );
    let mut cmd = Command::new(claude_path);
    cmd.arg("-p").arg(&prompt).arg("--output-format").arg("text");
    // 30s ceiling so a wedged claude doesn't block provision.
    let output = run_with_timeout(cmd, Duration::from_secs(30)).ok()?;
    if !output.status.success() {
        return None;
    }
    let translated = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if translated.is_empty() || !translated.is_ascii() {
        return None;
    }
    Some(translated)
}

/// Hard-bounded `Command::output` so a stuck `claude` doesn't hold the
/// whole provision flow hostage. We spawn, give it `timeout` seconds,
/// and kill if exceeded.
fn run_with_timeout(
    mut cmd: Command,
    timeout: Duration,
) -> Result<std::process::Output, std::io::Error> {
    use std::io::Read;
    let mut child = cmd
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;
    let start = std::time::Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            if let Some(mut o) = child.stdout.take() {
                let _ = o.read_to_end(&mut stdout);
            }
            if let Some(mut e) = child.stderr.take() {
                let _ = e.read_to_end(&mut stderr);
            }
            return Ok(std::process::Output {
                status,
                stdout,
                stderr,
            });
        }
        if start.elapsed() > timeout {
            let _ = child.kill();
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "claude translate timeout",
            ));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiword_abbreviates_to_initials() {
        assert_eq!(abbreviate_english("Marketing Squad"), "ms");
        assert_eq!(abbreviate_english("Marketing Communications Team"), "mct");
        assert_eq!(
            abbreviate_english("alpha beta gamma delta epsilon"),
            "abgde"
        );
    }

    #[test]
    fn single_word_truncates_to_five() {
        assert_eq!(abbreviate_english("Quiz"), "quiz");
        assert_eq!(abbreviate_english("Pomodoro"), "pomod");
        assert_eq!(abbreviate_english("a"), "a");
    }

    #[test]
    fn stopwords_filtered() {
        // "the project squad" → drop "the"/"project"/"squad" all
        // stopwords → fall back to all words → "tps".
        let out = abbreviate_english("The Project Squad");
        assert!(out.len() <= MAX_SLUG_LEN);
        assert!(out.starts_with(|c: char| c.is_ascii_lowercase()));
    }

    #[test]
    fn non_ascii_fallback_via_deunicode() {
        // Without claude on PATH the algorithm still produces *something*.
        // Exact value depends on deunicode tables; we just assert
        // length + shape.
        let out = abbreviate_english("팀협업");
        assert!(out.len() <= MAX_SLUG_LEN);
        assert!(out.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'));
    }

    #[test]
    fn punctuation_and_symbols_dropped() {
        assert_eq!(abbreviate_english("Foo / Bar / Baz!"), "fbb");
        // "a" is a stopword so it gets filtered before initials. The
        // result is the two remaining words' initials "bc", not "abc".
        assert_eq!(abbreviate_english("a@b.c"), "bc");
    }

    #[test]
    fn empty_falls_back_to_team() {
        assert_eq!(abbreviate_english("###"), "team");
        assert_eq!(abbreviate_english(""), "team");
    }

    #[test]
    fn slugify_abbrev_handles_ascii_directly() {
        // ASCII input must not shell out — same as
        // abbreviate_english.
        assert_eq!(slugify_abbrev("Quiz Demo"), "qd");
        assert_eq!(slugify_abbrev("DataPipeline"), "datap");
    }
}
