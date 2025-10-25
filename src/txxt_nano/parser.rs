//! Parser module for the txxt format
//!
//! This module contains the parsing logic for the txxt format,
//! including AST definitions and the parser implementation.

pub mod ast;
#[allow(clippy::module_inception)]
pub mod parser;

pub use ast::{ContentItem, Document, Paragraph, Session};
pub use parser::{document, parse};

/// Main parser function that takes source text and returns a parsed document
pub fn parse_document(
    source: &str,
) -> Result<Document, Vec<chumsky::prelude::Simple<crate::txxt_nano::lexer::Token>>> {
    let tokens = crate::txxt_nano::lexer::lex(source);
    parse(tokens)
}
