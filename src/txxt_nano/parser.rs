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
pub mod ast;
pub mod ast_conversion;
pub mod ast_position;
pub mod ast_tag_serializer;
pub mod ast_treeviz;
pub mod combinators;
pub mod document;
pub mod intermediate_ast;
pub mod labels;
pub mod parameters;
#[allow(clippy::module_inception)]
pub mod parser;
pub mod source_location;
#[cfg(test)]
mod tests;

pub use ast::{ContentItem, Document, List, ListItem, Paragraph, Position, Session, Span};
pub use ast_position::format_at_position;
pub use ast_tag_serializer::serialize_document as serialize_ast_tag;
pub use ast_treeviz::to_treeviz_str;
pub use document::document;
pub use parser::{parse, parse_with_source, parse_with_source_positions};
pub use source_location::SourceLocation;

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
