//! `.env.agents` — flat KEY="VALUE" file with per-agent secrets.
//!
//! Format conventions (matches genesis predecessor):
//! - One assignment per line: `KEY=VALUE` or `KEY="quoted value"`.
//! - Lines starting with `#` are comments and preserved on round-trip.
//! - Blank lines are preserved on round-trip.
//! - Keys are case-sensitive ASCII.
//!
//! Round-trip preservation is important: `genasis init` writes the file,
//! humans add comments, `genasis upgrade` rewrites it without losing the
//! comments.

use std::fmt::Write as _;
use std::path::Path;

use indexmap::IndexMap;

use crate::error::{Error, Result};
use crate::fs::{atomic_write, read_to_string_optional};

/// One physical line in the file. We store comments and blanks verbatim so
/// upgrades do not eat human annotations.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Line {
    Comment(String),
    Blank,
    Entry {
        key: String,
        value: String,
        quoted: bool,
    },
}

#[derive(Debug, Clone, Default)]
pub struct EnvFile {
    lines: Vec<Line>,
}

impl EnvFile {
    /// Parse a `.env.agents`-style file. Missing file → empty `EnvFile`.
    pub fn read(path: &Path) -> Result<Self> {
        let body = read_to_string_optional(path)?.unwrap_or_default();
        Self::from_str(&body)
    }

    /// Parse from a string buffer.
    pub fn from_str(s: &str) -> Result<Self> {
        let mut lines = Vec::new();
        for raw in s.lines() {
            let trimmed = raw.trim_start();
            if trimmed.is_empty() {
                lines.push(Line::Blank);
                continue;
            }
            if trimmed.starts_with('#') {
                lines.push(Line::Comment(raw.to_string()));
                continue;
            }
            let (k, v_raw) = trimmed
                .split_once('=')
                .ok_or_else(|| Error::Config(format!("env: missing `=` in line: {raw:?}")))?;
            let key = k.trim().to_string();
            let (value, quoted) = parse_value(v_raw);
            lines.push(Line::Entry { key, value, quoted });
        }
        Ok(Self { lines })
    }

    /// Serialise back to the on-disk format.
    pub fn to_string(&self) -> String {
        let mut out = String::new();
        for ln in &self.lines {
            match ln {
                Line::Blank => out.push('\n'),
                Line::Comment(c) => {
                    out.push_str(c);
                    out.push('\n');
                }
                Line::Entry { key, value, quoted } => {
                    if *quoted || needs_quoting(value) {
                        let _ = writeln!(out, "{key}=\"{}\"", escape_dq(value));
                    } else {
                        let _ = writeln!(out, "{key}={value}");
                    }
                }
            }
        }
        out
    }

    /// Get a value by key (last-write-wins if duplicates).
    pub fn get(&self, key: &str) -> Option<&str> {
        self.lines.iter().rev().find_map(|ln| match ln {
            Line::Entry { key: k, value, .. } if k == key => Some(value.as_str()),
            _ => None,
        })
    }

    /// Set or overwrite a key. New entries are appended at the end.
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let key = key.into();
        let value = value.into();
        for ln in self.lines.iter_mut() {
            if let Line::Entry {
                key: k,
                value: v,
                quoted,
            } = ln
            {
                if k == &key {
                    *v = value;
                    *quoted = needs_quoting(v);
                    return;
                }
            }
        }
        self.lines.push(Line::Entry {
            key,
            value: value.clone(),
            quoted: needs_quoting(&value),
        });
    }

    /// Remove a key (no-op if absent).
    pub fn remove(&mut self, key: &str) {
        self.lines
            .retain(|ln| !matches!(ln, Line::Entry { key: k, .. } if k == key));
    }

    /// All entries as an in-order map (last write wins).
    pub fn entries(&self) -> IndexMap<String, String> {
        let mut m = IndexMap::new();
        for ln in &self.lines {
            if let Line::Entry { key, value, .. } = ln {
                m.insert(key.clone(), value.clone());
            }
        }
        m
    }

    /// Atomically write to disk.
    pub fn write(&self, path: &Path) -> Result<()> {
        atomic_write(path, self.to_string().as_bytes())
    }
}

fn parse_value(raw: &str) -> (String, bool) {
    let trimmed = raw.trim();
    if let Some(inner) = trimmed.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
        (unescape_dq(inner), true)
    } else if let Some(inner) = trimmed
        .strip_prefix('\'')
        .and_then(|s| s.strip_suffix('\''))
    {
        (inner.to_string(), true)
    } else {
        (trimmed.to_string(), false)
    }
}

fn needs_quoting(v: &str) -> bool {
    v.is_empty()
        || v.chars()
            .any(|c| c.is_whitespace() || matches!(c, '#' | '"' | '\'' | '=' | '$'))
}

fn escape_dq(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn unescape_dq(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn round_trips_comments_blanks_and_quoting() {
        let raw =
            "# top comment\n\nPLANE_URL=\"https://example.com\"\nPLANE_TOKEN=plain\n# trailing\n";
        let env = EnvFile::from_str(raw).unwrap();
        // Output equals input verbatim.
        assert_eq!(env.to_string(), raw);
    }

    #[test]
    fn set_appends_when_new() {
        let mut env = EnvFile::default();
        env.set("A", "1");
        env.set("B", "two words");
        let s = env.to_string();
        assert!(s.contains("A=1\n"));
        assert!(s.contains("B=\"two words\"\n"));
    }

    #[test]
    fn set_replaces_existing_in_place() {
        let mut env = EnvFile::from_str("A=1\nB=2\n").unwrap();
        env.set("A", "10");
        assert_eq!(env.get("A"), Some("10"));
        assert_eq!(env.to_string(), "A=10\nB=2\n");
    }

    #[test]
    fn remove_deletes_entry() {
        let mut env = EnvFile::from_str("A=1\nB=2\n").unwrap();
        env.remove("A");
        assert_eq!(env.to_string(), "B=2\n");
    }

    #[test]
    fn write_then_read_round_trip() {
        let dir = tempdir().unwrap();
        let p = dir.path().join(".env.agents");
        let mut env = EnvFile::default();
        env.set("PLANE_URL", "https://example.com");
        env.set("PLANE_TOKEN_PM", "secret with spaces");
        env.write(&p).unwrap();

        let reread = EnvFile::read(&p).unwrap();
        assert_eq!(reread.get("PLANE_URL"), Some("https://example.com"));
        assert_eq!(reread.get("PLANE_TOKEN_PM"), Some("secret with spaces"));
    }

    #[test]
    fn rejects_malformed_line() {
        let err = EnvFile::from_str("not_an_assignment\n").unwrap_err();
        assert!(matches!(err, Error::Config(_)));
    }
}
