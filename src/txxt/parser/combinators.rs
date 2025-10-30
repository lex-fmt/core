//! Parser combinator functions for building the txxt parser.

use chumsky::prelude::*;
use std::ops::Range;
use std::sync::Arc;

use crate::txxt::ast::position::SourceLocation;
use crate::txxt::ast::{Location, Paragraph, Parameter, TextContent};
use crate::txxt::lexer::Token;
use crate::txxt::parser::elements::labels::parse_label_from_tokens;
use crate::txxt::parser::elements::parameters::{convert_parameter, parse_parameters_from_tokens};

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

/// Helper: convert a byte range to a location using source location
fn byte_range_to_location(source: &str, range: &Range<usize>) -> Option<Location> {
    if range.start > range.end {
        return None;
    }
    let source_loc = SourceLocation::new(source);
    Some(source_loc.range_to_location(range))
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
pub(crate) fn compute_location_from_optional_locations(
    locations: &[Option<Location>],
) -> Option<Location> {
    let actual_locations: Vec<Location> = locations.iter().filter_map(|s| *s).collect();
    if actual_locations.is_empty() {
        None
    } else {
        Some(compute_location_from_locations(&actual_locations))
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

/// Parse a list item line - a line that starts with a list marker
/// Phase 5: Now returns extracted text with location information
pub(crate) fn list_item_line(
    source: Arc<String>,
) -> impl Parser<TokenLocation, (String, Range<usize>), Error = ParserError> + Clone {
    let rest_of_line = filter(|(t, _location): &TokenLocation| is_text_token(t)).repeated();

    let dash_pattern = filter(|(t, _): &TokenLocation| matches!(t, Token::Dash))
        .then(filter(|(t, _): &TokenLocation| {
            matches!(t, Token::Whitespace)
        }))
        .chain(rest_of_line);

    let ordered_pattern =
        filter(|(t, _): &TokenLocation| matches!(t, Token::Number(_) | Token::Text(_)))
            .then(filter(|(t, _): &TokenLocation| {
                matches!(t, Token::Period | Token::CloseParen)
            }))
            .then(filter(|(t, _): &TokenLocation| {
                matches!(t, Token::Whitespace)
            }))
            .chain(rest_of_line);

    let paren_pattern = filter(|(t, _): &TokenLocation| matches!(t, Token::OpenParen))
        .then(filter(|(t, _): &TokenLocation| {
            matches!(t, Token::Number(_))
        }))
        .then(filter(|(t, _): &TokenLocation| {
            matches!(t, Token::CloseParen)
        }))
        .then(filter(|(t, _): &TokenLocation| {
            matches!(t, Token::Whitespace)
        }))
        .chain(rest_of_line);

    dash_pattern
        .or(ordered_pattern)
        .or(paren_pattern)
        .map(move |tokens_with_locations| {
            extract_tokens_to_text_and_location(&source, tokens_with_locations)
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
                        None
                    } else {
                        let range = compute_byte_range_bounds(locations);
                        byte_range_to_location(&source, &range)
                    };
                    TextContent::from_string(text, line_location)
                })
                .collect();

            // Compute overall location from all collected line locations
            let location = {
                let all_locations: Vec<Range<usize>> =
                    line_locations_list.into_iter().flatten().collect();
                if all_locations.is_empty() {
                    None
                } else {
                    let range = compute_byte_range_bounds(&all_locations);
                    byte_range_to_location(&source, &range)
                }
            };

            Paragraph { lines, location }
        })
}

/// Parse a definition subject
/// Phase 5: Now returns extracted text with location information
pub(crate) fn definition_subject(
    source: Arc<String>,
) -> impl Parser<TokenLocation, (String, Range<usize>), Error = ParserError> + Clone {
    filter(|(t, _location): &TokenLocation| !matches!(t, Token::Colon | Token::Newline))
        .repeated()
        .at_least(1)
        .map(move |tokens_with_locations| {
            extract_tokens_to_text_and_location(&source, tokens_with_locations)
        })
        .then_ignore(token(Token::Colon))
        .then_ignore(token(Token::Newline))
}

/// Parse a session title
/// Phase 5: Now returns extracted text with location information
pub(crate) fn session_title(
    source: Arc<String>,
) -> impl Parser<TokenLocation, (String, Range<usize>), Error = ParserError> + Clone {
    text_line()
        .then_ignore(token(Token::Newline))
        .then_ignore(token(Token::BlankLine))
        .map(move |locations| {
            let text = extract_text_from_locations(&source, &locations);
            let location = compute_byte_range_bounds(&locations);
            (text, location)
        })
}

/// Parse the bounded region between :: markers
/// Phase 5: Now returns extracted label text, label location, and final Parameter types
pub(crate) fn annotation_header(
    source: Arc<String>,
) -> impl Parser<
    TokenLocation,
    (Option<String>, Option<Range<usize>>, Vec<Parameter>),
    Error = ParserError,
> + Clone {
    let bounded_region =
        filter(|(t, _): &TokenLocation| !matches!(t, Token::TxxtMarker | Token::Newline))
            .repeated()
            .at_least(1);

    bounded_region.validate(move |tokens, location, emit| {
        if tokens.is_empty() {
            emit(ParserError::expected_input_found(location, None, None));
            return (None, None, Vec::new());
        }

        let (label_location, mut i) = parse_label_from_tokens(&tokens);

        if label_location.is_none() && i == 0 {
            while i < tokens.len() && matches!(tokens[i].0, Token::Whitespace) {
                i += 1;
            }
        }

        let params_with_locations = parse_parameters_from_tokens(&tokens[i..]);

        // Extract label text if present
        let label = label_location.as_ref().map(|location| {
            let text = if location.start < location.end && location.end <= source.len() {
                source[location.start..location.end].trim().to_string()
            } else {
                String::new()
            };
            text
        });

        // Convert parameters to final types
        let params = params_with_locations
            .into_iter()
            .map(|p| convert_parameter(&source, p))
            .collect();

        (label, label_location, params)
    })
}
