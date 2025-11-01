//! Public API for the parser.

use chumsky::prelude::*;
use std::ops::Range;

use crate::txxt::ast::Document;
use crate::txxt::lexer::Token;
use crate::txxt::parser::ast_build::document::document;

/// Type alias for token with location
type TokenLocation = (Token, Range<usize>);

/// Type alias for parser error
type ParserError = Simple<TokenLocation>;

/// Parse with source text - the primary parsing function
///
/// Parses tokens with location information and source text to produce a Document.
/// All parsed documents include complete location information automatically.
pub fn parse(tokens: Vec<TokenLocation>, source: &str) -> Result<Document, Vec<ParserError>> {
    document(source).parse(tokens)
}

/// Backward-compatibility shim: prefer `parse`
#[allow(dead_code)]
#[deprecated(note = "Use parse(tokens, source) instead")]
pub fn parse_with_source(
    tokens: Vec<TokenLocation>,
    source: &str,
) -> Result<Document, Vec<ParserError>> {
    parse(tokens, source)
}
