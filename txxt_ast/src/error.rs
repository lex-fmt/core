//! Error types for the AST module

use std::fmt;

/// Error type for position-based AST lookups
#[derive(Debug, Clone, PartialEq)]
pub enum PositionLookupError {
    /// The position format is invalid or missing required parameters
    InvalidPositionFormat(String),
    /// No element found at the given position
    NotFound(String),
}

impl fmt::Display for PositionLookupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PositionLookupError::InvalidPositionFormat(msg) => {
                write!(f, "Invalid position format: {}", msg)
            }
            PositionLookupError::NotFound(msg) => write!(f, "Not found: {}", msg),
        }
    }
}

impl std::error::Error for PositionLookupError {}
