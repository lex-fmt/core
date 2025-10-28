//! AST definitions and utilities for the txxt format
//!
//! This module provides the core Abstract Syntax Tree (AST) definitions,
//! along with utilities for working with AST nodes, tracking source positions,
//! and performing position-based lookups.
//!
//! ## Modules
//!
//! - `span` - Position and Span types for source code locations
//! - `node` - AST node type definitions and trait implementations
//! - `position` - Source location utilities for converting byte offsets
//! - `lookup` - Position-based AST node lookup functionality
//! - `error` - Error types for AST operations

pub mod error;
pub mod lookup;
pub mod node;
pub mod position;
pub mod span;
pub mod text_content;

// Re-export commonly used types at module root
pub use error::PositionLookupError;
pub use lookup::format_at_position;
pub use node::{
    Annotation, AstNode, Container, ContentItem, Definition, Document, ForeignBlock, Label, List,
    ListItem, Paragraph, Parameter, Session, TextNode,
};
pub use position::SourceLocation;
pub use span::{Position, Span};
pub use text_content::TextContent;
