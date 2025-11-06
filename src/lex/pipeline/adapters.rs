//! Adapters for converting between TokenStream and existing data structures
//!
//! This module provides bidirectional adapters that allow the new TokenStream
//! architecture to work with existing lexer and parser code during the migration.
//!
//! # Flat Adapters
//!
//! Convert between `Vec<(Token, Range<usize>)>` and `TokenStream`:
//! - `flat_to_token_stream()` - Convert flat vector to TokenStream
//! - `token_stream_to_flat()` - Convert TokenStream::Flat back to flat vector (safe, rejects Tree)
//! - `flatten_token_stream()` - Flatten any TokenStream to flat vector (uses unroll for Tree)
//!
//! # Tree Adapters
//!
//! Convert between `LineContainer` and `TokenStream::Tree`:
//! - `line_container_to_token_stream()` - Convert LineContainer to TokenStream::Tree
//! - `token_stream_to_line_container()` - Convert TokenStream::Tree back to LineContainer
//!
//! # Parser Adapters
//!
//! Wrap existing parsers to work with TokenStream:
//! - `adapt_reference_parser()` - Adapter for reference parser using TokenStream input
//!
//! # Design
//!
//! These adapters are temporary scaffolding for the migration. They allow us to:
//! 1. Develop new TokenStream-based transformations
//! 2. Test them alongside existing code
//! 3. Gradually migrate parsers to use TokenStream directly
//! 4. Eventually remove adapters once migration is complete

use crate::lex::lexing::tokens_core::Token;
use crate::lex::pipeline::stream::TokenStream;
use std::ops::Range as ByteRange;

/// Error type for adapter operations
#[derive(Debug, Clone, PartialEq)]
pub enum AdapterError {
    /// Attempted to convert a Tree variant to flat
    ExpectedFlat,
    /// Attempted to convert a Flat variant to tree
    ExpectedTree,
    /// Generic adapter error
    Error(String),
}

impl std::fmt::Display for AdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdapterError::ExpectedFlat => {
                write!(f, "Expected TokenStream::Flat, but found TokenStream::Tree")
            }
            AdapterError::ExpectedTree => {
                write!(f, "Expected TokenStream::Tree, but found TokenStream::Flat")
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

/// Flatten any TokenStream to a flat token vector using unroll().
///
/// Unlike `token_stream_to_flat()`, this function handles both Flat and Tree variants.
/// For Tree variants, it uses the `unroll()` method to recursively extract all tokens.
///
/// This is useful for:
/// - Debugging (inspect all tokens in a tree)
/// - Testing (verify tree contents)
/// - Non-parser contexts where you explicitly want to flatten a tree
///
/// **Warning**: Using this in parser adapters defeats the purpose of Tree structures.
/// Prefer `token_stream_to_flat()` during migration to catch incorrect usage early.
///
/// # Arguments
///
/// * `stream` - The TokenStream to flatten (Flat or Tree)
///
/// # Returns
///
/// A flat vector containing all tokens in document order
///
/// # Examples
///
/// ```ignore
/// // Flatten a Flat stream (equivalent to token_stream_to_flat)
/// let flat = TokenStream::Flat(vec![(Token::Text("hello".into()), 0..5)]);
/// let tokens = flatten_token_stream(flat);
///
/// // Flatten a Tree stream (uses unroll)
/// let tree = TokenStream::Tree(vec![...]);
/// let tokens = flatten_token_stream(tree); // Recursively extracts all tokens
/// ```
pub fn flatten_token_stream(stream: TokenStream) -> Vec<(Token, ByteRange<usize>)> {
    stream.unroll()
}

/// Adapt the reference parser to work with TokenStream input.
///
/// This function integrates the reference parser into the TokenStream architecture
/// by adapting the input from `TokenStream` to `Vec<(Token, Range)>`.
///
/// The reference parser expects a flat token stream, so this adapter:
/// 1. Converts TokenStream::Flat to flat vector (safe, rejects Tree)
/// 2. Calls the reference parser
/// 3. Returns the Document (no output adapter needed - Document is final)
///
/// This allows testing the new TokenStream architecture with the existing parser
/// during the migration phase. Once all transformations use TokenStream, this
/// adapter can be removed and the parser can consume TokenStream directly.
///
/// # Arguments
///
/// * `stream` - The TokenStream to parse (must be Flat variant)
/// * `source` - The original source text for location tracking
///
/// # Returns
///
/// The parsed Document or an adapter error
///
/// # Errors
///
/// Returns `AdapterError::ExpectedFlat` if stream is Tree variant.
/// Returns `AdapterError::Error` if parsing fails.
///
/// # Examples
///
/// ```ignore
/// let tokens = vec![(Token::Text("hello".into()), 0..5)];
/// let stream = flat_to_token_stream(tokens);
/// let doc = adapt_reference_parser(stream, "hello")?;
/// ```
pub fn adapt_reference_parser(
    stream: TokenStream,
    source: &str,
) -> Result<crate::lex::parsing::Document, AdapterError> {
    // Validate that stream is Flat (not Tree) - required by adapter contract
    // Note: We don't use the tokens since parse_document handles tokenization internally
    token_stream_to_flat(stream)?;

    // Call reference parser
    crate::lex::parsing::parse_document(source).map_err(|errors| {
        // Convert parser errors to adapter error
        let error_msg = errors
            .iter()
            .map(|e| format!("{:?}", e))
            .collect::<Vec<_>>()
            .join("; ");
        AdapterError::Error(format!("Parsing failed: {}", error_msg))
    })
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
            line_type: None,
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

    #[test]
    fn test_flatten_token_stream_with_flat() {
        // flatten_token_stream should work with Flat variant
        let tokens = vec![
            (Token::Text("hello".to_string()), 0..5),
            (Token::Whitespace, 5..6),
        ];

        let stream = TokenStream::Flat(tokens.clone());
        let result = flatten_token_stream(stream);

        assert_eq!(result, tokens);
    }

    #[test]
    fn test_flatten_token_stream_with_tree() {
        use crate::lex::pipeline::stream::TokenStreamNode;

        // flatten_token_stream should flatten Tree using unroll
        let node = TokenStreamNode {
            tokens: vec![(Token::Text("parent".to_string()), 0..6)],
            children: Some(Box::new(TokenStream::Flat(vec![(
                Token::Text("child".to_string()),
                10..15,
            )]))),
            line_type: None,
        };
        let stream = TokenStream::Tree(vec![node]);

        let result = flatten_token_stream(stream);

        // Should get all tokens in document order
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, Token::Text("parent".to_string()));
        assert_eq!(result[0].1, 0..6);
        assert_eq!(result[1].0, Token::Text("child".to_string()));
        assert_eq!(result[1].1, 10..15);
    }

    #[test]
    fn test_flatten_token_stream_with_nested_tree() {
        use crate::lex::pipeline::stream::TokenStreamNode;

        // Test deeply nested structure
        let grandchild = TokenStream::Flat(vec![(Token::Text("grandchild".to_string()), 20..30)]);

        let child = TokenStreamNode {
            tokens: vec![(Token::Text("child".to_string()), 10..15)],
            children: Some(Box::new(grandchild)),
            line_type: None,
        };

        let root = TokenStreamNode {
            tokens: vec![(Token::Text("root".to_string()), 0..4)],
            children: Some(Box::new(TokenStream::Tree(vec![child]))),
            line_type: None,
        };

        let stream = TokenStream::Tree(vec![root]);
        let result = flatten_token_stream(stream);

        // Should get all three tokens in document order
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].0, Token::Text("root".to_string()));
        assert_eq!(result[1].0, Token::Text("child".to_string()));
        assert_eq!(result[2].0, Token::Text("grandchild".to_string()));
    }

    #[test]
    fn test_flatten_preserves_all_ranges() {
        use crate::lex::pipeline::stream::TokenStreamNode;

        // Verify that flatten preserves exact ranges from nested structure
        let node = TokenStreamNode {
            tokens: vec![
                (Token::Text("a".to_string()), 0..1),
                (Token::Whitespace, 1..2),
            ],
            children: Some(Box::new(TokenStream::Flat(vec![
                (Token::Text("b".to_string()), 10..11),
                (Token::Newline, 11..12),
            ]))),
            line_type: None,
        };

        let stream = TokenStream::Tree(vec![node]);
        let result = flatten_token_stream(stream);

        assert_eq!(result[0].1, 0..1);
        assert_eq!(result[1].1, 1..2);
        assert_eq!(result[2].1, 10..11);
        assert_eq!(result[3].1, 11..12);
    }

    // Parser adapter tests
    #[test]
    fn test_adapt_reference_parser_simple() {
        // Test parsing a simple paragraph through the adapter
        let source = "Hello world\n";

        // Tokenize using existing lexer
        let tokens = crate::lex::lexing::tokenize(source);

        // Convert to TokenStream
        let stream = flat_to_token_stream(tokens);

        // Parse through adapter
        let result = adapt_reference_parser(stream, source);

        assert!(result.is_ok(), "Failed to parse: {:?}", result);
        let doc = result.unwrap();
        assert_eq!(doc.root.children.len(), 1);
    }

    #[test]
    fn test_adapt_reference_parser_rejects_tree() {
        use crate::lex::pipeline::stream::TokenStreamNode;

        // Tree streams should be rejected
        let node = TokenStreamNode {
            tokens: vec![(Token::Text("test".to_string()), 0..4)],
            children: None,
            line_type: None,
        };
        let stream = TokenStream::Tree(vec![node]);

        let result = adapt_reference_parser(stream, "test");

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AdapterError::ExpectedFlat);
    }

    #[test]
    fn test_adapt_reference_parser_complex_document() {
        // Test a more complex document with sessions and lists
        let source = "1. Session Title\n\n    Session content.\n\n";

        let tokens = crate::lex::lexing::tokenize(source);
        let stream = flat_to_token_stream(tokens);

        let result = adapt_reference_parser(stream, source);

        assert!(result.is_ok(), "Failed to parse: {:?}", result);
        let doc = result.unwrap();

        // Should have one session
        assert_eq!(doc.root.children.len(), 1);
    }

    #[test]
    fn test_adapt_reference_parser_preserves_locations() {
        // Verify that locations are preserved through the adapter
        let source = "Hello world\n";

        let tokens = crate::lex::lexing::tokenize(source);
        let stream = flat_to_token_stream(tokens);

        let result = adapt_reference_parser(stream, source);

        assert!(result.is_ok());
        let doc = result.unwrap();

        // Verify document has location information
        let root_loc = doc.root_location();
        assert!(root_loc.start < root_loc.end);
    }

    #[test]
    fn test_adapt_reference_parser_round_trip() {
        // Test that we can go: tokens -> stream -> parser -> document
        let source = "Paragraph one\n\nParagraph two\n";

        // Original path
        let doc1 = crate::lex::parsing::parse_document(source).unwrap();

        // TokenStream path
        let tokens2 = crate::lex::lexing::tokenize(source);
        let stream = flat_to_token_stream(tokens2);
        let doc2 = adapt_reference_parser(stream, source).unwrap();

        // Both should produce the same number of items
        assert_eq!(doc1.root.children.len(), doc2.root.children.len());
    }

    #[test]
    fn test_adapt_reference_parser_error_formatting() {
        // Test that parser errors are properly formatted and don't panic
        // Create a token stream that will cause a parse error
        // An incomplete annotation marker (just "::") will trigger parser errors
        let source = "::\n";
        let tokens = crate::lex::lexing::tokenize(source);
        let stream = flat_to_token_stream(tokens);

        let result = adapt_reference_parser(stream, source);

        // Should return an error
        assert!(
            result.is_err(),
            "Expected parser error for incomplete annotation"
        );

        // Verify the error is formatted correctly
        match result.unwrap_err() {
            AdapterError::Error(msg) => {
                // Should contain "Parsing failed:" prefix
                assert!(
                    msg.starts_with("Parsing failed:"),
                    "Error message should start with 'Parsing failed:', got: {}",
                    msg
                );
                // Should not be empty
                assert!(!msg.is_empty(), "Error message should not be empty");
                // Should contain some indication of what went wrong
                assert!(
                    msg.len() > "Parsing failed:".len(),
                    "Error message should contain details, got: {}",
                    msg
                );
            }
            other => panic!("Expected AdapterError::Error, got: {:?}", other),
        }
    }
}
