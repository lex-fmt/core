//! AST definitions and utilities for the txxt format
//!
//! This module provides the core Abstract Syntax Tree (AST) definitions,
//! along with utilities for working with AST nodes, tracking source positions,
//! and performing position-based lookups.
//!
//! ## Modules
//!
//! - `span` - Position and Span types for source code locations
//! - `elements` - AST node type definitions organized by element type
//! - `traits` - Common traits for AST nodes
//! - `position` - Source location utilities for converting byte offsets
//! - `lookup` - Position-based AST node lookup functionality
//! - `error` - Error types for AST operations

pub mod elements;
pub mod error;
pub mod lookup;
pub mod position;
pub mod span;
pub mod text_content;
pub mod traits;

// Re-export commonly used types at module root
pub use elements::{
    Annotation, ContentItem, Definition, Document, ForeignBlock, Label, List, ListItem, Paragraph,
    Parameter, Session,
};
pub use error::PositionLookupError;
pub use lookup::{find_nodes_at_position, format_at_position};
pub use position::SourceLocation;
pub use span::{Position, Span};
pub use text_content::TextContent;
pub use traits::{AstNode, Container, TextNode};
#[cfg(test)]
mod location_test;
