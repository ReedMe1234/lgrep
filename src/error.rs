//! Custom error types for lgrep
//!
//! Uses thiserror for ergonomic error definitions with automatic
//! Display and Error trait implementations.

use thiserror::Error;

/// Application-specific errors for lgrep
#[derive(Error, Debug)]
pub enum LgrepError {
    /// IO operations failed
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Embedding model failed to load or embed
    #[error("Embedding error: {0}")]
    Embedding(String),

    /// Vector index operations failed
    #[error("Index error: {0}")]
    Index(String),

    /// Serialization/deserialization failed
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    /// JSON parsing failed
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// No index exists at the expected location
    #[error("No index found. Run `lgrep index` first.")]
    NoIndex,

    /// Invalid file or directory path
    #[error("Invalid path: {0}")]
    InvalidPath(String),

    /// File watcher errors
    #[error("Watch error: {0}")]
    Watch(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, LgrepError>;
