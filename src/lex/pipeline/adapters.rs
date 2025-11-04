//! Adapters for converting between TokenStream and existing data structures
//!
//! This module provides bidirectional adapters that allow the new TokenStream
//! architecture to work with existing lexer and parser code during the migration.
//!
//! # Flat Adapters
//!
//! Convert between `Vec<(Token, Range<usize>)>` and `TokenStream::Flat`:
//! - `flat_to_token_stream()` - Convert flat vector to TokenStream
//! - `token_stream_to_flat()` - Convert TokenStream back to flat vector
//!
//! # Design
//!
//! These adapters are temporary scaffolding for the migration. They allow us to:
//! 1. Develop new TokenStream-based transformations
//! 2. Test them alongside existing code
//! 3. Gradually migrate parsers to use TokenStream directly
//! 4. Eventually remove adapters once migration is complete

use crate::lex::lexers::tokens::Token;
use crate::lex::pipeline::stream::TokenStream;
use std::ops::Range as ByteRange;

/// Error type for adapter operations
#[derive(Debug, Clone, PartialEq)]
pub enum AdapterError {
    /// Attempted to convert a Tree variant to flat
    ExpectedFlat,
    /// Generic adapter error
    Error(String),
}

impl std::fmt::Display for AdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdapterError::ExpectedFlat => {
                write!(f, "Expected TokenStream::Flat, but found TokenStream::Tree")
            }
            AdapterError::Error(msg) => write!(f, "Adapter error: {}", msg),
        }
    }
}

impl std::error::Error for AdapterError {}

/// Convert a flat token vector to a TokenStream.
///
/// This is a simple wrapper that creates a `TokenStream::Flat` variant.
/// Used to adapt existing flat token vectors for use with TokenStream-based
/// transformations.
///
/// # Arguments
///
/// * `tokens` - The flat vector of (Token, Range) pairs
///
/// # Returns
///
/// A `TokenStream::Flat` containing the tokens
///
/// # Examples
///
/// ```ignore
/// let tokens = vec![
///     (Token::Text("hello".into()), 0..5),
///     (Token::Newline, 5..6),
/// ];
/// let stream = flat_to_token_stream(tokens);
/// assert!(matches!(stream, TokenStream::Flat(_)));
/// ```
pub fn flat_to_token_stream(tokens: Vec<(Token, ByteRange<usize>)>) -> TokenStream {
    TokenStream::Flat(tokens)
}

/// Convert a TokenStream back to a flat token vector.
///
/// This adapter extracts tokens from a `TokenStream::Flat` variant.
/// If the stream is a `TokenStream::Tree`, it could use `unroll()` to flatten it,
/// but for safety during migration we return an error instead to catch misuse.
///
/// # Arguments
///
/// * `stream` - The TokenStream to convert
///
/// # Returns
///
/// The flat vector of tokens if the stream was `TokenStream::Flat`,
/// or an error if it was `TokenStream::Tree`.
///
/// # Errors
///
/// Returns `AdapterError::ExpectedFlat` if the stream is a Tree variant.
///
/// # Examples
///
/// ```ignore
/// let stream = TokenStream::Flat(vec![(Token::Text("hello".into()), 0..5)]);
/// let tokens = token_stream_to_flat(stream)?;
/// assert_eq!(tokens.len(), 1);
/// ```
pub fn token_stream_to_flat(
    stream: TokenStream,
) -> Result<Vec<(Token, ByteRange<usize>)>, AdapterError> {
    match stream {
        TokenStream::Flat(tokens) => Ok(tokens),
        TokenStream::Tree(_) => Err(AdapterError::ExpectedFlat),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_to_token_stream() {
        let tokens = vec![
            (Token::Text("hello".to_string()), 0..5),
            (Token::Whitespace, 5..6),
            (Token::Text("world".to_string()), 6..11),
        ];

        let stream = flat_to_token_stream(tokens.clone());

        match stream {
            TokenStream::Flat(result_tokens) => {
                assert_eq!(result_tokens, tokens);
            }
            _ => panic!("Expected TokenStream::Flat"),
        }
    }

    #[test]
    fn test_flat_to_token_stream_empty() {
        let tokens = vec![];
        let stream = flat_to_token_stream(tokens);

        match stream {
            TokenStream::Flat(result_tokens) => {
                assert_eq!(result_tokens.len(), 0);
            }
            _ => panic!("Expected TokenStream::Flat"),
        }
    }

    #[test]
    fn test_token_stream_to_flat() {
        let tokens = vec![
            (Token::Text("hello".to_string()), 0..5),
            (Token::Whitespace, 5..6),
        ];

        let stream = TokenStream::Flat(tokens.clone());
        let result = token_stream_to_flat(stream).unwrap();

        assert_eq!(result, tokens);
    }

    #[test]
    fn test_token_stream_to_flat_empty() {
        let stream = TokenStream::Flat(vec![]);
        let result = token_stream_to_flat(stream).unwrap();

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_token_stream_to_flat_rejects_tree() {
        use crate::lex::pipeline::stream::TokenStreamNode;

        let node = TokenStreamNode {
            tokens: vec![(Token::Text("test".to_string()), 0..4)],
            children: None,
        };
        let stream = TokenStream::Tree(vec![node]);

        let result = token_stream_to_flat(stream);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AdapterError::ExpectedFlat);
    }

    #[test]
    fn test_round_trip_flat() {
        let original = vec![
            (Token::Text("hello".to_string()), 0..5),
            (Token::Whitespace, 5..6),
            (Token::Text("world".to_string()), 6..11),
            (Token::Newline, 11..12),
        ];

        // Convert to stream
        let stream = flat_to_token_stream(original.clone());

        // Convert back to flat
        let result = token_stream_to_flat(stream).unwrap();

        assert_eq!(result, original);
    }

    #[test]
    fn test_round_trip_preserves_ranges() {
        let original = vec![
            (Token::Text("hello".to_string()), 0..5),
            (Token::Whitespace, 5..6),
            (Token::Text("world".to_string()), 6..11),
        ];

        let stream = flat_to_token_stream(original.clone());
        let result = token_stream_to_flat(stream).unwrap();

        // Verify ranges are preserved exactly
        assert_eq!(result[0].1, 0..5);
        assert_eq!(result[1].1, 5..6);
        assert_eq!(result[2].1, 6..11);
    }

    #[test]
    fn test_round_trip_preserves_token_types() {
        let original = vec![
            (Token::Text("test".to_string()), 0..4),
            (Token::Newline, 4..5),
            (Token::Whitespace, 5..6),
            (Token::Dash, 6..7),
            (Token::Colon, 7..8),
        ];

        let stream = flat_to_token_stream(original.clone());
        let result = token_stream_to_flat(stream).unwrap();

        // Verify all token types preserved
        assert_eq!(result.len(), original.len());
        for (orig, res) in original.iter().zip(result.iter()) {
            assert_eq!(orig.0, res.0);
        }
    }

    #[test]
    fn test_adapter_with_synthetic_tokens() {
        // Test with tokens that have embedded source tokens (like Indent, Dedent, BlankLine)
        let original = vec![
            (Token::Text("hello".to_string()), 0..5),
            (Token::Newline, 5..6),
            (
                Token::Indent(vec![(Token::Indentation, 6..10)]),
                0..0, // Placeholder span
            ),
            (Token::Text("indented".to_string()), 10..18),
            (Token::Newline, 18..19),
            (
                Token::Dedent(vec![]), // No source tokens
                0..0,                  // Placeholder span
            ),
        ];

        let stream = flat_to_token_stream(original.clone());
        let result = token_stream_to_flat(stream).unwrap();

        assert_eq!(result, original);
    }

    #[test]
    fn test_adapter_with_blank_line_tokens() {
        let original = vec![
            (Token::Text("line1".to_string()), 0..5),
            (Token::Newline, 5..6),
            (
                Token::BlankLine(vec![(Token::Newline, 6..7)]),
                0..0, // Placeholder span
            ),
            (Token::Text("line2".to_string()), 7..12),
        ];

        let stream = flat_to_token_stream(original.clone());
        let result = token_stream_to_flat(stream).unwrap();

        assert_eq!(result, original);
    }

    #[test]
    fn test_multiple_round_trips() {
        // Verify that multiple conversions don't corrupt data
        let original = vec![
            (Token::Text("test".to_string()), 0..4),
            (Token::Whitespace, 4..5),
        ];

        let mut current = original.clone();

        // Do 5 round trips
        for _ in 0..5 {
            let stream = flat_to_token_stream(current.clone());
            current = token_stream_to_flat(stream).unwrap();
        }

        assert_eq!(current, original);
    }
}
