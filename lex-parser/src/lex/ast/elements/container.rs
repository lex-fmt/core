//! Container element
//!
//! Container represents a collection of nested children elements.
//! This is used for true parent>child relationships (Sessions, Definitions, etc.)
//! and is distinct from "core items" (lines in paragraphs, items in lists).
//!
//! The Container types provide:
//! - Type safety distinguishing children from core items
//! - Type-level enforcement of nesting rules (Session vs General containers)
//! - Uniform handling of nested content
//! - Location tracking for nested element spans
//!
//! ## Container Types
//!
//! - **SessionContainer**: Can contain any ContentItem including Sessions
//!   - Used by: Document.root, Session.children
//! - **GeneralContainer**: Can contain any ContentItem EXCEPT Sessions
//!   - Used by: Definition.children, Annotation.children, ListItem.children
//! - **ListContainer**: Homogeneous container for ListItem variants only
//!   - Used by: List.items
//! - **VerbatimContainer**: Homogeneous container for VerbatimLine nodes only
//!   - Used by: VerbatimBlock.children

use super::super::range::Range;
use super::super::traits::{AstNode, Visitor};
use super::content_item::ContentItem;
use std::fmt;

/// SessionContainer allows any ContentItem including nested Sessions
///
/// Used for document-level containers where unlimited Session nesting is allowed.
#[derive(Debug, Clone, PartialEq)]
pub struct SessionContainer {
    pub children: Vec<ContentItem>,
    pub location: Range,
}

/// GeneralContainer allows any ContentItem EXCEPT Sessions
///
/// Used for Definition, Annotation, and ListItem children where Session nesting
/// is prohibited.
#[derive(Debug, Clone, PartialEq)]
pub struct GeneralContainer {
    pub children: Vec<ContentItem>,
    pub location: Range,
}

/// ListContainer is a homogeneous container for ListItem variants only
///
/// Used by List.items to enforce that lists only contain list items.
#[derive(Debug, Clone, PartialEq)]
pub struct ListContainer {
    pub children: Vec<ContentItem>,
    pub location: Range,
}

/// VerbatimContainer is a homogeneous container for VerbatimLine nodes only
///
/// Used by VerbatimBlock.children to enforce that verbatim blocks only contain
/// verbatim lines (content from other formats).
#[derive(Debug, Clone, PartialEq)]
pub struct VerbatimContainer {
    pub children: Vec<ContentItem>,
    pub location: Range,
}

/// Legacy type alias for backward compatibility during migration
#[deprecated(note = "Use specialized container types instead")]
pub type Container = SessionContainer;

// Macro to implement common container methods
macro_rules! impl_container {
    ($container_type:ident, $node_type_name:expr) => {
        impl $container_type {
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

        impl AstNode for $container_type {
            fn node_type(&self) -> &'static str {
                $node_type_name
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
        impl std::ops::Deref for $container_type {
            type Target = Vec<ContentItem>;

            fn deref(&self) -> &Self::Target {
                &self.children
            }
        }

        impl std::ops::DerefMut for $container_type {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.children
            }
        }

        impl fmt::Display for $container_type {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({} items)", $node_type_name, self.children.len())
            }
        }

        // Implement IntoIterator to allow for loops over Container
        impl<'a> IntoIterator for &'a $container_type {
            type Item = &'a ContentItem;
            type IntoIter = std::slice::Iter<'a, ContentItem>;

            fn into_iter(self) -> Self::IntoIter {
                self.children.iter()
            }
        }

        impl<'a> IntoIterator for &'a mut $container_type {
            type Item = &'a mut ContentItem;
            type IntoIter = std::slice::IterMut<'a, ContentItem>;

            fn into_iter(self) -> Self::IntoIter {
                self.children.iter_mut()
            }
        }
    };
}

// Apply implementations to all container types
impl_container!(SessionContainer, "SessionContainer");
impl_container!(GeneralContainer, "GeneralContainer");
impl_container!(ListContainer, "ListContainer");
impl_container!(VerbatimContainer, "VerbatimContainer");

#[cfg(test)]
mod tests {
    use super::super::paragraph::Paragraph;
    use super::*;

    #[test]
    fn test_session_container_creation() {
        let container = SessionContainer::empty();
        assert_eq!(container.len(), 0);
        assert!(container.is_empty());
    }

    #[test]
    fn test_general_container_creation() {
        let container = GeneralContainer::empty();
        assert_eq!(container.len(), 0);
        assert!(container.is_empty());
    }

    #[test]
    fn test_list_container_creation() {
        let container = ListContainer::empty();
        assert_eq!(container.len(), 0);
        assert!(container.is_empty());
    }

    #[test]
    fn test_verbatim_container_creation() {
        let container = VerbatimContainer::empty();
        assert_eq!(container.len(), 0);
        assert!(container.is_empty());
    }

    #[test]
    fn test_container_with_items() {
        let para = Paragraph::from_line("Test".to_string());
        let container = SessionContainer::new(vec![ContentItem::Paragraph(para)]);
        assert_eq!(container.len(), 1);
        assert!(!container.is_empty());
    }

    #[test]
    fn test_container_push() {
        let mut container = GeneralContainer::empty();
        let para = Paragraph::from_line("Test".to_string());
        container.push(ContentItem::Paragraph(para));
        assert_eq!(container.len(), 1);
    }

    #[test]
    fn test_container_deref() {
        let para = Paragraph::from_line("Test".to_string());
        let container = ListContainer::new(vec![ContentItem::Paragraph(para)]);
        // Should be able to use Vec methods directly via Deref
        assert_eq!(container.len(), 1);
        assert!(!container.is_empty());
    }
}
