//! Token processing utilities for location tracking
//!
//! This module provides the core utilities for the Immutable Log Architecture.
//! All functions here are pure and thoroughly unit tested.
//!
//! # Architecture
//!
//! The Logos lexer produces `(Token, Range<usize>)` pairs - this is the ground truth.
//! Transformations create aggregate tokens that store these original pairs in `source_tokens`.
//! This module provides utilities to:
//! 1. Unroll aggregate tokens back to flat lists
//! 2. Compute bounding boxes from token ranges
//! 3. Convert byte ranges to human-readable locations
//! 4. Extract text from ranges

use crate::txxt::ast::location::{Location, SourceLocation};
use crate::txxt::lexers::tokens::Token;
use std::ops::Range;

/// Trait that any token structure can implement to provide access to source tokens.
///
/// This enables the unrolling system to work with any parser's token representation.
pub trait SourceTokenProvider {
    /// Get the original Logos tokens that comprise this token.
    ///
    /// For atomic tokens (direct from Logos), this returns a slice containing just that token.
    /// For aggregate tokens (from transformations), this returns all the original tokens.
    fn source_tokens(&self) -> &[(Token, Range<usize>)];
}

/// Unroll a collection of tokens to a flat list of original Logos tokens.
///
/// This recursively extracts all `source_tokens` from aggregate structures,
/// returning a flat list of the original `(Token, Range<usize>)` pairs that
/// came directly from the Logos lexer.
///
/// # Example
///
/// ```rust,ignore
/// let line_tokens: Vec<LineToken> = /* ... parsed tokens ... */;
/// let flat_tokens = unroll(&line_tokens);
/// // flat_tokens now contains all original Logos tokens
/// ```
pub fn unroll<T: SourceTokenProvider>(tokens: &[T]) -> Vec<(Token, Range<usize>)> {
    tokens
        .iter()
        .flat_map(|t| t.source_tokens().iter().cloned())
        .collect()
}

/// Compute the bounding box (minimum start, maximum end) from a list of tokens.
///
/// Returns the smallest `Range<usize>` that encompasses all token ranges.
///
/// # Panics
///
/// Panics if the token list is empty. Callers should ensure tokens are non-empty.
///
/// # Example
///
/// ```rust,ignore
/// let tokens = vec![
///     (Token::Text("hello".into()), 0..5),
///     (Token::Whitespace, 5..6),
///     (Token::Text("world".into()), 6..11),
/// ];
/// let bbox = compute_bounding_box(&tokens);
/// assert_eq!(bbox, 0..11);
/// ```
pub fn compute_bounding_box(tokens: &[(Token, Range<usize>)]) -> Range<usize> {
    assert!(
        !tokens.is_empty(),
        "Cannot compute bounding box from empty token list"
    );

    let min_start = tokens
        .iter()
        .map(|(_, range)| range.start)
        .min()
        .unwrap_or(0);
    let max_end = tokens.iter().map(|(_, range)| range.end).max().unwrap_or(0);

    min_start..max_end
}

/// Convert a byte range to a human-readable Location (line:column).
///
/// This performs the one-time conversion from machine representation (`Range<usize>`)
/// to human representation (`Location` with line and column numbers).
///
/// # Arguments
///
/// * `range` - The byte offset range from the source string
/// * `source` - The original source string (needed to count newlines)
///
/// # Example
///
/// ```rust,ignore
/// let location = range_to_location(0..5, "hello world");
/// assert_eq!(location.start.line, 0);
/// assert_eq!(location.start.column, 0);
/// assert_eq!(location.end.line, 0);
/// assert_eq!(location.end.column, 5);
/// ```
pub fn range_to_location(range: Range<usize>, source: &str) -> Location {
    let source_location = SourceLocation::new(source);
    source_location.range_to_location(&range)
}

/// Extract text from the source string at the given range.
///
/// # Arguments
///
/// * `range` - The byte offset range to extract
/// * `source` - The original source string
///
/// # Example
///
/// ```rust,ignore
/// let text = extract_text(0..5, "hello world");
/// assert_eq!(text, "hello");
/// ```
pub fn extract_text(range: Range<usize>, source: &str) -> String {
    source[range].to_string()
}

/// High-level convenience: convert tokens directly to a Location.
///
/// This combines `compute_bounding_box` and `range_to_location` for convenience.
///
/// # Panics
///
/// Panics if tokens is empty.
pub fn tokens_to_location(tokens: &[(Token, Range<usize>)], source: &str) -> Location {
    let range = compute_bounding_box(tokens);
    range_to_location(range, source)
}

/// High-level convenience: extract text directly from tokens.
///
/// This combines `compute_bounding_box` and `extract_text` for convenience.
///
/// # Panics
///
/// Panics if tokens is empty.
pub fn tokens_to_text(tokens: &[(Token, Range<usize>)], source: &str) -> String {
    let range = compute_bounding_box(tokens);
    extract_text(range, source)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::txxt::ast::location::Position;

    // Mock token provider for testing
    struct MockToken {
        tokens: Vec<(Token, Range<usize>)>,
    }

    impl SourceTokenProvider for MockToken {
        fn source_tokens(&self) -> &[(Token, Range<usize>)] {
            &self.tokens
        }
    }

    #[test]
    fn test_compute_bounding_box_single_token() {
        let tokens = vec![(Token::Text("hello".to_string()), 0..5)];
        let bbox = compute_bounding_box(&tokens);
        assert_eq!(bbox, 0..5);
    }

    #[test]
    fn test_compute_bounding_box_multiple_contiguous() {
        let tokens = vec![
            (Token::Text("hello".to_string()), 0..5),
            (Token::Whitespace, 5..6),
            (Token::Text("world".to_string()), 6..11),
        ];
        let bbox = compute_bounding_box(&tokens);
        assert_eq!(bbox, 0..11);
    }

    #[test]
    fn test_compute_bounding_box_non_contiguous() {
        // In case tokens have gaps (shouldn't happen normally, but test it)
        let tokens = vec![
            (Token::Text("hello".to_string()), 0..5),
            (Token::Text("world".to_string()), 10..15),
        ];
        let bbox = compute_bounding_box(&tokens);
        assert_eq!(bbox, 0..15);
    }

    #[test]
    #[should_panic(expected = "Cannot compute bounding box from empty token list")]
    fn test_compute_bounding_box_empty_panics() {
        let tokens: Vec<(Token, Range<usize>)> = vec![];
        compute_bounding_box(&tokens);
    }

    #[test]
    fn test_extract_text_simple() {
        let source = "hello world";
        assert_eq!(extract_text(0..5, source), "hello");
        assert_eq!(extract_text(6..11, source), "world");
    }

    #[test]
    fn test_extract_text_multiline() {
        let source = "line one\nline two\nline three";
        assert_eq!(extract_text(0..8, source), "line one");
        assert_eq!(extract_text(9..17, source), "line two");
    }

    #[test]
    fn test_extract_text_unicode() {
        let source = "hello 世界";
        // "世界" is 6 bytes (3 bytes per character)
        let text = extract_text(6..12, source);
        assert_eq!(text, "世界");
    }

    #[test]
    fn test_range_to_location_single_line() {
        let source = "hello world";
        let location = range_to_location(0..5, source);
        assert_eq!(location.start, Position::new(0, 0));
        assert_eq!(location.end, Position::new(0, 5));
    }

    #[test]
    fn test_range_to_location_multiline() {
        let source = "line one\nline two\nline three";
        let location = range_to_location(0..17, source);
        // Should span from (0,0) to end of "line two"
        assert_eq!(location.start, Position::new(0, 0));
        assert_eq!(location.end, Position::new(1, 8));
    }

    #[test]
    fn test_unroll_single_token() {
        let mock = MockToken {
            tokens: vec![(Token::Text("hello".to_string()), 0..5)],
        };
        let unrolled = unroll(&[mock]);
        assert_eq!(unrolled.len(), 1);
        assert_eq!(unrolled[0].1, 0..5);
    }

    #[test]
    fn test_unroll_multiple_tokens() {
        let mock1 = MockToken {
            tokens: vec![(Token::Text("hello".to_string()), 0..5)],
        };
        let mock2 = MockToken {
            tokens: vec![
                (Token::Whitespace, 5..6),
                (Token::Text("world".to_string()), 6..11),
            ],
        };
        let unrolled = unroll(&[mock1, mock2]);
        assert_eq!(unrolled.len(), 3);
        assert_eq!(unrolled[0].1, 0..5);
        assert_eq!(unrolled[1].1, 5..6);
        assert_eq!(unrolled[2].1, 6..11);
    }

    #[test]
    fn test_tokens_to_location_convenience() {
        let source = "hello world";
        let tokens = vec![
            (Token::Text("hello".to_string()), 0..5),
            (Token::Whitespace, 5..6),
            (Token::Text("world".to_string()), 6..11),
        ];
        let location = tokens_to_location(&tokens, source);
        assert_eq!(location.start, Position::new(0, 0));
        assert_eq!(location.end, Position::new(0, 11));
    }

    #[test]
    fn test_tokens_to_text_convenience() {
        let source = "hello world";
        let tokens = vec![
            (Token::Text("hello".to_string()), 0..5),
            (Token::Whitespace, 5..6),
        ];
        let text = tokens_to_text(&tokens, source);
        assert_eq!(text, "hello ");
    }
}
