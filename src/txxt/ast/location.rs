//! Position and location tracking for source code locations
//!
//! This module defines the data structures for representing positions and locations in source code,
//! as well as utilities for converting byte offsets to line/column positions.
//!
//! ## How Location Tracking Works in txxt
//!
//! Location tracking flows through the entire compilation pipeline from raw source code to AST nodes.
//! Here's the high-level architecture:
//!
//! ### 1. Tokenization (Lexer)
//!
//! The lexer (`src/txxt/lexer.rs` and `src/txxt/lexer/indentation.rs`) produces tokens paired with
//! **byte-offset ranges** into the source. The logos crate provides automatic span tracking:
//!
//! ```text
//! Source: "Hello\nWorld"
//!         012345678901
//!                  ↓
//! Lexer produces: (Token::Text("Hello"), 0..5)
//!                 (Token::Newline, 5..6)
//!                 (Token::Text("World"), 6..11)
//! ```
//!
//! The lexer pipeline applies several transformations while preserving byte ranges:
//! - **Whitespace processing**: Removes whitespace tokens, preserves ranges
//! - **Indentation transformation**: Converts Indent tokens to IndentLevel tokens with meaningful ranges
//! - **Blank line transformation**: Converts multiple Newlines to BlankLine tokens with aggregated ranges
//!
//! See `src/txxt/lexer/transformations/` for implementation details.
//!
//! ### 2. Byte-to-Line Conversion (SourceLocation)
//!
//! Before building AST nodes, byte ranges are converted to line:column positions using [`SourceLocation`].
//! This is a one-time setup (not repeated per-token):
//!
//! ```text
//! SourceLocation pre-computes line start byte offsets:
//!   line_starts = [0, 6]  (first line starts at byte 0, second at byte 6)
//!
//! byte_to_position(8) uses binary search:
//!   - Find line: binary_search(&8) finds index 1 (line 1)
//!   - Column: 8 - 6 = 2
//!   - Returns: Position { line: 1, column: 2 }
//! ```
//!
//! This is O(log n) per conversion, efficient for large documents.
//!
//! ### 3. Parser (AST Construction)
//!
//! The parser (`src/txxt/parser/`) consumes tokens with byte ranges and builds AST nodes with
//! [`Location`] objects. Location assignment happens bottom-up:
//!
//! #### Bottom-Up Construction
//! Each parser combinator:
//! 1. Parses child elements (which have locations)
//! 2. Converts byte ranges to `Location` objects via `byte_range_to_location()`
//! 3. Aggregates child locations: `compute_location_from_locations(&[child1, child2, ...])`
//! 4. Creates parent node with aggregated location
//!
//! #### Example: Session Parser (`src/txxt/parser/elements/sessions.rs`)
//!
//! ```text
//! Input tokens:  "Session Title" [tokens] DedentLevel
//!                              ↓
//! 1. Parse title → TextContent with Location(0:0..0:13)
//! 2. Parse content (recursive) → [Paragraph, List, ...] each with Location
//! 3. Compute title_location from title bytes
//! 4. Aggregate: [title_location] + [para_location, list_location]
//! 5. Create Session with bounding box location
//! ```
//!
//! #### Location Aggregation
//! The `compute_location_from_locations()` function (in `src/txxt/parser/combinators.rs`) creates
//! a bounding box:
//!
//! ```text
//! Input:  [Location(0:0..1:5), Location(2:0..2:10), Location(1:5..1:15)]
//! Output: Location(0:0..2:10)  ← Spans from earliest start to latest end
//! ```
//!
//! ### 4. Complete Document Structure
//!
//! The final document has location information at every level:
//!
//! ```text
//! Document
//!   ├─ root: Session (Location)
//!   │  ├─ content[0]: Session "Intro" (Location)
//!   │  │  ├─ Paragraph (Location)
//!   │  │  │  └─ TextLine (Location)
//!   │  │  └─ Paragraph (Location)
//!   │  │     └─ TextLine (Location)
//!   │  └─ content[1]: List (Location)
//!   │     ├─ ListItem (Location)
//!   │     └─ ListItem (Location)
//!   └─ metadata: [Annotation, ...] (each with Location)
//! ```
//!
//! Every element knows its exact position in the source: start line:column to end line:column.
//!
//! ## Key Design Characteristics
//!
//! - **Mandatory locations**: All AST nodes (except Document) have required `location: Location` fields
//! - **No null locations**: Default position is (0, 0) to (0, 0), never None
//! - **Byte ranges preserved**: Parser uses byte ranges from lexer tokens, not reconstructed positions
//! - **Unicode-aware**: Handles multi-byte UTF-8 characters correctly via `char_indices()`
//! - **Efficient conversion**: O(log n) binary search for byte-to-position conversion
//!
//! ## Testing
//!
//! Location accuracy is verified at multiple levels:
//! - **Unit tests**: `SourceLocation::byte_to_position()` with ASCII, Unicode, single/multi-line
//! - **Integration tests**: Document → Session → Paragraph → TextLine hierarchies
//! - **Position queries**: `find_nodes_at_position()` returns correctly ordered (deepest-first) nodes
//! - **Edge cases**: Empty documents, single-line, deeply nested, overlapping regions
//!
//! See the module tests below (lines 127+) for comprehensive coverage.

use std::fmt;
use std::ops::Range;

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

impl Default for Position {
    fn default() -> Self {
        Self::new(0, 0)
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

impl Default for Location {
    fn default() -> Self {
        Self::new(Position::default(), Position::default())
    }
}

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

    /// Convert a byte range to a location
    pub fn range_to_location(&self, range: &Range<usize>) -> Location {
        Location::new(
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
    fn test_range_to_location_single_line() {
        let loc = SourceLocation::new("Hello World");
        let location = loc.range_to_location(&(0..5));

        assert_eq!(location.start, Position::new(0, 0));
        assert_eq!(location.end, Position::new(0, 5));
    }

    #[test]
    fn test_range_to_location_multiline() {
        let loc = SourceLocation::new("Hello\nWorld\nTest");
        let location = loc.range_to_location(&(6..12));

        assert_eq!(location.start, Position::new(1, 0));
        assert_eq!(location.end, Position::new(2, 0));
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
}

#[cfg(test)]
mod ast_integration_tests {
    use crate::txxt::ast::{
        elements::Session,
        location::{Location, Position},
        traits::{AstNode, Container},
    };

    #[test]
    fn test_start_position() {
        let location = Location::new(Position::new(1, 0), Position::new(1, 10));
        let session = Session::with_title("Title".to_string()).at(location);
        assert_eq!(session.start_position(), Position::new(1, 0));
    }

    #[test]
    fn test_find_nodes_at_position() {
        use crate::txxt::ast::elements::ContentItem;
        use crate::txxt::ast::elements::Document;
        use crate::txxt::ast::find_nodes_at_position;

        let location1 = Location::new(Position::new(1, 0), Position::new(1, 10));
        let location2 = Location::new(Position::new(2, 0), Position::new(2, 10));
        let session1 = Session::with_title("Title1".to_string()).at(location1);
        let session2 = Session::with_title("Title2".to_string()).at(location2);
        let document = Document::with_content(vec![
            ContentItem::Session(session1),
            ContentItem::Session(session2),
        ]);
        let nodes = find_nodes_at_position(&document, Position::new(1, 5));
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].node_type(), "Session");
        assert_eq!(nodes[0].display_label(), "Title1");
    }

    #[test]
    fn test_find_nested_nodes_at_position() {
        use crate::txxt::ast::elements::{ContentItem, Document, Paragraph};
        use crate::txxt::ast::find_nodes_at_position;

        let para_location = Location::new(Position::new(2, 0), Position::new(2, 10));
        let paragraph = Paragraph::from_line("Nested".to_string()).at(para_location);
        let session_location = Location::new(Position::new(1, 0), Position::new(3, 0));
        let mut session = Session::with_title("Title".to_string()).at(session_location);
        session
            .children_mut()
            .push(ContentItem::Paragraph(paragraph));
        let document = Document::with_content(vec![ContentItem::Session(session)]);
        let nodes = find_nodes_at_position(&document, Position::new(2, 5));
        // Now we get only the deepest element: TextLine
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].node_type(), "TextLine");
    }
}
