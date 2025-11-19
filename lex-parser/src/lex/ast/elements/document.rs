//! Document element
//!
//!     The document node serves two purposes:
//!         - Contains the document tree.
//!         - Contains document-level annotations, including non-content metadata (like file name,
//!           parser version, etc).
//!
//!     Lex documents are plain text, utf-8 encoded files with the file extension .lex. Line width
//!     is not limited, and is considered a presentation detail. Best practice dictates only
//!     limiting line length when publishing, not while authoring.
//!
//!     The document node holds the document metadata and the content's root node, which is a
//!     session node. The structure of the document then is a tree of sessions, which can be nested
//!     arbitrarily. This creates powerful addressing capabilities as one can target any sub-session
//!     from an index.
//!
//!     This structure makes the entire AST homogeneous - the document's content is accessed through
//!     the standard Session interface, making traversal and transformation logic consistent
//!     throughout the tree.
//!
//!     For more details on document structure and sessions, see the [ast](crate::lex::ast) module.
//!
//! Learn More:
//! - Paragraphs: specs/v1/elements/paragraphs.lex
//! - Lists: specs/v1/elements/lists.lex
//! - Sessions: specs/v1/elements/sessions.lex
//! - Annotations: specs/v1/elements/annotations.lex
//! - Definitions: specs/v1/elements/definitions.lex
//! - Verbatim blocks: specs/v1/elements/verbatim.lex
//!
//! Examples:
//! - Document-level metadata via annotations
//! - All body content accessible via document.root.children

use super::super::range::Range;
use super::super::traits::{AstNode, Container, Visitor};
use super::annotation::Annotation;
use super::content_item::ContentItem;
use super::session::Session;
use super::typed_content;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub annotations: Vec<Annotation>,
    // all content is attached to the root node
    pub root: Session,
}

impl Document {
    pub fn new() -> Self {
        Self {
            annotations: Vec::new(),
            root: Session::with_title(String::new()),
        }
    }

    pub fn with_content(content: Vec<ContentItem>) -> Self {
        let mut root = Session::with_title(String::new());
        let session_content = typed_content::into_session_contents(content);
        root.children = super::container::SessionContainer::from_typed(session_content);
        Self {
            annotations: Vec::new(),
            root,
        }
    }

    /// Construct a document from an existing root session.
    pub fn from_root(root: Session) -> Self {
        Self {
            annotations: Vec::new(),
            root,
        }
    }

    pub fn with_annotations_and_content(
        annotations: Vec<Annotation>,
        content: Vec<ContentItem>,
    ) -> Self {
        let mut root = Session::with_title(String::new());
        let session_content = typed_content::into_session_contents(content);
        root.children = super::container::SessionContainer::from_typed(session_content);
        Self { annotations, root }
    }

    pub fn with_root_location(mut self, location: Range) -> Self {
        self.root.location = location;
        self
    }

    pub fn root_session(&self) -> &Session {
        &self.root
    }

    pub fn root_session_mut(&mut self) -> &mut Session {
        &mut self.root
    }

    pub fn into_root(self) -> Session {
        self.root
    }

    /// All annotations attached directly to the document (document-level metadata).
    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    /// Mutable access to document-level annotations.
    pub fn annotations_mut(&mut self) -> &mut Vec<Annotation> {
        &mut self.annotations
    }

    /// Iterate over document-level annotation blocks in source order.
    pub fn iter_annotations(&self) -> std::slice::Iter<'_, Annotation> {
        self.annotations.iter()
    }

    /// Iterate over all content items nested inside document-level annotations.
    pub fn iter_annotation_contents(&self) -> impl Iterator<Item = &ContentItem> {
        self.annotations
            .iter()
            .flat_map(|annotation| annotation.children())
    }
}

impl AstNode for Document {
    fn node_type(&self) -> &'static str {
        "Document"
    }

    fn display_label(&self) -> String {
        format!(
            "Document ({} annotations, {} items)",
            self.annotations.len(),
            self.root.children.len()
        )
    }

    fn range(&self) -> &Range {
        &self.root.location
    }

    fn accept(&self, visitor: &mut dyn Visitor) {
        for annotation in &self.annotations {
            annotation.accept(visitor);
        }
        self.root.accept(visitor);
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
            "Document({} annotations, {} items)",
            self.annotations.len(),
            self.root.children.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::range::Position;
    use super::super::paragraph::{Paragraph, TextLine};
    use super::super::session::Session;
    use super::*;
    use crate::lex::ast::text_content::TextContent;
    use crate::lex::ast::traits::AstNode;

    #[test]
    fn test_document_creation() {
        let doc = Document::with_content(vec![
            ContentItem::Paragraph(Paragraph::from_line("Para 1".to_string())),
            ContentItem::Session(Session::with_title("Section 1".to_string())),
        ]);
        assert_eq!(doc.annotations.len(), 0);
        assert_eq!(doc.root.children.len(), 2);
    }

    #[test]
    fn test_document_element_at() {
        let text_line1 = TextLine::new(TextContent::from_string("First".to_string(), None))
            .at(Range::new(0..0, Position::new(0, 0), Position::new(0, 5)));
        let para1 = Paragraph::new(vec![ContentItem::TextLine(text_line1)]).at(Range::new(
            0..0,
            Position::new(0, 0),
            Position::new(0, 5),
        ));

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

        let result = doc.root.element_at(Position::new(1, 3));
        assert!(result.is_some(), "Expected to find element at position");
        assert!(result.unwrap().is_text_line());
    }

    #[test]
    fn test_document_traits() {
        let doc = Document::with_content(vec![ContentItem::Paragraph(Paragraph::from_line(
            "Line".to_string(),
        ))]);

        assert_eq!(doc.node_type(), "Document");
        assert_eq!(doc.display_label(), "Document (0 annotations, 1 items)");
        assert_eq!(doc.root.children.len(), 1);
    }

    #[test]
    fn test_root_session_accessors() {
        let doc = Document::with_content(vec![ContentItem::Session(Session::with_title(
            "Section".to_string(),
        ))]);

        assert_eq!(doc.root_session().children.len(), 1);

        let mut doc = doc;
        doc.root_session_mut().title = TextContent::from_string("Updated".to_string(), None);
        assert_eq!(doc.root_session().title.as_string(), "Updated");

        let root = doc.into_root();
        assert_eq!(root.title.as_string(), "Updated");
    }
}
