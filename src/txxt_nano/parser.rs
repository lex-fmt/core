//! Parser module for the txxt format
//!
//! This module contains the parsing logic for the txxt format,
//! including AST definitions and the parser implementation.
//!
//! ## Testing
//!
//! All parser tests must follow strict guidelines. See the [testing module](crate::txxt_nano::testing)
//! for comprehensive documentation on using verified txxt sources and AST assertions.

pub mod api;
pub mod combinators;
pub mod conversion;
pub mod document;
pub mod elements;
pub mod labels;
pub mod parameters;
#[allow(clippy::module_inception)]
pub mod parser;
#[cfg(test)]
mod tests;

// Re-export AST types and utilities from the ast module
pub use crate::txxt_nano::ast::{
    format_at_position, Annotation, AstNode, Container, ContentItem, Definition, Document,
    ForeignBlock, Label, List, ListItem, Paragraph, Parameter, Position, Session, SourceLocation,
    Span, TextNode,
};

pub use crate::txxt_nano::formats::{serialize_ast_tag, to_treeviz_str};
pub use document::document;
pub use parser::{parse, parse_with_source, parse_with_source_positions};

/// Type alias for parse result with spanned tokens
type ParseResult = Result<
    Document,
    Vec<chumsky::prelude::Simple<(crate::txxt_nano::lexer::Token, std::ops::Range<usize>)>>,
>;

/// Main parser function that takes source text and returns a parsed document
/// This is the primary entry point for parsing txxt documents
pub fn parse_document(source: &str) -> ParseResult {
    let tokens_with_spans = crate::txxt_nano::lexer::lex_with_spans(source);
    parse_with_source(tokens_with_spans, source)
}
