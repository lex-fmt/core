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
