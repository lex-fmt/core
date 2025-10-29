//! Foreign block element definition

use super::super::span::Span;
use super::super::text_content::TextContent;
use super::super::traits::AstNode;
use super::annotation::Annotation;
use std::fmt;

/// A foreign block represents content from another format/system
#[derive(Debug, Clone, PartialEq)]
pub struct ForeignBlock {
    pub subject: TextContent,
    pub content: TextContent,
    pub closing_annotation: Annotation,
    pub span: Option<Span>,
}

impl ForeignBlock {
    pub fn new(subject: String, content: String, closing_annotation: Annotation) -> Self {
        Self {
            subject: TextContent::from_string(subject, None),
            content: TextContent::from_string(content, None),
            closing_annotation,
            span: None,
        }
    }
    pub fn marker(subject: String, closing_annotation: Annotation) -> Self {
        Self {
            subject: TextContent::from_string(subject, None),
            content: TextContent::from_string(String::new(), None),
            closing_annotation,
            span: None,
        }
    }
    pub fn with_span(mut self, span: Option<Span>) -> Self {
        self.span = span;
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
