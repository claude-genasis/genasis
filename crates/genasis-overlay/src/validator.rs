//! Inspect an existing fence to decide whether it's safe to replace.
//!
//! There are three mutually exclusive states a Genasis-touched file can be in:
//!
//! 1. **Pristine**: a fence exists, its body matches its recorded hash,
//!    and the recorded version equals the version we want to write.
//!    → Skip; nothing to do.
//! 2. **Outdated**: a fence exists, its body matches its hash, but the
//!    recorded version differs from the version we want to write.
//!    → Safe to replace; the user has not edited Genasis-owned content.
//! 3. **Tampered**: a fence exists but the body does **not** match the
//!    recorded hash — a human has hand-edited Genasis-owned content.
//!    → Refuse to overwrite without `--force`.
//!
//! No fence at all is also a valid state (handled by the merger via `inject`).

use genasis_core::error::Result;
use genasis_core::marker::{find, Fence};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FenceState {
    /// No fence in the file at all.
    Absent,
    /// Existing fence is byte-identical to the proposed one (same hash, same version, same role).
    Pristine,
    /// Existing fence is intact but its version is different from the proposed.
    Outdated { existing_version: String },
    /// Body has been hand-edited; hash mismatch.
    Tampered { existing: Fence },
    /// Existing fence is for a different role than we're trying to write.
    /// This is a misconfiguration, not a tamper. Caller decides whether to refuse.
    RoleMismatch {
        existing_role: String,
        proposed_role: String,
    },
}

pub fn inspect(file_text: &str, proposed: &Fence) -> Result<FenceState> {
    let existing = match find(file_text)? {
        None => return Ok(FenceState::Absent),
        Some((f, _)) => f,
    };

    if existing.role != proposed.role {
        return Ok(FenceState::RoleMismatch {
            existing_role: existing.role,
            proposed_role: proposed.role.clone(),
        });
    }

    if !existing.body_matches_hash() {
        return Ok(FenceState::Tampered { existing });
    }

    if existing.version == proposed.version && existing.hash == proposed.hash {
        return Ok(FenceState::Pristine);
    }

    Ok(FenceState::Outdated {
        existing_version: existing.version,
    })
}

/// Decision for the merger: should we write this proposed fence?
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteDecision {
    /// Inject (no fence) or replace (existing fence safe to overwrite).
    Apply,
    /// Already up to date — leave the file untouched.
    Skip { reason: &'static str },
    /// Refuse to write; surface a warning to the user.
    Refuse { reason: String },
}

impl WriteDecision {
    pub fn is_apply(&self) -> bool {
        matches!(self, WriteDecision::Apply)
    }
}

/// Translate a [`FenceState`] into a write decision.
///
/// `force` (the `--force` flag) overrides the Tampered / RoleMismatch refusals.
pub fn decide(state: &FenceState, force: bool) -> WriteDecision {
    match state {
        FenceState::Absent | FenceState::Outdated { .. } => WriteDecision::Apply,
        FenceState::Pristine => WriteDecision::Skip {
            reason: "fence already up to date",
        },
        FenceState::Tampered { .. } => {
            if force {
                WriteDecision::Apply
            } else {
                WriteDecision::Refuse {
                    reason:
                        "fence body has been hand-edited (hash mismatch); pass --force to overwrite"
                            .into(),
                }
            }
        }
        FenceState::RoleMismatch {
            existing_role,
            proposed_role,
        } => {
            if force {
                WriteDecision::Apply
            } else {
                WriteDecision::Refuse {
                    reason: format!(
                        "existing fence is for role `{existing_role}` but we want `{proposed_role}`; pass --force to overwrite"
                    ),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use genasis_core::marker::upsert;

    fn proposed_v1() -> Fence {
        Fence::new("frontend", "1.0", "BODY V1")
    }

    #[test]
    fn absent_returns_absent() {
        let s = inspect("# nothing here\n", &proposed_v1()).unwrap();
        assert_eq!(s, FenceState::Absent);
        assert!(decide(&s, false).is_apply());
    }

    #[test]
    fn pristine_returns_pristine_and_skips() {
        let with_fence = upsert("---\nname: frontend\n---\n", &proposed_v1()).unwrap();
        let s = inspect(&with_fence, &proposed_v1()).unwrap();
        assert_eq!(s, FenceState::Pristine);
        assert!(matches!(decide(&s, false), WriteDecision::Skip { .. }));
    }

    #[test]
    fn outdated_version_is_safe_to_replace() {
        let with_v1 = upsert("---\nname: frontend\n---\n", &proposed_v1()).unwrap();
        let proposed_v2 = Fence::new("frontend", "2.0", "BODY V2");
        let s = inspect(&with_v1, &proposed_v2).unwrap();
        match s {
            FenceState::Outdated {
                ref existing_version,
            } => assert_eq!(existing_version, "1.0"),
            other => panic!("unexpected state: {other:?}"),
        }
        assert!(decide(&s, false).is_apply());
    }

    #[test]
    fn tampered_body_refuses_without_force() {
        let with_fence = upsert("---\nname: frontend\n---\n", &proposed_v1()).unwrap();
        // simulate a human edit inside the fence body
        let tampered = with_fence.replace("BODY V1", "BODY V1 + human edit");
        let s = inspect(&tampered, &proposed_v1()).unwrap();
        assert!(matches!(s, FenceState::Tampered { .. }));
        match decide(&s, false) {
            WriteDecision::Refuse { .. } => (),
            other => panic!("expected refuse, got {other:?}"),
        }
        assert!(decide(&s, true).is_apply());
    }

    #[test]
    fn role_mismatch_refuses_without_force() {
        let with_fence = upsert("---\nname: x\n---\n", &proposed_v1()).unwrap();
        let other_role = Fence::new("backend", "1.0", "x");
        let s = inspect(&with_fence, &other_role).unwrap();
        assert!(matches!(s, FenceState::RoleMismatch { .. }));
        assert!(matches!(decide(&s, false), WriteDecision::Refuse { .. }));
        assert!(decide(&s, true).is_apply());
    }
}
