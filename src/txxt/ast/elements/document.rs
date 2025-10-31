//! Document element
//!
//! A document is the root of a txxt tree. It can contain metadata
//! (as annotations) and a sequence of content elements (paragraphs,
//! sessions, lists, foreign blocks, definitions, annotations).
//!
//! ## Structure
//! - Metadata: zero or more leading annotations that apply to the whole document
//! - Content: ordered list of content items making up the body
//!
//! ## Trait Implementations
//!
//! Document implements `AstNode` and `Container` to enable uniform tree traversal
//! and visualization. However, Document's structure differs from other nodes:
//! - Metadata (annotations) are stored separately from content
//! - The `AstNode::accept()` visitor visits metadata first, then content
//! - Snapshots (via `snapshot_from_document()`) include only content, not metadata
//!
//! **Note:** This partial alignment with other nodes is temporary. Issue #103 Phase 2
//! will further restructure Document by introducing a Session root node, making the
//! structure fully homogeneous with the rest of the AST.
//!
//! Learn More:
//! - Paragraphs: docs/specs/v1/elements/paragraphs.txxt
//! - Lists: docs/specs/v1/elements/lists.txxt
//! - Annotations: docs/specs/v1/elements/annotations.txxt
//! - Definitions: docs/specs/v1/elements/definitions.txxt
//! - Foreign blocks: docs/specs/v1/elements/foreign.txxt
//! - Issue #103: Fix Document node in the AST
//!
//! Examples:
//! - Document-level metadata at the top via annotations
//! - Body mixing paragraphs, sessions, lists, and definitions

use super::super::location::{Location, Position};
use super::super::traits::{AstNode, Container, Visitor};
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

impl AstNode for Document {
    fn node_type(&self) -> &'static str {
        "Document"
    }

    fn display_label(&self) -> String {
        format!(
            "Document ({} metadata, {} items)",
            self.metadata.len(),
            self.content.len()
        )
    }

    fn location(&self) -> Option<Location> {
        self.location
    }

    fn accept(&self, visitor: &mut dyn Visitor) {
        for annotation in &self.metadata {
            annotation.accept(visitor);
        }
        super::super::traits::visit_children(visitor, &self.content);
    }
}

impl Container for Document {
    fn label(&self) -> &str {
        "Document"
    }

    fn children(&self) -> &[ContentItem] {
        &self.content
    }

    fn children_mut(&mut self) -> &mut Vec<ContentItem> {
        &mut self.content
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
            TextLine::new(TextContent::from_string("First".to_string(), None)).with_location(Some(
                Location::new(Position::new(0, 0), Position::new(0, 5)),
            ));
        let para1 = Paragraph::new(vec![ContentItem::TextLine(text_line1)]).with_location(Some(
            Location::new(Position::new(0, 0), Position::new(0, 5)),
        ));

        // Create paragraph 2 with properly located TextLine
        let text_line2 =
            TextLine::new(TextContent::from_string("Second".to_string(), None)).with_location(
                Some(Location::new(Position::new(1, 0), Position::new(1, 6))),
            );
        let para2 = Paragraph::new(vec![ContentItem::TextLine(text_line2)]).with_location(Some(
            Location::new(Position::new(1, 0), Position::new(1, 6)),
        ));

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

    #[test]
    fn test_document_traits() {
        use crate::txxt::ast::traits::{AstNode, Container};

        let doc = Document::with_content(vec![ContentItem::Paragraph(Paragraph::from_line(
            "Line".to_string(),
        ))]);

        assert_eq!(doc.node_type(), "Document");
        assert_eq!(doc.display_label(), "Document (0 metadata, 1 items)");
        assert_eq!(Container::label(&doc), "Document");
        assert_eq!(Container::children(&doc).len(), 1);
    }
}
