//! Definition element
//!
//!  Definitions are a core element for explaining terms and concepts.
//!  They pair a subject (the term being defined) with its content, the definition body.
//!
//! Syntax:
//!     <text-span>+ <colon> <line-break>
//!     <indent> <content> ... any number of content elements
//!     <dedent>
//!
//! Examples:
//!     Cache:
//!         Temporary storage for frequently accessed data.
//!
//!     Microservice:
//!         An architectural style that structures applications as loosely coupled services.
//!
//!         Each service is independently deployable and scalable.
//!
//! Learn More:
//! - The definition spec: docs/specs/v1/elements/definitions.txxt
//! - The definition sample: docs/specs/v1/samples/element-based/definitions/definitions.simple.txxt

use super::super::location::{Location, Position};
use super::super::text_content::TextContent;
use super::super::traits::{AstNode, Container, Visitor};
use super::content_item::ContentItem;
use std::fmt;

/// A definition provides a subject and associated content
#[derive(Debug, Clone, PartialEq)]
pub struct Definition {
    pub subject: TextContent,
    pub content: Vec<ContentItem>,
    pub location: Location,
}

impl Definition {
    fn default_location() -> Location {
        Location::new(Position::new(0, 0), Position::new(0, 0))
    }
    pub fn new(subject: TextContent, content: Vec<ContentItem>) -> Self {
        Self {
            subject,
            content,
            location: Self::default_location(),
        }
    }
    pub fn with_subject(subject: String) -> Self {
        Self {
            subject: TextContent::from_string(subject, None),
            content: Vec::new(),
            location: Self::default_location(),
        }
    }
    pub fn with_location(mut self, location: Location) -> Self {
        self.location = location;
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
    fn location(&self) -> Option<Location> {
        Some(self.location)
    }

    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_definition(self);
        super::super::traits::visit_children(visitor, &self.content);
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
    fn test_definition() {
        let location = super::super::super::location::Location::new(
            super::super::super::location::Position::new(1, 0),
            super::super::super::location::Position::new(1, 10),
        );
        let definition = Definition::with_subject("Subject".to_string()).with_location(location);
        assert_eq!(definition.location, location);
    }
}
