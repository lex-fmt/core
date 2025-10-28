//! Public API for the parser.

use chumsky::prelude::*;
use std::ops::Range;

use crate::txxt_nano::lexer::Token;
use crate::txxt_nano::parser::ast::Document;
use crate::txxt_nano::parser::document::document;
use crate::txxt_nano::parser::ast_conversion::{
    convert_document, convert_document_with_positions,
};

/// Type alias for token with span
type TokenSpan = (Token, Range<usize>);

/// Type alias for parser error
type ParserError = Simple<TokenSpan>;

/// Parse with source text - extracts actual content from spans
pub fn parse_with_source(
    tokens_with_spans: Vec<TokenSpan>,
    source: &str,
) -> Result<Document, Vec<ParserError>> {
    let doc_with_spans = document().parse(tokens_with_spans)?;
    Ok(convert_document(source, doc_with_spans))
}

/// Parse a txxt document from tokens with source, preserving position information
pub fn parse_with_source_positions(
    tokens_with_spans: Vec<TokenSpan>,
    source: &str,
) -> Result<Document, Vec<ParserError>> {
    let doc_with_spans = document().parse(tokens_with_spans)?;
    Ok(convert_document_with_positions(source, doc_with_spans))
}

/// Parse a txxt document from a token stream (legacy - doesn't preserve source text)
pub fn parse(tokens: Vec<Token>) -> Result<Document, Vec<Simple<Token>>> {
    let tokens_with_spans: Vec<TokenSpan> = tokens.into_iter().map(|t| (t, 0..0)).collect();
    parse_with_source(tokens_with_spans, "")
        .map_err(|errs| errs.into_iter().map(|e| e.map(|(t, _)| t)).collect())
}
