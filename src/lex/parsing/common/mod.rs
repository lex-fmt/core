//! Common parser module
//!
//! This module contains shared interfaces for parser implementations.

use crate::lex::parsing::Document;
use crate::lex::pipeline::stream::TokenStream;
use std::fmt;

/// Input type for parsers
pub type ParserInput = TokenStream;

/// Errors that can occur during parsing
#[derive(Debug, Clone)]
pub enum ParseError {
    /// Generic error message
    Error(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Error(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for ParseError {}

/// Trait for parser implementations
pub trait Parser {
    /// Parse the input into a document
    fn parse(&self, input: ParserInput, source: &str) -> Result<Document, ParseError>;
}
