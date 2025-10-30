//! Label parsing for annotations
//!
//! This module handles parsing of annotation labels in the txxt format.
//! Labels are optional identifiers that appear at the start of annotations.
//!
//! Grammar: `<label> = <letter> (<letter> | <digit> | "_" | "-" | ".")*`
//!
//! A label is distinguished from a parameter key by the absence of an '=' sign
//! immediately following it (after optional whitespace).

use crate::txxt::lexer::Token;
use chumsky::{prelude::*, Stream};
use std::ops::Range;

/// Type alias for token with location
type TokenLocation = (Token, Range<usize>);

/// Parse label from a token slice
///
/// Extracts a label location from the beginning of the token slice if present.
/// A label is identified as an identifier (Text, Dash, Number, Period tokens)
/// that is NOT followed by an equals sign.
///
/// # Arguments
/// * `tokens` - Slice of tokens to parse
///
/// # Returns
/// * `Some(Range<usize>)` - The location of the label in the source text
/// * `None` - No label found (either no identifier, or identifier followed by '=')
///
/// # Examples
/// ```text
/// :: note ::           -> Some(label_location for "note")
/// :: note key=val ::   -> Some(label_location for "note")
/// :: key=val ::        -> None (this is a parameter, not a label)
/// ```
pub(crate) fn parse_label_from_tokens(tokens: &[TokenLocation]) -> (Option<Range<usize>>, usize) {
    if tokens.is_empty() {
        return (None, 0);
    }

    type ParserError = Simple<TokenLocation>;

    let whitespace = filter(|(token, _): &TokenLocation| matches!(token, Token::Whitespace));

    let leading_whitespace = whitespace
        .clone()
        .repeated()
        .map(|items: Vec<TokenLocation>| items.len());

    let identifier = filter(|(token, _): &TokenLocation| {
        matches!(
            token,
            Token::Text(_) | Token::Dash | Token::Number(_) | Token::Period
        )
    })
    .repeated()
    .at_least(1)
    .map(|items: Vec<TokenLocation>| {
        let start = items.first().map(|(_, span)| span.start).unwrap_or(0);
        let end = items.last().map(|(_, span)| span.end).unwrap_or(start);
        let count = items.len();
        (start..end, count)
    });

    let trailing_whitespace = whitespace
        .clone()
        .repeated()
        .map(|items: Vec<TokenLocation>| items.len());

    let parser = leading_whitespace
        .clone()
        .then(identifier)
        .then(trailing_whitespace.clone())
        .map(
            |((leading_count, (label_range, label_len)), trailing_count)| {
                (leading_count, label_range, label_len, trailing_count)
            },
        )
        .then(any::<TokenLocation, ParserError>().repeated());

    let stream = Stream::from_iter(
        0..0,
        tokens
            .iter()
            .cloned()
            .map(|(token, span)| ((token, span.clone()), span)),
    );

    match parser.parse(stream) {
        Ok(((leading_count, label_range, label_len, trailing_count), remainder)) => {
            let consumed = leading_count + label_len + trailing_count;

            if remainder
                .first()
                .map(|(token, _)| matches!(token, Token::Equals))
                .unwrap_or(false)
            {
                return (None, 0);
            }

            (Some(label_range), consumed)
        }
        Err(_) => {
            // Directly count leading whitespace tokens in the original slice
            let leading_only = tokens
                .iter()
                .take_while(|(token, _)| matches!(token, Token::Whitespace))
                .count();
            (None, leading_only)
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::txxt::lexer::lex_with_locations;
    use crate::txxt::parser::parse_with_source;

    #[test]
    fn test_annotation_with_label_only() {
        let source = ":: note ::\n\nText. {{paragraph}}\n";
        let tokens = lex_with_locations(source);
        let doc = parse_with_source(tokens, source).unwrap();

        let annotation = doc.content[0].as_annotation().unwrap();
        assert_eq!(annotation.label.value, "note");
        assert_eq!(annotation.parameters.len(), 0);
    }

    #[test]
    fn test_annotation_with_label_and_parameters() {
        let source = ":: warning severity=high ::\n\nText. {{paragraph}}\n";
        let tokens = lex_with_locations(source);
        let doc = parse_with_source(tokens, source).unwrap();

        let annotation = doc.content[0].as_annotation().unwrap();
        assert_eq!(annotation.label.value, "warning");
        assert_eq!(annotation.parameters.len(), 1);
        assert_eq!(annotation.parameters[0].key, "severity");
    }

    #[test]
    fn test_annotation_with_dotted_label() {
        let source = ":: python.typing ::\n\nText. {{paragraph}}\n";
        let tokens = lex_with_locations(source);
        let doc = parse_with_source(tokens, source).unwrap();

        let annotation = doc.content[0].as_annotation().unwrap();
        assert_eq!(annotation.label.value, "python.typing");
        assert_eq!(annotation.parameters.len(), 0);
    }

    #[test]
    fn test_annotation_parameters_only_no_label() {
        let source = ":: version=3.11 ::\n\nText. {{paragraph}}\n";
        let tokens = lex_with_locations(source);
        let doc = parse_with_source(tokens, source).unwrap();

        let annotation = doc.content[0].as_annotation().unwrap();
        assert_eq!(annotation.label.value, ""); // Empty label
        assert_eq!(annotation.parameters.len(), 1);
        assert_eq!(annotation.parameters[0].key, "version");
        assert_eq!(annotation.parameters[0].value, Some("3.11".to_string()));
    }

    #[test]
    fn test_annotation_with_dashed_label() {
        let source = ":: code-review ::\n\nText. {{paragraph}}\n";
        let tokens = lex_with_locations(source);
        let doc = parse_with_source(tokens, source).unwrap();

        let annotation = doc.content[0].as_annotation().unwrap();
        assert_eq!(annotation.label.value, "code-review");
        assert_eq!(annotation.parameters.len(), 0);
    }
}
