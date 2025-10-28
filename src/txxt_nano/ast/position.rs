//! Source location utilities for converting byte offsets to line/column positions
//!
//! This module provides utilities to convert byte offsets within source code
//! to human-readable line and column positions, useful for error reporting
//! and position tracking in AST nodes.

use super::span::{Position, Span};
use std::ops::Range;

/// Provides fast conversion from byte offsets to line/column positions
pub struct SourceLocation {
    /// Byte offsets where each line starts
    line_starts: Vec<usize>,
}

impl SourceLocation {
    /// Create a new SourceLocation from source code
    pub fn new(source: &str) -> Self {
        let mut line_starts = vec![0];

        for (byte_pos, ch) in source.char_indices() {
            if ch == '\n' {
                line_starts.push(byte_pos + 1);
            }
        }

        Self { line_starts }
    }

    /// Convert a byte offset to a line/column position
    pub fn byte_to_position(&self, byte_offset: usize) -> Position {
        let line = self
            .line_starts
            .binary_search(&byte_offset)
            .unwrap_or_else(|i| i - 1);

        let column = byte_offset - self.line_starts[line];

        Position::new(line, column)
    }

    /// Convert a byte range to a span
    pub fn range_to_span(&self, range: &Range<usize>) -> Span {
        Span::new(
            self.byte_to_position(range.start),
            self.byte_to_position(range.end),
        )
    }

    /// Get the total number of lines in the source
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    /// Get the byte offset for the start of a line
    pub fn line_start(&self, line: usize) -> Option<usize> {
        self.line_starts.get(line).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_to_position_single_line() {
        let loc = SourceLocation::new("Hello");
        assert_eq!(loc.byte_to_position(0), Position::new(0, 0));
        assert_eq!(loc.byte_to_position(1), Position::new(0, 1));
        assert_eq!(loc.byte_to_position(4), Position::new(0, 4));
    }

    #[test]
    fn test_byte_to_position_multiline() {
        let loc = SourceLocation::new("Hello\nworld\ntest");

        // First line
        assert_eq!(loc.byte_to_position(0), Position::new(0, 0));
        assert_eq!(loc.byte_to_position(5), Position::new(0, 5));

        // Second line
        assert_eq!(loc.byte_to_position(6), Position::new(1, 0));
        assert_eq!(loc.byte_to_position(10), Position::new(1, 4));

        // Third line
        assert_eq!(loc.byte_to_position(12), Position::new(2, 0));
        assert_eq!(loc.byte_to_position(15), Position::new(2, 3));
    }

    #[test]
    fn test_byte_to_position_with_unicode() {
        let loc = SourceLocation::new("Hello\nwörld");
        // Unicode characters take multiple bytes
        assert_eq!(loc.byte_to_position(6), Position::new(1, 0));
        assert_eq!(loc.byte_to_position(7), Position::new(1, 1));
    }

    #[test]
    fn test_range_to_span_single_line() {
        let loc = SourceLocation::new("Hello World");
        let span = loc.range_to_span(&(0..5));

        assert_eq!(span.start, Position::new(0, 0));
        assert_eq!(span.end, Position::new(0, 5));
    }

    #[test]
    fn test_range_to_span_multiline() {
        let loc = SourceLocation::new("Hello\nWorld\nTest");
        let span = loc.range_to_span(&(6..12));

        assert_eq!(span.start, Position::new(1, 0));
        assert_eq!(span.end, Position::new(2, 0));
    }

    #[test]
    fn test_line_count() {
        assert_eq!(SourceLocation::new("single").line_count(), 1);
        assert_eq!(SourceLocation::new("line1\nline2").line_count(), 2);
        assert_eq!(SourceLocation::new("line1\nline2\nline3").line_count(), 3);
    }

    #[test]
    fn test_line_start() {
        let loc = SourceLocation::new("Hello\nWorld\nTest");

        assert_eq!(loc.line_start(0), Some(0));
        assert_eq!(loc.line_start(1), Some(6));
        assert_eq!(loc.line_start(2), Some(12));
        assert_eq!(loc.line_start(3), None);
    }

    #[test]
    fn test_position_contains() {
        let span = Span::new(Position::new(1, 5), Position::new(2, 10));

        // Start position
        assert!(span.contains(Position::new(1, 5)));

        // End position
        assert!(span.contains(Position::new(2, 10)));

        // Inside span
        assert!(span.contains(Position::new(1, 8)));
        assert!(span.contains(Position::new(2, 0)));

        // Outside span
        assert!(!span.contains(Position::new(1, 4)));
        assert!(!span.contains(Position::new(2, 11)));
        assert!(!span.contains(Position::new(0, 0)));
    }

    #[test]
    fn test_span_overlaps() {
        let span1 = Span::new(Position::new(0, 0), Position::new(1, 5));
        let span2 = Span::new(Position::new(1, 0), Position::new(2, 5));
        let span3 = Span::new(Position::new(3, 0), Position::new(4, 5));

        // Overlapping spans
        assert!(span1.overlaps(span2));
        assert!(span2.overlaps(span1));

        // Non-overlapping spans
        assert!(!span1.overlaps(span3));
        assert!(!span3.overlaps(span1));
    }
}
