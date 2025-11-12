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
//!
//! ## Accessing Container Children
//!
//! The `.children` field is private. Use one of these access patterns:
//!
//! **Deref coercion** (preferred for Vec operations):
//! ```ignore
//! let session = Session::new(...);
//! for child in &session.children {  // Deref to &Vec<ContentItem>
//!     // process child
//! }
//! let count = session.children.len();  // Works via Deref
//! ```
//!
//! **ContentItem polymorphic access**:
//! ```ignore
//! fn process(item: &ContentItem) {
//!     if let Some(children) = item.children() {
//!         // Access children polymorphically
//!     }
//! }
//! ```
//!
//! **Container trait**:
//! ```ignore
//! fn process<T: Container>(container: &T) {
//!     let children = container.children();  // Returns &[ContentItem]
//! }
//! ```

use super::super::range::Range;
use super::super::traits::{AstNode, Visitor};
use super::content_item::ContentItem;
use super::typed_content::{ContentElement, ListContent, SessionContent, VerbatimContent};
use std::fmt;
use std::marker::PhantomData;

// ============================================================================
// CONTAINER POLICY TRAITS
// ============================================================================

/// Policy trait defining what content is allowed in a container.
///
/// This trait provides compile-time information about nesting rules.
/// Each policy type defines which element types can be contained.
pub trait ContainerPolicy: 'static {
    /// The typed content variant this policy accepts
    type ContentType: Into<ContentItem> + Clone;

    /// Whether this container allows Session elements
    const ALLOWS_SESSIONS: bool;

    /// Whether this container allows Annotation elements
    const ALLOWS_ANNOTATIONS: bool;

    /// Human-readable name for error messages
    const POLICY_NAME: &'static str;
}

/// Policy for Session containers - allows all elements including Sessions
///
/// Used by: Document.root, Session.children
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionPolicy;

impl ContainerPolicy for SessionPolicy {
    type ContentType = SessionContent;

    const ALLOWS_SESSIONS: bool = true;
    const ALLOWS_ANNOTATIONS: bool = true;
    const POLICY_NAME: &'static str = "SessionPolicy";
}

/// Policy for general containers - allows all elements EXCEPT Sessions
///
/// Used by: Definition.children, Annotation.children, ListItem.children
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GeneralPolicy;

impl ContainerPolicy for GeneralPolicy {
    type ContentType = ContentElement;

    const ALLOWS_SESSIONS: bool = false;
    const ALLOWS_ANNOTATIONS: bool = true;
    const POLICY_NAME: &'static str = "GeneralPolicy";
}

/// Policy for list containers - only allows ListItem elements
///
/// Used by: List.items
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListPolicy;

impl ContainerPolicy for ListPolicy {
    type ContentType = ListContent;

    const ALLOWS_SESSIONS: bool = false;
    const ALLOWS_ANNOTATIONS: bool = false;
    const POLICY_NAME: &'static str = "ListPolicy";
}

/// Policy for verbatim containers - only allows VerbatimLine elements
///
/// Used by: VerbatimBlock.children
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerbatimPolicy;

impl ContainerPolicy for VerbatimPolicy {
    type ContentType = VerbatimContent;

    const ALLOWS_SESSIONS: bool = false;
    const ALLOWS_ANNOTATIONS: bool = false;
    const POLICY_NAME: &'static str = "VerbatimPolicy";
}

// ============================================================================
// CONTAINER TYPES
// ============================================================================

/// Generic container with compile-time policy enforcement
///
/// The policy type parameter P determines what content is allowed in this container.
/// See the ContainerPolicy trait for available policies.
#[derive(Debug, Clone, PartialEq)]
pub struct Container<P: ContainerPolicy> {
    children: Vec<ContentItem>,
    pub location: Range,
    _policy: PhantomData<P>,
}

// ============================================================================
// TYPE ALIASES
// ============================================================================

/// SessionContainer allows any ContentItem including nested Sessions
///
/// Used for document-level containers where unlimited Session nesting is allowed.
pub type SessionContainer = Container<SessionPolicy>;

/// GeneralContainer allows any ContentItem EXCEPT Sessions
///
/// Used for Definition, Annotation, and ListItem children where Session nesting
/// is prohibited.
pub type GeneralContainer = Container<GeneralPolicy>;

/// ListContainer is a homogeneous container for ListItem variants only
///
/// Used by List.items to enforce that lists only contain list items.
pub type ListContainer = Container<ListPolicy>;

/// VerbatimContainer is a homogeneous container for VerbatimLine nodes only
///
/// Used by VerbatimBlock.children to enforce that verbatim blocks only contain
/// verbatim lines (content from other formats).
pub type VerbatimContainer = Container<VerbatimPolicy>;

// ============================================================================
// GENERIC CONTAINER IMPLEMENTATION
// ============================================================================

impl<P: ContainerPolicy> Container<P> {
    /// Create a new container with the given children (legacy, accepts ContentItem)
    ///
    /// Note: This bypasses type checking. Prefer `from_typed` for type-safe construction.
    pub fn new(children: Vec<ContentItem>) -> Self {
        Self {
            children,
            location: Range::default(),
            _policy: PhantomData,
        }
    }

    /// Create a type-safe container from typed content
    ///
    /// This is the preferred way to create containers as it enforces nesting rules
    /// at compile time via the policy's ContentType.
    ///
    /// # Future Work
    ///
    /// Currently, element constructors (Session::new, Definition::new, etc.) still
    /// accept Vec<ContentItem> for backward compatibility. A future refactoring
    /// could update these to accept typed content directly, enabling full compile-time
    /// enforcement throughout the construction pipeline.
    pub fn from_typed(children: Vec<P::ContentType>) -> Self {
        Self {
            children: children.into_iter().map(|c| c.into()).collect(),
            location: Range::default(),
            _policy: PhantomData,
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

impl<P: ContainerPolicy> AstNode for Container<P> {
    fn node_type(&self) -> &'static str {
        P::POLICY_NAME
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
impl<P: ContainerPolicy> std::ops::Deref for Container<P> {
    type Target = Vec<ContentItem>;

    fn deref(&self) -> &Self::Target {
        &self.children
    }
}

impl<P: ContainerPolicy> std::ops::DerefMut for Container<P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.children
    }
}

impl<P: ContainerPolicy> fmt::Display for Container<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({} items)", P::POLICY_NAME, self.children.len())
    }
}

// Implement IntoIterator to allow for loops over Container
impl<'a, P: ContainerPolicy> IntoIterator for &'a Container<P> {
    type Item = &'a ContentItem;
    type IntoIter = std::slice::Iter<'a, ContentItem>;

    fn into_iter(self) -> Self::IntoIter {
        self.children.iter()
    }
}

impl<'a, P: ContainerPolicy> IntoIterator for &'a mut Container<P> {
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
