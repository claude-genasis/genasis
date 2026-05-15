//! genasis-core — shared primitives for the Genasis workspace.
//!
//! Modules are intentionally small: `config` (genasis.toml schema), `env`
//! (.env.agents), `fs` (atomic write/backup), `marker` (overlay fence), and
//! `error` (shared error type). M1 fills these in; M0 only stubs the surface.

pub mod config;
pub mod env;
pub mod error;
pub mod frontmatter;
pub mod fs;
pub mod manifest;
pub mod marker;
pub mod slug;

pub use error::{Error, Result};
