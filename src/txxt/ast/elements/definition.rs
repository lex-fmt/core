//! Definition element definition

use super::super::location::Location;
use super::super::text_content::TextContent;
use super::super::traits::{AstNode, Container, Visitor};
use super::content_item::ContentItem;
use std::fmt;

/// A definition provides a subject and associated content
#[derive(Debug, Clone, PartialEq)]
pub struct Definition {
    pub subject: TextContent,
    pub content: Vec<ContentItem>,
    pub location: Option<Location>,
}

impl Definition {
    pub fn new(subject: TextContent, content: Vec<ContentItem>) -> Self {
        Self {
            subject,
            content,
            location: None,
        }
    }
    pub fn with_subject(subject: String) -> Self {
        Self {
            subject: TextContent::from_string(subject, None),
            content: Vec::new(),
            location: None,
        }
    }
    pub fn with_location(mut self, location: Option<Location>) -> Self {
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
        self.location
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
    fn test_definition_with_location() {
        let location = super::super::super::location::Location::new(
            super::super::super::location::Position::new(1, 0),
            super::super::super::location::Position::new(1, 10),
        );
        let definition =
            Definition::with_subject("Subject".to_string()).with_location(Some(location));
        assert_eq!(definition.location, Some(location));
    }
}
