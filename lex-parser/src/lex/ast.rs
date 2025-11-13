//! AST definitions and utilities for the lex format
//!
//! This module provides the core Abstract Syntax Tree (AST) definitions,
//! along with utilities for working with AST nodes, tracking source positions,
//! and performing position-based lookups.
//!
//! ## How Location Tracking Works in lex
//!
//! Location tracking flows through the entire compilation pipeline from raw source code to AST nodes.
//!
//! ### 1. Tokenization (Lexer)
//!
//! The lexer produces tokens paired with **byte-offset ranges** into the source:
//!
//! ```text
//! Source: "Hello\nWorld"
//!         012345678901
//!                  ↓
//! Lexer: (Token::Text("Hello"), 0..5)
//!        (Token::Newline, 5..6)
//!        (Token::Text("World"), 6..11)
//! ```
//!
//! The lexer pipeline applies transformations while preserving byte ranges:
//! - Whitespace processing - removes tokens, preserves ranges
//! - Indentation transformation - converts to semantic Indent/Dedent tokens
//! - Blank line transformation - aggregates multiple Newlines
//!
//! ### 2. Byte-to-Line Conversion
//!
//! Before building AST nodes, byte ranges are converted to line:column positions
//! using `SourceLocation` (one-time setup, O(log n) per conversion):
//!
//! ```text
//! SourceLocation pre-computes line starts: [0, 6]
//! byte_to_position(8) → binary search → Position { line: 1, column: 2 }
//! ```
//!
//! ### 3. Parser (AST Construction)
//!
//! The parser builds AST nodes with `Range` objects via bottom-up construction:
//! 1. Parse child elements (which have locations)
//! 2. Convert byte ranges to `Range` objects
//! 3. Aggregate child locations via `compute_location_from_locations()`
//! 4. Create parent node with aggregated location (bounding box)
//!
//! See `src/lex/building/location.rs` for the canonical implementations.
//!
//! ### 4. Complete Document Structure
//!
//! The final document has location information at every level - every element
//! knows its exact position in the source (start line:column to end line:column).
//!
//! ## Modules
//!
//! - `range` - Position and Range types for source code locations
//! - `elements` - AST node type definitions organized by element type
//! - `traits` - Common traits for AST nodes (AstNode, Container, TextNode, Visitor)
//! - `lookup` - Position-based AST node lookup functionality
//! - `snapshot` - Normalized intermediate representation for serialization
//! - `error` - Error types for AST operations
//!
//! ## Type-Safe Containers
//!
//! Containers such as `Session`, `Definition`, and `Annotation` now take typed
//! vectors (`SessionContent`, `ContentElement`, etc.) so invalid nesting is ruled
//! out at compile time. See `docs/architecture/type-safe-containers.md` for
//! details and compile-fail examples.

pub mod elements;
pub mod error;
pub mod range;
pub mod snapshot;
pub mod text_content;
pub mod traits;

// Re-export commonly used types at module root
pub use elements::{
    Annotation, ContentItem, Definition, Document, Label, List, ListItem, Paragraph, Parameter,
    Session, TextLine, Verbatim,
};
pub use error::PositionLookupError;
pub use range::{Position, Range, SourceLocation};
pub use snapshot::{snapshot_from_content, snapshot_from_document, snapshot_node, AstSnapshot};
pub use text_content::TextContent;
pub use traits::{AstNode, Container, TextNode, Visitor};

// Convenience functions that delegate to Document methods
// These are provided for backwards compatibility with existing code

/// Find nodes at a given position in the document
///
/// This is a convenience wrapper around `Document::find_nodes_at_position()`.
/// Returns a vector containing the deepest AST node at the given position.
#[inline]
pub fn find_nodes_at_position(document: &Document, position: Position) -> Vec<&dyn AstNode> {
    document.root.find_nodes_at_position(position)
}

/// Format information about nodes at a given position
///
/// This is a convenience wrapper around `Document::format_at_position()`.
/// Returns a formatted string describing the AST nodes at the given position.
#[inline]
pub fn format_at_position(document: &Document, position: Position) -> String {
    document.root.format_at_position(position)
}
