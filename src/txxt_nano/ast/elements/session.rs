//! Session element definition

use super::super::span::Location;
use super::super::text_content::TextContent;
use super::super::traits::{AstNode, Container};
use super::content_item::ContentItem;
use std::fmt;

/// A session represents a hierarchical container with a title
#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    pub title: TextContent,
    pub content: Vec<ContentItem>,
    pub span: Option<Location>,
}

impl Session {
    pub fn new(title: TextContent, content: Vec<ContentItem>) -> Self {
        Self {
            title,
            content,
            span: None,
        }
    }
    pub fn with_title(title: String) -> Self {
        Self {
            title: TextContent::from_string(title, None),
            content: Vec::new(),
            span: None,
        }
    }
    pub fn with_span(mut self, span: Option<Location>) -> Self {
        self.span = span;
        self
    }
}

impl AstNode for Session {
    fn node_type(&self) -> &'static str {
        "Session"
    }
    fn display_label(&self) -> String {
        self.title.as_string().to_string()
    }
    fn location(&self) -> Option<Location> {
        self.span
    }
}

impl Container for Session {
    fn label(&self) -> &str {
        self.title.as_string()
    }
    fn children(&self) -> &[ContentItem] {
        &self.content
    }
    fn children_mut(&mut self) -> &mut Vec<ContentItem> {
        &mut self.content
    }
}

impl fmt::Display for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Session('{}', {} items)",
            self.title.as_string(),
            self.content.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::paragraph::Paragraph;
    use super::*;

    #[test]
    fn test_session_creation() {
        let mut session = Session::with_title("Introduction".to_string());
        session
            .children_mut()
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Content".to_string(),
            )));
        assert_eq!(session.label(), "Introduction");
        assert_eq!(session.content.len(), 1);
    }

    #[test]
    fn test_session_with_span() {
        let span = super::super::super::span::Location::new(
            super::super::super::span::Position::new(1, 0),
            super::super::super::span::Position::new(1, 10),
        );
        let session = Session::with_title("Title".to_string()).with_span(Some(span));
        assert_eq!(session.span, Some(span));
    }
}
