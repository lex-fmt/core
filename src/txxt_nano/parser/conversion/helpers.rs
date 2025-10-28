//! Helper functions for parser conversions
//!
//! Common patterns extracted from conversion logic to reduce duplication.

use super::super::ast::Span;
use super::super::source_location::SourceLocation;
use crate::txxt_nano::lexer::Token;
use std::ops::Range;

/// Check if a token is a text-like token (content that can appear in lines)
///
/// This includes: Text, Whitespace, Numbers, Punctuation, and common symbols
pub(crate) fn is_text_token(token: &Token) -> bool {
    matches!(
        token,
        Token::Text(_)
            | Token::Whitespace
            | Token::Number(_)
            | Token::Dash
            | Token::Period
            | Token::OpenParen
            | Token::CloseParen
            | Token::Colon
            | Token::Comma
            | Token::Quote
            | Token::Equals
    )
}

/// Convert span ranges to a Span position object using source location mapping
///
/// Handles both single spans and multiple spans, extracting the start and end positions.
pub(crate) fn spans_to_span_position(
    spans: &[Range<usize>],
    source_loc: &SourceLocation,
) -> Option<Span> {
    if spans.is_empty() {
        return None;
    }

    let start_range = spans.first();
    let end_range = spans.last();

    match (start_range, end_range) {
        (Some(start), Some(end)) => Some(Span::new(
            source_loc.byte_to_position(start.start),
            source_loc.byte_to_position(end.end),
        )),
        _ => None,
    }
}

/// Convert nested spans (Vec<Vec<Range<usize>>>) to a Span position
///
/// Flattens the nested structure to find the outermost start and end positions.
pub(crate) fn nested_spans_to_span_position(
    spans: &[Vec<Range<usize>>],
    source_loc: &SourceLocation,
) -> Option<Span> {
    if spans.is_empty() {
        return None;
    }

    let start_range = spans.first().and_then(|s| s.first());
    let end_range = spans.last().and_then(|s| s.last());

    match (start_range, end_range) {
        (Some(start), Some(end)) => Some(Span::new(
            source_loc.byte_to_position(start.start),
            source_loc.byte_to_position(end.end),
        )),
        _ => None,
    }
}
