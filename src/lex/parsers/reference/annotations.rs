//! Annotation element parsing
//!
//! This module re-exports annotation builders from the consolidated builders module.
//! All AST node building and tests have been moved to builders.rs.

// Re-export annotation builders from consolidated builders module
pub(crate) use super::builders::build_annotation_parser;
