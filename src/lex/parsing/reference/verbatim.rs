//! Verbatim block element parsing
//!
//! This module re-exports verbatim block builders from the consolidated builders module.
//! All AST node building and tests have been moved to builders.rs.

// Re-export verbatim block builders from consolidated builders module
pub(crate) use super::builders::verbatim_block;
