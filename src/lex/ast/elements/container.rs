//! Container element
//!
//! Container represents a collection of nested children elements.
//! This is used for true parent>child relationships (Sessions, Definitions, etc.)
//! and is distinct from "core items" (lines in paragraphs, items in lists).
//!
//! The Container type provides:
//! - Type safety distinguishing children from core items
//! - Uniform handling of nested content
//! - Location tracking for nested element spans

use super::super::range::Range;
use super::super::traits::{AstNode, Visitor};
use super::content_item::ContentItem;
use std::fmt;

/// Container represents nested children elements
///
/// Used for true parent>child relationships where heterogeneous elements
/// can be nested (Sessions, Definitions, Annotations, ListItems).
/// Also used for core items (lines in Paragraphs, items in Lists).
#[derive(Debug, Clone, PartialEq)]
pub struct Container {
    pub children: Vec<ContentItem>,
    pub location: Range,
}

impl Container {
    /// Create a new container with the given children
    pub fn new(children: Vec<ContentItem>) -> Self {
        Self {
            children,
            location: Range::default(),
        }
    }

    /// Create an empty container
    pub fn empty() -> Self {
        Self::new(Vec::new())
    }

    /// Set the location for this container (builder pattern)
    pub fn at(mut self, location: Range) -> Self {
        self.location = location;
        self
    }

    /// Get the number of children
    pub fn len(&self) -> usize {
        self.children.len()
    }

    /// Check if the container is empty
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    /// Add a child to the container
    pub fn push(&mut self, item: ContentItem) {
        self.children.push(item);
    }

    /// Get an iterator over the children
    pub fn iter(&self) -> std::slice::Iter<'_, ContentItem> {
        self.children.iter()
    }

    /// Get a mutable iterator over the children
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, ContentItem> {
        self.children.iter_mut()
    }
}

impl AstNode for Container {
    fn node_type(&self) -> &'static str {
        "Container"
    }

    fn display_label(&self) -> String {
        format!("{} items", self.children.len())
    }

    fn range(&self) -> &Range {
        &self.location
    }

    fn accept(&self, visitor: &mut dyn Visitor) {
        // Container itself doesn't have a visit method
        // It delegates to its children
        super::super::traits::visit_children(visitor, &self.children);
    }
}

// Implement Deref for ergonomic access to the inner Vec
impl std::ops::Deref for Container {
    type Target = Vec<ContentItem>;

    fn deref(&self) -> &Self::Target {
        &self.children
    }
}

impl std::ops::DerefMut for Container {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.children
    }
}

impl fmt::Display for Container {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Container({} items)", self.children.len())
    }
}

// Implement IntoIterator to allow for loops over Container
impl<'a> IntoIterator for &'a Container {
    type Item = &'a ContentItem;
    type IntoIter = std::slice::Iter<'a, ContentItem>;

    fn into_iter(self) -> Self::IntoIter {
        self.children.iter()
    }
}

impl<'a> IntoIterator for &'a mut Container {
    type Item = &'a mut ContentItem;
    type IntoIter = std::slice::IterMut<'a, ContentItem>;

    fn into_iter(self) -> Self::IntoIter {
        self.children.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::super::paragraph::Paragraph;
    use super::*;

    #[test]
    fn test_container_creation() {
        let container = Container::empty();
        assert_eq!(container.len(), 0);
        assert!(container.is_empty());
    }

    #[test]
    fn test_container_with_items() {
        let para = Paragraph::from_line("Test".to_string());
        let container = Container::new(vec![ContentItem::Paragraph(para)]);
        assert_eq!(container.len(), 1);
        assert!(!container.is_empty());
    }

    #[test]
    fn test_container_push() {
        let mut container = Container::empty();
        let para = Paragraph::from_line("Test".to_string());
        container.push(ContentItem::Paragraph(para));
        assert_eq!(container.len(), 1);
    }

    #[test]
    fn test_container_deref() {
        let para = Paragraph::from_line("Test".to_string());
        let container = Container::new(vec![ContentItem::Paragraph(para)]);
        // Should be able to use Vec methods directly via Deref
        assert_eq!(container.len(), 1);
        assert!(!container.is_empty());
    }
}
