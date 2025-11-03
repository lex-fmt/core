//! Annotation
//!
//! Annotations are a core element in lex, but not the document's content , rather it's metadata one.
//! They provide not only a way for authors and collaborators to register non content related
//! information, but the right hooks for tooling to build on top of lex (e.g., comments, review
//! metadata, publishing hints).
//!
//! As such they provide : -
//! - labels: a way to identify the annotation
//! - parameters[optional]:  a way to provide structured data
//! - Optional content, like all other elements:
//!     - Nestable containter that can host any element but sessions
//!     - Shorthand for for single or no content annotations.
//!
//!
//! Syntax:
//!   Short Hand Form:
//!     <lex-marker> <label> <parameters>? <lex-marker>
//!   Long Hand Form:
//!     <lex-marker> <label> <parameters>? <lex-marker>
//!     <indent> <content> ... any number of content elements
//!     <dedent> <lex-marker>
//!
//!  Examples:
//!      Label only:
//!         :: image ::  
//!      Label and parameters:
//!         :: note severity=high :: Check this carefully
//!      Marker form (no content):
//!         :: debug ::
//!      Params-only:
//!         :: type=python :: (params-only, no label)
//!      Long Form:
//!         :: label ::
//!             John has reviewed this paragraph. Hence we're only lacking:
//!             - Janest's approval
//!             - OK from legal
//! Learn More:
//! - The annotation spec: docs/specs/v1/elements/annotations.lex
//! - The annotation sample: docs/specs/v1/samples/element-based/annotations/annotations.simple.lex
//! - Labels: docs/specs/v1/elements/labels.lex
//! - Parameters: docs/specs/v1/elements/parameters.lex

use super::super::range::{Position, Range};
use super::super::traits::{AstNode, Container, Visitor};
use super::content_item::ContentItem;
use super::label::Label;
use super::parameter::Parameter;
use std::fmt;

/// An annotation represents some metadata about an ast element.
#[derive(Debug, Clone, PartialEq)]
pub struct Annotation {
    pub label: Label,
    pub parameters: Vec<Parameter>,
    pub content: Vec<ContentItem>,
    pub location: Range,
}

impl Annotation {
    fn default_location() -> Range {
        Range::new(0..0, Position::new(0, 0), Position::new(0, 0))
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
    #[deprecated(note = "Use at(location) instead")]
    pub fn with_location(self, location: Range) -> Self {
        self.at(location)
    }
    /// Preferred builder
    pub fn at(mut self, location: Range) -> Self {
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
    fn range(&self) -> &Range {
        &self.location
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
        let location = super::super::super::range::Range::new(
            0..0,
            super::super::super::range::Position::new(1, 0),
            super::super::super::range::Position::new(1, 10),
        );
        let annotation = Annotation::marker(Label::new("test".to_string())).at(location.clone());
        assert_eq!(annotation.location, location);
    }
}
