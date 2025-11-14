//! List element
//!
//! A list is an ordered collection of items, each with its own text
//! and optional nested content. Lists can be used to structure
//! outlines, steps, or bullet points.
//!
//! Lists have decoration styles, for example the plain one (dashes) or various ordering formats, as numerical, alphabetical, roman, etc.
//!
//! Lists must have a minimum of 2 items.  And it's not ilegal to have mixed decorations in a list, as the parser will consider the first item's decoration to set the list type. The ordering doesn't have to be correct, as lists itself are ordered, they are just a marker, but tooling will order them under demand.
//!
//! Examples:
//!    A flat list with the plain decoration:
//!         - Bread
//!         - Milk
//! They can be nested, and have other styles:
//!    1. Groceries
//!        - Bread
//!        - Milk
//!
//!
//! Learn More:
//! - Lists spec: docs/specs/v1/elements/lists.lex
//! - Labels (used by annotations in lists): docs/specs/v1/elements/labels.lex
//! - Parameters (used by annotations in lists): docs/specs/v1/elements/parameters.lex

use super::super::range::{Position, Range};
use super::super::text_content::TextContent;
use super::super::traits::AstNode;
use super::super::traits::Container;
use super::super::traits::Visitor;
use super::annotation::Annotation;
use super::container::{GeneralContainer, ListContainer};
use super::content_item::ContentItem;
use super::typed_content::{ContentElement, ListContent};
use std::fmt;

/// A list contains multiple list items
#[derive(Debug, Clone, PartialEq)]
pub struct List {
    pub items: ListContainer,
    pub annotations: Vec<Annotation>,
    pub location: Range,
}

/// A list item has text and optional nested content
#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    pub text: Vec<TextContent>,
    pub children: GeneralContainer,
    pub annotations: Vec<Annotation>,
    pub location: Range,
}

impl List {
    fn default_location() -> Range {
        Range::new(0..0, Position::new(0, 0), Position::new(0, 0))
    }
    pub fn new(items: Vec<ListItem>) -> Self {
        let typed_items = items
            .into_iter()
            .map(ListContent::ListItem)
            .collect::<Vec<_>>();
        Self {
            items: ListContainer::from_typed(typed_items),
            annotations: Vec::new(),
            location: Self::default_location(),
        }
    }

    /// Preferred builder
    pub fn at(mut self, location: Range) -> Self {
        self.location = location;
        self
    }

    /// Annotations attached to this list as a whole.
    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    /// Mutable access to list annotations.
    pub fn annotations_mut(&mut self) -> &mut Vec<Annotation> {
        &mut self.annotations
    }

    /// Iterate over annotation blocks on the list element.
    pub fn iter_annotations(&self) -> std::slice::Iter<'_, Annotation> {
        self.annotations.iter()
    }

    /// Iterate over all content items nested inside list-level annotations.
    pub fn iter_annotation_contents(&self) -> impl Iterator<Item = &ContentItem> {
        self.annotations
            .iter()
            .flat_map(|annotation| annotation.children())
    }
}

impl AstNode for List {
    fn node_type(&self) -> &'static str {
        "List"
    }
    fn display_label(&self) -> String {
        format!("{} items", self.items.len())
    }
    fn range(&self) -> &Range {
        &self.location
    }

    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_list(self);
        super::super::traits::visit_children(visitor, &self.items);
    }
}

impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "List({} items)", self.items.len())
    }
}

impl ListItem {
    fn default_location() -> Range {
        Range::new(0..0, Position::new(0, 0), Position::new(0, 0))
    }
    pub fn new(text: String) -> Self {
        Self {
            text: vec![TextContent::from_string(text, None)],
            children: GeneralContainer::empty(),
            annotations: Vec::new(),
            location: Self::default_location(),
        }
    }
    pub fn with_content(text: String, children: Vec<ContentElement>) -> Self {
        Self {
            text: vec![TextContent::from_string(text, None)],
            children: GeneralContainer::from_typed(children),
            annotations: Vec::new(),
            location: Self::default_location(),
        }
    }
    /// Create a ListItem with TextContent that may have location information
    pub fn with_text_content(text_content: TextContent, children: Vec<ContentElement>) -> Self {
        Self {
            text: vec![text_content],
            children: GeneralContainer::from_typed(children),
            annotations: Vec::new(),
            location: Self::default_location(),
        }
    }

    /// Preferred builder
    pub fn at(mut self, location: Range) -> Self {
        self.location = location;
        self
    }
    pub fn text(&self) -> &str {
        self.text[0].as_string()
    }

    /// Annotations attached to this list item.
    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    /// Mutable access to list-item annotations.
    pub fn annotations_mut(&mut self) -> &mut Vec<Annotation> {
        &mut self.annotations
    }

    /// Iterate annotation blocks associated with this list item.
    pub fn iter_annotations(&self) -> std::slice::Iter<'_, Annotation> {
        self.annotations.iter()
    }

    /// Iterate all content items nested inside the list item's annotations.
    pub fn iter_annotation_contents(&self) -> impl Iterator<Item = &ContentItem> {
        self.annotations
            .iter()
            .flat_map(|annotation| annotation.children())
    }
}

impl AstNode for ListItem {
    fn node_type(&self) -> &'static str {
        "ListItem"
    }
    fn display_label(&self) -> String {
        let text = self.text();
        if text.len() > 50 {
            format!("{}...", &text[..50])
        } else {
            text.to_string()
        }
    }
    fn range(&self) -> &Range {
        &self.location
    }

    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_list_item(self);
        super::super::traits::visit_children(visitor, &self.children);
    }
}

impl Container for ListItem {
    fn label(&self) -> &str {
        self.text[0].as_string()
    }
    fn children(&self) -> &[ContentItem] {
        &self.children
    }
    fn children_mut(&mut self) -> &mut Vec<ContentItem> {
        &mut self.children
    }
}

impl fmt::Display for ListItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ListItem('{}')", self.text())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list() {
        let location = super::super::super::range::Range::new(
            0..0,
            super::super::super::range::Position::new(1, 0),
            super::super::super::range::Position::new(1, 10),
        );
        let list = List::new(vec![]).at(location.clone());
        assert_eq!(list.location, location);
    }
}
