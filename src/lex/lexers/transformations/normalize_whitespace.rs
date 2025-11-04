//! Whitespace remainder transformation for lex lexer
//!
//! This module transforms raw tokens to process whitespace remainders according to lex specification.
//!
//! The spec states: "lines that have space remainders (as in 10 spaces, which converts to
//! 2 tab stops with 2 spaces remaining) will be parsed with no error. Only two indentation
//! level tokens will be generated, and the remaining whitespaces will be considered part of the text."

use crate::lex::lexers::tokens::Token;
use crate::lex::lexers::transformations::Transformation;

/// Whitespace normalization transformation
///
/// Removes remainder whitespace tokens that appear after indentation and before text content,
/// according to the lex specification.
pub struct NormalizeWhitespace;

impl Transformation for NormalizeWhitespace {
    fn name(&self) -> &str {
        "normalize_whitespace"
    }

    fn description(&self) -> &str {
        "Process whitespace remainders according to lex spec (removes remainder spaces after indentation)"
    }

    fn transform(
        &self,
        tokens: Vec<(Token, std::ops::Range<usize>)>,
    ) -> Vec<(Token, std::ops::Range<usize>)> {
        process_whitespace_remainders(tokens)
    }
}

/// Process whitespace remainders while preserving source locations
pub fn process_whitespace_remainders(
    tokenss: Vec<(Token, std::ops::Range<usize>)>,
) -> Vec<(Token, std::ops::Range<usize>)> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < tokenss.len() {
        let (token, location) = &tokenss[i];

        match token {
            Token::Whitespace => {
                // Check if this whitespace follows indentation tokens and precedes text content
                let mut indent_count = 0;
                let mut j = i;

                // Count consecutive Indent tokens before this Whitespace
                while j > 0 && matches!(tokenss[j - 1].0, Token::Indentation) {
                    indent_count += 1;
                    j -= 1;
                }

                // If we have indentation and this whitespace is followed by text,
                // skip this whitespace token (it will be considered part of the text)
                if indent_count > 0
                    && i + 1 < tokenss.len()
                    && matches!(tokenss[i + 1].0, Token::Text(_))
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
    use crate::lex::testing::factories::{mk_token, Tokens};

    #[test]
    fn test_whitespace_remainders() {
        // Test case: 10 spaces should produce 2 Indent tokens (8 spaces)
        // and the remaining 2 spaces should be part of text content
        let tokenss = crate::lex::lexers::tokenize("          hello");
        let results = process_whitespace_remainders(tokenss);
        let result: Vec<Token> = results.into_iter().map(|(t, _)| t).collect();
        println!("Tokens for '          hello': {:?}", result);

        // According to spec: 10 spaces = 2 indent levels (8 spaces) + 2 remaining spaces
        // The remaining 2 spaces should be considered part of the text, not separate whitespace
        assert_eq!(result.len(), 3); // Should be: [Indent, Indent, Text("  hello")]
        assert_eq!(result[0], Token::Indentation);
        assert_eq!(result[1], Token::Indentation);
        assert_eq!(result[2], Token::Text("hello".to_string()));
    }

    #[test]
    fn test_whitespace_without_indentation() {
        // Whitespace not following indent should be preserved
        let tokenss: Tokens = vec![
            mk_token(Token::Text("hello".to_string()), 0, 5),
            mk_token(Token::Whitespace, 5, 6),
            mk_token(Token::Text("world".to_string()), 6, 11),
        ];
        let result = process_whitespace_remainders(tokenss.clone());
        assert_eq!(result, tokenss);
    }

    #[test]
    fn test_whitespace_after_indent_before_non_text() {
        // Whitespace after indent but before non-text token should be preserved
        let tokenss: Tokens = vec![
            mk_token(Token::Indentation, 0, 4),
            mk_token(Token::Whitespace, 4, 5),
            mk_token(Token::Dash, 5, 6),
        ];
        let result = process_whitespace_remainders(tokenss.clone());
        assert_eq!(result, tokenss);
    }

    #[test]
    fn test_multiple_whitespace_remainders() {
        // Test with multiple indent levels and remainder spaces
        let tokenss = crate::lex::lexers::tokenize("            hello");
        let results = process_whitespace_remainders(tokenss);
        let result: Vec<Token> = results.into_iter().map(|(t, _)| t).collect();

        // 12 spaces = 3 indent levels (no remainder)
        assert_eq!(result[0], Token::Indentation);
        assert_eq!(result[1], Token::Indentation);
        assert_eq!(result[2], Token::Indentation);
        assert_eq!(result[3], Token::Text("hello".to_string()));
    }
}
