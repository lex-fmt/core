//! Public API for the parser.

use chumsky::prelude::*;
use std::ops::Range;

use crate::txxt_nano::ast::Document;
use crate::txxt_nano::lexer::Token;
use crate::txxt_nano::parser::elements::document::document;

/// Type alias for token with span
type TokenSpan = (Token, Range<usize>);

/// Type alias for parser error
type ParserError = Simple<TokenSpan>;

/// Parse with source text - returns final Document with full span information
pub fn parse_with_source(
    tokens_with_spans: Vec<TokenSpan>,
    source: &str,
) -> Result<Document, Vec<ParserError>> {
    document(source).parse(tokens_with_spans)
}

/// Parse a txxt document from tokens with source, preserving position information
/// Note: All parsed documents now include complete span information automatically
pub fn parse_with_source_positions(
    tokens_with_spans: Vec<TokenSpan>,
    source: &str,
) -> Result<Document, Vec<ParserError>> {
    document(source).parse(tokens_with_spans)
}

/// Parse a txxt document from a token stream (legacy - doesn't preserve source text)
pub fn parse(tokens: Vec<Token>) -> Result<Document, Vec<Simple<Token>>> {
    let tokens_with_spans: Vec<TokenSpan> = tokens.into_iter().map(|t| (t, 0..0)).collect();
    parse_with_source(tokens_with_spans, "")
        .map_err(|errs| errs.into_iter().map(|e| e.map(|(t, _)| t)).collect())
}
