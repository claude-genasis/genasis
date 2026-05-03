//! genasis-templates — Tera templates embedded at compile time.
//!
//! Templates are organised under `templates/` and are loaded via
//! `include_dir!()` so that the release binary is fully self-contained.
//!
//! M2/M6/M7 fill in the real templates. M0 provides only the GENASIS.md
//! header, the genasis.toml schema, and a few placeholders.

use include_dir::{include_dir, Dir};

pub static TEMPLATES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates");

/// Look up a template body by relative path (e.g. `"GENASIS.md.tera"`).
pub fn get(path: &str) -> Option<&'static str> {
    TEMPLATES.get_file(path).and_then(|f| f.contents_utf8())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn genasis_md_template_present() {
        assert!(get("GENASIS.md.tera").is_some());
    }
}
