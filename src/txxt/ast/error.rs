//! Error types for AST operations

use std::fmt;

/// Errors that can occur during AST position lookup operations
#[derive(Debug, Clone)]
pub enum PositionLookupError {
    /// Invalid position format string
    InvalidPositionFormat(String),
    /// Element not found at the specified position
    NotFound { line: usize, column: usize },
}

impl fmt::Display for PositionLookupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PositionLookupError::InvalidPositionFormat(msg) => {
                write!(f, "Invalid position format: {}", msg)
            }
            PositionLookupError::NotFound { line, column } => {
                write!(f, "No element found at position {}:{}", line, column)
            }
        }
    }
}

impl std::error::Error for PositionLookupError {}
