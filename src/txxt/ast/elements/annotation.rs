//! Annotation
//!
//! Annotations are a core element in txxt, but a metadata one. They provide not only a way for authors and collaborators to register non content related information, but the right hooks for tooling to build on top of txxt (e.g., comments, review metadata, publishing hints).
//!
// As such they provide labels (a way to identify the annotation) and parameters (a way to provide structured metadata for tooling).
//!
//! Syntax Forms:
//!
//! Syntax patterns (informal):
//! - `:: label :: content`
//! - `:: label key=value ::`
//!
//! The full form:
//!
//!
//! :: label ::
//!     indented paragraph or list
//! ::
//!
//! Examples:
//! - `:: reviewed by=alice ::`
//!
//! Learn More:
//! - The annotation spec: docs/specs/v1/elements/annotations.txxt
//! - The annotation sample: docs/specs/v1/samples/element-based/annotations/annotations.simple.txxt
//! - Labels: docs/specs/v1/elements/labels.txxt
//! - Parameters: docs/specs/v1/elements/parameters.txxt

use super::super::location::{Location, Position};
use super::super::traits::{AstNode, Container, Visitor};
use super::content_item::ContentItem;
use super::label::Label;
use super::parameter::Parameter;
use std::fmt;

/// An annotation represents a labeled element with parameters and content
#[derive(Debug, Clone, PartialEq)]
pub struct Annotation {
    pub label: Label,
    pub parameters: Vec<Parameter>,
    pub content: Vec<ContentItem>,
    pub location: Location,
}

impl Annotation {
    fn default_location() -> Location {
        Location::new(Position::new(0, 0), Position::new(0, 0))
    }
    pub fn new(label: Label, parameters: Vec<Parameter>, content: Vec<ContentItem>) -> Self {
        Self {
            label,
            parameters,
            content,
            location: Self::default_location(),
        }
    }
    pub fn marker(label: Label) -> Self {
        Self {
            label,
            parameters: Vec::new(),
            content: Vec::new(),
            location: Self::default_location(),
        }
    }
    pub fn with_parameters(label: Label, parameters: Vec<Parameter>) -> Self {
        Self {
            label,
            parameters,
            content: Vec::new(),
            location: Self::default_location(),
        }
    }
    pub fn with_location(mut self, location: Location) -> Self {
        self.location = location;
        self
    }
}

impl AstNode for Annotation {
    fn node_type(&self) -> &'static str {
        "Annotation"
    }
    fn display_label(&self) -> String {
        if self.parameters.is_empty() {
            self.label.value.clone()
        } else {
            format!("{} ({} params)", self.label.value, self.parameters.len())
        }
    }
    fn location(&self) -> Option<Location> {
        Some(self.location)
    }

    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_annotation(self);
        super::super::traits::visit_children(visitor, &self.content);
    }
}

impl Container for Annotation {
    fn label(&self) -> &str {
        &self.label.value
    }
    fn children(&self) -> &[ContentItem] {
        &self.content
    }
    fn children_mut(&mut self) -> &mut Vec<ContentItem> {
        &mut self.content
    }
}

impl fmt::Display for Annotation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Annotation('{}', {} params, {} items)",
            self.label.value,
            self.parameters.len(),
            self.content.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_annotation_with_location() {
        let location = super::super::super::location::Location::new(
            super::super::super::location::Position::new(1, 0),
            super::super::super::location::Position::new(1, 10),
        );
        let annotation =
            Annotation::marker(Label::new("test".to_string())).with_location(location);
        assert_eq!(annotation.location, location);
    }
}
