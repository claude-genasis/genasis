//! Orchestrate fence injection / replacement / removal across the agents
//! produced by `detector::scan`.
//!
//! The merger is split into a **planning phase** (pure, no IO) and an
//! **apply phase** (writes via `genasis-core::fs::atomic_write` after
//! `snapshot`).
//!
//! Planning emits a [`MergePlan`] consisting of one [`PlannedChange`] per
//! file. Callers (cmd_attach / cmd_detach / cmd_upgrade) can render the plan
//! as a dry-run diff (see [`crate::dry_run`]) and then optionally apply it.

use std::path::PathBuf;

use genasis_core::error::{Error, Result};
use genasis_core::fs as gfs;
use genasis_core::marker::{remove as fence_remove, upsert as fence_upsert, Fence};
use tera::Tera;

use crate::detector::DetectedAgent;
use crate::role_inference::{Classified, Role};
use crate::validator::{decide, inspect, WriteDecision};

/// One per-file change the merger intends to make.
#[derive(Debug, Clone)]
pub struct PlannedChange {
    pub path: PathBuf,
    pub role: ClassifiedRoleSnapshot,
    pub before: String,
    pub after: String,
    pub action: PlannedAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlannedAction {
    /// Inject a new fence (no fence existed).
    Inject,
    /// Replace an existing outdated fence.
    Replace,
    /// Skip — already up to date.
    Skip(&'static str),
    /// Refuse — surface a warning and leave the file untouched.
    Refuse(String),
    /// Remove an existing fence (used by `detach`).
    Remove,
    /// No fence exists; nothing to remove (used by `detach`).
    NoFenceToRemove,
}

#[derive(Debug, Clone)]
pub enum ClassifiedRoleSnapshot {
    Known(Role),
    Custom(String),
}

impl From<&Classified> for ClassifiedRoleSnapshot {
    fn from(c: &Classified) -> Self {
        match c {
            Classified::Known(r) => ClassifiedRoleSnapshot::Known(*r),
            Classified::Custom(s) => ClassifiedRoleSnapshot::Custom(s.clone()),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MergePlan {
    pub changes: Vec<PlannedChange>,
}

impl MergePlan {
    pub fn writable(&self) -> impl Iterator<Item = &PlannedChange> {
        self.changes.iter().filter(|c| {
            matches!(
                c.action,
                PlannedAction::Inject | PlannedAction::Replace | PlannedAction::Remove
            )
        })
    }

    pub fn refused(&self) -> impl Iterator<Item = &PlannedChange> {
        self.changes
            .iter()
            .filter(|c| matches!(c.action, PlannedAction::Refuse(_)))
    }
}

/// Inputs that drive overlay rendering.
///
/// Templates are looked up in [`genasis_templates::TEMPLATES`] under
/// `agent-overlays/<role-slug>.patch.md.tera`. The Tera context comes from
/// `context_json` so callers can inject project-specific values without this
/// crate knowing the schema.
pub struct AttachOptions {
    pub fence_version: String,
    /// JSON object of values made available to every template (e.g.
    /// `{"project_name": "demo", "plane_url": "..."}`).
    pub context: serde_json::Value,
    /// `--force` overrides Tampered / RoleMismatch refusals.
    pub force: bool,
}

impl AttachOptions {
    pub fn new(fence_version: impl Into<String>) -> Self {
        Self {
            fence_version: fence_version.into(),
            context: serde_json::json!({}),
            force: false,
        }
    }
}

/// Build a plan for `genasis attach`. Pure — no IO beyond what the detector
/// has already done.
pub fn plan_attach(agents: &[DetectedAgent], opts: &AttachOptions) -> Result<MergePlan> {
    let mut changes = Vec::with_capacity(agents.len());
    let tera = build_tera()?;

    for agent in agents {
        let role_slug = match &agent.classification {
            Classified::Known(r) => r.slug(),
            Classified::Custom(_) => {
                changes.push(PlannedChange {
                    path: agent.path.clone(),
                    role: (&agent.classification).into(),
                    before: agent.raw.clone(),
                    after: agent.raw.clone(),
                    action: PlannedAction::Skip("custom agent — no overlay template"),
                });
                continue;
            }
        };

        let body = match render_overlay(&tera, role_slug, &opts.context) {
            Ok(b) => b,
            Err(Error::Overlay(_)) => {
                // Template missing for this role → skip silently.
                changes.push(PlannedChange {
                    path: agent.path.clone(),
                    role: (&agent.classification).into(),
                    before: agent.raw.clone(),
                    after: agent.raw.clone(),
                    action: PlannedAction::Skip("no overlay template for role"),
                });
                continue;
            }
            Err(e) => return Err(e),
        };

        let proposed = Fence::new(role_slug, &opts.fence_version, body);
        let state = inspect(&agent.raw, &proposed)?;
        let decision = decide(&state, opts.force);

        let (after, action) = match decision {
            WriteDecision::Apply => {
                let updated = fence_upsert(&agent.raw, &proposed)?;
                if agent.has_existing_fence {
                    (updated, PlannedAction::Replace)
                } else {
                    (updated, PlannedAction::Inject)
                }
            }
            WriteDecision::Skip { reason } => (agent.raw.clone(), PlannedAction::Skip(reason)),
            WriteDecision::Refuse { reason } => (agent.raw.clone(), PlannedAction::Refuse(reason)),
        };

        changes.push(PlannedChange {
            path: agent.path.clone(),
            role: (&agent.classification).into(),
            before: agent.raw.clone(),
            after,
            action,
        });
    }

    Ok(MergePlan { changes })
}

/// Build a plan for `genasis detach`.
pub fn plan_detach(agents: &[DetectedAgent]) -> Result<MergePlan> {
    let mut changes = Vec::with_capacity(agents.len());
    for agent in agents {
        if !agent.has_existing_fence {
            changes.push(PlannedChange {
                path: agent.path.clone(),
                role: (&agent.classification).into(),
                before: agent.raw.clone(),
                after: agent.raw.clone(),
                action: PlannedAction::NoFenceToRemove,
            });
            continue;
        }
        let after = fence_remove(&agent.raw)?;
        changes.push(PlannedChange {
            path: agent.path.clone(),
            role: (&agent.classification).into(),
            before: agent.raw.clone(),
            after,
            action: PlannedAction::Remove,
        });
    }
    Ok(MergePlan { changes })
}

/// Apply a plan — snapshot then atomic write per file. Skipped/Refused/
/// NoFenceToRemove changes are no-ops on disk.
pub fn apply(plan: &MergePlan) -> Result<AppliedReport> {
    let mut written = Vec::new();
    let mut backups = Vec::new();
    for change in &plan.changes {
        match change.action {
            PlannedAction::Inject | PlannedAction::Replace | PlannedAction::Remove => {
                if let Some(b) = gfs::snapshot(&change.path)? {
                    backups.push(b);
                }
                gfs::atomic_write(&change.path, change.after.as_bytes())?;
                written.push(change.path.clone());
            }
            _ => {}
        }
    }
    Ok(AppliedReport { written, backups })
}

#[derive(Debug, Clone, Default)]
pub struct AppliedReport {
    pub written: Vec<PathBuf>,
    pub backups: Vec<PathBuf>,
}

fn build_tera() -> Result<Tera> {
    build_tera_lang("en")
}

/// Build a Tera bundle from `templates/<lang>/agent-overlays/`. Falls back to
/// the legacy flat path for compatibility with older callers.
pub fn build_tera_lang(lang: &str) -> Result<Tera> {
    let mut tera = Tera::default();
    let dir_path = format!("{lang}/agent-overlays");
    let overlays_dir = genasis_templates::TEMPLATES
        .get_dir(&dir_path)
        .or_else(|| genasis_templates::TEMPLATES.get_dir("agent-overlays"))
        .ok_or_else(|| {
            Error::Overlay(format!(
                "templates/{lang}/agent-overlays missing from binary"
            ))
        })?;
    for file in overlays_dir.files() {
        let name = file
            .path()
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if !name.ends_with(".patch.md.tera") {
            continue;
        }
        let body = file
            .contents_utf8()
            .ok_or_else(|| Error::Overlay(format!("non-utf8 template: {name}")))?;
        tera.add_raw_template(name, body)
            .map_err(|e| Error::Overlay(format!("tera add {name}: {e}")))?;
    }
    Ok(tera)
}

fn render_overlay(tera: &Tera, role_slug: &str, ctx: &serde_json::Value) -> Result<String> {
    let template_name = format!("{role_slug}.patch.md.tera");
    if tera.get_template_names().all(|n| n != template_name) {
        return Err(Error::Overlay(format!("no template for role {role_slug}")));
    }
    let context = tera::Context::from_value(ctx.clone())
        .map_err(|e| Error::Overlay(format!("tera context: {e}")))?;
    let mut rendered = tera
        .render(&template_name, &context)
        .map_err(|e| Error::Overlay(format!("tera render {role_slug}: {e}")))?;
    // Trim trailing newline so `Fence::render` controls newline placement.
    while rendered.ends_with('\n') {
        rendered.pop();
    }
    Ok(rendered)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detector::DetectedAgent;
    use crate::role_inference::Role;

    fn make_agent(role: Role, raw: &str) -> DetectedAgent {
        DetectedAgent {
            path: PathBuf::from(format!("/tmp/{}.md", role.slug())),
            name: role.slug().into(),
            classification: Classified::Known(role),
            raw: raw.into(),
            has_existing_fence: false,
        }
    }

    #[test]
    fn skip_when_role_is_custom() {
        // M6 added templates for all 10 ECC roles, so a Known(role) lookup
        // always finds a template. We exercise the Skip path with a Custom
        // classification (an agent file whose `name:` doesn't match any
        // built-in role).
        let agent = DetectedAgent {
            path: PathBuf::from("/tmp/loop-operator.md"),
            name: "loop-operator".into(),
            classification: Classified::Custom("loop-operator".into()),
            raw: "---\nname: loop-operator\n---\n".into(),
            has_existing_fence: false,
        };
        let plan = plan_attach(&[agent], &AttachOptions::new("1.0")).unwrap();
        assert_eq!(plan.changes.len(), 1);
        assert!(matches!(plan.changes[0].action, PlannedAction::Skip(_)));
    }

    #[test]
    fn detach_no_fence_is_noop() {
        let agents = vec![make_agent(
            Role::Frontend,
            "---\nname: frontend\n---\n# body\n",
        )];
        let plan = plan_detach(&agents).unwrap();
        assert!(matches!(
            plan.changes[0].action,
            PlannedAction::NoFenceToRemove
        ));
    }

    #[test]
    fn custom_agent_is_skipped() {
        let agent = DetectedAgent {
            path: PathBuf::from("/tmp/custom.md"),
            name: "loop-operator".into(),
            classification: Classified::Custom("loop-operator".into()),
            raw: "---\nname: loop-operator\n---\nbody\n".into(),
            has_existing_fence: false,
        };
        let plan = plan_attach(&[agent], &AttachOptions::new("1.0")).unwrap();
        match &plan.changes[0].action {
            PlannedAction::Skip(reason) => assert!(reason.contains("custom")),
            other => panic!("expected skip, got {other:?}"),
        }
    }
}
