//! Categorise design-system.md changes into impact areas so the ticket
//! emitter can produce one Plane issue per affected area.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ImpactArea {
    ColorTokens,
    Typography,
    Spacing,
    Layout,
    Components,
    Motion,
    Other,
}

impl ImpactArea {
    pub fn slug(self) -> &'static str {
        match self {
            ImpactArea::ColorTokens => "color-tokens",
            ImpactArea::Typography => "typography",
            ImpactArea::Spacing => "spacing",
            ImpactArea::Layout => "layout",
            ImpactArea::Components => "components",
            ImpactArea::Motion => "motion",
            ImpactArea::Other => "other",
        }
    }
}

/// Heuristic categorisation: scan a section's title or content for
/// keywords. We keep this deliberately simple — `cmd_design swap` just
/// needs a list of areas that *probably* changed so the ticket emitter
/// can spawn parallel improvement issues.
pub fn categorize(line: &str) -> ImpactArea {
    let l = line.to_ascii_lowercase();
    if l.contains("color") || l.contains("colour") || l.contains("palette") || l.contains("oklch") {
        ImpactArea::ColorTokens
    } else if l.contains("font") || l.contains("typography") || l.contains("text-") {
        ImpactArea::Typography
    } else if l.contains("space") || l.contains("padding") || l.contains("margin") || l.contains("gap") {
        ImpactArea::Spacing
    } else if l.contains("grid") || l.contains("layout") || l.contains("container") || l.contains("breakpoint") {
        ImpactArea::Layout
    } else if l.contains("button") || l.contains("card") || l.contains("input") || l.contains("component") {
        ImpactArea::Components
    } else if l.contains("animation") || l.contains("transition") || l.contains("motion") || l.contains("ease") {
        ImpactArea::Motion
    } else {
        ImpactArea::Other
    }
}

/// Compare `before` and `after` line by line, return the set of impact
/// areas affected by changed lines.
pub fn changed_areas(before: &str, after: &str) -> Vec<ImpactArea> {
    use std::collections::BTreeSet;
    let mut areas: BTreeSet<&'static str> = BTreeSet::new();
    let before_lines: Vec<&str> = before.lines().collect();
    let after_lines: Vec<&str> = after.lines().collect();
    let len = before_lines.len().max(after_lines.len());
    for i in 0..len {
        let b = before_lines.get(i).copied().unwrap_or("");
        let a = after_lines.get(i).copied().unwrap_or("");
        if b != a {
            let area = categorize(if a.is_empty() { b } else { a });
            areas.insert(area.slug());
        }
    }
    areas.into_iter().filter_map(slug_to_area).collect()
}

fn slug_to_area(s: &str) -> Option<ImpactArea> {
    Some(match s {
        "color-tokens" => ImpactArea::ColorTokens,
        "typography" => ImpactArea::Typography,
        "spacing" => ImpactArea::Spacing,
        "layout" => ImpactArea::Layout,
        "components" => ImpactArea::Components,
        "motion" => ImpactArea::Motion,
        "other" => ImpactArea::Other,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyword_categorisation() {
        assert_eq!(categorize("--color-primary: oklch(60%);"), ImpactArea::ColorTokens);
        assert_eq!(categorize("font-size: 14px"), ImpactArea::Typography);
        assert_eq!(categorize("padding: 8px"), ImpactArea::Spacing);
        assert_eq!(categorize("grid-template-columns:"), ImpactArea::Layout);
        assert_eq!(categorize("Button primary"), ImpactArea::Components);
        assert_eq!(categorize("transition: 200ms"), ImpactArea::Motion);
        assert_eq!(categorize("misc note"), ImpactArea::Other);
    }

    #[test]
    fn changed_areas_collects_unique() {
        let before = "color-primary: red\nfont-size: 14px\n";
        let after = "color-primary: blue\nfont-size: 14px\n";
        let a = changed_areas(before, after);
        assert!(a.contains(&ImpactArea::ColorTokens));
        assert!(!a.contains(&ImpactArea::Typography));
    }
}
