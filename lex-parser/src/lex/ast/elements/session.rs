//! Session element
//!
//! A session is the main structural element of lex documents. Sessions can be arbitrarily nested
//! and contain required titles and content.
//!
//! Sessions establish hierarchy within a document via their title e and nested content, like all
//! major elements in lex.
//!
//! Structure:
//! - Title: short text identifying the session
//! - Content: any elements allowed in the body
//!
//! The title can be any text content, and is often decorated with an ordering indicator, just like lists,
//! and in lex all the numerical, alphabetical, and roman numeral indicators are supported.
//!
//! Examples:
//!
//! Welcome to The Lex format
//!
//!     Lex is a plain text document format. ...
//!
//! 1.4 The Finale
//!
//!     Here is where we stop.
//!
use super::super::range::{Position, Range};
use super::super::text_content::TextContent;
use super::super::traits::{AstNode, Container, Visitor};
use super::container::SessionContainer;
use super::content_item::ContentItem;
use super::typed_content::SessionContent;
use std::fmt;

/// A session represents a hierarchical container with a title
#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    pub title: TextContent,
    pub children: SessionContainer,
    pub location: Range,
}

impl Session {
    fn default_location() -> Range {
        Range::new(0..0, Position::new(0, 0), Position::new(0, 0))
    }
    pub fn new(title: TextContent, children: Vec<SessionContent>) -> Self {
        Self {
            title,
            children: SessionContainer::from_typed(children),
            location: Self::default_location(),
        }
    }
    pub fn with_title(title: String) -> Self {
        Self {
            title: TextContent::from_string(title, None),
            children: SessionContainer::empty(),
            location: Self::default_location(),
        }
    }

    /// Preferred builder
    pub fn at(mut self, location: Range) -> Self {
        self.location = location;
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
    fn range(&self) -> &Range {
        &self.location
    }

    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_session(self);
        super::super::traits::visit_children(visitor, &self.children);
    }
}

impl Container for Session {
    fn label(&self) -> &str {
        self.title.as_string()
    }
    fn children(&self) -> &[ContentItem] {
        &self.children
    }
    fn children_mut(&mut self) -> &mut Vec<ContentItem> {
        &mut self.children
    }
}

impl fmt::Display for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Session('{}', {} items)",
            self.title.as_string(),
            self.children.len()
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
        assert_eq!(session.children.len(), 1);
    }

    #[test]
    fn test_session() {
        let location = super::super::super::range::Range::new(
            0..0,
            super::super::super::range::Position::new(1, 0),
            super::super::super::range::Position::new(1, 10),
        );
        let session = Session::with_title("Title".to_string()).at(location.clone());
        assert_eq!(session.location, location);
    }
}
