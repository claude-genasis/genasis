//! Workspace-level integration test: marker fence idempotency.
//!
//! Exercises the public `marker` API end-to-end on representative agent file
//! shapes (with frontmatter, without frontmatter, with existing GENASIS
//! fence, with embedded HTML comments that are *not* GENASIS fences).

use genasis_core::marker::{find, remove, upsert, Fence};

fn fence(role: &str, body: &str) -> Fence {
    Fence::new(role, "1.0", body)
}

#[test]
fn upsert_inject_then_replace_then_remove_chain() {
    let original = "---\nname: frontend\ntools: Bash, Read\n---\n\n# Frontend Agent\n\nBody.\n";

    // Inject
    let after_inject = upsert(original, &fence("frontend", "BODY V1")).unwrap();
    assert!(find(&after_inject).unwrap().is_some());

    // Replace (different body)
    let after_replace = upsert(&after_inject, &fence("frontend", "BODY V2")).unwrap();
    let (parsed, _) = find(&after_replace).unwrap().unwrap();
    assert_eq!(parsed.body, "BODY V2");

    // Re-upsert with the same fence is a no-op
    let after_idempotent = upsert(&after_replace, &fence("frontend", "BODY V2")).unwrap();
    assert_eq!(after_replace, after_idempotent);

    // Remove brings us back to within a trailing-newline of the original
    let stripped = remove(&after_idempotent).unwrap();
    assert_eq!(stripped, original, "remove must restore the original");
}

#[test]
fn html_comments_outside_fence_are_preserved() {
    let original = "---\nname: planner\n---\n\n<!-- some unrelated HTML comment -->\n\n# Planner\n";
    let with_fence = upsert(original, &fence("planner", "scrum body")).unwrap();
    assert!(with_fence.contains("<!-- some unrelated HTML comment -->"));
    let stripped = remove(&with_fence).unwrap();
    assert_eq!(stripped, original);
}

#[test]
fn no_frontmatter_files_supported() {
    let original = "# Agent\n\nbody only\n";
    let with_fence = upsert(original, &fence("custom", "x")).unwrap();
    assert!(with_fence.starts_with("<!-- GENASIS:BEGIN"));
    assert!(with_fence.ends_with("# Agent\n\nbody only\n"));
}

#[test]
fn version_bump_changes_only_fence_metadata_and_body() {
    let original = "---\nname: backend\n---\n\nbody\n";
    let v1 = upsert(original, &Fence::new("backend", "1.0", "old")).unwrap();
    let v2 = upsert(&v1, &Fence::new("backend", "1.1", "new")).unwrap();
    assert!(v2.contains("version=1.1"));
    assert!(v2.contains("new"));
    assert!(!v2.contains("old"));
    assert!(v2.ends_with("body\n"));
}
