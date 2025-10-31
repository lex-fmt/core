//! Parser combinator functions for building the txxt parser.

use chumsky::prelude::*;
use std::ops::Range;
use std::sync::Arc;

use crate::txxt::ast::location::SourceLocation;
use crate::txxt::ast::{Location, Paragraph, TextContent};
use crate::txxt::lexer::Token;

/// Type alias for token with location
type TokenLocation = (Token, Range<usize>);

/// Type alias for parser error
type ParserError = Simple<TokenLocation>;

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

/// Convert a byte range to a Location (line:column positions)
///
/// This is the canonical implementation used throughout the parser.
/// Converts byte offsets from token ranges to line/column coordinates
/// using the SourceLocation utility (O(log n) binary search).
pub(crate) fn byte_range_to_location(source: &str, range: &Range<usize>) -> Location {
    if range.start > range.end {
        return Location::default();
    }
    let source_loc = SourceLocation::new(source);
    source_loc.range_to_location(range)
}

/// Helper: compute location bounds from multiple locations
pub(crate) fn compute_location_from_locations(locations: &[Location]) -> Location {
    use crate::txxt::ast::location::Position;
    let start_line = locations.iter().map(|sp| sp.start.line).min().unwrap_or(0);
    let start_col = locations
        .iter()
        .map(|sp| sp.start.column)
        .min()
        .unwrap_or(0);
    let end_line = locations.iter().map(|sp| sp.end.line).max().unwrap_or(0);
    let end_col = locations.iter().map(|sp| sp.end.column).max().unwrap_or(0);
    Location::new(
        Position::new(start_line, start_col),
        Position::new(end_line, end_col),
    )
}

/// Helper: compute location bounds from multiple optional locations
pub(crate) fn compute_location_from_optional_locations(locations: &[Option<Location>]) -> Location {
    let actual_locations: Vec<Location> = locations.iter().filter_map(|s| *s).collect();
    if actual_locations.is_empty() {
        Location::default()
    } else {
        compute_location_from_locations(&actual_locations)
    }
}

/// Helper: compute location bounds from byte ranges
pub(crate) fn compute_byte_range_bounds(ranges: &[Range<usize>]) -> Range<usize> {
    if ranges.is_empty() {
        0..0
    } else {
        let start = ranges.iter().map(|r| r.start).min().unwrap_or(0);
        let end = ranges.iter().map(|r| r.end).max().unwrap_or(0);
        start..end
    }
}

/// Helper: extract text from multiple locations
pub(crate) fn extract_text_from_locations(source: &str, locations: &[Range<usize>]) -> String {
    if locations.is_empty() {
        return String::new();
    }
    let start = locations.first().map(|s| s.start).unwrap_or(0);
    let end = locations.last().map(|s| s.end).unwrap_or(0);

    if start >= end || end > source.len() {
        return String::new();
    }

    source[start..end].trim().to_string()
}

/// Helper: extract tokens to text and byte range location
/// Converts a vector of token-location pairs to (extracted_text, byte_range)
pub(crate) fn extract_tokens_to_text_and_location(
    source: &Arc<String>,
    tokens: Vec<TokenLocation>,
) -> (String, Range<usize>) {
    let locations: Vec<Range<usize>> = tokens.into_iter().map(|(_, s)| s).collect();
    let text = extract_text_from_locations(source, &locations);
    let location = compute_byte_range_bounds(&locations);
    (text, location)
}

/// Helper: match a specific token type, ignoring the location
pub(crate) fn token(t: Token) -> impl Parser<TokenLocation, (), Error = ParserError> + Clone {
    filter(move |(tok, _)| tok == &t).ignored()
}

/// Parse a text line (sequence of text and whitespace tokens)
/// Returns the collected locations for this line
pub(crate) fn text_line(
) -> impl Parser<TokenLocation, Vec<Range<usize>>, Error = ParserError> + Clone {
    filter(|(t, _location): &TokenLocation| is_text_token(t))
        .repeated()
        .at_least(1)
        .map(|tokens_with_locations: Vec<TokenLocation>| {
            // Collect all locations for this line
            tokens_with_locations.into_iter().map(|(_, s)| s).collect()
        })
}

/// Parse a paragraph
/// Phase 5: Now populates location information
pub(crate) fn paragraph(
    source: Arc<String>,
) -> impl Parser<TokenLocation, Paragraph, Error = ParserError> + Clone {
    text_line()
        .then_ignore(token(Token::Newline))
        .repeated()
        .at_least(1)
        .map(move |line_locations_list: Vec<Vec<Range<usize>>>| {
            let lines = line_locations_list
                .iter()
                .map(|locations| {
                    let text = extract_text_from_locations(&source, locations);
                    // Compute location for this line
                    let line_location = if locations.is_empty() {
                        Location::default()
                    } else {
                        let range = compute_byte_range_bounds(locations);
                        byte_range_to_location(&source, &range)
                    };
                    let text_content = TextContent::from_string(text, Some(line_location));
                    let text_line =
                        crate::txxt::ast::TextLine::new(text_content).with_location(line_location);
                    crate::txxt::ast::ContentItem::TextLine(text_line)
                })
                .collect();

            // Compute overall location from all collected line locations
            let location = {
                let all_locations: Vec<Range<usize>> =
                    line_locations_list.into_iter().flatten().collect();
                if all_locations.is_empty() {
                    Location::default()
                } else {
                    let range = compute_byte_range_bounds(&all_locations);
                    byte_range_to_location(&source, &range)
                }
            };

            Paragraph { lines, location }
        })
}
