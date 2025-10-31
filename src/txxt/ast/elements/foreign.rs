//! Foreign block element
//!
//! A foreign block embeds content from another system or format
//! between a subject marker and a closing annotation. Its content
//! is treated as opaque text by the txxt AST.
//!
//! Structure:
//! - Subject: identifies the kind of foreign content
//! - Content: raw text payload, not parsed into txxt elements
//! - Closing annotation: terminates the block and may carry metadata
//!
//! Learn More:
//! - Foreign blocks spec: docs/specs/v1/elements/foreign.txxt

use super::super::location::Location;
use super::super::text_content::TextContent;
use super::super::traits::AstNode;
use super::super::traits::Visitor;
use super::annotation::Annotation;
use std::fmt;

/// A foreign block represents content from another format/system
#[derive(Debug, Clone, PartialEq)]
pub struct ForeignBlock {
    pub subject: TextContent,
    pub content: TextContent,
    pub closing_annotation: Annotation,
    pub location: Option<Location>,
}

impl ForeignBlock {
    pub fn new(subject: String, content: String, closing_annotation: Annotation) -> Self {
        Self {
            subject: TextContent::from_string(subject, None),
            content: TextContent::from_string(content, None),
            closing_annotation,
            location: None,
        }
    }
    pub fn marker(subject: String, closing_annotation: Annotation) -> Self {
        Self {
            subject: TextContent::from_string(subject, None),
            content: TextContent::from_string(String::new(), None),
            closing_annotation,
            location: None,
        }
    }
    pub fn with_location(mut self, location: Option<Location>) -> Self {
        self.location = location;
        self
    }
}

impl AstNode for ForeignBlock {
    fn node_type(&self) -> &'static str {
        "ForeignBlock"
    }
    fn display_label(&self) -> String {
        let subject_text = self.subject.as_string();
        if subject_text.len() > 50 {
            format!("{}...", &subject_text[..50])
        } else {
            subject_text.to_string()
        }
    }
    fn location(&self) -> Option<Location> {
        self.location
    }

    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_foreign_block(self);
        // ForeignBlock has no children to visit - content is opaque
    }
}

impl fmt::Display for ForeignBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ForeignBlock('{}', {} chars, closing: {})",
            self.subject.as_string(),
            self.content.as_string().len(),
            self.closing_annotation.label.value
        )
    }
}
