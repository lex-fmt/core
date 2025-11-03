//! Document element
//!
//! The document node serves two purposes:
//! - Contains the document tree.
//! - Contains document-level metadata , including non content related (like file name, parser version, etc)
//!
//! This structure makes the entire AST homogeneous - the document's content
//! is accessed through the standard Session interface, making traversal and
//! transformation logic consistent throughout the tree.
//!
//! Learn More:
//! - Paragraphs: docs/specs/v1/elements/paragraphs.lex
//! - Lists: docs/specs/v1/elements/lists.lex
//! - Sessions: docs/specs/v1/elements/sessions.lex
//! - Annotations: docs/specs/v1/elements/annotations.lex
//! - Definitions: docs/specs/v1/elements/definitions.lex
//! - Foreign blocks: docs/specs/v1/elements/foreign.lex
//!
//! Examples:
//! - Document-level metadata via annotations
//! - All body content accessible via document.root.content

use super::super::range::{Position, Range};
use super::super::traits::{AstNode, Container, Visitor};
use super::annotation::Annotation;
use super::content_item::ContentItem;
use super::foreign::ForeignBlock;
use super::list::List;
use super::paragraph::Paragraph;
use super::session::Session;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub metadata: Vec<Annotation>,
    // all content is attached to the root node
    pub root: Session,
}

impl Document {
    pub fn new() -> Self {
        Self {
            metadata: Vec::new(),
            root: Session::with_title(String::new()),
        }
    }

    pub fn with_content(content: Vec<ContentItem>) -> Self {
        let mut root = Session::with_title(String::new());
        root.content = content;
        Self {
            metadata: Vec::new(),
            root,
        }
    }

    pub fn with_metadata_and_content(metadata: Vec<Annotation>, content: Vec<ContentItem>) -> Self {
        let mut root = Session::with_title(String::new());
        root.content = content;
        Self { metadata, root }
    }

    pub fn with_root_location(mut self, location: Range) -> Self {
        self.root.location = location;
        self
    }

    pub fn iter_items(&self) -> impl Iterator<Item = &ContentItem> {
        self.root.content.iter()
    }

    pub fn iter_paragraphs(&self) -> impl Iterator<Item = &Paragraph> {
        self.root
            .content
            .iter()
            .filter_map(|item| item.as_paragraph())
    }

    pub fn iter_sessions(&self) -> impl Iterator<Item = &Session> {
        self.root
            .content
            .iter()
            .filter_map(|item| item.as_session())
    }

    pub fn iter_lists(&self) -> impl Iterator<Item = &List> {
        self.root.content.iter().filter_map(|item| item.as_list())
    }

    pub fn iter_foreign_blocks(&self) -> impl Iterator<Item = &ForeignBlock> {
        self.root
            .content
            .iter()
            .filter_map(|item| item.as_foreign_block())
    }

    /// Convenience accessor for the root session's location
    pub fn root_location(&self) -> Range {
        self.root.location.clone()
    }

    pub fn count_by_type(&self) -> (usize, usize, usize, usize) {
        let paragraphs = self.iter_paragraphs().count();
        let sessions = self.iter_sessions().count();
        let lists = self.iter_lists().count();
        let foreign_blocks = self.iter_foreign_blocks().count();
        (paragraphs, sessions, lists, foreign_blocks)
    }

    /// Has to be the deepest element, as ancestors are supersets the deepest node location.
    /// Returns the deepest (most nested) element that contains the position
    pub fn element_at(&self, pos: Position) -> Option<&ContentItem> {
        for item in &self.root.content {
            if let Some(result) = item.element_at(pos) {
                return Some(result);
            }
        }
        None
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
            self.root.content.len()
        )
    }

    fn range(&self) -> &Range {
        &self.root.location
    }

    fn accept(&self, visitor: &mut dyn Visitor) {
        for annotation in &self.metadata {
            annotation.accept(visitor);
        }
        self.root.accept(visitor);
    }
}

impl Container for Document {
    fn label(&self) -> &str {
        "Document"
    }

    fn children(&self) -> &[ContentItem] {
        &self.root.content
    }

    fn children_mut(&mut self) -> &mut Vec<ContentItem> {
        &mut self.root.content
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
            self.root.content.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::range::Position;
    use super::super::paragraph::Paragraph;
    use super::super::session::Session;
    use super::*;

    #[test]
    fn test_document_creation() {
        let doc = Document::with_content(vec![
            ContentItem::Paragraph(Paragraph::from_line("Para 1".to_string())),
            ContentItem::Session(Session::with_title("Section 1".to_string())),
        ]);
        assert_eq!(doc.root.content.len(), 2);
        assert_eq!(doc.metadata.len(), 0);
    }

    #[test]
    fn test_document_element_at() {
        use crate::lex::ast::elements::paragraph::TextLine;
        use crate::lex::ast::text_content::TextContent;

        // Create paragraph 1 with properly located TextLine
        let text_line1 = TextLine::new(TextContent::from_string("First".to_string(), None))
            .at(Range::new(0..0, Position::new(0, 0), Position::new(0, 5)));
        let para1 = Paragraph::new(vec![ContentItem::TextLine(text_line1)]).at(Range::new(
            0..0,
            Position::new(0, 0),
            Position::new(0, 5),
        ));

        // Create paragraph 2 with properly located TextLine
        let text_line2 = TextLine::new(TextContent::from_string("Second".to_string(), None))
            .at(Range::new(0..0, Position::new(1, 0), Position::new(1, 6)));
        let para2 = Paragraph::new(vec![ContentItem::TextLine(text_line2)]).at(Range::new(
            0..0,
            Position::new(1, 0),
            Position::new(1, 6),
        ));

        let doc = Document::with_content(vec![
            ContentItem::Paragraph(para1),
            ContentItem::Paragraph(para2),
        ]);

        let result = doc.element_at(Position::new(1, 3));
        // We get the deepest element: TextLine
        assert!(result.is_some(), "Expected to find element at position");
        assert!(
            result.unwrap().is_text_line(),
            "Expected TextLine, got {}",
            result.unwrap().node_type()
        );
    }

    #[test]
    fn test_document_traits() {
        use crate::lex::ast::traits::{AstNode, Container};

        let doc = Document::with_content(vec![ContentItem::Paragraph(Paragraph::from_line(
            "Line".to_string(),
        ))]);

        assert_eq!(doc.node_type(), "Document");
        assert_eq!(doc.display_label(), "Document (0 metadata, 1 items)");
        assert_eq!(Container::label(&doc), "Document");
        assert_eq!(Container::children(&doc).len(), 1);
    }
}
