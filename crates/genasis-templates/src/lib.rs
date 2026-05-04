//! genasis-templates — Tera templates embedded at compile time.
//!
//! Templates live under two parallel locale subtrees, `templates/en/` and
//! `templates/ko/`, both loaded via `include_dir!()` so the release binary
//! is self-contained. The CLI's `--lang` selection chooses which subtree
//! is consulted by [`get_lang`] / [`get`].

use include_dir::{include_dir, Dir};

pub static TEMPLATES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates");

/// Look up a template body using the legacy single-tree path
/// (e.g. `"en/GENASIS.md.tera"` or `"ko/agent-overlays/frontend.patch.md.tera"`).
pub fn get(path: &str) -> Option<&'static str> {
    TEMPLATES.get_file(path).and_then(|f| f.contents_utf8())
}

/// Look up a template body for a specific BCP-47 locale code.
///
/// `lang` is `"en"` or `"ko"`; `relative` is the path inside the locale
/// subtree (e.g. `"GENASIS.md.tera"` or
/// `"agent-overlays/frontend.patch.md.tera"`).
pub fn get_lang(lang: &str, relative: &str) -> Option<&'static str> {
    let combined = format!("{lang}/{relative}");
    TEMPLATES
        .get_file(&combined)
        .and_then(|f| f.contents_utf8())
}

/// Locale subtrees that are guaranteed to exist at build time.
pub const SUPPORTED_LANGS: &[&str] = &["en", "ko"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_genasis_md_present() {
        assert!(get_lang("en", "GENASIS.md.tera").is_some());
    }

    #[test]
    fn korean_genasis_md_present() {
        assert!(get_lang("ko", "GENASIS.md.tera").is_some());
    }

    #[test]
    fn english_frontend_overlay_present() {
        assert!(get_lang("en", "agent-overlays/frontend.patch.md.tera").is_some());
    }

    #[test]
    fn korean_frontend_overlay_present() {
        assert!(get_lang("ko", "agent-overlays/frontend.patch.md.tera").is_some());
    }

    #[test]
    fn unknown_locale_returns_none() {
        assert!(get_lang("xx", "GENASIS.md.tera").is_none());
    }

    #[test]
    fn english_and_korean_have_same_top_level_files() {
        let en: std::collections::BTreeSet<_> = TEMPLATES
            .get_dir("en")
            .expect("en/ subtree must exist")
            .files()
            .map(|f| f.path().file_name().unwrap().to_string_lossy().to_string())
            .collect();
        let ko: std::collections::BTreeSet<_> = TEMPLATES
            .get_dir("ko")
            .expect("ko/ subtree must exist")
            .files()
            .map(|f| f.path().file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert_eq!(en, ko, "top-level filename parity required");
    }
}
