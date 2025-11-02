//! Location utilities for AST node building
//!
//! Provides shared location handling utilities used by both reference and linebased parsers.
//! These utilities handle the conversion from byte ranges to line/column positions and
//! compute bounding boxes for container nodes (sessions, lists, definitions, etc.).

use std::ops::Range;

use crate::txxt::ast::location::SourceLocation;
use crate::txxt::ast::traits::AstNode;
use crate::txxt::ast::{ContentItem, Location};

/// Convert a byte range to a Location (line:column positions)
///
/// This is the canonical implementation used throughout both parsers.
/// Converts byte offsets from token ranges to line/column coordinates
/// using the SourceLocation utility (O(log n) binary search).
pub fn byte_range_to_location(source: &str, range: &Range<usize>) -> Location {
    debug_assert!(
        range.start <= range.end,
        "Invalid byte range: {}..{} (start > end)",
        range.start,
        range.end
    );
    let source_loc = SourceLocation::new(source);
    source_loc.range_to_location(range)
}

/// Compute location bounds from multiple locations
///
/// Creates a bounding box that encompasses all provided locations by taking:
/// - The minimum start line/column across all locations
/// - The maximum end line/column across all locations
///
/// This matches both parsers' approach for location aggregation.
pub fn compute_location_from_locations(locations: &[Location]) -> Location {
    use crate::txxt::ast::location::Position;
    let start_line = locations.iter().map(|sp| sp.start.line).min().unwrap_or(0);
    let start_col = locations
        .iter()
        .map(|sp| sp.start.column)
        .min()
        .unwrap_or(0);
    let end_line = locations.iter().map(|sp| sp.end.line).max().unwrap_or(0);
    let end_col = locations.iter().map(|sp| sp.end.column).max().unwrap_or(0);
    Location::new(
        Position::new(start_line, start_col),
        Position::new(end_line, end_col),
    )
}

/// Aggregate location from a primary location and child content items
///
/// Creates a bounding box that encompasses the primary location and all child content.
/// This is commonly used when building container nodes (sessions, lists, definitions)
/// that need to include the location of their title/header and all child items.
///
/// # Example
/// ```ignore
/// let location = aggregate_locations(title_location, &session_content);
/// ```
pub fn aggregate_locations(primary: Location, children: &[ContentItem]) -> Location {
    let mut sources = vec![primary];
    sources.extend(children.iter().map(|item| item.location()));
    compute_location_from_locations(&sources)
}

/// Compute location bounds from byte ranges
///
/// Finds the minimum start and maximum end across all byte ranges.
/// Used when combining multiple token ranges into a single location.
pub fn compute_byte_range_bounds(ranges: &[Range<usize>]) -> Range<usize> {
    if ranges.is_empty() {
        0..0
    } else {
        let start = ranges.iter().map(|r| r.start).min().unwrap_or(0);
        let end = ranges.iter().map(|r| r.end).max().unwrap_or(0);
        start..end
    }
}

/// Create a default location (0,0)..(0,0)
///
/// Used when source span information is not available.
pub fn default_location() -> Location {
    Location {
        start: crate::txxt::ast::location::Position { line: 0, column: 0 },
        end: crate::txxt::ast::location::Position { line: 0, column: 0 },
    }
}
