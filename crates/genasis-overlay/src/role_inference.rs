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
}

/// Either a known role or a custom name (preserved verbatim).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Classified {
    Known(Role),
    Custom(String),
}

/// Best-effort role inference from the `name:` frontmatter field.
pub fn infer_from_name(name: &str) -> Classified {
    let n = name.trim().to_ascii_lowercase();
    match n.as_str() {
        "pm" | "scrum-master" | "product-manager" | "scrum_master" | "scrummaster" => {
            Classified::Known(Role::Pm)
        }
        "planner" | "plan" => Classified::Known(Role::Planner),
        "architect" | "system-architect" => Classified::Known(Role::Architect),
        "frontend" | "fe" | "web" | "ui" => Classified::Known(Role::Frontend),
        "backend" | "be" | "api" => Classified::Known(Role::Backend),
        "qa" | "tester" | "e2e-runner" | "e2e_runner" => Classified::Known(Role::Qa),
        "designer" | "ux" | "ui-designer" => Classified::Known(Role::Designer),
        "security" | "sec" | "security-reviewer" | "security_reviewer" => {
            Classified::Known(Role::Security)
        }
        "devops" | "infra" | "platform" => Classified::Known(Role::Devops),
        "code-reviewer" | "reviewer" | "code_reviewer" => Classified::Known(Role::CodeReviewer),
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
}
