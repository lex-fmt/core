//! Whitespace normalization mapper for TokenStream pipeline
//!
//! This mapper removes remainder Whitespace tokens that appear between
//! Indentation tokens and Text tokens. This handles the case where indentation
//! leaves trailing spaces before the actual text content begins.
//!
//! # Logic
//!
//! For each Whitespace token:
//! - Check if preceded by consecutive Indentation token(s)
//! - Check if followed by a Text token
//! - If both conditions are met, remove the Whitespace token
//! - Otherwise, preserve it
//!
//! This is a pure adaptation of the existing normalize_whitespace transformation
//! to the TokenStream architecture.

use crate::lex::lexers::tokens_core::Token;
use crate::lex::pipeline::mapper::{StreamMapper, TransformationError};
use crate::lex::pipeline::stream::TokenStream;
use std::ops::Range as ByteRange;

/// A mapper that normalizes whitespace by removing remainder spaces after indentation.
///
/// This transformation only operates on flat token streams and preserves all
/// token ranges exactly as they appear in the source.
pub struct NormalizeWhitespaceMapper;

impl NormalizeWhitespaceMapper {
    /// Create a new NormalizeWhitespaceMapper.
    pub fn new() -> Self {
        NormalizeWhitespaceMapper
    }
}

impl Default for NormalizeWhitespaceMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamMapper for NormalizeWhitespaceMapper {
    fn map_flat(
        &mut self,
        tokens: Vec<(Token, ByteRange<usize>)>,
    ) -> Result<TokenStream, TransformationError> {
        let mut result = Vec::new();
        let mut i = 0;

        while i < tokens.len() {
            let (token, location) = &tokens[i];

            match token {
                Token::Whitespace => {
                    // Count consecutive Indent tokens before this Whitespace
                    let mut indent_count = 0;
                    let mut j = i;
                    while j > 0 && matches!(tokens[j - 1].0, Token::Indentation) {
                        indent_count += 1;
                        j -= 1;
                    }

                    // If we have indentation and this whitespace is followed by text,
                    // skip this whitespace token (it will be considered part of the text)
                    if indent_count > 0
                        && i + 1 < tokens.len()
                        && matches!(tokens[i + 1].0, Token::Text(_))
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

        Ok(TokenStream::Flat(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::lexers::tokens_core::Token;

    #[test]
    fn test_normalize_whitespace_with_indentation() {
        // Test: Indentation followed by whitespace followed by text
        // The whitespace should be removed
        let tokens = vec![
            (Token::Indentation, 0..4),
            (Token::Whitespace, 4..5),
            (Token::Text("hello".to_string()), 5..10),
        ];

        let mut mapper = NormalizeWhitespaceMapper::new();
        let result = mapper.map_flat(tokens).unwrap();

        match result {
            TokenStream::Flat(output) => {
                assert_eq!(output.len(), 2);
                assert!(matches!(output[0].0, Token::Indentation));
                assert!(matches!(output[1].0, Token::Text(_)));
                // Whitespace should be removed
            }
            _ => panic!("Expected Flat stream"),
        }
    }

    #[test]
    fn test_normalize_whitespace_without_indentation() {
        // Test: Whitespace not preceded by indentation should be preserved
        let tokens = vec![
            (Token::Text("hello".to_string()), 0..5),
            (Token::Whitespace, 5..6),
            (Token::Text("world".to_string()), 6..11),
        ];

        let mut mapper = NormalizeWhitespaceMapper::new();
        let result = mapper.map_flat(tokens).unwrap();

        match result {
            TokenStream::Flat(output) => {
                assert_eq!(output.len(), 3);
                assert!(matches!(output[0].0, Token::Text(_)));
                assert!(matches!(output[1].0, Token::Whitespace));
                assert!(matches!(output[2].0, Token::Text(_)));
            }
            _ => panic!("Expected Flat stream"),
        }
    }

    #[test]
    fn test_normalize_whitespace_multiple_indent_levels() {
        // Test: Multiple indentation tokens followed by whitespace and text
        let tokens = vec![
            (Token::Indentation, 0..4),
            (Token::Indentation, 4..8),
            (Token::Whitespace, 8..9),
            (Token::Text("nested".to_string()), 9..15),
        ];

        let mut mapper = NormalizeWhitespaceMapper::new();
        let result = mapper.map_flat(tokens).unwrap();

        match result {
            TokenStream::Flat(output) => {
                assert_eq!(output.len(), 3);
                assert!(matches!(output[0].0, Token::Indentation));
                assert!(matches!(output[1].0, Token::Indentation));
                assert!(matches!(output[2].0, Token::Text(_)));
                // Whitespace should be removed
            }
            _ => panic!("Expected Flat stream"),
        }
    }

    #[test]
    fn test_normalize_whitespace_not_followed_by_text() {
        // Test: Indentation followed by whitespace NOT followed by text
        // The whitespace should be preserved
        let tokens = vec![
            (Token::Indentation, 0..4),
            (Token::Whitespace, 4..5),
            (Token::Newline, 5..6),
        ];

        let mut mapper = NormalizeWhitespaceMapper::new();
        let result = mapper.map_flat(tokens).unwrap();

        match result {
            TokenStream::Flat(output) => {
                assert_eq!(output.len(), 3);
                assert!(matches!(output[0].0, Token::Indentation));
                assert!(matches!(output[1].0, Token::Whitespace));
                assert!(matches!(output[2].0, Token::Newline));
            }
            _ => panic!("Expected Flat stream"),
        }
    }

    #[test]
    fn test_normalize_whitespace_preserves_ranges() {
        // Test: Verify that byte ranges are preserved exactly
        let tokens = vec![
            (Token::Indentation, 0..4),
            (Token::Whitespace, 4..5),
            (Token::Text("test".to_string()), 5..9),
        ];

        let mut mapper = NormalizeWhitespaceMapper::new();
        let result = mapper.map_flat(tokens).unwrap();

        match result {
            TokenStream::Flat(output) => {
                assert_eq!(output[0].1, 0..4);
                assert_eq!(output[1].1, 5..9);
            }
            _ => panic!("Expected Flat stream"),
        }
    }

    #[test]
    fn test_normalize_whitespace_empty_input() {
        let tokens = vec![];

        let mut mapper = NormalizeWhitespaceMapper::new();
        let result = mapper.map_flat(tokens).unwrap();

        match result {
            TokenStream::Flat(output) => {
                assert_eq!(output.len(), 0);
            }
            _ => panic!("Expected Flat stream"),
        }
    }

    #[test]
    fn test_normalize_whitespace_no_whitespace_tokens() {
        // Test: No whitespace tokens to process
        let tokens = vec![
            (Token::Indentation, 0..4),
            (Token::Text("hello".to_string()), 4..9),
            (Token::Newline, 9..10),
        ];

        let mut mapper = NormalizeWhitespaceMapper::new();
        let result = mapper.map_flat(tokens).unwrap();

        match result {
            TokenStream::Flat(output) => {
                assert_eq!(output.len(), 3);
                assert!(matches!(output[0].0, Token::Indentation));
                assert!(matches!(output[1].0, Token::Text(_)));
                assert!(matches!(output[2].0, Token::Newline));
            }
            _ => panic!("Expected Flat stream"),
        }
    }
}
