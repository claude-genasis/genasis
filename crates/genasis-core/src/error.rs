//! Shared error type for the Genasis workspace.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("not implemented yet: {0}")]
    NotImplemented(&'static str),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("toml parse: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("json: {0}")]
    Json(#[from] serde_json::Error),

    #[error("config: {0}")]
    Config(String),

    #[error("overlay: {0}")]
    Overlay(String),

    #[error("provider: {0}")]
    Provider(String),

    #[error("db: {0}")]
    Db(String),
}

pub type Result<T> = std::result::Result<T, Error>;
