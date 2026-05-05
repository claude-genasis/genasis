//! Green-field bootstrap — scaffold canonical agent files when
//! `.claude/agents/` is empty (or partially populated).
//!
//! See ADR-010. This is a separate stage from `attach`: bootstrap creates
//! the *base* file (frontmatter + role header) when it is missing, and
//! `attach` then injects the patch fence into it. The two layers have
//! distinct ownership:
//!
//! | Layer | Owner | Lifecycle |
//! |---|---|---|
//! | Base   | user    | one-shot emit; user-editable thereafter |
//! | Patch  | genasis | rewritten by attach/upgrade via hash diff |
//!
//! Default behaviour is **opt-in** — callers (e.g. `genasis bootstrap`,
//! `genasis init --bootstrap`) explicitly invoke `plan_bootstrap`.
//! `attach` does not auto-call this when `.claude/agents/` is empty.
//!
//! Wired in M14.2.

use std::path::{Path, PathBuf};

use genasis_core::error::{Error, Result};
use genasis_core::fs as gfs;
use tera::Tera;

use crate::role_inference::Role;

/// Inputs that drive base-file scaffolding.
pub struct BootstrapOptions {
    /// BCP-47 locale code (`"en"` / `"ko"`). Picks the
    /// `templates/<lang>/agents/` subtree.
    pub lang: String,
    /// Roles to scaffold. Defaults to [`Role::ALL`].
    pub roles: Vec<Role>,
    /// JSON object passed to every Tera render. Empty by default — the
    /// base templates do not currently use any context variables, but
    /// the channel is here for future thickening (e.g. project_name in
    /// the role header).
    pub context: serde_json::Value,
}

impl Default for BootstrapOptions {
    fn default() -> Self {
        Self {
            lang: "en".to_string(),
            roles: Role::ALL.to_vec(),
            context: serde_json::json!({}),
        }
    }
}

impl BootstrapOptions {
    pub fn new(lang: impl Into<String>) -> Self {
        Self {
            lang: lang.into(),
            ..Default::default()
        }
    }

    pub fn with_roles(mut self, roles: Vec<Role>) -> Self {
        self.roles = roles;
        self
    }

    pub fn with_context(mut self, ctx: serde_json::Value) -> Self {
        self.context = ctx;
        self
    }
}

/// One per-role scaffolding decision.
#[derive(Debug, Clone)]
pub struct BootstrapChange {
    pub role: Role,
    pub path: PathBuf,
    pub action: BootstrapAction,
}

#[derive(Debug, Clone)]
pub enum BootstrapAction {
    /// File missing → render and write `body`.
    Create { body: String },
    /// File already exists → leave it alone.
    Skip { reason: &'static str },
}

#[derive(Debug, Clone, Default)]
pub struct BootstrapPlan {
    pub changes: Vec<BootstrapChange>,
}

impl BootstrapPlan {
    /// Iterator over changes that will write to disk.
    pub fn creates(&self) -> impl Iterator<Item = &BootstrapChange> {
        self.changes
            .iter()
            .filter(|c| matches!(c.action, BootstrapAction::Create { .. }))
    }

    pub fn skips(&self) -> impl Iterator<Item = &BootstrapChange> {
        self.changes
            .iter()
            .filter(|c| matches!(c.action, BootstrapAction::Skip { .. }))
    }
}

#[derive(Debug, Clone, Default)]
pub struct BootstrapReport {
    pub written: Vec<PathBuf>,
}

/// Plan a bootstrap pass. Pure — checks file existence only, no writes.
pub fn plan_bootstrap(project_root: &Path, opts: &BootstrapOptions) -> Result<BootstrapPlan> {
    let agents_dir = project_root.join(".claude").join("agents");
    let tera = build_tera(&opts.lang)?;
    let ctx = tera::Context::from_value(opts.context.clone())
        .map_err(|e| Error::Overlay(format!("tera context: {e}")))?;

    let mut changes = Vec::with_capacity(opts.roles.len());
    for role in &opts.roles {
        let slug = role.slug();
        let path = agents_dir.join(format!("{slug}.md"));
        let action = if path.exists() {
            BootstrapAction::Skip { reason: "exists" }
        } else {
            let body = render_base(&tera, slug, &ctx)?;
            BootstrapAction::Create { body }
        };
        changes.push(BootstrapChange {
            role: *role,
            path,
            action,
        });
    }
    Ok(BootstrapPlan { changes })
}

/// Apply a bootstrap plan. `Skip` actions are no-ops; `Create` writes the
/// rendered body via [`gfs::atomic_write`] (which auto-creates the
/// parent directory).
pub fn apply_bootstrap(plan: &BootstrapPlan) -> Result<BootstrapReport> {
    let mut written = Vec::new();
    for change in &plan.changes {
        if let BootstrapAction::Create { body } = &change.action {
            gfs::atomic_write(&change.path, body.as_bytes())?;
            written.push(change.path.clone());
        }
    }
    Ok(BootstrapReport { written })
}

fn build_tera(lang: &str) -> Result<Tera> {
    let mut tera = Tera::default();
    let dir_path = format!("{lang}/agents");
    let agents_dir = genasis_templates::TEMPLATES
        .get_dir(&dir_path)
        .ok_or_else(|| {
            Error::Overlay(format!(
                "templates/{lang}/agents missing from binary (M14.1 not yet shipped?)"
            ))
        })?;
    for file in agents_dir.files() {
        let name = file
            .path()
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if !name.ends_with(".md.tera") {
            continue;
        }
        let body = file
            .contents_utf8()
            .ok_or_else(|| Error::Overlay(format!("non-utf8 base template: {name}")))?;
        tera.add_raw_template(name, body)
            .map_err(|e| Error::Overlay(format!("tera add {name}: {e}")))?;
    }
    Ok(tera)
}

fn render_base(tera: &Tera, role_slug: &str, ctx: &tera::Context) -> Result<String> {
    let template_name = format!("{role_slug}.md.tera");
    if tera.get_template_names().all(|n| n != template_name) {
        return Err(Error::Overlay(format!(
            "no base template for role {role_slug} — missing from templates/{{lang}}/agents/"
        )));
    }
    let mut rendered = tera
        .render(&template_name, ctx)
        .map_err(|e| Error::Overlay(format!("tera render base {role_slug}: {e}")))?;
    if !rendered.ends_with('\n') {
        rendered.push('\n');
    }
    Ok(rendered)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn empty_project_creates_all_ten_roles() {
        let d = tempdir().unwrap();
        let plan = plan_bootstrap(d.path(), &BootstrapOptions::new("en")).unwrap();
        assert_eq!(plan.changes.len(), 10);
        assert_eq!(plan.creates().count(), 10);
        assert_eq!(plan.skips().count(), 0);
    }

    #[test]
    fn existing_files_are_skipped() {
        let d = tempdir().unwrap();
        let agents = d.path().join(".claude/agents");
        fs::create_dir_all(&agents).unwrap();
        // Pre-populate two role files; bootstrap should skip them.
        fs::write(
            agents.join("frontend.md"),
            "---\nname: frontend\n---\n# user-authored\n",
        )
        .unwrap();
        fs::write(
            agents.join("backend.md"),
            "---\nname: backend\n---\n# user-authored\n",
        )
        .unwrap();
        let plan = plan_bootstrap(d.path(), &BootstrapOptions::new("en")).unwrap();
        assert_eq!(plan.creates().count(), 8);
        let skipped: Vec<_> = plan.skips().map(|c| c.role).collect();
        assert!(skipped.contains(&Role::Frontend));
        assert!(skipped.contains(&Role::Backend));
    }

    #[test]
    fn apply_writes_only_create_actions() {
        let d = tempdir().unwrap();
        let plan = plan_bootstrap(d.path(), &BootstrapOptions::new("en")).unwrap();
        let report = apply_bootstrap(&plan).unwrap();
        assert_eq!(report.written.len(), 10);
        // All 10 files now exist on disk.
        for role in Role::ALL {
            let p = d.path().join(format!(".claude/agents/{}.md", role.slug()));
            assert!(p.exists(), "missing {}", p.display());
        }
    }

    #[test]
    fn rendered_base_carries_required_frontmatter_keys() {
        // Every base template must declare name/description/tools/model/color.
        let d = tempdir().unwrap();
        let plan = plan_bootstrap(d.path(), &BootstrapOptions::new("en")).unwrap();
        for change in plan.creates() {
            let body = match &change.action {
                BootstrapAction::Create { body } => body,
                _ => unreachable!(),
            };
            assert!(body.starts_with("---\n"), "missing frontmatter open: {}", change.role.slug());
            for key in ["name:", "description:", "tools:", "model:", "color:"] {
                assert!(
                    body.contains(key),
                    "{} base missing key {key}",
                    change.role.slug()
                );
            }
            // name: <slug> must match the file stem, so detector classifies as Known(_).
            let expected_name = format!("name: {}", change.role.slug());
            assert!(
                body.contains(&expected_name),
                "{} base name does not match stem (expected `{expected_name}`)",
                change.role.slug()
            );
        }
    }

    #[test]
    fn korean_locale_subtree_loads() {
        let d = tempdir().unwrap();
        let plan = plan_bootstrap(d.path(), &BootstrapOptions::new("ko")).unwrap();
        assert_eq!(plan.creates().count(), 10);
    }

    #[test]
    fn unknown_locale_errors() {
        let d = tempdir().unwrap();
        let err = plan_bootstrap(d.path(), &BootstrapOptions::new("xx")).unwrap_err();
        assert!(format!("{err:?}").contains("agents missing"));
    }

    #[test]
    fn role_subset_only_plans_chosen_roles() {
        let d = tempdir().unwrap();
        let opts = BootstrapOptions::new("en").with_roles(vec![Role::Frontend, Role::Backend]);
        let plan = plan_bootstrap(d.path(), &opts).unwrap();
        assert_eq!(plan.changes.len(), 2);
        assert_eq!(plan.creates().count(), 2);
    }

    #[test]
    fn idempotent_second_apply_is_a_noop() {
        let d = tempdir().unwrap();
        let plan1 = plan_bootstrap(d.path(), &BootstrapOptions::new("en")).unwrap();
        apply_bootstrap(&plan1).unwrap();
        let plan2 = plan_bootstrap(d.path(), &BootstrapOptions::new("en")).unwrap();
        assert_eq!(plan2.creates().count(), 0);
        assert_eq!(plan2.skips().count(), 10);
        let report = apply_bootstrap(&plan2).unwrap();
        assert!(report.written.is_empty());
    }
}
