//! Parser module for the txxt format
//!
//! This module contains the parsing logic for the txxt format,
//! including AST definitions and the parser implementation.
//!
//! ## Testing
//!
//! All parser tests must follow strict guidelines. See the [testing module](crate::txxt_nano::testing)
//! for comprehensive documentation on using verified txxt sources and AST assertions.

pub mod ast;
pub mod ast_tag_serializer;
pub mod ast_treeviz;
pub mod parameters;
#[allow(clippy::module_inception)]
pub mod parser;

pub use ast::{ContentItem, Document, List, ListItem, Paragraph, Session};
pub use ast_tag_serializer::serialize_document as serialize_ast_tag;
pub use ast_treeviz::to_treeviz_str;
pub use parser::{document, parse, parse_with_source};

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

/// Legacy parser function (for backward compatibility with tests)
/// This version doesn't preserve source text - use parse_document instead for real usage
#[deprecated(note = "Use parse_document instead to preserve source text")]
pub fn parse_tokens_only(
    tokens: Vec<crate::txxt_nano::lexer::Token>,
) -> Result<Document, Vec<chumsky::prelude::Simple<crate::txxt_nano::lexer::Token>>> {
    parse(tokens)
}
