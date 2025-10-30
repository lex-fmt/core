//! Parameter parsing for annotations
//!
//! This module handles parsing of annotation parameters in the txxt format.
//! Parameters are key-value pairs separated by commas within annotation markers.
//!
//! Grammar: `<parameters> = <parameter> ("," <parameter>)*`
//! Where: `<parameter> = <key> "=" <value>`

use crate::txxt_nano::ast::Parameter;
use crate::txxt_nano::lexer::Token;
use std::ops::Range;

/// Type alias for token with span
type TokenLocation = (Token, Range<usize>);

/// Parameter with source text spans for later extraction
#[derive(Debug, Clone)]
pub(crate) struct ParameterWithSpans {
    pub(crate) key_location: Range<usize>,
    pub(crate) value_location: Option<Range<usize>>,
}

/// Convert a parameter from spans to final AST
///
/// Extracts the key and value text from the source using the stored spans
pub(crate) fn convert_parameter(source: &str, param: ParameterWithSpans) -> Parameter {
    let key = extract_text(source, &param.key_location).to_string();

    let value = param
        .value_location
        .map(|value_location| extract_text(source, &value_location).to_string());

    Parameter {
        key,
        value,
        span: None,
    }
}

/// Extract text from source using a span range
fn extract_text<'a>(source: &'a str, span: &Range<usize>) -> &'a str {
    &source[span.start..span.end]
}

/// Helper function to parse parameters from a token slice
///
/// Simplified parameter parsing:
/// 1. Split by comma
/// 2. For each segment, split by '=' to get key/value
/// 3. Whitespace around parameters is ignored
pub(crate) fn parse_parameters_from_tokens(tokens: &[TokenLocation]) -> Vec<ParameterWithSpans> {
    let mut params = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        // Skip leading whitespace
        while i < tokens.len() && matches!(tokens[i].0, Token::Whitespace) {
            i += 1;
        }

        if i >= tokens.len() {
            break;
        }

        // Parse key: identifier tokens (Text, Dash, Number)
        let key_start = i;
        while i < tokens.len()
            && matches!(tokens[i].0, Token::Text(_) | Token::Dash | Token::Number(_))
        {
            i += 1;
        }

        if i == key_start {
            // No key found, skip to next comma
            while i < tokens.len() && !matches!(tokens[i].0, Token::Comma) {
                i += 1;
            }
            if i < tokens.len() {
                i += 1; // Skip comma
            }
            continue;
        }

        let key_location = {
            let first_location = &tokens[key_start].1;
            let last_location = &tokens[i - 1].1;
            first_location.start..last_location.end
        };

        // Skip whitespace before '='
        while i < tokens.len() && matches!(tokens[i].0, Token::Whitespace) {
            i += 1;
        }

        // Require '=' sign (no boolean parameters)
        if i >= tokens.len() || !matches!(tokens[i].0, Token::Equals) {
            // Skip this malformed parameter and move to next comma
            while i < tokens.len() && !matches!(tokens[i].0, Token::Comma) {
                i += 1;
            }
            if i < tokens.len() {
                i += 1; // Skip comma
            }
            continue;
        }

        i += 1; // Skip '='

        // Skip whitespace after '='
        while i < tokens.len() && matches!(tokens[i].0, Token::Whitespace) {
            i += 1;
        }

        // Parse value - could be quoted or unquoted
        let value_location = if i < tokens.len() && matches!(tokens[i].0, Token::Quote) {
            i += 1; // Skip opening quote
            let val_start = i;

            // Collect until closing quote
            while i < tokens.len() && !matches!(tokens[i].0, Token::Quote) {
                i += 1;
            }

            let val_location = if val_start < i {
                let first_location = &tokens[val_start].1;
                let last_location = &tokens[i - 1].1;
                Some(first_location.start..last_location.end)
            } else {
                Some(0..0) // Empty quoted string
            };

            if i < tokens.len() && matches!(tokens[i].0, Token::Quote) {
                i += 1; // Skip closing quote
            }

            val_location
        } else {
            // Unquoted value: collect until comma or whitespace
            let val_start = i;
            while i < tokens.len() && !matches!(tokens[i].0, Token::Comma | Token::Whitespace) {
                i += 1;
            }

            if val_start < i {
                let first_location = &tokens[val_start].1;
                let last_location = &tokens[i - 1].1;
                Some(first_location.start..last_location.end)
            } else {
                // No value found, skip this parameter
                while i < tokens.len() && !matches!(tokens[i].0, Token::Comma) {
                    i += 1;
                }
                if i < tokens.len() {
                    i += 1; // Skip comma
                }
                continue;
            }
        };

        params.push(ParameterWithSpans {
            key_location,
            value_location,
        });

        // Skip trailing whitespace
        while i < tokens.len() && matches!(tokens[i].0, Token::Whitespace) {
            i += 1;
        }

        // Skip comma separator
        if i < tokens.len() && matches!(tokens[i].0, Token::Comma) {
            i += 1;
        }
    }

    params
}

#[cfg(test)]
mod tests {
    use crate::txxt_nano::lexer::lex_with_locations;
    use crate::txxt_nano::parser::parse_with_source;

    #[test]
    fn test_annotation_comma_separated_parameters() {
        let source = ":: warning severity=high,priority=urgent ::\n\nText. {{paragraph}}\n";
        let tokens = lex_with_locations(source);
        let doc = parse_with_source(tokens, source).unwrap();

        let annotation = doc.content[0].as_annotation().unwrap();
        assert_eq!(annotation.label.value, "warning");
        assert_eq!(annotation.parameters.len(), 2);
        assert_eq!(annotation.parameters[0].key, "severity");
        assert_eq!(annotation.parameters[0].value, Some("high".to_string()));
        assert_eq!(annotation.parameters[1].key, "priority");
        assert_eq!(annotation.parameters[1].value, Some("urgent".to_string()));
    }

    #[test]
    fn test_annotation_quoted_string_values() {
        let source =
            ":: note author=\"Jane Doe\" title=\"Important Note\" ::\n\nText. {{paragraph}}\n";
        let tokens = lex_with_locations(source);
        let doc = parse_with_source(tokens, source).unwrap();

        let annotation = doc.content[0].as_annotation().unwrap();
        assert_eq!(annotation.label.value, "note");
        assert_eq!(annotation.parameters.len(), 2);
        assert_eq!(annotation.parameters[0].key, "author");
        assert_eq!(annotation.parameters[0].value, Some("Jane Doe".to_string()));
        assert_eq!(annotation.parameters[1].key, "title");
        assert_eq!(
            annotation.parameters[1].value,
            Some("Important Note".to_string())
        );
    }

    #[test]
    fn test_annotation_mixed_separators_and_quotes() {
        let source = ":: task priority=high,status=\"in progress\",assigned=alice ::\n\nText. {{paragraph}}\n";
        let tokens = lex_with_locations(source);
        let doc = parse_with_source(tokens, source).unwrap();

        let annotation = doc.content[0].as_annotation().unwrap();
        assert_eq!(annotation.parameters.len(), 3);
        assert_eq!(annotation.parameters[0].key, "priority");
        assert_eq!(annotation.parameters[0].value, Some("high".to_string()));
        assert_eq!(annotation.parameters[1].key, "status");
        assert_eq!(
            annotation.parameters[1].value,
            Some("in progress".to_string())
        );
        assert_eq!(annotation.parameters[2].key, "assigned");
        assert_eq!(annotation.parameters[2].value, Some("alice".to_string()));
    }

    #[test]
    fn test_annotation_whitespace_around_commas() {
        // Test that whitespace around commas is properly ignored
        let source = ":: note key1=val1 , key2=val2 , key3=val3 ::\n\nText. {{paragraph}}\n";
        let tokens = lex_with_locations(source);
        let doc = parse_with_source(tokens, source).unwrap();

        let annotation = doc.content[0].as_annotation().unwrap();
        assert_eq!(annotation.label.value, "note");
        assert_eq!(annotation.parameters.len(), 3);
        assert_eq!(annotation.parameters[0].key, "key1");
        assert_eq!(annotation.parameters[0].value, Some("val1".to_string()));
        assert_eq!(annotation.parameters[1].key, "key2");
        assert_eq!(annotation.parameters[1].value, Some("val2".to_string()));
        assert_eq!(annotation.parameters[2].key, "key3");
        assert_eq!(annotation.parameters[2].value, Some("val3".to_string()));
    }
}
