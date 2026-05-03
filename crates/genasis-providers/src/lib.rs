//! genasis-providers — adapters for external collaboration systems.
//!
//! Plane and Mattermost both ship with a flavor system (upstream / custom /
//! auto). See blueprint.md §5 (Flavor 시스템).

pub mod github;
pub mod mattermost;
pub mod plane;

pub use mattermost::MattermostProvider;
pub use plane::PlaneProvider;
