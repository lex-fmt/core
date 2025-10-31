//! Paragraph element
//!
//! A paragraph is a block of one or more text lines. It represents
//! narrative content. Empty lines separate paragraphs.
//!
//! Properties:
//! - Stores lines as `TextLine` items
//! - Preserves intra-paragraph line breaks
//!
//! Learn More:
//! - Paragraphs spec: docs/specs/v1/elements/paragraphs.txxt
//!
//! Examples:
//! - A single paragraph spans multiple lines until a blank line
//! - Blank lines separate paragraphs; lists and sessions break flow

use super::super::location::{Location, Position};
use super::super::text_content::TextContent;
use super::super::traits::{AstNode, TextNode, Visitor};
use std::fmt;

/// A text line within a paragraph
#[derive(Debug, Clone, PartialEq)]
pub struct TextLine {
    pub content: TextContent,
    pub location: Location,
}

impl TextLine {
    fn default_location() -> Location {
        Location::new(Position::new(0, 0), Position::new(0, 0))
    }
    pub fn new(content: TextContent) -> Self {
        Self {
            content,
            location: Self::default_location(),
        }
    }

    pub fn with_location(mut self, location: Location) -> Self {
        self.location = location;
        self
    }

    pub fn text(&self) -> &str {
        self.content.as_string()
    }
}

impl AstNode for TextLine {
    fn node_type(&self) -> &'static str {
        "TextLine"
    }

    fn display_label(&self) -> String {
        let text = self.text();
        if text.len() > 50 {
            format!("{}...", &text[..50])
        } else {
            text.to_string()
        }
    }

    fn location(&self) -> Option<Location> {
        Some(self.location)
    }

    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_text_line(self);
    }
}

impl fmt::Display for TextLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TextLine('{}')", self.text())
    }
}

/// A paragraph represents a block of text lines
#[derive(Debug, Clone, PartialEq)]
pub struct Paragraph {
    /// Lines stored as ContentItems (each a TextLine wrapping TextContent)
    pub lines: Vec<super::content_item::ContentItem>,
    pub location: Location,
}

impl Paragraph {
    fn default_location() -> Location {
        Location::new(Position::new(0, 0), Position::new(0, 0))
    }
    pub fn new(lines: Vec<super::content_item::ContentItem>) -> Self {
        Self {
            lines,
            location: Self::default_location(),
        }
    }
    pub fn from_line(line: String) -> Self {
        Self {
            lines: vec![super::content_item::ContentItem::TextLine(TextLine::new(
                TextContent::from_string(line, None),
            ))],
            location: Self::default_location(),
        }
    }
    pub fn with_location(mut self, location: Location) -> Self {
        self.location = location;
        // When a paragraph's location is set in tests, we should also update
        // the location of the single child TextLine for consistency, as this
        // is what the parser would do.
        if self.lines.len() == 1 {
            if let Some(super::content_item::ContentItem::TextLine(text_line)) =
                self.lines.get_mut(0)
            {
                text_line.location = location;
            }
        }
        self
    }
    pub fn text(&self) -> String {
        self.lines
            .iter()
            .filter_map(|item| {
                if let super::content_item::ContentItem::TextLine(tl) = item {
                    Some(tl.text().to_string())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl AstNode for Paragraph {
    fn node_type(&self) -> &'static str {
        "Paragraph"
    }
    fn display_label(&self) -> String {
        let text = self.text();
        if text.len() > 50 {
            format!("{}...", &text[..50])
        } else {
            text
        }
    }
    fn location(&self) -> Option<Location> {
        Some(self.location)
    }

    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_paragraph(self);
        // Visit child TextLines
        super::super::traits::visit_children(visitor, &self.lines);
    }
}

impl TextNode for Paragraph {
    fn text(&self) -> String {
        self.lines
            .iter()
            .filter_map(|item| {
                if let super::content_item::ContentItem::TextLine(tl) = item {
                    Some(tl.text().to_string())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
    fn lines(&self) -> &[TextContent] {
        // This is a compatibility method - we no longer store raw TextContent
        // Return empty slice since we've moved to ContentItem::TextLine
        &[]
    }
}

impl fmt::Display for Paragraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Paragraph({} lines)", self.lines.len())
    }
}

#[cfg(test)]
mod tests {
    use super::super::content_item::ContentItem;
    use super::*;

    #[test]
    fn test_paragraph_creation() {
        let para = Paragraph::new(vec![
            ContentItem::TextLine(TextLine::new(TextContent::from_string(
                "Hello".to_string(),
                None,
            ))),
            ContentItem::TextLine(TextLine::new(TextContent::from_string(
                "World".to_string(),
                None,
            ))),
        ]);
        assert_eq!(para.lines.len(), 2);
        assert_eq!(para.text(), "Hello\nWorld");
    }

    #[test]
    fn test_paragraph_with_location() {
        let location = Location::new(
            super::super::super::location::Position::new(0, 0),
            super::super::super::location::Position::new(0, 5),
        );
        let para = Paragraph::from_line("Hello".to_string()).with_location(location);

        assert_eq!(para.location, location);
    }
}
