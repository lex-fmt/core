//! Position and location tracking for source code locations
//!
//! This module defines the data structures for representing positions
//! and locations in source code.

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

/// Represents a location in source code (start and end positions)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Location {
    pub start: Position,
    pub end: Position,
}

impl Location {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// Check if a position is contained within this location
    pub fn contains(&self, pos: Position) -> bool {
        (self.start.line < pos.line
            || (self.start.line == pos.line && self.start.column <= pos.column))
            && (self.end.line > pos.line
                || (self.end.line == pos.line && self.end.column >= pos.column))
    }

    /// Check if another location overlaps with this location
    pub fn overlaps(&self, other: Location) -> bool {
        self.contains(other.start)
            || self.contains(other.end)
            || other.contains(self.start)
            || other.contains(self.end)
    }
}

impl fmt::Display for Location {
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
    fn test_location_creation() {
        let start = Position::new(0, 0);
        let end = Position::new(2, 5);
        let location = Location::new(start, end);

        assert_eq!(location.start, start);
        assert_eq!(location.end, end);
    }

    #[test]
    fn test_location_contains_single_line() {
        let location = Location::new(Position::new(0, 0), Position::new(0, 10));

        assert!(location.contains(Position::new(0, 0)));
        assert!(location.contains(Position::new(0, 5)));
        assert!(location.contains(Position::new(0, 10)));

        assert!(!location.contains(Position::new(0, 11)));
        assert!(!location.contains(Position::new(1, 0)));
    }

    #[test]
    fn test_location_contains_multiline() {
        let location = Location::new(Position::new(1, 5), Position::new(2, 10));

        // Before location
        assert!(!location.contains(Position::new(1, 4)));
        assert!(!location.contains(Position::new(0, 5)));

        // In location
        assert!(location.contains(Position::new(1, 5)));
        assert!(location.contains(Position::new(1, 10)));
        assert!(location.contains(Position::new(2, 0)));
        assert!(location.contains(Position::new(2, 10)));

        // After location
        assert!(!location.contains(Position::new(2, 11)));
        assert!(!location.contains(Position::new(3, 0)));
    }

    #[test]
    fn test_location_overlaps() {
        let location1 = Location::new(Position::new(0, 0), Position::new(1, 5));
        let location2 = Location::new(Position::new(1, 0), Position::new(2, 5));
        let location3 = Location::new(Position::new(3, 0), Position::new(4, 5));

        assert!(location1.overlaps(location2));
        assert!(location2.overlaps(location1));
        assert!(!location1.overlaps(location3));
        assert!(!location3.overlaps(location1));
    }

    #[test]
    fn test_position_display() {
        let pos = Position::new(5, 10);
        assert_eq!(format!("{}", pos), "5:10");
    }

    #[test]
    fn test_location_display() {
        let location = Location::new(Position::new(1, 0), Position::new(2, 5));
        assert_eq!(format!("{}", location), "1:0..2:5");
    }
}
