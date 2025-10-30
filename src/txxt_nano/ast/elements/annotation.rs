//! Annotation element definition

use super::super::span::Location;
use super::super::traits::{AstNode, Container};
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
    pub span: Option<Location>,
}

impl Annotation {
    pub fn new(label: Label, parameters: Vec<Parameter>, content: Vec<ContentItem>) -> Self {
        Self {
            label,
            parameters,
            content,
            span: None,
        }
    }
    pub fn marker(label: Label) -> Self {
        Self {
            label,
            parameters: Vec::new(),
            content: Vec::new(),
            span: None,
        }
    }
    pub fn with_parameters(label: Label, parameters: Vec<Parameter>) -> Self {
        Self {
            label,
            parameters,
            content: Vec::new(),
            span: None,
        }
    }
    pub fn with_span(mut self, span: Option<Location>) -> Self {
        self.span = span;
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
        self.span
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
    fn test_annotation_with_span() {
        let span = super::super::super::span::Location::new(
            super::super::super::span::Position::new(1, 0),
            super::super::super::span::Position::new(1, 10),
        );
        let annotation = Annotation::marker(Label::new("test".to_string())).with_span(Some(span));
        assert_eq!(annotation.span, Some(span));
    }
}
