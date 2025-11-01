//! Parser module for the txxt format
//!
//! This module contains the parsing logic for the txxt format,
//! including two parser implementations:
//!
//! - **Reference Parser**: Traditional combinator-based parser (reference/)
//! - **Grammar Engine**: Regex-based grammar-driven parser (grammarengine/)
//!
//! Both parsers share common AST building blocks (ast_build/).
//!
//! ## Testing
//!
//! All parser tests must follow strict guidelines. See the [testing module](crate::txxt::testing)
//! for comprehensive documentation on using verified txxt sources and AST assertions.

// Parser implementations
pub mod ast_build;
pub mod grammarengine;
pub mod reference;

// Re-export AST types and utilities from the ast module
pub use crate::txxt::ast::{
    format_at_position, Annotation, AstNode, Container, ContentItem, Definition, Document,
    ForeignBlock, Label, List, ListItem, Location, Paragraph, Parameter, Position, Session,
    SourceLocation, TextNode,
};

pub use crate::txxt::formats::{serialize_ast_tag, to_treeviz_str};
pub use ast_build::document;
pub use reference::parse;

/// Type alias for parse result with spanned tokens
type ParseResult = Result<
    Document,
    Vec<chumsky::prelude::Simple<(crate::txxt::lexer::Token, std::ops::Range<usize>)>>,
>;

/// Main parser function that takes source text and returns a parsed document
/// This is the primary entry point for parsing txxt documents
pub fn parse_document(source: &str) -> ParseResult {
    let tokens = crate::txxt::lexer::lex(source);
    parse(tokens, source)
}
