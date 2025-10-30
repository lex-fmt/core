//! Paragraph element definition

use super::super::span::{Position, Span};
use super::super::text_content::TextContent;
use super::super::traits::{AstNode, NodeStartLocation, TextNode};
use std::fmt;

/// A paragraph represents a block of text lines
#[derive(Debug, Clone, PartialEq)]
pub struct Paragraph {
    pub lines: Vec<TextContent>,
    pub span: Option<Span>,
}

impl Paragraph {
    pub fn new(lines: Vec<TextContent>) -> Self {
        Self { lines, span: None }
    }
    pub fn from_line(line: String) -> Self {
        Self {
            lines: vec![TextContent::from_string(line, None)],
            span: None,
        }
    }
    pub fn with_span(mut self, span: Option<Span>) -> Self {
        self.span = span;
        self
    }
    pub fn text(&self) -> String {
        self.lines
            .iter()
            .map(|line| line.as_string().to_string())
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
}

impl TextNode for Paragraph {
    fn text(&self) -> String {
        self.lines
            .iter()
            .map(|line| line.as_string().to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }
    fn lines(&self) -> &[TextContent] {
        &self.lines
    }
}

impl NodeStartLocation for Paragraph {
    fn start_location(&self) -> Option<Position> {
        self.span.map(|s| s.start)
    }
}

impl fmt::Display for Paragraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Paragraph({} lines)", self.lines.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paragraph_creation() {
        let para = Paragraph::new(vec![
            TextContent::from_string("Hello".to_string(), None),
            TextContent::from_string("World".to_string(), None),
        ]);
        assert_eq!(para.lines.len(), 2);
        assert_eq!(para.text(), "Hello\nWorld");
    }

    #[test]
    fn test_paragraph_with_span() {
        let span = Span::new(
            super::super::super::span::Position::new(0, 0),
            super::super::super::span::Position::new(0, 5),
        );
        let para = Paragraph::from_line("Hello".to_string()).with_span(Some(span));

        assert_eq!(para.span, Some(span));
    }
}
