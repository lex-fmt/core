//! Location utilities for AST node building
//!
//! Provides shared location handling utilities used by the parser/AST builder.
//! These utilities handle the conversion from byte ranges to line/column positions and
//! compute bounding boxes for container nodes (sessions, lists, definitions, etc.).
//!
//! ## Relationship to `ast/range.rs`
//!
//! This module builds on top of `ast/range.rs`:
//! - `ast/range.rs` provides the foundation types (`Position`, `Range`, `SourceLocation`)
//! - This module provides high-level helpers for AST construction
//!
//! The separation maintains clean architecture:
//! - `ast/range.rs` = Pure types with no AST dependencies
//! - `building/location.rs` = Builder utilities that work with AST nodes

use std::ops::Range as ByteRange;

use crate::lex::ast::range::SourceLocation;
use crate::lex::ast::traits::AstNode;
use crate::lex::ast::{ContentItem, Range};

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
