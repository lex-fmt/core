//! Common parser module
//!
//! This module contains shared interfaces for parser implementations.

pub mod interface;

pub use interface::{
    LineBasedParserImpl, ParseError, Parser, ParserInput, ParserRegistry, ReferenceParserImpl,
};
