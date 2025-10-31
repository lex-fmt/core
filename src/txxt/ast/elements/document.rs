//! Document element
//!
//! A document is the root of a txxt tree. It can contain metadata
//! (as annotations) and a sequence of content elements (paragraphs,
//! sessions, lists, foreign blocks, definitions, annotations).
//!
//! Structure:
//! - Metadata: zero or more leading annotations that apply to the whole document
//! - Content: ordered list of content items making up the body
//!
//! Learn More:
//! - Paragraphs: docs/specs/v1/elements/paragraphs.txxt
//! - Lists: docs/specs/v1/elements/lists.txxt
//! - Annotations: docs/specs/v1/elements/annotations.txxt
//! - Definitions: docs/specs/v1/elements/definitions.txxt
//! - Foreign blocks: docs/specs/v1/elements/foreign.txxt
//!
//! Examples:
//! - Document-level metadata at the top via annotations
//! - Body mixing paragraphs, sessions, lists, and definitions

use super::super::location::{Location, Position};
use super::annotation::Annotation;
use super::content_item::ContentItem;
use super::foreign::ForeignBlock;
use super::list::List;
use super::paragraph::Paragraph;
use super::session::Session;
use std::fmt;

/// A document represents the root of a txxt AST
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub metadata: Vec<Annotation>,
    pub content: Vec<ContentItem>,
    pub location: Option<Location>,
}

impl Document {
    pub fn new() -> Self {
        Self {
            metadata: Vec::new(),
            content: Vec::new(),
            location: None,
        }
    }

    pub fn with_content(content: Vec<ContentItem>) -> Self {
        Self {
            metadata: Vec::new(),
            content,
            location: None,
        }
    }

    pub fn with_metadata_and_content(metadata: Vec<Annotation>, content: Vec<ContentItem>) -> Self {
        Self {
            metadata,
            content,
            location: None,
        }
    }

    pub fn with_location(mut self, location: Option<Location>) -> Self {
        self.location = location;
        self
    }

    pub fn iter_items(&self) -> impl Iterator<Item = &ContentItem> {
        self.content.iter()
    }

    pub fn iter_paragraphs(&self) -> impl Iterator<Item = &Paragraph> {
        self.content.iter().filter_map(|item| item.as_paragraph())
    }

    pub fn iter_sessions(&self) -> impl Iterator<Item = &Session> {
        self.content.iter().filter_map(|item| item.as_session())
    }

    pub fn iter_lists(&self) -> impl Iterator<Item = &List> {
        self.content.iter().filter_map(|item| item.as_list())
    }

    pub fn iter_foreign_blocks(&self) -> impl Iterator<Item = &ForeignBlock> {
        self.content
            .iter()
            .filter_map(|item| item.as_foreign_block())
    }

    pub fn count_by_type(&self) -> (usize, usize, usize, usize) {
        let paragraphs = self.iter_paragraphs().count();
        let sessions = self.iter_sessions().count();
        let lists = self.iter_lists().count();
        let foreign_blocks = self.iter_foreign_blocks().count();
        (paragraphs, sessions, lists, foreign_blocks)
    }

    /// Find all elements at the given position, returning them in order from deepest to shallowest
    pub fn elements_at(&self, pos: Position) -> Vec<&ContentItem> {
        let mut results = Vec::new();
        for item in &self.content {
            if let Some(mut items) = item.elements_at(pos) {
                results.append(&mut items);
            }
        }
        results
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Document {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Document({} metadata, {} items)",
            self.metadata.len(),
            self.content.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::location::Position;
    use super::super::paragraph::Paragraph;
    use super::super::session::Session;
    use super::*;

    #[test]
    fn test_document_creation() {
        let doc = Document::with_content(vec![
            ContentItem::Paragraph(Paragraph::from_line("Para 1".to_string())),
            ContentItem::Session(Session::with_title("Section 1".to_string())),
        ]);
        assert_eq!(doc.content.len(), 2);
        assert_eq!(doc.metadata.len(), 0);
    }

    #[test]
    fn test_document_elements_at() {
        use crate::txxt::ast::elements::paragraph::TextLine;
        use crate::txxt::ast::text_content::TextContent;

        // Create paragraph 1 with properly located TextLine
        let text_line1 =
            TextLine::new(TextContent::from_string("First".to_string(), None)).with_location(
                Location::new(Position::new(0, 0), Position::new(0, 5)),
            );
        let para1 = Paragraph::new(vec![ContentItem::TextLine(text_line1)]).with_location(
            Location::new(Position::new(0, 0), Position::new(0, 5)),
        );

        // Create paragraph 2 with properly located TextLine
        let text_line2 =
            TextLine::new(TextContent::from_string("Second".to_string(), None)).with_location(
                Location::new(Position::new(1, 0), Position::new(1, 6)),
            );
        let para2 = Paragraph::new(vec![ContentItem::TextLine(text_line2)]).with_location(
            Location::new(Position::new(1, 0), Position::new(1, 6)),
        );

        let doc = Document::with_content(vec![
            ContentItem::Paragraph(para1),
            ContentItem::Paragraph(para2),
        ]);

        let results = doc.elements_at(Position::new(1, 3));
        // We get: TextLine (deepest), Paragraph (shallowest)
        // Document.elements_at returns results from content items, not including Document itself
        assert_eq!(
            results.len(),
            2,
            "Expected 2 results, got: {:?}",
            results.iter().map(|r| r.node_type()).collect::<Vec<_>>()
        );
        assert!(
            results[0].is_text_line(),
            "results[0] is {}",
            results[0].node_type()
        );
        assert!(
            results[1].is_paragraph(),
            "results[1] is {}",
            results[1].node_type()
        );
    }
}
