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

use genasis_core::error::Result;
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
    /// File missing on disk → render and write `body`. `source_alias`
    /// records which entry from [`Role::aliases`](crate::Role::aliases)
    /// resolved to a catalog file (often the canonical slug, but may
    /// be a field-observed alias like `frontend-developer` when the
    /// canonical `frontend.md` isn't shipped — see ADR-011 v1.0.0
    /// catalog and the `real_catalog_sim` integration test).
    Create { body: String, source_alias: String },
    /// File already exists → leave it alone.
    Skip { reason: &'static str },
    /// No alias from [`Role::aliases`](crate::Role::aliases) matched
    /// any catalog file for this role. Bootstrap **does not abort** —
    /// the remaining roles still get planned, and the user sees a
    /// warning listing what was tried. This makes a partial catalog
    /// (e.g. v1.0.0 missing `pm.md`) usable instead of fatal.
    Missing { tried: Vec<String> },
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

    /// Iterator over roles for which no catalog alias resolved.
    /// `apply_bootstrap` skips these silently; CLI callers surface
    /// them as warnings so the user can patch the catalog or
    /// hand-author the base file.
    pub fn missing(&self) -> impl Iterator<Item = &BootstrapChange> {
        self.changes
            .iter()
            .filter(|c| matches!(c.action, BootstrapAction::Missing { .. }))
    }
}

#[derive(Debug, Clone, Default)]
pub struct BootstrapReport {
    pub written: Vec<PathBuf>,
}

/// Plan a bootstrap pass. Pure — checks file existence only, no writes.
///
/// ADR-011: `store` is the loaded agents catalog from disk cache. Base
/// agent `.md` files are read from `store.get_file("base/{alias}.md")`
/// where `alias` comes from [`Role::aliases`](crate::Role::aliases) —
/// canonical slug first, field-observed aliases after. First match
/// wins; if none match, the role is recorded as
/// [`BootstrapAction::Missing`] instead of aborting the whole plan.
/// This lets a partial catalog (e.g. v1.0.0 missing `pm.md` but
/// shipping `product-manager.md`) still install what it can.
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
            resolve_via_aliases(*role, store)
        };
        changes.push(BootstrapChange {
            role: *role,
            path,
            action,
        });
    }
    Ok(BootstrapPlan { changes })
}

fn resolve_via_aliases(role: Role, store: &genasis_templates::AgentStore) -> BootstrapAction {
    let mut tried = Vec::new();
    for alias in role.aliases() {
        let base_path = format!("base/{alias}.md");
        tried.push((*alias).to_string());
        if let Some(body) = store.get_file(&base_path) {
            return BootstrapAction::Create {
                body,
                source_alias: (*alias).to_string(),
            };
        }
    }
    BootstrapAction::Missing { tried }
}

/// Apply a bootstrap plan. `Skip` and `Missing` actions are no-ops;
/// only `Create` writes the rendered body via [`gfs::atomic_write`]
/// (which auto-creates the parent directory). Missing roles are
/// expected to surface to the user via the CLI consumer's logging
/// path so they can be hand-authored or the catalog patched.
pub fn apply_bootstrap(plan: &BootstrapPlan) -> Result<BootstrapReport> {
    let mut written = Vec::new();
    for change in &plan.changes {
        if let BootstrapAction::Create { body, .. } = &change.action {
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
            let (body, source_alias) = match &change.action {
                BootstrapAction::Create { body, source_alias } => (body, source_alias),
                _ => unreachable!(),
            };
            assert!(
                body.starts_with("---\n"),
                "missing frontmatter: {}",
                change.role.slug()
            );
            for key in ["name:", "description:", "tools:", "model:", "color:"] {
                assert!(body.contains(key), "{} missing {key}", change.role.slug());
            }
            // Mock store writes files at the canonical slug, so the
            // first alias should always resolve and source_alias
            // must equal slug.
            assert_eq!(
                source_alias,
                change.role.slug(),
                "mock store should resolve via canonical slug"
            );
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
    fn partial_catalog_yields_missing_actions_not_error() {
        // ADR-017 §field-feedback: real users running v1.0.0 hit this
        // path — the catalog only ships some canonical slugs, the
        // rest are field aliases. Bootstrap must not abort; it should
        // emit Missing for unresolved roles and Create for resolved
        // ones.
        let catalog = tempdir().unwrap();
        fs::create_dir_all(catalog.path().join("base")).unwrap();
        fs::write(
            catalog.path().join("manifest.json"),
            r#"{"version":"0.0.1"}"#,
        )
        .unwrap();
        // Only write one role's base file; the rest are absent.
        fs::write(
            catalog.path().join("base/pm.md"),
            "---\nname: pm\n---\n# PM\n",
        )
        .unwrap();
        let store = AgentStore::from_dir(catalog.path().to_path_buf()).unwrap();

        let d = tempdir().unwrap();
        let plan = plan_bootstrap(d.path(), &BootstrapOptions::default(), &store).unwrap();

        assert_eq!(plan.changes.len(), Role::ALL.len());
        assert_eq!(plan.creates().count(), 1, "only pm.md resolves");
        assert_eq!(
            plan.missing().count(),
            Role::ALL.len() - 1,
            "every other role records what it tried"
        );

        // The Missing action carries the alias list so callers can
        // print a useful diagnostic.
        let frontend_missing = plan
            .missing()
            .find(|c| c.role == Role::Frontend)
            .expect("frontend is missing");
        match &frontend_missing.action {
            BootstrapAction::Missing { tried } => {
                assert!(tried.contains(&"frontend".to_string()));
                assert!(tried.contains(&"frontend-developer".to_string()));
            }
            other => panic!("expected Missing, got {other:?}"),
        }

        // apply_bootstrap must not write anything for Missing rows.
        let report = apply_bootstrap(&plan).unwrap();
        assert_eq!(report.written.len(), 1);
    }

    #[test]
    fn alias_walk_picks_field_observed_filename_when_slug_absent() {
        // v1.0.0-style catalog: no `frontend.md` but yes
        // `frontend-developer.md`. The alias walk must pick it up.
        let catalog = tempdir().unwrap();
        fs::create_dir_all(catalog.path().join("base")).unwrap();
        fs::write(
            catalog.path().join("manifest.json"),
            r#"{"version":"0.0.1"}"#,
        )
        .unwrap();
        fs::write(
            catalog.path().join("base/frontend-developer.md"),
            "---\nname: frontend-developer\n---\n# FE Dev\n",
        )
        .unwrap();
        let store = AgentStore::from_dir(catalog.path().to_path_buf()).unwrap();

        let d = tempdir().unwrap();
        let opts = BootstrapOptions::default().with_roles(vec![Role::Frontend]);
        let plan = plan_bootstrap(d.path(), &opts, &store).unwrap();

        assert_eq!(plan.changes.len(), 1);
        match &plan.changes[0].action {
            BootstrapAction::Create { source_alias, .. } => {
                assert_eq!(source_alias, "frontend-developer");
            }
            other => panic!("expected Create via alias, got {other:?}"),
        }
    }
}
