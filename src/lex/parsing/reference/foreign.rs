//! Verbatim block element parsing
//!
//! This module re-exports foreign block builders from the consolidated builders module.
//! All AST node building and tests have been moved to builders.rs.

// Re-export foreign block builders from consolidated builders module
pub(crate) use super::builders::foreign_block;
