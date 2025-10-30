//! Parser module for the txxt format
//!
//! This module contains the parsing logic for the txxt format,
//! including AST definitions and the parser implementation.
//!
//! ## Testing
//!
//! All parser tests must follow strict guidelines. See the [testing module](crate::txxt::testing)
//! for comprehensive documentation on using verified txxt sources and AST assertions.

pub mod api;
pub mod combinators;
pub mod elements;
#[allow(clippy::module_inception)]
pub mod parser;
#[cfg(test)]
mod tests;

// Re-export AST types and utilities from the ast module
pub use crate::txxt::ast::{
    format_at_position, Annotation, AstNode, Container, ContentItem, Definition, Document,
    ForeignBlock, Label, List, ListItem, Location, Paragraph, Parameter, Position, Session,
    SourceLocation, TextNode,
};

pub use crate::txxt::formats::{serialize_ast_tag, to_treeviz_str};
pub use elements::document::document;
pub use parser::parse_with_source;

/// Type alias for parse result with spanned tokens
type ParseResult = Result<
    Document,
    Vec<chumsky::prelude::Simple<(crate::txxt::lexer::Token, std::ops::Range<usize>)>>,
>;

/// Main parser function that takes source text and returns a parsed document
/// This is the primary entry point for parsing txxt documents
pub fn parse_document(source: &str) -> ParseResult {
    let tokens_with_locations = crate::txxt::lexer::lex_with_locations(source);
    parse_with_source(tokens_with_locations, source)
}
