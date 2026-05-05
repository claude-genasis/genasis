//! genasis-overlay — non-destructive overlay engine.
//!
//! See blueprint.md §3 (Marker Fence Spec) and progress.md M2 for the
//! milestone that fills these modules in. M0 only declares the surface so the
//! workspace builds.

pub mod bootstrap;
pub mod detector;
pub mod dry_run;
pub mod merger;
pub mod role_inference;
pub mod validator;

pub use bootstrap::{
    apply_bootstrap, plan_bootstrap, BootstrapAction, BootstrapChange, BootstrapOptions,
    BootstrapPlan, BootstrapReport,
};
pub use detector::{scan, DetectedAgent, DetectionReport};
pub use dry_run::{counts as plan_counts, summary, unified_diff, Counts};
pub use merger::{
    apply, plan_attach, plan_detach, AppliedReport, AttachOptions, MergePlan, PlannedAction,
    PlannedChange,
};
pub use role_inference::{infer_from_name, Classified, Role};
pub use validator::{decide, inspect, FenceState, WriteDecision};
