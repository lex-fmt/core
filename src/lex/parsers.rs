//! Parser module for the lex format
//!
//! This module contains two independent parser implementations:
//!
//! - **Reference Parser**: Traditional combinator-based parser (reference/)
//!   - Contains element parsers and parser combinators
//! - **Grammar Engine**: Regex-based grammar-driven parser (linebased/)
//!   - Uses regex matching and pattern unwrapping
//!
//! No shared code between parsers (each is completely independent).
//!
//! ## Testing
//!
//! All parser tests must follow strict guidelines. See the [testing module](crate::lex::testing)
//! for comprehensive documentation on using verified lex sources and AST assertions.

// Parser implementations
pub mod ast;
pub mod common;
pub mod linebased;
pub mod reference;

// Re-export common parser interfaces
pub use common::{ParseError, Parser, ParserInput};

// Re-export AST types and utilities from the ast module
pub use crate::lex::ast::{
    format_at_position, Annotation, AstNode, Container, ContentItem, Definition, Document,
    ForeignBlock, Label, List, ListItem, Paragraph, Parameter, Position, Range, Session,
    SourceLocation, TextNode,
};

pub use crate::lex::formats::{serialize_ast_tag, to_treeviz_str};
pub use reference::document;
pub use reference::parse;

/// Type alias for parse result with spanned tokens
type ParseResult = Result<
    Document,
    Vec<chumsky::prelude::Simple<(crate::lex::lexers::Token, std::ops::Range<usize>)>>,
>;

/// Main parser function that takes source text and returns a parsed document
/// This is the primary entry point for parsing lex documents
pub fn parse_document(source: &str) -> ParseResult {
    let source_with_newline = crate::lex::lexers::ensure_source_ends_with_newline(source);
    let token_stream = crate::lex::lexers::base_tokenization::tokenize(&source_with_newline);
    let tokens = crate::lex::lexers::lex(token_stream);
    parse(tokens, source)
}
