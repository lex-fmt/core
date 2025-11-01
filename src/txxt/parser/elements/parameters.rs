//! Parameter parsing for annotations
//!
//! This module handles parsing of annotation parameters in the txxt format.
//! Parameters are key-value pairs separated by commas within annotation markers.
//!
//! Grammar: `<parameters> = <parameter> ("," <parameter>)*`
//! Where: `<parameter> = <key> "=" <value>`

use crate::txxt::ast::Parameter;
use crate::txxt::lexer::Token;
use crate::txxt::parser::combinators::byte_range_to_location;
use chumsky::{prelude::*, Stream};
use std::ops::Range;

/// Type alias for token with location
type TokenLocation = (Token, Range<usize>);
type ParserError = Simple<TokenLocation>;

/// Parameter with source text locations for later extraction
#[derive(Debug, Clone)]
pub(crate) struct ParameterWithLocations {
    pub(crate) key_location: Range<usize>,
    pub(crate) value_location: Option<Range<usize>>,
    pub(crate) range: Range<usize>,
}

/// Convert a parameter from locations to final AST
///
/// Extracts the key and value text from the source using the stored locations
pub(crate) fn convert_parameter(source: &str, param: ParameterWithLocations) -> Parameter {
    let key = extract_text(source, &param.key_location).to_string();

    let value = param
        .value_location
        .map(|value_location| extract_text(source, &value_location).to_string());

    let location = byte_range_to_location(source, &param.range);

    Parameter {
        key,
        value: value.unwrap_or_default(),
        location,
    }
}

/// Extract text from source using a location range
fn extract_text<'a>(source: &'a str, location: &Range<usize>) -> &'a str {
    &source[location.start..location.end]
}

/// Helper function to parse parameters from a token slice
///
/// Simplified parameter parsing:
/// 1. Split by comma
/// 2. For each segment, split by '=' to get key/value
/// 3. Whitespace around parameters is ignored
pub(crate) fn parse_parameters_from_tokens(
    tokens: &[TokenLocation],
) -> Vec<ParameterWithLocations> {
    if tokens.is_empty() {
        return Vec::new();
    }

    #[derive(Clone)]
    struct ParsedValue {
        location: Option<Range<usize>>,
        last_consumed: Range<usize>,
    }

    let whitespace_token = filter::<TokenLocation, _, ParserError>(|(token, _): &TokenLocation| {
        matches!(token, Token::Whitespace)
    })
    .ignored();
    let whitespace0 = whitespace_token.repeated().ignored();

    let key_segment = filter::<TokenLocation, _, ParserError>(|(token, _): &TokenLocation| {
        matches!(token, Token::Text(_) | Token::Dash | Token::Number(_))
    })
    .map(|(_, span)| span.clone())
    .repeated()
    .at_least(1)
    .map(|segments: Vec<Range<usize>>| {
        let start = segments.first().map(|range| range.start).unwrap_or(0);
        let end = segments.last().map(|range| range.end).unwrap_or(start);
        start..end
    });

    let equals = filter::<TokenLocation, _, ParserError>(|(token, _): &TokenLocation| {
        matches!(token, Token::Equals)
    })
    .ignored();

    let unquoted_value_segment =
        filter::<TokenLocation, _, ParserError>(|(token, _): &TokenLocation| {
            !matches!(token, Token::Comma | Token::Whitespace)
        })
        .map(|(_, span)| span.clone());

    let unquoted_value =
        unquoted_value_segment
            .repeated()
            .at_least(1)
            .map(|segments: Vec<Range<usize>>| {
                let start = segments.first().map(|range| range.start).unwrap_or(0);
                let end = segments.last().map(|range| range.end).unwrap_or(start);
                let last_consumed = segments.last().cloned().unwrap_or(start..end);
                ParsedValue {
                    location: Some(start..end),
                    last_consumed,
                }
            });

    let quoted_inner = filter::<TokenLocation, _, ParserError>(|(token, _): &TokenLocation| {
        !matches!(token, Token::Quote)
    })
    .map(|(_, span)| span.clone())
    .repeated();

    let closing_quote = filter::<TokenLocation, _, ParserError>(|(token, _): &TokenLocation| {
        matches!(token, Token::Quote)
    })
    .map(|(_, span)| span.clone());

    let quoted_value = filter::<TokenLocation, _, ParserError>(|(token, _): &TokenLocation| {
        matches!(token, Token::Quote)
    })
    .map(|(_, span)| span.clone())
    .then(quoted_inner.then(closing_quote.or_not()))
    .map(|(opening_span, (inner_segments, closing_span))| {
        let location = if inner_segments.is_empty() {
            Some(0..0)
        } else {
            let start = inner_segments.first().map(|range| range.start).unwrap_or(0);
            let end = inner_segments
                .last()
                .map(|range| range.end)
                .unwrap_or(start);
            Some(start..end)
        };

        let last_consumed = closing_span
            .clone()
            .or_else(|| inner_segments.last().cloned())
            .unwrap_or_else(|| opening_span.clone());

        ParsedValue {
            location,
            last_consumed,
        }
    });

    let value = quoted_value.or(unquoted_value);

    let parameter = key_segment
        .then_ignore(whitespace0)
        .then_ignore(equals)
        .then_ignore(whitespace0)
        .then(value)
        .map(|(key_location, parsed_value)| {
            let ParsedValue {
                location,
                last_consumed,
            } = parsed_value;

            let range_start = key_location.start;
            let range_end = location
                .as_ref()
                .filter(|loc| loc.end > range_start)
                .map(|loc| loc.end)
                .unwrap_or(last_consumed.end);

            ParameterWithLocations {
                key_location,
                value_location: location,
                range: range_start..range_end,
            }
        });

    let comma = filter::<TokenLocation, _, ParserError>(|(token, _): &TokenLocation| {
        matches!(token, Token::Comma)
    })
    .ignored();
    let parameter_with_separator = parameter
        .then_ignore(whitespace0)
        .then_ignore(comma.then_ignore(whitespace0).or_not());

    let parser = whitespace0
        .ignore_then(parameter_with_separator.repeated())
        .then_ignore(whitespace0);

    let stream = Stream::from_iter(
        0..0,
        tokens
            .iter()
            .cloned()
            .map(|(token, span)| ((token, span.clone()), span)),
    );

    parser.parse(stream).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use crate::txxt::lexer::lex_with_locations;
    use crate::txxt::parser::parse_with_source;

    #[test]
    fn test_annotation_comma_separated_parameters() {
        let source = ":: warning severity=high,priority=urgent ::\n\nText. {{paragraph}}\n";
        let tokens = lex_with_locations(source);
        let doc = parse_with_source(tokens, source).unwrap();

        let annotation = doc.root_session.content[0].as_annotation().unwrap();
        assert_eq!(annotation.label.value, "warning");
        assert_eq!(annotation.parameters.len(), 2);
        assert_eq!(annotation.parameters[0].key, "severity");
        assert_eq!(annotation.parameters[0].value, "high".to_string());
        assert_eq!(annotation.parameters[1].key, "priority");
        assert_eq!(annotation.parameters[1].value, "urgent".to_string());
    }

    #[test]
    fn test_annotation_quoted_string_values() {
        let source =
            ":: note author=\"Jane Doe\" title=\"Important Note\" ::\n\nText. {{paragraph}}\n";
        let tokens = lex_with_locations(source);
        let doc = parse_with_source(tokens, source).unwrap();

        let annotation = doc.root_session.content[0].as_annotation().unwrap();
        assert_eq!(annotation.label.value, "note");
        assert_eq!(annotation.parameters.len(), 2);
        assert_eq!(annotation.parameters[0].key, "author");
        assert_eq!(annotation.parameters[0].value, "Jane Doe".to_string());
        assert_eq!(annotation.parameters[1].key, "title");
        assert_eq!(
            annotation.parameters[1].value,
            "Important Note".to_string()
        );
    }

    #[test]
    fn test_annotation_mixed_separators_and_quotes() {
        let source = ":: task priority=high,status=\"in progress\",assigned=alice ::\n\nText. {{paragraph}}\n";
        let tokens = lex_with_locations(source);
        let doc = parse_with_source(tokens, source).unwrap();

        let annotation = doc.root_session.content[0].as_annotation().unwrap();
        assert_eq!(annotation.parameters.len(), 3);
        assert_eq!(annotation.parameters[0].key, "priority");
        assert_eq!(annotation.parameters[0].value, "high".to_string());
        assert_eq!(annotation.parameters[1].key, "status");
        assert_eq!(
            annotation.parameters[1].value,
            "in progress".to_string()
        );
        assert_eq!(annotation.parameters[2].key, "assigned");
        assert_eq!(annotation.parameters[2].value, "alice".to_string());
    }

    #[test]
    fn test_annotation_whitespace_around_commas() {
        // Test that whitespace around commas is properly ignored
        let source = ":: note key1=val1 , key2=val2 , key3=val3 ::\n\nText. {{paragraph}}\n";
        let tokens = lex_with_locations(source);
        let doc = parse_with_source(tokens, source).unwrap();

        let annotation = doc.root_session.content[0].as_annotation().unwrap();
        assert_eq!(annotation.label.value, "note");
        assert_eq!(annotation.parameters.len(), 3);
        assert_eq!(annotation.parameters[0].key, "key1");
        assert_eq!(annotation.parameters[0].value, "val1".to_string());
        assert_eq!(annotation.parameters[1].key, "key2");
        assert_eq!(annotation.parameters[1].value, "val2".to_string());
        assert_eq!(annotation.parameters[2].key, "key3");
        assert_eq!(annotation.parameters[2].value, "val3".to_string());
    }

    #[test]
    fn test_annotation_boolean_parameter_is_not_parsed() {
        let source = ":: warning draft,priority=urgent ::\n\nText. {{paragraph}}\n";
        let tokens = lex_with_locations(source);
        let doc = parse_with_source(tokens, source).unwrap();

        let annotation = doc.root_session.content[0].as_annotation().unwrap();
        assert_eq!(annotation.label.value, "warning");
        assert_eq!(annotation.parameters.len(), 1);
        assert_eq!(annotation.parameters[0].key, "priority");
        assert_eq!(annotation.parameters[0].value, "urgent".to_string());
    }
}
