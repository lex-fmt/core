//! Definition element definition

use super::super::span::Span;
use super::super::text_content::TextContent;
use super::super::traits::{AstNode, Container};
use super::content_item::ContentItem;
use std::fmt;

/// A definition provides a subject and associated content
#[derive(Debug, Clone, PartialEq)]
pub struct Definition {
    pub subject: TextContent,
    pub content: Vec<ContentItem>,
    pub span: Option<Span>,
}

impl Definition {
    pub fn new(subject: TextContent, content: Vec<ContentItem>) -> Self {
        Self {
            subject,
            content,
            span: None,
        }
    }
    pub fn with_subject(subject: String) -> Self {
        Self {
            subject: TextContent::from_string(subject, None),
            content: Vec::new(),
            span: None,
        }
    }
    pub fn with_span(mut self, span: Option<Span>) -> Self {
        self.span = span;
        self
    }
}

impl AstNode for Definition {
    fn node_type(&self) -> &'static str {
        "Definition"
    }
    fn display_label(&self) -> String {
        let subject_text = self.subject.as_string();
        if subject_text.len() > 50 {
            format!("{}...", &subject_text[..50])
        } else {
            subject_text.to_string()
        }
    }
    fn span(&self) -> Option<Span> {
        self.span
    }
}

impl Container for Definition {
    fn label(&self) -> &str {
        self.subject.as_string()
    }
    fn children(&self) -> &[ContentItem] {
        &self.content
    }
    fn children_mut(&mut self) -> &mut Vec<ContentItem> {
        &mut self.content
    }
}

impl fmt::Display for Definition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Definition('{}', {} items)",
            self.subject.as_string(),
            self.content.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_definition_with_span() {
        let span = super::super::super::span::Span::new(
            super::super::super::span::Position::new(1, 0),
            super::super::super::span::Position::new(1, 10),
        );
        let definition = Definition::with_subject("Subject".to_string()).with_span(Some(span));
        assert_eq!(definition.span, Some(span));
    }
}
