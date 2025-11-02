//! Definition element parsing
//!
//! This module re-exports definition builders from the consolidated builders module.
//! All AST node building and tests have been moved to builders.rs.

// Re-export definition builders from consolidated builders module
pub(crate) use super::builders::build_definition_parser;
