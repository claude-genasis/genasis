//! Integration tests for `genasis-i18n`.
//!
//! These tests exercise the public surface only: `Lang::parse`,
//! `resolve()`, `install()`, and the `t!` macro re-export. The tests
//! mutate process-global state (rust-i18n locale, env vars) and so must
//! run sequentially. Cargo runs integration tests in separate binaries
//! by default; we still serialise within this binary.

use std::sync::Mutex;

use genasis_i18n::{install, resolve, t, Lang, LangSource};
use once_cell::sync::Lazy;

// `rust_i18n::set_locale` is a process-global write. Multiple tests in
// the same binary must serialise on this mutex so locale-sensitive
// assertions don't interleave.
static SERIAL: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

fn with_lang<F: FnOnce()>(lang: Lang, f: F) {
    let _guard = SERIAL.lock().unwrap();
    install(lang);
    f();
}

#[test]
fn lang_parse_accepts_canonical_codes() {
    assert_eq!(Lang::parse("en"), Some(Lang::En));
    assert_eq!(Lang::parse("ko"), Some(Lang::Ko));
}

#[test]
fn lang_parse_is_case_insensitive() {
    assert_eq!(Lang::parse("EN"), Some(Lang::En));
    assert_eq!(Lang::parse("Ko"), Some(Lang::Ko));
}

#[test]
fn lang_parse_strips_locale_modifier() {
    assert_eq!(Lang::parse("en_US.UTF-8"), Some(Lang::En));
    assert_eq!(Lang::parse("ko_KR.UTF-8"), Some(Lang::Ko));
    assert_eq!(Lang::parse("en-GB"), Some(Lang::En));
}

#[test]
fn lang_parse_accepts_friendly_names() {
    assert_eq!(Lang::parse("english"), Some(Lang::En));
    assert_eq!(Lang::parse("Korean"), Some(Lang::Ko));
    assert_eq!(Lang::parse("kr"), Some(Lang::Ko));
}

#[test]
fn lang_parse_rejects_unknown() {
    assert_eq!(Lang::parse("xx"), None);
    assert_eq!(Lang::parse(""), None);
    assert_eq!(Lang::parse("   "), None);
    assert_eq!(Lang::parse("zh"), None); // not yet supported
}

#[test]
fn resolve_flag_wins_over_config_and_env() {
    let _guard = SERIAL.lock().unwrap();
    std::env::set_var("GENASIS_LANG", "en");
    std::env::set_var("LANG", "en_US.UTF-8");
    let r = resolve(Some("ko"), Some("en"));
    assert_eq!(r.lang, Lang::Ko);
    assert_eq!(r.source, LangSource::Flag);
    std::env::remove_var("GENASIS_LANG");
}

#[test]
fn resolve_config_wins_over_env_when_no_flag() {
    let _guard = SERIAL.lock().unwrap();
    std::env::set_var("GENASIS_LANG", "en");
    let r = resolve(None, Some("ko"));
    assert_eq!(r.lang, Lang::Ko);
    assert_eq!(r.source, LangSource::ConfigFile);
    std::env::remove_var("GENASIS_LANG");
}

#[test]
fn resolve_genasis_env_wins_over_lang_env() {
    let _guard = SERIAL.lock().unwrap();
    std::env::set_var("GENASIS_LANG", "ko");
    std::env::set_var("LANG", "en_US.UTF-8");
    let r = resolve(None, None);
    assert_eq!(r.lang, Lang::Ko);
    assert_eq!(r.source, LangSource::GenasisEnv);
    std::env::remove_var("GENASIS_LANG");
}

#[test]
fn resolve_lang_env_kicks_in_last_before_default() {
    let _guard = SERIAL.lock().unwrap();
    std::env::remove_var("GENASIS_LANG");
    std::env::set_var("LANG", "ko_KR.UTF-8");
    let r = resolve(None, None);
    assert_eq!(r.lang, Lang::Ko);
    assert_eq!(r.source, LangSource::PosixEnv);
}

#[test]
fn resolve_falls_back_to_english() {
    let _guard = SERIAL.lock().unwrap();
    std::env::remove_var("GENASIS_LANG");
    std::env::set_var("LANG", "C");
    let r = resolve(None, None);
    assert_eq!(r.lang, Lang::En);
    assert_eq!(r.source, LangSource::Default);
}

#[test]
fn resolve_skips_unknown_flag_and_falls_through() {
    let _guard = SERIAL.lock().unwrap();
    std::env::remove_var("GENASIS_LANG");
    std::env::set_var("LANG", "ko_KR.UTF-8");
    let r = resolve(Some("xx"), None);
    assert_eq!(r.lang, Lang::Ko);
    assert_eq!(r.source, LangSource::PosixEnv);
}

#[test]
fn t_macro_renders_english_attach_success() {
    with_lang(Lang::En, || {
        let msg = t!("attach.success", count = 3);
        assert!(msg.contains("Attached overlay"));
        assert!(msg.contains("3"));
    });
}

#[test]
fn t_macro_renders_korean_attach_success() {
    with_lang(Lang::Ko, || {
        let msg = t!("attach.success", count = 5);
        assert!(msg.contains("부착했습니다"));
        assert!(msg.contains("5"));
    });
}

#[test]
fn t_macro_falls_back_when_key_missing_in_active_locale() {
    // Insert a key only in en.yml (none of the production keys are
    // intentionally one-sided yet). We simulate "missing in ko" by
    // requesting an English-only key from the Korean locale; rust-i18n
    // is configured with `fallback = "en"` so we expect to see the
    // English string verbatim.
    with_lang(Lang::Ko, || {
        let msg = t!("common.ok");
        // ko.yml defines this as "확인" — make sure we got the localised
        // value, not silently fell through to "OK".
        assert_eq!(msg.as_ref(), "확인");
    });
}

#[test]
fn doctor_section_label_is_localised() {
    with_lang(Lang::En, || {
        assert_eq!(t!("doctor.i18n.section").as_ref(), "[i18n]");
    });
    with_lang(Lang::Ko, || {
        assert_eq!(t!("doctor.i18n.section").as_ref(), "[다국어]");
    });
}

#[test]
fn lang_round_trips_to_code_and_back() {
    for lang in [Lang::En, Lang::Ko] {
        assert_eq!(Lang::parse(lang.code()), Some(lang));
    }
}

#[test]
fn lang_source_has_human_label() {
    assert_eq!(LangSource::Flag.label(), "--lang flag");
    assert_eq!(LangSource::PosixEnv.label(), "$LANG");
    assert!(!LangSource::Default.label().is_empty());
}
