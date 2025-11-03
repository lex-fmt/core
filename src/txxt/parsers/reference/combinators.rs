//! Parser combinator functions for building the txxt parser
//!
//! This module re-exports combinators and utilities from the consolidated builders module.
//! All location utilities, text extraction, and parser combinators have been moved to builders.rs.

// Re-export parsers and utilities from consolidated builders module
// Only re-export what's actually used by the parser
pub(crate) use super::builders::paragraph;

// Import location utilities from common module
#[allow(unused_imports)]
pub(crate) use crate::txxt::parsers::common::location::{
    aggregate_locations, byte_range_to_location, compute_byte_range_bounds,
    compute_location_from_locations,
};

// Re-export other utilities from builders
#[allow(unused_imports)]
pub(crate) use super::builders::{
    extract_text_from_locations, extract_tokens_to_text_and_location, is_text_token, text_line,
    ParserError, TokenLocation,
};
