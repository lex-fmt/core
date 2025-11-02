//! Common parser module
//!
//! This module contains shared interfaces and utilities for parser implementations.

pub mod interface;
pub mod location;

pub use interface::{
    LineBasedParserImpl, ParseError, Parser, ParserInput, ParserRegistry, ReferenceParserImpl,
};
pub use location::{
    aggregate_locations, byte_range_to_location, compute_byte_range_bounds,
    compute_location_from_locations, default_location,
};
