//! AST traits - Common interfaces for uniform node access
//!
//! This module defines the common traits that provide uniform access
//! to AST node information across all node types.

use super::elements::ContentItem;
use super::span::{Location, Position};
use super::text_content::TextContent;

/// Common interface for all AST nodes
pub trait AstNode {
    fn node_type(&self) -> &'static str;
    fn display_label(&self) -> String;
    fn location(&self) -> Option<Location>;
    fn get_location(&self) -> Option<Position> {
        self.location().map(|s| s.start)
    }
}

/// Trait for container nodes that have a label and children
pub trait Container: AstNode {
    fn label(&self) -> &str;
    fn children(&self) -> &[ContentItem];
    fn children_mut(&mut self) -> &mut Vec<ContentItem>;
}

/// Trait for leaf nodes that contain text
pub trait TextNode: AstNode {
    fn text(&self) -> String;
    fn lines(&self) -> &[TextContent];
}
