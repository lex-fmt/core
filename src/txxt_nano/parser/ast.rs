//! Abstract Syntax Tree (AST) definitions for the txxt format
//!
//! This module defines the data structures that represent the parsed
//! structure of a txxt document.

use std::fmt;

/// A complete txxt document
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    /// Top-level content items (paragraphs and sessions)
    pub items: Vec<ContentItem>,
}

/// A content item can be either a paragraph or a session
#[derive(Debug, Clone, PartialEq)]
pub enum ContentItem {
    Paragraph(Paragraph),
    Session(Session),
}

/// A paragraph is a block of text content
#[derive(Debug, Clone, PartialEq)]
pub struct Paragraph {
    /// The text lines that make up this paragraph
    pub lines: Vec<String>,
}

/// A session is a titled section that can contain nested content
#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    /// The title of the session
    pub title: String,
    /// Content items within this session (paragraphs and nested sessions)
    pub content: Vec<ContentItem>,
}

impl Document {
    /// Create a new empty document
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Create a document with the given items
    pub fn with_items(items: Vec<ContentItem>) -> Self {
        Self { items }
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl Paragraph {
    /// Create a new paragraph with the given lines
    pub fn new(lines: Vec<String>) -> Self {
        Self { lines }
    }

    /// Create a paragraph from a single line
    pub fn from_line(line: String) -> Self {
        Self { lines: vec![line] }
    }

    /// Get the full text of the paragraph
    pub fn text(&self) -> String {
        self.lines.join("\n")
    }
}

impl Session {
    /// Create a new session with the given title and content
    pub fn new(title: String, content: Vec<ContentItem>) -> Self {
        Self { title, content }
    }

    /// Create a session with just a title and no content
    pub fn with_title(title: String) -> Self {
        Self {
            title,
            content: Vec::new(),
        }
    }
}

impl fmt::Display for Document {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Document({} items)", self.items.len())
    }
}

impl fmt::Display for ContentItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContentItem::Paragraph(p) => write!(f, "Paragraph({} lines)", p.lines.len()),
            ContentItem::Session(s) => {
                write!(f, "Session('{}', {} items)", s.title, s.content.len())
            }
        }
    }
}

impl fmt::Display for Paragraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Paragraph({} lines)", self.lines.len())
    }
}

impl fmt::Display for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Session('{}', {} items)", self.title, self.content.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paragraph_creation() {
        let para = Paragraph::new(vec!["Hello".to_string(), "World".to_string()]);
        assert_eq!(para.lines.len(), 2);
        assert_eq!(para.text(), "Hello\nWorld");
    }

    #[test]
    fn test_session_creation() {
        let session = Session::new(
            "Introduction".to_string(),
            vec![ContentItem::Paragraph(Paragraph::from_line(
                "Content".to_string(),
            ))],
        );
        assert_eq!(session.title, "Introduction");
        assert_eq!(session.content.len(), 1);
    }

    #[test]
    fn test_document_creation() {
        let doc = Document::with_items(vec![
            ContentItem::Paragraph(Paragraph::from_line("Para 1".to_string())),
            ContentItem::Session(Session::with_title("Section 1".to_string())),
        ]);
        assert_eq!(doc.items.len(), 2);
    }
}
