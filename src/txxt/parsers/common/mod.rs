//! Common parser module
//!
//! This module contains shared interfaces and utilities for parser implementations.

pub mod builders;
pub mod interface;
pub mod location;
pub mod token_processing;

pub use builders::{
    build_annotation, build_definition, build_foreign_block, build_list, build_list_item,
    build_paragraph, build_session, extract_text_from_span,
};
pub use interface::{
    LineBasedParserImpl, ParseError, Parser, ParserInput, ParserRegistry, ReferenceParserImpl,
};
pub use location::{
    aggregate_locations, byte_range_to_location, compute_byte_range_bounds,
    compute_location_from_locations, default_location,
};
