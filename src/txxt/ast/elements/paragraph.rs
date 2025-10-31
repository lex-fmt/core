//! Paragraph element definition

use super::super::location::Location;
use super::super::text_content::TextContent;
use super::super::traits::{AstNode, TextNode};
use std::fmt;

/// A text line within a paragraph
#[derive(Debug, Clone, PartialEq)]
pub struct TextLine {
    pub content: TextContent,
    pub location: Option<Location>,
}

impl TextLine {
    pub fn new(content: TextContent) -> Self {
        Self {
            content,
            location: None,
        }
    }

    pub fn with_location(mut self, location: Option<Location>) -> Self {
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
        self.location
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
    pub location: Option<Location>,
}

impl Paragraph {
    pub fn new(lines: Vec<super::content_item::ContentItem>) -> Self {
        Self {
            lines,
            location: None,
        }
    }
    pub fn from_line(line: String) -> Self {
        Self {
            lines: vec![super::content_item::ContentItem::TextLine(TextLine::new(
                TextContent::from_string(line, None),
            ))],
            location: None,
        }
    }
    pub fn with_location(mut self, location: Option<Location>) -> Self {
        self.location = location;
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
        self.location
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
        let para = Paragraph::from_line("Hello".to_string()).with_location(Some(location));

        assert_eq!(para.location, Some(location));
    }
}
