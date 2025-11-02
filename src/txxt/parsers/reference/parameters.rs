//! Parameter parsing for annotations
//!
//! This module re-exports parameter parsing from the consolidated builders module.
//! All parameter parsing logic and tests have been moved to builders.rs.

// Re-export parameter parsing from consolidated builders module
// Note: These are re-exported but may not be used directly in the current parser
// architecture as they're called internally by annotation_header
#[allow(unused_imports)]
pub(crate) use super::builders::{
    convert_parameter, parse_parameters_from_tokens, ParameterWithLocations,
};
