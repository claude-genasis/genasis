//! genasis-templates — Agents catalog: fetch, cache, and load.
//!
//! ADR-011: Templates/agents are NO LONGER embedded in the binary via
//! `include_dir!()`. Instead they are distributed as GitHub Release
//! tarballs (`agents-v1.x.tar.gz`) and fetched at runtime.
//!
//! The catalog is cached at `~/.cache/genasis/agents/v{version}/` and
//! loaded from disk when needed. The version is pinned in
//! `genasis.toml [agents].version`.

pub mod cache;
pub mod registry;
pub mod store;

pub use cache::{cache_dir, is_cached, list_cached, remove_cached, store_tarball};
pub use registry::{check_latest, fetch_tarball};
pub use store::{load, AgentStore};

/// Locale subtrees supported by overlays.
pub const SUPPORTED_LANGS: &[&str] = &["en", "ko"];
