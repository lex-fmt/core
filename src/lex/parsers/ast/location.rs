//! Location utilities for AST node building
//!
//! Provides shared location handling utilities used by both reference and linebased parsers.
//! These utilities handle the conversion from byte ranges to line/column positions and
//! compute bounding boxes for container nodes (sessions, lists, definitions, etc.).

use std::ops::Range as ByteRange;

use crate::lex::ast::range::SourceLocation;
use crate::lex::ast::traits::AstNode;
use crate::lex::ast::{ContentItem, Range};
use crate::lex::lexers::tokens_core::Token;

// ============================================================================
// BYTE RANGE TO AST RANGE CONVERSION
// ============================================================================

/// Convert a byte range to an AST Range (line:column positions)
///
/// This is the canonical implementation used throughout the AST building pipeline.
/// Converts byte offsets from token ranges to line/column coordinates
/// using the SourceLocation utility (O(log n) binary search).
///
/// # Arguments
///
/// * `range` - Byte offset range from the source string
/// * `source` - Original source string (needed to count newlines)
///
/// # Returns
///
/// An AST Range with line/column positions
pub(super) fn byte_range_to_ast_range(range: ByteRange<usize>, source: &str) -> Range {
    let source_location = SourceLocation::new(source);
    source_location.byte_range_to_ast_range(&range)
}

/// Convert a byte range to a Location (line:column positions) - Legacy API
///
/// This is a wrapper around byte_range_to_ast_range with different parameter order
/// for backwards compatibility. Prefer using byte_range_to_ast_range directly.
#[deprecated(note = "Use byte_range_to_ast_range instead")]
pub fn byte_range_to_location(source: &str, range: &ByteRange<usize>) -> Range {
    byte_range_to_ast_range(range.clone(), source)
}

/// High-level convenience: convert tokens directly to an AST Range
///
/// This combines computing the bounding box and converting to AST Range.
///
/// # Panics
///
/// Panics if tokens is empty.
#[allow(dead_code)]
pub(super) fn tokens_to_ast_range(tokens: &[(Token, ByteRange<usize>)], source: &str) -> Range {
    let range = super::token::processing::compute_bounding_box(tokens);
    byte_range_to_ast_range(range, source)
}

// ============================================================================
// AST RANGE AGGREGATION
// ============================================================================

/// Compute location bounds from multiple locations
///
/// Creates a bounding box that encompasses all provided locations by taking:
/// - The minimum start line/column across all locations
/// - The maximum end line/column across all locations
///
/// This matches both parsers' approach for location aggregation.
///
/// Note: This function is public for use by parser implementations.
pub fn compute_location_from_locations(locations: &[Range]) -> Range {
    use crate::lex::ast::range::Position;
    let start_line = locations.iter().map(|sp| sp.start.line).min().unwrap_or(0);
    let start_col = locations
        .iter()
        .map(|sp| sp.start.column)
        .min()
        .unwrap_or(0);
    let end_line = locations.iter().map(|sp| sp.end.line).max().unwrap_or(0);
    let end_col = locations.iter().map(|sp| sp.end.column).max().unwrap_or(0);
    Range::new(
        0..0, // This is an aggregated range, the original spans may not be contiguous
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
pub(super) fn aggregate_locations(primary: Range, children: &[ContentItem]) -> Range {
    let mut sources = vec![primary];
    sources.extend(children.iter().map(|item| item.range().clone()));
    compute_location_from_locations(&sources)
}

/// Compute location bounds from byte ranges
///
/// Finds the minimum start and maximum end across all byte ranges.
/// Used when combining multiple token ranges into a single location.
#[allow(dead_code)]
pub(super) fn compute_byte_range_bounds(ranges: &[ByteRange<usize>]) -> ByteRange<usize> {
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
pub fn default_location() -> Range {
    Range {
        span: 0..0,
        start: crate::lex::ast::range::Position { line: 0, column: 0 },
        end: crate::lex::ast::range::Position { line: 0, column: 0 },
    }
}
