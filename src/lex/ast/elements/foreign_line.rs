//! Foreign line element
//!
//! A foreign line represents a single line of foreign content within a foreign block.
//! This is the "lead item" for foreign blocks, similar to how sessions have titles
//! and definitions have subjects.
//!
//! The foreign line handles the indentation wall - stripping the common indentation
//! from all content lines to preserve content integrity regardless of nesting level.
//!
//! Structure:
//! - content: The raw text content of the foreign line
//! - location: The byte range and position information
//!
//! Note: Foreign lines are typically collected as children of a ForeignBlock, but
//! a foreign block can forgo content entirely (e.g., for binary markers).

use super::super::range::{Position, Range};
use super::super::text_content::TextContent;
use super::super::traits::AstNode;
use super::super::traits::Visitor;
use std::fmt;

/// A foreign line represents a single line of foreign content
#[derive(Debug, Clone, PartialEq)]
pub struct ForeignLine {
    pub content: TextContent,
    pub location: Range,
}

impl ForeignLine {
    fn default_location() -> Range {
        Range::new(0..0, Position::new(0, 0), Position::new(0, 0))
    }

    pub fn new(content: String) -> Self {
        Self {
            content: TextContent::from_string(content, None),
            location: Self::default_location(),
        }
    }

    pub fn from_text_content(content: TextContent) -> Self {
        Self {
            content,
            location: Self::default_location(),
        }
    }

    /// Preferred builder
    pub fn at(mut self, location: Range) -> Self {
        self.location = location;
        self
    }
}

impl AstNode for ForeignLine {
    fn node_type(&self) -> &'static str {
        "ForeignLine"
    }

    fn display_label(&self) -> String {
        let content_text = self.content.as_string();
        if content_text.len() > 50 {
            format!("{}...", &content_text[..50])
        } else {
            content_text.to_string()
        }
    }

    fn range(&self) -> &Range {
        &self.location
    }

    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_foreign_line(self);
        // ForeignLine has no children - it's a leaf node
    }
}

impl fmt::Display for ForeignLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ForeignLine({} chars)", self.content.as_string().len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_foreign_line_creation() {
        let line = ForeignLine::new("    code line".to_string());
        assert_eq!(line.content.as_string(), "    code line");
    }

    #[test]
    fn test_foreign_line_with_location() {
        let location = Range::new(0..12, Position::new(1, 0), Position::new(1, 12));
        let line = ForeignLine::new("    code line".to_string()).at(location.clone());
        assert_eq!(line.location, location);
    }
}
