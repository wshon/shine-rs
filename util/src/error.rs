//! Error types for utility functions
//!
//! This module defines error types used by utility functions.

use std::fmt;

/// Error type for utility operations
#[derive(Debug)]
pub enum UtilError {
    /// I/O operation failed
    IoError(std::io::Error),
    /// Validation error
    ValidationError(String),
}

impl fmt::Display for UtilError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UtilError::IoError(err) => write!(f, "I/O error: {}", err),
            UtilError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for UtilError {}

impl From<std::io::Error> for UtilError {
    fn from(err: std::io::Error) -> Self {
        UtilError::IoError(err)
    }
}

/// Result type for utility operations
pub type UtilResult<T> = std::result::Result<T, UtilError>;