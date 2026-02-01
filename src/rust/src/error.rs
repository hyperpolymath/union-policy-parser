// SPDX-License-Identifier: PMPL-1.0-or-later
//! Error types for union-policy-parser

use thiserror::Error;
use std::path::PathBuf;

#[derive(Error, Debug)]
pub enum PolicyError {
    #[error("Failed to parse A2ML file: {0}")]
    ParseError(String),

    #[error("Validation failed: {0}")]
    ValidationError(String),

    #[error("Missing required clause: {0}")]
    MissingClause(String),

    #[error("Invalid clause value for '{clause}': expected {expected}, got {actual}")]
    InvalidClauseValue {
        clause: String,
        expected: String,
        actual: String,
    },

    #[error("Schema error: {0}")]
    SchemaError(String),

    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Template error: {0}")]
    TemplateError(String),

    #[error("Unknown union: {0}")]
    UnknownUnion(String),
}

pub type Result<T> = std::result::Result<T, PolicyError>;
