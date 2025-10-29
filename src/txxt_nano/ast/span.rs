//! Position and span tracking for source code locations
//!
//! This module defines the data structures for representing positions
//! and spans in source code.

use std::fmt;

/// Represents a position in source code (line and column)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// Represents a span in source code (start and end positions)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// Check if a position is contained within this span
    pub fn contains(&self, pos: Position) -> bool {
        (self.start.line < pos.line
            || (self.start.line == pos.line && self.start.column <= pos.column))
            && (self.end.line > pos.line
                || (self.end.line == pos.line && self.end.column >= pos.column))
    }

    /// Check if another span overlaps with this span
    pub fn overlaps(&self, other: Span) -> bool {
        self.contains(other.start)
            || self.contains(other.end)
            || other.contains(self.start)
            || other.contains(self.end)
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_creation() {
        let pos = Position::new(5, 10);
        assert_eq!(pos.line, 5);
        assert_eq!(pos.column, 10);
    }

    #[test]
    fn test_position_comparison() {
        let pos1 = Position::new(1, 5);
        let pos2 = Position::new(1, 5);
        let pos3 = Position::new(2, 3);

        assert_eq!(pos1, pos2);
        assert_ne!(pos1, pos3);
        assert!(pos1 < pos3);
    }

    #[test]
    fn test_span_creation() {
        let start = Position::new(0, 0);
        let end = Position::new(2, 5);
        let span = Span::new(start, end);

        assert_eq!(span.start, start);
        assert_eq!(span.end, end);
    }

    #[test]
    fn test_span_contains_single_line() {
        let span = Span::new(Position::new(0, 0), Position::new(0, 10));

        assert!(span.contains(Position::new(0, 0)));
        assert!(span.contains(Position::new(0, 5)));
        assert!(span.contains(Position::new(0, 10)));

        assert!(!span.contains(Position::new(0, 11)));
        assert!(!span.contains(Position::new(1, 0)));
    }

    #[test]
    fn test_span_contains_multiline() {
        let span = Span::new(Position::new(1, 5), Position::new(2, 10));

        // Before span
        assert!(!span.contains(Position::new(1, 4)));
        assert!(!span.contains(Position::new(0, 5)));

        // In span
        assert!(span.contains(Position::new(1, 5)));
        assert!(span.contains(Position::new(1, 10)));
        assert!(span.contains(Position::new(2, 0)));
        assert!(span.contains(Position::new(2, 10)));

        // After span
        assert!(!span.contains(Position::new(2, 11)));
        assert!(!span.contains(Position::new(3, 0)));
    }

    #[test]
    fn test_span_overlaps() {
        let span1 = Span::new(Position::new(0, 0), Position::new(1, 5));
        let span2 = Span::new(Position::new(1, 0), Position::new(2, 5));
        let span3 = Span::new(Position::new(3, 0), Position::new(4, 5));

        assert!(span1.overlaps(span2));
        assert!(span2.overlaps(span1));
        assert!(!span1.overlaps(span3));
        assert!(!span3.overlaps(span1));
    }

    #[test]
    fn test_position_display() {
        let pos = Position::new(5, 10);
        assert_eq!(format!("{}", pos), "5:10");
    }

    #[test]
    fn test_span_display() {
        let span = Span::new(Position::new(1, 0), Position::new(2, 5));
        assert_eq!(format!("{}", span), "1:0..2:5");
    }
}
