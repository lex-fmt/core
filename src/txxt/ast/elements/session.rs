//! Session element
//!
//! A session is the main structural element of txxt documents. Sessions can be arbitrarily nested
//! and contain required titles and content.
//!
//! Sessions establish hierarchy within a document via their title e and nested content, like all
//! major elements in txxt.
//!
//! Structure:
//! - Title: short text identifying the session
//! - Content: any elements allowed in the body
//!
//! The title can be any text content, and is often decorated with an ordering indicator, just like lists,
//! and in txxt all the numerical, alphabetical, and roman numeral indicators are supported.
//!
//! Examples:
//!
//! Welcome to The Txxt format
//!
//!     Txxt is a plain text document format. ...
//!
//! 1.4 The Finale
//!
//!     Here is where we stop.
//!
use super::super::location::{Location, Position};
use super::super::text_content::TextContent;
use super::super::traits::{AstNode, Container, Visitor};
use super::content_item::ContentItem;
use std::fmt;

/// A session represents a hierarchical container with a title
#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    pub title: TextContent,
    pub content: Vec<ContentItem>,
    pub location: Location,
}

impl Session {
    fn default_location() -> Location {
        Location::new(Position::new(0, 0), Position::new(0, 0))
    }
    pub fn new(title: TextContent, content: Vec<ContentItem>) -> Self {
        Self {
            title,
            content,
            location: Self::default_location(),
        }
    }
    pub fn with_title(title: String) -> Self {
        Self {
            title: TextContent::from_string(title, None),
            content: Vec::new(),
            location: Self::default_location(),
        }
    }
    #[deprecated(note = "Use at(location) instead")]
    pub fn with_location(self, location: Location) -> Self {
        self.at(location)
    }
    /// Preferred builder
    pub fn at(mut self, location: Location) -> Self {
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
    fn location(&self) -> Location {
        self.location
    }

    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_session(self);
        super::super::traits::visit_children(visitor, &self.content);
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
    fn test_session() {
        let location = super::super::super::location::Location::new(
            super::super::super::location::Position::new(1, 0),
            super::super::super::location::Position::new(1, 10),
        );
        let session = Session::with_title("Title".to_string()).at(location);
        assert_eq!(session.location, location);
    }
}
