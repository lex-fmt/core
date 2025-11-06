//! Parsing module for the lex format
//!
//! This module provides the complete processing pipeline from source text to AST:
//! 1. **Lexing**: Tokenization of source text
//! 2. **Analysis**: Syntactic analysis to produce IR nodes
//! 3. **Building**: Construction of AST from IR nodes
//!
//! ## Independent Analyzer Implementations
//!
//! - **Reference Analyzer**: Traditional combinator-based analyzer (reference/)
//!   - Contains element parsers and parser combinators
//! - **Linebased Analyzer**: Regex-based grammar-driven analyzer (linebased/)
//!   - Uses regex matching and pattern unwrapping
//!
//! No shared code between analyzers (each is completely independent).
//!
//! ## Terminology
//!
//! - **parse**: Colloquial term for the entire process (lexing + analysis + building)
//! - **analyze/analysis**: The syntactic analysis phase specifically
//! - **build**: The AST construction phase specifically
//!
//! ## Testing
//!
//! All parser tests must follow strict guidelines. See the [testing module](crate::lex::testing)
//! for comprehensive documentation on using verified lex sources and AST assertions.

// Parser implementations
pub mod builder;
pub mod common;
pub mod ir;
pub mod linebased;
pub mod reference;

// Re-export common parser interfaces
pub use common::{ParseError, Parser, ParserInput};

// Re-export AST types and utilities from the ast module
pub use crate::lex::ast::{
    format_at_position, Annotation, AstNode, Container, ContentItem, Definition, Document, Label,
    List, ListItem, Paragraph, Parameter, Position, Range, Session, SourceLocation, TextNode,
    Verbatim,
};

pub use crate::lex::formats::{serialize_ast_tag, to_treeviz_str};
pub use reference::parse;

/// Type alias for processing result with spanned tokens
type ProcessResult = Result<
    Document,
    Vec<chumsky::prelude::Simple<(crate::lex::lexing::Token, std::ops::Range<usize>)>>,
>;

/// Process source text through the complete pipeline: lex, analyze, and build.
///
/// This is the primary entry point for processing lex documents. It performs:
/// 1. **Lexing**: Tokenizes the source text
/// 2. **Analysis**: Performs syntactic analysis to produce IR nodes
/// 3. **Building**: Constructs the final AST from IR nodes
///
/// # Arguments
///
/// * `source` - The source text to process
///
/// # Returns
///
/// A `Document` containing the complete AST, or parsing errors.
///
/// # Example
///
/// ```rust,ignore
/// use lex::lex::parsing::process_full;
///
/// let source = "Hello world\n";
/// let document = process_full(source)?;
/// ```
pub fn process_full(source: &str) -> ProcessResult {
    let source_with_newline = crate::lex::lexing::ensure_source_ends_with_newline(source);
    let token_stream = crate::lex::lexing::base_tokenization::tokenize(&source_with_newline);
    let tokens = crate::lex::lexing::lex(token_stream);
    let parse_tree = parse(tokens, source)?;
    let builder = builder::AstBuilder::new(source);
    Ok(builder.build(parse_tree))
}

/// Alias for `process_full` to maintain backward compatibility.
///
/// The term "parse" colloquially refers to the entire processing pipeline
/// (lexing + analysis + building), even though technically parsing is just
/// the syntactic analysis phase.
pub fn parse_document(source: &str) -> ProcessResult {
    process_full(source)
}
