//! Minimal YAML frontmatter splitter.
//!
//! We do **not** parse the YAML — we only locate the head/body boundary so
//! the overlay merger knows where to inject fences and so the detector can
//! pull a few well-known scalar fields (`name`, `description`, `tools`,
//! `model`, `color`).
//!
//! Recognised forms:
//! - `---\n<head>\n---\n<body>` (Unix newlines)
//! - `---\r\n<head>\r\n---\r\n<body>` (Windows newlines)
//!
//! Anything else: `frontmatter` is `None` and `body` is the entire file.

use crate::error::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frontmatter<'a> {
    /// Raw text between the opening `---` and the closing `---`, exclusive.
    pub raw: &'a str,
    /// Byte position immediately after the closing `---\n` (or `---\r\n`).
    pub body_start: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Split<'a> {
    pub frontmatter: Option<Frontmatter<'a>>,
    pub body: &'a str,
}

pub fn split(text: &str) -> Split<'_> {
    let (open_len, close_pat) = if let Some(rest) = text.strip_prefix("---\n") {
        (text.len() - rest.len(), "\n---\n")
    } else if let Some(rest) = text.strip_prefix("---\r\n") {
        (text.len() - rest.len(), "\n---\r\n")
    } else {
        return Split {
            frontmatter: None,
            body: text,
        };
    };

    let after_open = &text[open_len..];
    if let Some(close_idx) = after_open.find(close_pat) {
        let raw = &after_open[..close_idx];
        let body_start = open_len + close_idx + close_pat.len();
        return Split {
            frontmatter: Some(Frontmatter { raw, body_start }),
            body: &text[body_start..],
        };
    }

    Split {
        frontmatter: None,
        body: text,
    }
}

/// Pull a top-level scalar field from a YAML frontmatter block.
///
/// Recognises both `key: value` and `key: "quoted value"` on a single line.
/// Multiline / nested values return `None` (we don't need them — the only
/// fields we read are `name`, `description`, `tools`, `model`, `color`).
pub fn read_scalar<'a>(frontmatter: &'a str, key: &str) -> Option<&'a str> {
    for line in frontmatter.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }
        let (k, v) = match trimmed.split_once(':') {
            Some(kv) => kv,
            None => continue,
        };
        if k.trim() != key {
            continue;
        }
        let v = v.trim();
        if v.is_empty() || v == ">" || v == "|" {
            return None; // multiline form — caller can't use this helper
        }
        if let Some(stripped) = v.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
            return Some(stripped);
        }
        if let Some(stripped) = v.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')) {
            return Some(stripped);
        }
        return Some(v);
    }
    None
}

/// `read_scalar` plus an explicit error if the key is required.
pub fn require_scalar<'a>(frontmatter: &'a str, key: &str) -> Result<&'a str> {
    read_scalar(frontmatter, key)
        .ok_or_else(|| Error::Overlay(format!("frontmatter missing required scalar key `{key}`")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_basic_frontmatter() {
        let text = "---\nname: frontend\n---\n# Body\n";
        let s = split(text);
        let fm = s.frontmatter.expect("has frontmatter");
        assert_eq!(fm.raw, "name: frontend");
        assert_eq!(s.body, "# Body\n");
    }

    #[test]
    fn no_frontmatter_returns_full_body() {
        let text = "# Plain markdown\nbody.\n";
        let s = split(text);
        assert!(s.frontmatter.is_none());
        assert_eq!(s.body, text);
    }

    #[test]
    fn unterminated_frontmatter_yields_no_split() {
        let text = "---\nname: x\nno closing fence\n";
        let s = split(text);
        assert!(s.frontmatter.is_none());
    }

    #[test]
    fn read_scalar_handles_quotes_and_plain() {
        let fm = "name: frontend\nmodel: \"sonnet\"\ncolor: 'cyan'\n";
        assert_eq!(read_scalar(fm, "name"), Some("frontend"));
        assert_eq!(read_scalar(fm, "model"), Some("sonnet"));
        assert_eq!(read_scalar(fm, "color"), Some("cyan"));
    }

    #[test]
    fn read_scalar_returns_none_for_multiline_marker() {
        let fm = "description: >\n  long stuff\nname: x\n";
        assert!(read_scalar(fm, "description").is_none());
        assert_eq!(read_scalar(fm, "name"), Some("x"));
    }

    #[test]
    fn read_scalar_skips_comments() {
        let fm = "# name: wrong\nname: right\n";
        assert_eq!(read_scalar(fm, "name"), Some("right"));
    }
}
