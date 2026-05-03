//! Runtime localisation for the Genasis CLI / TUI.
//!
//! Backed by [`rust-i18n`]. The `t!()` macro is re-exported so call sites
//! depend on this crate, not on the underlying engine.
//!
//! ## Resolution priority
//!
//! 1. Explicit CLI flag (`--lang ko`).
//! 2. `genasis.toml` `[i18n] cli_lang` (caller's responsibility to pass in).
//! 3. `GENASIS_LANG` environment variable.
//! 4. POSIX `LANG` environment variable (e.g. `ko_KR.UTF-8` → `ko`).
//! 5. Fallback to `en`.
//!
//! ## Both languages installed simultaneously
//!
//! Not supported. Construction with [`Lang::Both`] is impossible by
//! design — `--lang both` is intercepted by the CLI layer with a dedicated
//! error message that cites `docs/impact-of-multilang-prompts.md`.

use std::env;
use std::fmt;

use once_cell::sync::OnceCell;

// `rust_i18n::i18n!` walks up from the current crate's manifest dir to
// resolve the locales path at compile time. We include the YAML files via
// the macro and surface `t!` to downstream crates by re-export.
rust_i18n::i18n!("locales", fallback = "en");

pub use rust_i18n::t;

/// Identifier for a supported runtime locale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lang {
    En,
    Ko,
}

impl Lang {
    /// BCP-47 short code.
    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Ko => "ko",
        }
    }

    /// Human-readable native name (used in the install prompt confirmation).
    pub fn native_name(self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::Ko => "한국어",
        }
    }

    /// Parse a user-supplied identifier. Accepts `en`, `EN`, `en_US`,
    /// `en-GB`, `English`, `english`, etc. Returns `None` for unknown
    /// values; the caller decides how to surface the error.
    pub fn parse(raw: &str) -> Option<Self> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return None;
        }
        let head = trimmed
            .split(|c: char| c == '_' || c == '-' || c == '.')
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        match head.as_str() {
            "en" | "english" | "eng" => Some(Lang::En),
            "ko" | "korean" | "kor" | "kr" => Some(Lang::Ko),
            _ => None,
        }
    }
}

impl Default for Lang {
    fn default() -> Self {
        Lang::En
    }
}

impl fmt::Display for Lang {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

/// Where the runtime locale came from. Surfaced by `genasis doctor`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LangSource {
    /// Explicit `--lang` flag.
    Flag,
    /// `genasis.toml` `[i18n] cli_lang`.
    ConfigFile,
    /// `GENASIS_LANG` environment variable.
    GenasisEnv,
    /// POSIX `LANG` environment variable.
    PosixEnv,
    /// No signal found; fell back to `en`.
    Default,
}

impl LangSource {
    pub fn label(self) -> &'static str {
        match self {
            LangSource::Flag => "--lang flag",
            LangSource::ConfigFile => "genasis.toml [i18n] cli_lang",
            LangSource::GenasisEnv => "$GENASIS_LANG",
            LangSource::PosixEnv => "$LANG",
            LangSource::Default => "default (en)",
        }
    }
}

/// The result of `Lang::resolve()`. Always contains a usable [`Lang`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Resolved {
    pub lang: Lang,
    pub source: LangSource,
}

impl Resolved {
    pub fn new(lang: Lang, source: LangSource) -> Self {
        Self { lang, source }
    }
}

/// Resolve the runtime locale from the priority chain.
///
/// `flag` is the value of `--lang` if the CLI consumed one.
/// `config` is the value of `genasis.toml [i18n] cli_lang` if loaded.
/// The remaining sources are pulled from process environment.
///
/// Unknown values at any tier are skipped with a `tracing::warn!`; the
/// next tier is consulted. The function never panics and always returns
/// a `Resolved`.
pub fn resolve(flag: Option<&str>, config: Option<&str>) -> Resolved {
    if let Some(raw) = flag {
        match Lang::parse(raw) {
            Some(lang) => return Resolved::new(lang, LangSource::Flag),
            None => tracing::warn!(
                value = %raw,
                "ignoring unknown --lang value; falling through to next source"
            ),
        }
    }
    if let Some(raw) = config {
        match Lang::parse(raw) {
            Some(lang) => return Resolved::new(lang, LangSource::ConfigFile),
            None => tracing::warn!(
                value = %raw,
                "ignoring unknown genasis.toml [i18n] cli_lang; falling through"
            ),
        }
    }
    if let Ok(raw) = env::var("GENASIS_LANG") {
        if let Some(lang) = Lang::parse(&raw) {
            return Resolved::new(lang, LangSource::GenasisEnv);
        }
        tracing::warn!(value = %raw, "ignoring unknown $GENASIS_LANG; falling through");
    }
    if let Ok(raw) = env::var("LANG") {
        if let Some(lang) = Lang::parse(&raw) {
            return Resolved::new(lang, LangSource::PosixEnv);
        }
    }
    Resolved::new(Lang::En, LangSource::Default)
}

/// Apply a resolved locale to the global `rust-i18n` runtime.
///
/// Idempotent. Subsequent `t!(...)` calls in any crate observe the new
/// locale immediately.
pub fn install(lang: Lang) {
    rust_i18n::set_locale(lang.code());
    INSTALLED.set(lang).ok();
}

/// Returns the locale most recently installed via [`install`], if any.
/// Useful for diagnostics; do not branch program logic on this.
pub fn current() -> Option<Lang> {
    INSTALLED.get().copied()
}

static INSTALLED: OnceCell<Lang> = OnceCell::new();

// Note: per-key parity diagnostics ("which keys are missing in ko.yml?")
// are implemented at doctor time (M12.11) by parsing the YAML files
// directly, not via the runtime t! macro. Keeping this crate's surface
// small avoids depending on private rust-i18n APIs.
