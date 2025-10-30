//! Label parsing for annotations
//!
//! This module handles parsing of annotation labels in the txxt format.
//! Labels are optional identifiers that appear at the start of annotations.
//!
//! Grammar: `<label> = <letter> (<letter> | <digit> | "_" | "-" | ".")*`
//!
//! A label is distinguished from a parameter key by the absence of an '=' sign
//! immediately following it (after optional whitespace).

use crate::txxt_nano::lexer::Token;
use std::ops::Range;

/// Type alias for token with span
type TokenLocation = (Token, Range<usize>);

/// Parse label from a token slice
///
/// Extracts a label span from the beginning of the token slice if present.
/// A label is identified as an identifier (Text, Dash, Number, Period tokens)
/// that is NOT followed by an equals sign.
///
/// # Arguments
/// * `tokens` - Slice of tokens to parse
///
/// # Returns
/// * `Some(Range<usize>)` - The span of the label in the source text
/// * `None` - No label found (either no identifier, or identifier followed by '=')
///
/// # Examples
/// ```text
/// :: note ::           -> Some(label_location for "note")
/// :: note key=val ::   -> Some(label_location for "note")
/// :: key=val ::        -> None (this is a parameter, not a label)
/// ```
pub(crate) fn parse_label_from_tokens(tokens: &[TokenLocation]) -> (Option<Range<usize>>, usize) {
    let mut i = 0;

    // Skip leading whitespace
    while i < tokens.len() && matches!(tokens[i].0, Token::Whitespace) {
        i += 1;
    }

    if i >= tokens.len() {
        return (None, i);
    }

    // Check if first word is a label by looking ahead for '='
    // A label is: word(s) followed by whitespace/comma (NOT equals)
    let start = i;

    // Collect identifier tokens (Text, Dash, Number, Period)
    while i < tokens.len()
        && matches!(
            tokens[i].0,
            Token::Text(_) | Token::Dash | Token::Number(_) | Token::Period
        )
    {
        i += 1;
    }

    if i == start {
        // No identifier found
        return (None, i);
    }

    // Check what comes after: if it's '=', this is NOT a label but a parameter key
    // Skip optional whitespace to check
    let mut peek = i;
    while peek < tokens.len() && matches!(tokens[peek].0, Token::Whitespace) {
        peek += 1;
    }

    if peek < tokens.len() && matches!(tokens[peek].0, Token::Equals) {
        // This is a parameter key, not a label
        (None, 0) // Return 0 to indicate we should restart parsing from beginning
    } else {
        // This is a label
        let first_location = &tokens[start].1;
        let last_location = &tokens[i - 1].1;
        let label_location = Some(first_location.start..last_location.end);

        // Skip trailing whitespace after label
        while i < tokens.len() && matches!(tokens[i].0, Token::Whitespace) {
            i += 1;
        }

        (label_location, i)
    }
}

#[cfg(test)]
mod tests {

    use crate::txxt_nano::lexer::lex_with_locations;
    use crate::txxt_nano::parser::parse_with_source;

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
