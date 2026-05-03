//! Render a [`MergePlan`] as human-readable output for the CLI.
//!
//! Two formats are supported:
//! - `summary`: one line per file with status emoji and reason.
//! - `unified_diff`: full unified diff per file (when stdout is a TTY and
//!   the user passed `--diff`).

use std::fmt::Write as _;

use similar::{ChangeTag, TextDiff};

use crate::merger::{MergePlan, PlannedAction, PlannedChange};

pub fn summary(plan: &MergePlan) -> String {
    let mut out = String::new();
    if plan.changes.is_empty() {
        return "no agents detected — nothing to do".into();
    }
    for c in &plan.changes {
        let _ = writeln!(out, "{} {}", action_glyph(&c.action), short_path(&c.path));
        match &c.action {
            PlannedAction::Refuse(r) => {
                let _ = writeln!(out, "    refused: {r}");
            }
            PlannedAction::Skip(r) => {
                let _ = writeln!(out, "    skipped: {r}");
            }
            _ => {}
        }
    }
    let counts = counts(plan);
    let _ = writeln!(
        out,
        "\nplan: inject={} replace={} remove={} skip={} refuse={} no-fence={}",
        counts.inject, counts.replace, counts.remove, counts.skip, counts.refuse, counts.no_fence,
    );
    out
}

pub fn unified_diff(plan: &MergePlan) -> String {
    let mut out = String::new();
    for c in &plan.changes {
        if !changes_disk(&c.action) || c.before == c.after {
            continue;
        }
        let _ = writeln!(out, "--- {}", short_path(&c.path));
        let _ = writeln!(out, "+++ {} (proposed)", short_path(&c.path));
        let diff = TextDiff::from_lines(&c.before, &c.after);
        for change in diff.iter_all_changes() {
            let sign = match change.tag() {
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
                ChangeTag::Equal => " ",
            };
            // Trim trailing newlines because `change` already carries one.
            let line = change.value().trim_end_matches('\n');
            let _ = writeln!(out, "{sign}{line}");
        }
        out.push('\n');
    }
    if out.is_empty() {
        out.push_str("no on-disk changes\n");
    }
    out
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Counts {
    pub inject: usize,
    pub replace: usize,
    pub remove: usize,
    pub skip: usize,
    pub refuse: usize,
    pub no_fence: usize,
}

pub fn counts(plan: &MergePlan) -> Counts {
    let mut c = Counts::default();
    for ch in &plan.changes {
        match ch.action {
            PlannedAction::Inject => c.inject += 1,
            PlannedAction::Replace => c.replace += 1,
            PlannedAction::Remove => c.remove += 1,
            PlannedAction::Skip(_) => c.skip += 1,
            PlannedAction::Refuse(_) => c.refuse += 1,
            PlannedAction::NoFenceToRemove => c.no_fence += 1,
        }
    }
    c
}

fn changes_disk(a: &PlannedAction) -> bool {
    matches!(a, PlannedAction::Inject | PlannedAction::Replace | PlannedAction::Remove)
}

fn action_glyph(a: &PlannedAction) -> &'static str {
    match a {
        PlannedAction::Inject => "[+]",
        PlannedAction::Replace => "[~]",
        PlannedAction::Remove => "[-]",
        PlannedAction::Skip(_) => "[=]",
        PlannedAction::Refuse(_) => "[!]",
        PlannedAction::NoFenceToRemove => "[ ]",
    }
}

fn short_path(p: &std::path::Path) -> String {
    let s = p.to_string_lossy().into_owned();
    if let Some(idx) = s.rfind(".claude/agents/") {
        return s[idx..].to_string();
    }
    if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
        return name.to_string();
    }
    s
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::merger::{ClassifiedRoleSnapshot, PlannedChange};
    use crate::role_inference::Role;

    fn change(action: PlannedAction) -> PlannedChange {
        PlannedChange {
            path: PathBuf::from(".claude/agents/frontend.md"),
            role: ClassifiedRoleSnapshot::Known(Role::Frontend),
            before: "old\n".into(),
            after: "new\n".into(),
            action,
        }
    }

    #[test]
    fn summary_includes_each_action_glyph() {
        let plan = MergePlan {
            changes: vec![
                change(PlannedAction::Inject),
                change(PlannedAction::Replace),
                change(PlannedAction::Remove),
                change(PlannedAction::Skip("ok")),
                change(PlannedAction::Refuse("nope".into())),
                change(PlannedAction::NoFenceToRemove),
            ],
        };
        let s = summary(&plan);
        for glyph in ["[+]", "[~]", "[-]", "[=]", "[!]", "[ ]"] {
            assert!(s.contains(glyph), "missing glyph {glyph}: {s}");
        }
    }

    #[test]
    fn unified_diff_only_includes_disk_changing_actions() {
        let plan = MergePlan {
            changes: vec![
                change(PlannedAction::Inject),
                change(PlannedAction::Skip("up to date")),
            ],
        };
        let d = unified_diff(&plan);
        assert!(d.contains("---"));
        assert!(d.contains("-old"));
        assert!(d.contains("+new"));
        // Skip wasn't included
        assert_eq!(d.matches("---").count(), 1);
    }

    #[test]
    fn counts_reflect_actions() {
        let plan = MergePlan {
            changes: vec![
                change(PlannedAction::Inject),
                change(PlannedAction::Inject),
                change(PlannedAction::Replace),
                change(PlannedAction::Skip("ok")),
            ],
        };
        let c = counts(&plan);
        assert_eq!(c.inject, 2);
        assert_eq!(c.replace, 1);
        assert_eq!(c.skip, 1);
        assert_eq!(c.remove, 0);
    }
}
