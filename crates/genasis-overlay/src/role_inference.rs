//! Map an existing agent file to one of Genasis's known roles.
//!
//! Inference is driven by the YAML frontmatter `name` field with a small
//! synonym table. Files that don't match are classified as `Custom`.
//!
//! Wired into the detector in M2; M1 seeds the table and surface.

use serde::{Deserialize, Serialize};

/// All roles Genasis ships overlay templates for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Role {
    Pm,
    Planner,
    Architect,
    Frontend,
    Backend,
    Qa,
    Designer,
    Security,
    Devops,
    CodeReviewer,
}

impl Role {
    pub const ALL: &'static [Role] = &[
        Role::Pm,
        Role::Planner,
        Role::Architect,
        Role::Frontend,
        Role::Backend,
        Role::Qa,
        Role::Designer,
        Role::Security,
        Role::Devops,
        Role::CodeReviewer,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            Role::Pm => "pm",
            Role::Planner => "planner",
            Role::Architect => "architect",
            Role::Frontend => "frontend",
            Role::Backend => "backend",
            Role::Qa => "qa",
            Role::Designer => "designer",
            Role::Security => "security",
            Role::Devops => "devops",
            Role::CodeReviewer => "code-reviewer",
        }
    }

    /// Candidate base-agent filenames (without `.md` extension) to try
    /// when resolving this role against a catalog tarball, in priority
    /// order. The canonical [`slug`](Self::slug) is always first; the
    /// remaining entries cover community naming conventions actually
    /// observed in the v1.0.0 catalog shipped via GitHub Releases.
    ///
    /// Used by [`plan_bootstrap`](crate::plan_bootstrap) when the
    /// canonical slug is absent from the catalog (e.g. v1.0.0 ships
    /// `frontend-developer.md` rather than `frontend.md`). Walking the
    /// alias list lets bootstrap succeed against partial catalogs
    /// instead of failing hard on the first miss.
    pub fn aliases(self) -> &'static [&'static str] {
        match self {
            Role::Pm => &["pm", "product-manager", "scrum-master", "chief-of-staff"],
            Role::Planner => &["planner"],
            Role::Architect => &["architect", "backend-architect", "cloud-architect"],
            Role::Frontend => &["frontend", "frontend-developer"],
            Role::Backend => &["backend", "backend-developer", "backend-architect"],
            Role::Qa => &["qa", "qa-expert", "qa-coordinator", "tester"],
            Role::Designer => &["designer", "design-system-architect"],
            Role::Security => &[
                "security",
                "security-engineer",
                "security-auditor",
                "security-reviewer",
            ],
            Role::Devops => &["devops", "devops-engineer", "platform-engineer"],
            Role::CodeReviewer => &["code-reviewer", "reviewer", "architect-reviewer"],
        }
    }
}

/// Either a known role or a custom name (preserved verbatim).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Classified {
    Known(Role),
    Custom(String),
}

/// Best-effort role inference from the `name:` frontmatter field.
///
/// v0.5.3 issue 라: the v1.0.0 catalog ships base agent files whose
/// frontmatter `name:` reads `frontend-developer` /
/// `backend-developer` / `product-manager` / `qa-expert` etc. —
/// NOT the canonical short slug `Role::slug` returns. When
/// `cmd_bootstrap` copies those files into `.claude/agents/<slug>.md`
/// and `cmd_attach` infers their role from the frontmatter, the
/// long-form name landed in `Classified::Custom` and the overlay
/// fence was silently skipped. The alias list below mirrors
/// `Role::aliases()` so resolution is symmetric in both directions
/// — filename → role (bootstrap) AND name → role (attach).
pub fn infer_from_name(name: &str) -> Classified {
    let n = name.trim().to_ascii_lowercase();
    match n.as_str() {
        "pm" | "scrum-master" | "product-manager" | "scrum_master" | "scrummaster"
        | "chief-of-staff" => Classified::Known(Role::Pm),
        "planner" | "plan" => Classified::Known(Role::Planner),
        "architect" | "system-architect" | "backend-architect" | "cloud-architect" => {
            Classified::Known(Role::Architect)
        }
        "frontend" | "fe" | "web" | "ui" | "frontend-developer" => {
            Classified::Known(Role::Frontend)
        }
        "backend" | "be" | "api" | "backend-developer" => Classified::Known(Role::Backend),
        "qa" | "tester" | "e2e-runner" | "e2e_runner" | "qa-expert" | "qa-coordinator" => {
            Classified::Known(Role::Qa)
        }
        "designer" | "ux" | "ui-designer" | "design-system-architect" => {
            Classified::Known(Role::Designer)
        }
        "security" | "sec" | "security-reviewer" | "security_reviewer" | "security-engineer"
        | "security-auditor" => Classified::Known(Role::Security),
        "devops" | "infra" | "platform" | "devops-engineer" | "platform-engineer" => {
            Classified::Known(Role::Devops)
        }
        "code-reviewer" | "reviewer" | "code_reviewer" | "architect-reviewer" => {
            Classified::Known(Role::CodeReviewer)
        }
        _ => Classified::Custom(name.trim().to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_canonical_names() {
        assert_eq!(infer_from_name("pm"), Classified::Known(Role::Pm));
        assert_eq!(
            infer_from_name("Frontend"),
            Classified::Known(Role::Frontend)
        );
        assert_eq!(infer_from_name("e2e-runner"), Classified::Known(Role::Qa));
    }

    #[test]
    fn unknown_falls_through_to_custom() {
        match infer_from_name("loop-operator") {
            Classified::Custom(s) => assert_eq!(s, "loop-operator"),
            _ => panic!(),
        }
    }

    #[test]
    fn slug_round_trips_for_every_role() {
        for r in Role::ALL {
            let slug = r.slug();
            match infer_from_name(slug) {
                Classified::Known(r2) => assert_eq!(*r, r2, "slug {slug} did not round-trip"),
                Classified::Custom(_) => panic!("slug {slug} not recognised"),
            }
        }
    }

    #[test]
    fn aliases_first_entry_is_canonical_slug() {
        // The first alias is always the canonical slug — that keeps
        // tarballs shipped with `pm.md` etc. resolving via the most
        // obvious path, and only falls back when the slug isn't there.
        for r in Role::ALL {
            assert_eq!(
                r.aliases().first().copied(),
                Some(r.slug()),
                "Role::{r:?}.aliases() must start with slug() = {:?}",
                r.slug()
            );
        }
    }

    #[test]
    fn aliases_cover_v1_catalog_field_observations() {
        // Spot-check the field-observed filenames in the v1.0.0
        // tarball (which is what real users get from GitHub Releases).
        // If a future catalog refresh renames these, the alias list
        // here should be expanded — not narrowed.
        assert!(Role::Frontend.aliases().contains(&"frontend-developer"));
        assert!(Role::Backend.aliases().contains(&"backend-developer"));
        assert!(Role::Devops.aliases().contains(&"devops-engineer"));
        assert!(Role::Pm.aliases().contains(&"product-manager"));
        assert!(Role::Qa.aliases().contains(&"qa-expert"));
        assert!(Role::Security.aliases().contains(&"security-engineer"));
    }
}
