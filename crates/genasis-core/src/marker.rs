//! Overlay marker fence — parse, serialise, hash, locate.
//!
//! Format (see blueprint.md §3.1):
//!
//! ```markdown
//! <!-- GENASIS:BEGIN role=frontend version=1.0 hash=a1b2c3d4 -->
//! ...content...
//! <!-- GENASIS:END -->
//! ```
//!
//! Invariants enforced here:
//! - At most **one** fence per file (the merger refuses to inject a second).
//! - The body is hashed with SHA-256, hex-encoded, and truncated to 8 chars.
//! - The hash recorded in the `BEGIN` line is computed over the *exact*
//!   `body` slice we serialise (no leading/trailing newline normalisation).
//! - Insert position: immediately after the YAML frontmatter terminator
//!   (`---` followed by a newline) when present, otherwise at the top of the
//!   file. Determined by [`insertion_anchor`].
//!
//! M1 scope: parser + serialiser + locator + idempotent inject/replace/remove.
//! Higher-level orchestration (detector, role inference, dry-run) lives in
//! `genasis-overlay`.

use std::fmt::Write as _;

use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{Error, Result};

pub const BEGIN_PREFIX: &str = "<!-- GENASIS:BEGIN";
pub const END_MARKER: &str = "<!-- GENASIS:END -->";

/// One overlay fence pulled out of a file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fence {
    pub role: String,
    pub version: String,
    pub hash: String,
    /// The body lines between the BEGIN and END markers, **without** the
    /// surrounding marker lines, joined with `\n`. No trailing newline.
    pub body: String,
}

impl Fence {
    /// Build a fence from a body, computing the hash automatically.
    pub fn new(role: impl Into<String>, version: impl Into<String>, body: impl Into<String>) -> Self {
        let body = body.into();
        let hash = compute_hash(&body);
        Self {
            role: role.into(),
            version: version.into(),
            hash,
            body,
        }
    }

    /// Re-serialise to the canonical fenced markdown form.
    pub fn render(&self) -> String {
        let mut out = String::with_capacity(self.body.len() + 128);
        let _ = writeln!(
            out,
            "<!-- GENASIS:BEGIN role={} version={} hash={} -->",
            self.role, self.version, self.hash
        );
        out.push_str(&self.body);
        if !self.body.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(END_MARKER);
        out.push('\n');
        out
    }

    /// True iff the recorded hash matches the body. False means a human
    /// edited the fence content directly — the merger must skip it.
    pub fn body_matches_hash(&self) -> bool {
        compute_hash(&self.body) == self.hash
    }
}

/// SHA-256(body) → first 8 hex chars.
pub fn compute_hash(body: &str) -> String {
    let digest = Sha256::digest(body.as_bytes());
    hex::encode(&digest[..4])
}

/// Locate the (single) fence in `text`, if any.
///
/// Returns `Ok(Some((fence, byte_range)))`. The range covers the entire
/// fenced block including both marker lines and any trailing newline that
/// belongs to the END marker line.
pub fn find(text: &str) -> Result<Option<(Fence, std::ops::Range<usize>)>> {
    let begin_idx = match text.find(BEGIN_PREFIX) {
        Some(i) => i,
        None => return Ok(None),
    };

    // The BEGIN line ends at the first \n after begin_idx.
    let begin_line_end = text[begin_idx..]
        .find('\n')
        .map(|n| begin_idx + n)
        .ok_or_else(|| Error::Overlay("BEGIN marker missing terminating newline".into()))?;
    let begin_line = &text[begin_idx..begin_line_end];

    // Detect a duplicate BEGIN beyond this one.
    if text[begin_line_end..].contains(BEGIN_PREFIX) {
        return Err(Error::Overlay(
            "multiple GENASIS:BEGIN fences in one file (overlay must be singleton)".into(),
        ));
    }

    let end_idx = text[begin_line_end..].find(END_MARKER).map(|n| begin_line_end + n);
    let end_idx = end_idx.ok_or_else(|| {
        Error::Overlay("GENASIS:BEGIN found but matching GENASIS:END is missing".into())
    })?;

    // Range: from begin_idx through the newline that follows END_MARKER (if any).
    let after_end = end_idx + END_MARKER.len();
    let range_end = if text[after_end..].starts_with('\n') {
        after_end + 1
    } else {
        after_end
    };

    let body = text[begin_line_end + 1..end_idx].trim_end_matches('\n').to_string();
    let attrs = parse_begin_attrs(begin_line)?;
    let fence = Fence {
        role: attrs.role,
        version: attrs.version,
        hash: attrs.hash,
        body,
    };
    Ok(Some((fence, begin_idx..range_end)))
}

struct BeginAttrs {
    role: String,
    version: String,
    hash: String,
}

fn parse_begin_attrs(begin_line: &str) -> Result<BeginAttrs> {
    let re = Regex::new(r"^<!-- GENASIS:BEGIN(.*?)-->\s*$").unwrap();
    let cap = re
        .captures(begin_line.trim_end())
        .ok_or_else(|| Error::Overlay(format!("malformed BEGIN line: {begin_line:?}")))?;
    let attrs_str = cap.get(1).map(|m| m.as_str()).unwrap_or("");

    let mut role = None;
    let mut version = None;
    let mut hash = None;
    for tok in attrs_str.split_whitespace() {
        let (k, v) = match tok.split_once('=') {
            Some(kv) => kv,
            None => continue,
        };
        match k {
            "role" => role = Some(v.to_string()),
            "version" => version = Some(v.to_string()),
            "hash" => hash = Some(v.to_string()),
            _ => {}
        }
    }
    Ok(BeginAttrs {
        role: role.ok_or_else(|| Error::Overlay("BEGIN missing role= attribute".into()))?,
        version: version.ok_or_else(|| Error::Overlay("BEGIN missing version= attribute".into()))?,
        hash: hash.ok_or_else(|| Error::Overlay("BEGIN missing hash= attribute".into()))?,
    })
}

/// Where to insert a fresh fence in `text`: immediately after the YAML
/// frontmatter terminator (the second `---\n`) if one exists, otherwise byte 0.
pub fn insertion_anchor(text: &str) -> usize {
    if !text.starts_with("---\n") && !text.starts_with("---\r\n") {
        return 0;
    }
    let after_first = if let Some(stripped) = text.strip_prefix("---\r\n") {
        text.len() - stripped.len()
    } else {
        4 // "---\n"
    };
    let rest = &text[after_first..];
    let close_pat = if text.contains("\r\n") { "\n---\r\n" } else { "\n---\n" };
    if let Some(pos) = rest.find(close_pat) {
        return after_first + pos + close_pat.len();
    }
    0
}

/// Inject `fence` into `text` after the frontmatter terminator (or at byte 0).
/// If `text` already contains a fence, returns an error — callers must use
/// [`replace`] explicitly.
pub fn inject(text: &str, fence: &Fence) -> Result<String> {
    if find(text)?.is_some() {
        return Err(Error::Overlay(
            "file already has a GENASIS fence; use replace() instead".into(),
        ));
    }
    let anchor = insertion_anchor(text);
    let rendered = fence.render();
    let mut out = String::with_capacity(text.len() + rendered.len() + 1);
    out.push_str(&text[..anchor]);
    if anchor > 0 && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&rendered);
    out.push_str(&text[anchor..]);
    Ok(out)
}

/// Replace the existing fence in `text` with `fence`. Idempotent: if the
/// existing fence is byte-equal after replacement, returns `text` unchanged.
pub fn replace(text: &str, fence: &Fence) -> Result<String> {
    let (_existing, range) = find(text)?.ok_or_else(|| {
        Error::Overlay("no existing GENASIS fence to replace; use inject() instead".into())
    })?;
    let rendered = fence.render();
    let mut out = String::with_capacity(text.len() + rendered.len());
    out.push_str(&text[..range.start]);
    out.push_str(&rendered);
    out.push_str(&text[range.end..]);
    Ok(out)
}

/// Inject if absent, replace if present. The merger's primary entry point.
pub fn upsert(text: &str, fence: &Fence) -> Result<String> {
    match find(text)? {
        None => inject(text, fence),
        Some(_) => replace(text, fence),
    }
}

/// Remove the fence (and its surrounding markers) from `text`. Returns
/// unchanged text if no fence is present.
pub fn remove(text: &str) -> Result<String> {
    match find(text)? {
        None => Ok(text.to_string()),
        Some((_, range)) => {
            let mut out = String::with_capacity(text.len());
            out.push_str(&text[..range.start]);
            out.push_str(&text[range.end..]);
            Ok(out)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_fence() -> Fence {
        Fence::new(
            "frontend",
            "1.0",
            "## (Genasis Overlay) Plane/Mattermost 계약\n- placeholder body line\n- another line",
        )
    }

    #[test]
    fn render_round_trip() {
        let f = sample_fence();
        let rendered = f.render();
        let parsed = find(&rendered).unwrap().expect("fence present").0;
        assert_eq!(parsed, f);
        assert!(parsed.body_matches_hash());
    }

    #[test]
    fn inject_after_frontmatter() {
        let original = "---\nname: frontend\ntools: Bash\n---\n\n# Frontend Agent\nbody.\n";
        let injected = inject(original, &sample_fence()).unwrap();
        // Frontmatter intact, fence after it, original body preserved.
        assert!(injected.starts_with("---\nname: frontend\ntools: Bash\n---\n"));
        assert!(injected.contains(BEGIN_PREFIX));
        assert!(injected.contains(END_MARKER));
        assert!(injected.contains("# Frontend Agent\nbody.\n"));
    }

    #[test]
    fn inject_without_frontmatter_lands_at_top() {
        let original = "# Plain markdown\nbody.\n";
        let injected = inject(original, &sample_fence()).unwrap();
        assert!(injected.starts_with(BEGIN_PREFIX));
        assert!(injected.ends_with("# Plain markdown\nbody.\n"));
    }

    #[test]
    fn upsert_is_idempotent() {
        let original = "---\nname: frontend\n---\n\n# body\n";
        let once = upsert(original, &sample_fence()).unwrap();
        let twice = upsert(&once, &sample_fence()).unwrap();
        assert_eq!(once, twice, "upsert must be idempotent");
    }

    #[test]
    fn replace_swaps_body_only() {
        let f1 = Fence::new("frontend", "1.0", "old body");
        let f2 = Fence::new("frontend", "1.1", "new body");
        let original = inject("---\nname: x\n---\n\nrest\n", &f1).unwrap();
        let replaced = replace(&original, &f2).unwrap();
        assert!(replaced.contains("new body"));
        assert!(!replaced.contains("old body"));
        assert!(replaced.contains("version=1.1"));
    }

    #[test]
    fn remove_strips_fence() {
        let original = "---\na: b\n---\n\nrest\n";
        let with_fence = inject(original, &sample_fence()).unwrap();
        let stripped = remove(&with_fence).unwrap();
        assert_eq!(stripped, original);
    }

    #[test]
    fn detects_human_modification_via_hash() {
        let f = sample_fence();
        let mut tampered = f.clone();
        tampered.body.push_str("\n# evil edit");
        assert!(!tampered.body_matches_hash());
    }

    #[test]
    fn rejects_duplicate_fences() {
        let single = sample_fence().render();
        let doubled = format!("{single}\n{single}");
        let err = find(&doubled).unwrap_err();
        assert!(matches!(err, Error::Overlay(_)));
    }

    #[test]
    fn rejects_unterminated_begin() {
        let bad = "<!-- GENASIS:BEGIN role=x version=1 hash=00000000 -->\nbody\n";
        let err = find(bad).unwrap_err();
        assert!(matches!(err, Error::Overlay(_)));
    }

    #[test]
    fn parse_attrs_tolerates_extra_whitespace() {
        let line = "<!-- GENASIS:BEGIN  role=qa   version=2.0  hash=deadbeef -->";
        let a = parse_begin_attrs(line).unwrap();
        assert_eq!(a.role, "qa");
        assert_eq!(a.version, "2.0");
        assert_eq!(a.hash, "deadbeef");
    }

    #[test]
    fn hash_is_8_hex_chars() {
        let h = compute_hash("anything");
        assert_eq!(h.len(), 8);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
