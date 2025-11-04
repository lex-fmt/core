//! Label parsing for annotations
//!
//! This module re-exports label parsing from the consolidated builders module.
//! All label parsing logic and tests have been moved to builders.rs.

// Re-export label parsing from consolidated builders module
// Note: parse_label_from_tokens is re-exported but may not be used directly
// in the current parser architecture as it's called internally by annotation_header
#[allow(unused_imports)]
pub(crate) use super::builders::parse_label_from_tokens;
