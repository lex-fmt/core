//! Whitespace remainder transformation for txxt lexer
//!
//! This module transforms raw tokens to process whitespace remainders according to txxt specification.
//!
//! The spec states: "lines that have space remainders (as in 10 spaces, which converts to
//! 2 tab stops with 2 spaces remaining) will be parsed with no error. Only two indentation
//! level tokens will be generated, and the remaining whitespaces will be considered part of the text."

use crate::txxt::lexer::tokens::Token;

/// Process whitespace remainders according to txxt specification
///
/// The spec states: "lines that have space remainders (as in 10 spaces, which converts to
/// 2 tab stops with 2 spaces remaining) will be parsed with no error. Only two indentation
/// level tokens will be generated, and the remaining whitespaces will be considered part of the text."
///
/// This function removes Whitespace tokens that follow Indent tokens and precede Text tokens,
/// effectively merging the whitespace remainder into the text content.
pub fn process_whitespace_remainders(
    tokens_with_locations: Vec<(Token, logos::Span)>,
) -> Vec<Token> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < tokens_with_locations.len() {
        let (token, _location) = &tokens_with_locations[i];

        match token {
            Token::Whitespace => {
                // Check if this whitespace follows indentation tokens and precedes text content
                let mut indent_count = 0;
                let mut j = i;

                // Count consecutive Indent tokens before this Whitespace
                while j > 0 && matches!(tokens_with_locations[j - 1].0, Token::Indent) {
                    indent_count += 1;
                    j -= 1;
                }

                // If we have indentation and this whitespace is followed by text,
                // skip this whitespace token (it will be considered part of the text)
                if indent_count > 0
                    && i + 1 < tokens_with_locations.len()
                    && matches!(tokens_with_locations[i + 1].0, Token::Text(_))
                {
                    // Skip this whitespace token
                    i += 1;
                    continue;
                }

                // Otherwise, keep the whitespace token as-is
                result.push(token.clone());
            }
            _ => {
                result.push(token.clone());
            }
        }
        i += 1;
    }

    result
}

/// Process whitespace remainders while preserving source locations
pub fn process_whitespace_remainders_with_locations(
    tokens_with_locations: Vec<(Token, std::ops::Range<usize>)>,
) -> Vec<(Token, std::ops::Range<usize>)> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < tokens_with_locations.len() {
        let (token, location) = &tokens_with_locations[i];

        match token {
            Token::Whitespace => {
                // Check if this whitespace follows indentation tokens and precedes text content
                let mut indent_count = 0;
                let mut j = i;

                // Count consecutive Indent tokens before this Whitespace
                while j > 0 && matches!(tokens_with_locations[j - 1].0, Token::Indent) {
                    indent_count += 1;
                    j -= 1;
                }

                // If we have indentation and this whitespace is followed by text,
                // skip this whitespace token (it will be considered part of the text)
                if indent_count > 0
                    && i + 1 < tokens_with_locations.len()
                    && matches!(tokens_with_locations[i + 1].0, Token::Text(_))
                {
                    // Skip this whitespace token
                    i += 1;
                    continue;
                }

                // Otherwise, keep the whitespace token with its location
                result.push((token.clone(), location.clone()));
            }
            _ => {
                result.push((token.clone(), location.clone()));
            }
        }
        i += 1;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whitespace_remainders() {
        // Test case: 10 spaces should produce 2 Indent tokens (8 spaces)
        // and the remaining 2 spaces should be part of text content
        let tokens_with_locations = crate::txxt::lexer::tokenize_with_locations("          hello");
        let result = process_whitespace_remainders(tokens_with_locations);
        println!("Tokens for '          hello': {:?}", result);

        // According to spec: 10 spaces = 2 indent levels (8 spaces) + 2 remaining spaces
        // The remaining 2 spaces should be considered part of the text, not separate whitespace
        assert_eq!(result.len(), 3); // Should be: [Indent, Indent, Text("  hello")]
        assert_eq!(result[0], Token::Indent);
        assert_eq!(result[1], Token::Indent);
        assert_eq!(result[2], Token::Text("hello".to_string()));
    }

    #[test]
    fn test_whitespace_without_indentation() {
        // Whitespace not following indent should be preserved
        let tokens_with_locations = vec![
            (Token::Text("hello".to_string()), 0..5),
            (Token::Whitespace, 5..6),
            (Token::Text("world".to_string()), 6..11),
        ];
        let result = process_whitespace_remainders_with_locations(tokens_with_locations.clone());
        assert_eq!(result, tokens_with_locations);
    }

    #[test]
    fn test_whitespace_after_indent_before_non_text() {
        // Whitespace after indent but before non-text token should be preserved
        let tokens_with_locations = vec![
            (Token::Indent, 0..4),
            (Token::Whitespace, 4..5),
            (Token::Dash, 5..6),
        ];
        let result = process_whitespace_remainders_with_locations(tokens_with_locations.clone());
        assert_eq!(result, tokens_with_locations);
    }

    #[test]
    fn test_multiple_whitespace_remainders() {
        // Test with multiple indent levels and remainder spaces
        let tokens_with_locations =
            crate::txxt::lexer::tokenize_with_locations("            hello");
        let result = process_whitespace_remainders(tokens_with_locations);

        // 12 spaces = 3 indent levels (no remainder)
        assert_eq!(result[0], Token::Indent);
        assert_eq!(result[1], Token::Indent);
        assert_eq!(result[2], Token::Indent);
        assert_eq!(result[3], Token::Text("hello".to_string()));
    }
}
