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

use crate::role_inference::Role;

/// Inputs that drive base-file scaffolding.
///
/// ADR-011: base agents are plain .md files read directly from the
/// AgentStore — no Tera rendering needed. The `lang` field is no longer
/// used for base file selection (base agents are language-neutral),
/// but kept for consistency with the overlay stage that follows.
pub struct BootstrapOptions {
    /// BCP-47 locale code — used by the subsequent `attach` stage for
    /// overlay rendering, not for base file selection.
    pub lang: String,
    /// Roles to scaffold. Defaults to [`Role::ALL`].
    pub roles: Vec<Role>,
}

impl Default for BootstrapOptions {
    fn default() -> Self {
        Self {
            lang: "en".to_string(),
            roles: Role::ALL.to_vec(),
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
///
/// ADR-011: `store` is the loaded agents catalog from disk cache.
/// Base agent .md files are read from `store.get_file("base/{role}.md")`.
pub fn plan_bootstrap(
    project_root: &Path,
    opts: &BootstrapOptions,
    store: &genasis_templates::AgentStore,
) -> Result<BootstrapPlan> {
    let agents_dir = project_root.join(".claude").join("agents");

    let mut changes = Vec::with_capacity(opts.roles.len());
    for role in &opts.roles {
        let slug = role.slug();
        let path = agents_dir.join(format!("{slug}.md"));
        let action = if path.exists() {
            BootstrapAction::Skip { reason: "exists" }
        } else {
            // ADR-011: read plain .md from catalog (no Tera rendering needed).
            let base_path = format!("base/{slug}.md");
            let body = store.get_file(&base_path).ok_or_else(|| {
                Error::Overlay(format!(
                    "base agent template missing from catalog: {base_path}"
                ))
            })?;
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

// ADR-011: build_tera / render_base removed — base agents are now plain .md
// files read directly from AgentStore. No Tera rendering needed for base files.

#[cfg(test)]
mod tests {
    use super::*;
    use genasis_templates::AgentStore;
    use std::fs;
    use tempfile::tempdir;

    /// Create a mock AgentStore with base .md files for all 10 roles.
    fn mock_store() -> (tempfile::TempDir, AgentStore) {
        let catalog = tempdir().unwrap();
        let base = catalog.path().join("base");
        fs::create_dir_all(&base).unwrap();
        fs::write(
            catalog.path().join("manifest.json"),
            r#"{"version":"0.0.1-test","roles":[]}"#,
        )
        .unwrap();
        for role in Role::ALL {
            let slug = role.slug();
            let content = format!(
                "---\nname: {slug}\ndescription: test {slug}\ntools: Read\nmodel: sonnet\ncolor: gray\n---\n\n# {slug} Agent\n\nTest base file.\n"
            );
            fs::write(base.join(format!("{slug}.md")), &content).unwrap();
        }
        let store = AgentStore::from_dir(catalog.path().to_path_buf()).unwrap();
        (catalog, store)
    }

    #[test]
    fn empty_project_creates_all_ten_roles() {
        let d = tempdir().unwrap();
        let (_cat, store) = mock_store();
        let plan = plan_bootstrap(d.path(), &BootstrapOptions::default(), &store).unwrap();
        assert_eq!(plan.changes.len(), 10);
        assert_eq!(plan.creates().count(), 10);
        assert_eq!(plan.skips().count(), 0);
    }

    #[test]
    fn existing_files_are_skipped() {
        let d = tempdir().unwrap();
        let agents = d.path().join(".claude/agents");
        fs::create_dir_all(&agents).unwrap();
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
        let (_cat, store) = mock_store();
        let plan = plan_bootstrap(d.path(), &BootstrapOptions::default(), &store).unwrap();
        assert_eq!(plan.creates().count(), 8);
        let skipped: Vec<_> = plan.skips().map(|c| c.role).collect();
        assert!(skipped.contains(&Role::Frontend));
        assert!(skipped.contains(&Role::Backend));
    }

    #[test]
    fn apply_writes_only_create_actions() {
        let d = tempdir().unwrap();
        let (_cat, store) = mock_store();
        let plan = plan_bootstrap(d.path(), &BootstrapOptions::default(), &store).unwrap();
        let report = apply_bootstrap(&plan).unwrap();
        assert_eq!(report.written.len(), 10);
        for role in Role::ALL {
            let p = d.path().join(format!(".claude/agents/{}.md", role.slug()));
            assert!(p.exists(), "missing {}", p.display());
        }
    }

    #[test]
    fn base_carries_required_frontmatter_keys() {
        let d = tempdir().unwrap();
        let (_cat, store) = mock_store();
        let plan = plan_bootstrap(d.path(), &BootstrapOptions::default(), &store).unwrap();
        for change in plan.creates() {
            let body = match &change.action {
                BootstrapAction::Create { body } => body,
                _ => unreachable!(),
            };
            assert!(body.starts_with("---\n"), "missing frontmatter: {}", change.role.slug());
            for key in ["name:", "description:", "tools:", "model:", "color:"] {
                assert!(body.contains(key), "{} missing {key}", change.role.slug());
            }
        }
    }

    #[test]
    fn role_subset_only_plans_chosen_roles() {
        let d = tempdir().unwrap();
        let (_cat, store) = mock_store();
        let opts = BootstrapOptions::default().with_roles(vec![Role::Frontend, Role::Backend]);
        let plan = plan_bootstrap(d.path(), &opts, &store).unwrap();
        assert_eq!(plan.changes.len(), 2);
        assert_eq!(plan.creates().count(), 2);
    }

    #[test]
    fn idempotent_second_apply_is_noop() {
        let d = tempdir().unwrap();
        let (_cat, store) = mock_store();
        let plan1 = plan_bootstrap(d.path(), &BootstrapOptions::default(), &store).unwrap();
        apply_bootstrap(&plan1).unwrap();
        let plan2 = plan_bootstrap(d.path(), &BootstrapOptions::default(), &store).unwrap();
        assert_eq!(plan2.creates().count(), 0);
        assert_eq!(plan2.skips().count(), 10);
    }

    #[test]
    fn missing_base_file_in_store_errors() {
        let catalog = tempdir().unwrap();
        fs::create_dir_all(catalog.path().join("base")).unwrap();
        fs::write(
            catalog.path().join("manifest.json"),
            r#"{"version":"0.0.1"}"#,
        )
        .unwrap();
        // Only write one role — the rest will be missing.
        fs::write(
            catalog.path().join("base/pm.md"),
            "---\nname: pm\n---\n# PM\n",
        )
        .unwrap();
        let store = AgentStore::from_dir(catalog.path().to_path_buf()).unwrap();

        let d = tempdir().unwrap();
        let err = plan_bootstrap(d.path(), &BootstrapOptions::default(), &store).unwrap_err();
        assert!(format!("{err:?}").contains("missing from catalog"));
    }
}
