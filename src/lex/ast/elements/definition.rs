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
//! - The definition spec: docs/specs/v1/elements/definitions.lex
//! - The definition sample: docs/specs/v1/samples/element-based/definitions/definitions.simple.lex

use super::super::range::{Position, Range};
use super::super::text_content::TextContent;
use super::super::traits::{AstNode, Container, Visitor};
use super::container::GeneralContainer;
use super::content_item::ContentItem;
use std::fmt;

/// A definition provides a subject and associated content
#[derive(Debug, Clone, PartialEq)]
pub struct Definition {
    pub subject: TextContent,
    pub children: GeneralContainer,
    pub location: Range,
}

impl Definition {
    fn default_location() -> Range {
        Range::new(0..0, Position::new(0, 0), Position::new(0, 0))
    }
    pub fn new(subject: TextContent, children: Vec<ContentItem>) -> Self {
        Self {
            subject,
            children: GeneralContainer::new(children),
            location: Self::default_location(),
        }
    }
    pub fn with_subject(subject: String) -> Self {
        Self {
            subject: TextContent::from_string(subject, None),
            children: GeneralContainer::empty(),
            location: Self::default_location(),
        }
    }
    /// Preferred builder
    pub fn at(mut self, location: Range) -> Self {
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
    fn range(&self) -> &Range {
        &self.location
    }

    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_definition(self);
        super::super::traits::visit_children(visitor, &self.children);
    }
}

impl Container for Definition {
    fn label(&self) -> &str {
        self.subject.as_string()
    }
    fn children(&self) -> &[ContentItem] {
        &self.children
    }
    fn children_mut(&mut self) -> &mut Vec<ContentItem> {
        &mut self.children
    }
}

impl fmt::Display for Definition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Definition('{}', {} items)",
            self.subject.as_string(),
            self.children.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_definition() {
        let location = super::super::super::range::Range::new(
            0..0,
            super::super::super::range::Position::new(1, 0),
            super::super::super::range::Position::new(1, 10),
        );
        let definition = Definition::with_subject("Subject".to_string()).at(location.clone());
        assert_eq!(definition.location, location);
    }
}
