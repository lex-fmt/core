//! Common parser module
//!
//! This module contains shared interfaces and utilities for parser implementations.

pub mod interface;

pub use interface::{
    LineBasedParserImpl, ParseError, Parser, ParserInput, ParserRegistry, ReferenceParserImpl,
};
