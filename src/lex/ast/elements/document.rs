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
//! - Verbatim blocks: docs/specs/v1/elements/verbatim.lex
//!
//! Examples:
//! - Document-level metadata via annotations
//! - All body content accessible via document.root.children

use super::super::range::{Position, Range};
use super::super::traits::{AstNode, Container, Visitor};
use super::annotation::Annotation;
use super::content_item::ContentItem;
use super::list::List;
use super::paragraph::Paragraph;
use super::session::Session;
use super::verbatim::Verbatim;
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
        root.children = super::container::SessionContainer::new(content);
        Self {
            metadata: Vec::new(),
            root,
        }
    }

    pub fn with_metadata_and_content(metadata: Vec<Annotation>, content: Vec<ContentItem>) -> Self {
        let mut root = Session::with_title(String::new());
        root.children = super::container::SessionContainer::new(content);
        Self { metadata, root }
    }

    pub fn with_root_location(mut self, location: Range) -> Self {
        self.root.location = location;
        self
    }

    pub fn iter_items(&self) -> impl Iterator<Item = &ContentItem> {
        self.root.children.iter()
    }

    pub fn iter_paragraphs(&self) -> impl Iterator<Item = &Paragraph> {
        self.root
            .children
            .iter()
            .filter_map(|item| item.as_paragraph())
    }

    pub fn iter_sessions(&self) -> impl Iterator<Item = &Session> {
        self.root
            .children
            .iter()
            .filter_map(|item| item.as_session())
    }

    pub fn iter_lists(&self) -> impl Iterator<Item = &List> {
        self.root.children.iter().filter_map(|item| item.as_list())
    }

    pub fn iter_verbatim_blocks(&self) -> impl Iterator<Item = &Verbatim> {
        self.root
            .children
            .iter()
            .filter_map(|item| item.as_verbatim_block())
    }

    /// Recursively iterate all paragraphs at any depth in the document
    pub fn iter_paragraphs_recursive(&self) -> Box<dyn Iterator<Item = &Paragraph> + '_> {
        Box::new(self.iter_all_nodes().filter_map(|item| item.as_paragraph()))
    }

    /// Recursively iterate all sessions at any depth in the document
    pub fn iter_sessions_recursive(&self) -> Box<dyn Iterator<Item = &Session> + '_> {
        Box::new(self.iter_all_nodes().filter_map(|item| item.as_session()))
    }

    /// Recursively iterate all lists at any depth in the document
    pub fn iter_lists_recursive(&self) -> Box<dyn Iterator<Item = &List> + '_> {
        Box::new(self.iter_all_nodes().filter_map(|item| item.as_list()))
    }

    /// Recursively iterate all verbatim blocks at any depth in the document
    pub fn iter_verbatim_blocks_recursive(&self) -> Box<dyn Iterator<Item = &Verbatim> + '_> {
        Box::new(
            self.iter_all_nodes()
                .filter_map(|item| item.as_verbatim_block()),
        )
    }

    /// Recursively iterate all list items at any depth in the document
    pub fn iter_list_items_recursive(&self) -> Box<dyn Iterator<Item = &super::ListItem> + '_> {
        Box::new(self.iter_all_nodes().filter_map(|item| item.as_list_item()))
    }

    /// Recursively iterate all definitions at any depth in the document
    pub fn iter_definitions_recursive(&self) -> Box<dyn Iterator<Item = &super::Definition> + '_> {
        Box::new(
            self.iter_all_nodes()
                .filter_map(|item| item.as_definition()),
        )
    }

    /// Recursively iterate all annotations at any depth in the document
    pub fn iter_annotations_recursive(&self) -> Box<dyn Iterator<Item = &Annotation> + '_> {
        Box::new(
            self.iter_all_nodes()
                .filter_map(|item| item.as_annotation()),
        )
    }

    /// Iterate all nodes in the document tree (depth-first pre-order traversal)
    pub fn iter_all_nodes(&self) -> Box<dyn Iterator<Item = &ContentItem> + '_> {
        Box::new(
            self.root
                .children
                .iter()
                .flat_map(|item| std::iter::once(item).chain(item.descendants())),
        )
    }

    /// Iterate all nodes with their depth (0 = root level children)
    pub fn iter_all_nodes_with_depth(
        &self,
    ) -> Box<dyn Iterator<Item = (&ContentItem, usize)> + '_> {
        Box::new(
            self.root
                .children
                .iter()
                .flat_map(|item| std::iter::once((item, 0)).chain(item.descendants_with_depth(1))),
        )
    }

    // ========== Phase 2: Predicate-based Filtering Methods ==========

    /// Find all paragraphs matching a predicate
    ///
    /// # Example
    /// ```ignore
    /// // Find paragraphs starting with "Hello"
    /// let results = doc.find_paragraphs(|p| p.text().starts_with("Hello"));
    /// ```
    pub fn find_paragraphs<F>(&self, predicate: F) -> Vec<&Paragraph>
    where
        F: Fn(&Paragraph) -> bool,
    {
        self.iter_paragraphs_recursive()
            .filter(|p| predicate(p))
            .collect()
    }

    /// Find all sessions matching a predicate
    ///
    /// # Example
    /// ```ignore
    /// // Find sessions with "Chapter" in the title
    /// let results = doc.find_sessions(|s| s.title.as_string().contains("Chapter"));
    /// ```
    pub fn find_sessions<F>(&self, predicate: F) -> Vec<&Session>
    where
        F: Fn(&Session) -> bool,
    {
        self.iter_sessions_recursive()
            .filter(|s| predicate(s))
            .collect()
    }

    /// Find all lists matching a predicate
    pub fn find_lists<F>(&self, predicate: F) -> Vec<&List>
    where
        F: Fn(&List) -> bool,
    {
        self.iter_lists_recursive()
            .filter(|l| predicate(l))
            .collect()
    }

    /// Find all definitions matching a predicate
    pub fn find_definitions<F>(&self, predicate: F) -> Vec<&super::Definition>
    where
        F: Fn(&super::Definition) -> bool,
    {
        self.iter_definitions_recursive()
            .filter(|d| predicate(d))
            .collect()
    }

    /// Find all annotations matching a predicate
    pub fn find_annotations<F>(&self, predicate: F) -> Vec<&Annotation>
    where
        F: Fn(&Annotation) -> bool,
    {
        self.iter_annotations_recursive()
            .filter(|a| predicate(a))
            .collect()
    }

    /// Find all nodes (of any type) matching a predicate
    ///
    /// # Example
    /// ```ignore
    /// // Find all sessions with more than 5 children
    /// let results = doc.find_nodes(|node| {
    ///     matches!(node, ContentItem::Session(_)) &&
    ///     node.children().map(|c| c.len() > 5).unwrap_or(false)
    /// });
    /// ```
    pub fn find_nodes<F>(&self, predicate: F) -> Vec<&ContentItem>
    where
        F: Fn(&ContentItem) -> bool,
    {
        self.iter_all_nodes().filter(|n| predicate(n)).collect()
    }

    /// Find all nodes at a specific depth (0 = root level children)
    ///
    /// # Example
    /// ```ignore
    /// // Find all top-level items
    /// let top_level = doc.find_nodes_at_depth(0);
    ///
    /// // Find all items 2 levels deep
    /// let deep_items = doc.find_nodes_at_depth(2);
    /// ```
    pub fn find_nodes_at_depth(&self, target_depth: usize) -> Vec<&ContentItem> {
        self.iter_all_nodes_with_depth()
            .filter(|(_, depth)| *depth == target_depth)
            .map(|(node, _)| node)
            .collect()
    }

    /// Find all nodes within a depth range (inclusive)
    ///
    /// # Example
    /// ```ignore
    /// // Find all items between depth 1 and 3
    /// let mid_level = doc.find_nodes_in_depth_range(1, 3);
    /// ```
    pub fn find_nodes_in_depth_range(
        &self,
        min_depth: usize,
        max_depth: usize,
    ) -> Vec<&ContentItem> {
        self.iter_all_nodes_with_depth()
            .filter(|(_, depth)| *depth >= min_depth && *depth <= max_depth)
            .map(|(node, _)| node)
            .collect()
    }

    /// Find all sessions at a specific depth
    pub fn find_sessions_at_depth(&self, target_depth: usize) -> Vec<&Session> {
        self.iter_all_nodes_with_depth()
            .filter(|(node, depth)| *depth == target_depth && node.is_session())
            .filter_map(|(node, _)| node.as_session())
            .collect()
    }

    /// Find all paragraphs at a specific depth
    pub fn find_paragraphs_at_depth(&self, target_depth: usize) -> Vec<&Paragraph> {
        self.iter_all_nodes_with_depth()
            .filter(|(node, depth)| *depth == target_depth && node.is_paragraph())
            .filter_map(|(node, _)| node.as_paragraph())
            .collect()
    }

    /// Find nodes matching both a predicate and a depth constraint
    ///
    /// # Example
    /// ```ignore
    /// // Find sessions at depth 2 with "Advanced" in the title
    /// let results = doc.find_nodes_with_depth(2, |node| {
    ///     node.as_session()
    ///         .map(|s| s.title.as_string().contains("Advanced"))
    ///         .unwrap_or(false)
    /// });
    /// ```
    pub fn find_nodes_with_depth<F>(&self, target_depth: usize, predicate: F) -> Vec<&ContentItem>
    where
        F: Fn(&ContentItem) -> bool,
    {
        self.iter_all_nodes_with_depth()
            .filter(|(node, depth)| *depth == target_depth && predicate(node))
            .map(|(node, _)| node)
            .collect()
    }

    /// Convenience accessor for the root session's location
    pub fn root_location(&self) -> Range {
        self.root.location.clone()
    }

    pub fn count_by_type(&self) -> (usize, usize, usize, usize) {
        let paragraphs = self.iter_paragraphs().count();
        let sessions = self.iter_sessions().count();
        let lists = self.iter_lists().count();
        let verbatim_blocks = self.iter_verbatim_blocks().count();
        (paragraphs, sessions, lists, verbatim_blocks)
    }

    /// Has to be the deepest element, as ancestors are supersets the deepest node location.
    /// Returns the deepest (most nested) element that contains the position
    pub fn element_at(&self, pos: Position) -> Option<&ContentItem> {
        for item in &self.root.children {
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
            self.root.children.len()
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
        &self.root.children
    }

    fn children_mut(&mut self) -> &mut Vec<ContentItem> {
        &mut self.root.children
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
            self.root.children.len()
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
        assert_eq!(doc.root.children.len(), 2);
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

    #[test]
    fn test_iter_paragraphs_recursive() {
        // Create a nested structure:
        // Document
        //   - Paragraph("Top level")
        //   - Session
        //     - Paragraph("Nested 1")
        //     - Session
        //       - Paragraph("Nested 2")
        let mut inner_session = Session::with_title("Inner".to_string());
        inner_session
            .children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Nested 2".to_string(),
            )));

        let mut outer_session = Session::with_title("Outer".to_string());
        outer_session
            .children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Nested 1".to_string(),
            )));
        outer_session
            .children
            .push(ContentItem::Session(inner_session));

        let doc = Document::with_content(vec![
            ContentItem::Paragraph(Paragraph::from_line("Top level".to_string())),
            ContentItem::Session(outer_session),
        ]);

        // Direct iteration should only find top-level paragraph
        assert_eq!(doc.iter_paragraphs().count(), 1);

        // Recursive iteration should find all 3 paragraphs
        let paragraphs: Vec<_> = doc.iter_paragraphs_recursive().collect();
        assert_eq!(paragraphs.len(), 3);
    }

    #[test]
    fn test_iter_sessions_recursive() {
        let inner_session = Session::with_title("Inner".to_string());
        let mut outer_session = Session::with_title("Outer".to_string());
        outer_session
            .children
            .push(ContentItem::Session(inner_session));

        let doc = Document::with_content(vec![ContentItem::Session(outer_session)]);

        // Direct iteration finds 1 session
        assert_eq!(doc.iter_sessions().count(), 1);

        // Recursive iteration finds both sessions
        assert_eq!(doc.iter_sessions_recursive().count(), 2);
    }

    #[test]
    fn test_iter_all_nodes_with_depth() {
        // Create nested structure to test depth tracking
        let mut inner_session = Session::with_title("Inner".to_string());
        inner_session
            .children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Deep".to_string(),
            )));

        let mut outer_session = Session::with_title("Outer".to_string());
        outer_session
            .children
            .push(ContentItem::Session(inner_session));

        let doc = Document::with_content(vec![
            ContentItem::Paragraph(Paragraph::from_line("Top".to_string())),
            ContentItem::Session(outer_session),
        ]);

        let nodes_with_depth: Vec<_> = doc.iter_all_nodes_with_depth().collect();

        // Should have: paragraph(0), paragraph's TextLine(1), outer_session(0),
        // inner_session(1), deep_paragraph(2), deep_paragraph's TextLine(3)
        assert_eq!(nodes_with_depth.len(), 6);

        // Check depths are correct for key nodes
        assert_eq!(nodes_with_depth[0].1, 0); // Top paragraph
        assert!(nodes_with_depth[0].0.is_paragraph());
        assert_eq!(nodes_with_depth[1].1, 1); // Top paragraph's TextLine
        assert!(nodes_with_depth[1].0.is_text_line());
        assert_eq!(nodes_with_depth[2].1, 0); // Outer session
        assert!(nodes_with_depth[2].0.is_session());
        assert_eq!(nodes_with_depth[3].1, 1); // Inner session
        assert!(nodes_with_depth[3].0.is_session());
    }

    #[test]
    fn test_query_api_example() {
        // Comprehensive example showing the new query APIs in action
        // Build a realistic nested document structure
        let mut chapter1 = Session::with_title("Chapter 1: Introduction".to_string());
        chapter1
            .children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Hello, this is the intro.".to_string(),
            )));

        let mut section1_1 = Session::with_title("Section 1.1".to_string());
        section1_1
            .children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Nested content here.".to_string(),
            )));
        chapter1.children.push(ContentItem::Session(section1_1));

        let mut chapter2 = Session::with_title("Chapter 2: Advanced".to_string());
        chapter2
            .children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Advanced topics.".to_string(),
            )));

        let doc = Document::with_content(vec![
            ContentItem::Paragraph(Paragraph::from_line("Document preamble.".to_string())),
            ContentItem::Session(chapter1),
            ContentItem::Session(chapter2),
        ]);

        // Example 1: Find all paragraphs recursively (previously required custom visitor)
        let all_paragraphs: Vec<_> = doc.iter_paragraphs_recursive().collect();
        assert_eq!(all_paragraphs.len(), 4); // preamble, intro, nested, advanced

        // Example 2: Find all sessions recursively
        let all_sessions: Vec<_> = doc.iter_sessions_recursive().collect();
        assert_eq!(all_sessions.len(), 3); // chapter1, section1.1, chapter2

        // Example 3: Filter paragraphs by content using iterator combinators
        let hello_paragraphs: Vec<_> = doc
            .iter_paragraphs_recursive()
            .filter(|p| p.text().contains("Hello"))
            .collect();
        assert_eq!(hello_paragraphs.len(), 1);

        // Example 4: Find deeply nested sessions (depth >= 1)
        let nested_sessions: Vec<_> = doc
            .iter_all_nodes_with_depth()
            .filter(|(node, depth)| node.is_session() && *depth >= 1)
            .collect();
        assert_eq!(nested_sessions.len(), 1); // only Section 1.1

        // Example 5: Count nodes at each depth level
        let depth_0_count = doc
            .iter_all_nodes_with_depth()
            .filter(|(_, depth)| *depth == 0)
            .count();
        assert!(depth_0_count > 0);
    }

    #[test]
    fn test_descendants_on_content_item() {
        // Create a session with nested content
        let mut session = Session::with_title("Test".to_string());
        session
            .children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Child".to_string(),
            )));

        let mut inner_session = Session::with_title("Inner".to_string());
        inner_session
            .children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Grandchild".to_string(),
            )));
        session.children.push(ContentItem::Session(inner_session));

        let item = ContentItem::Session(session);

        // Descendants should include all nested items (including TextLines within paragraphs)
        // child_paragraph, child_textline, inner_session, grandchild_paragraph, grandchild_textline
        let descendants: Vec<_> = item.descendants().collect();
        assert_eq!(descendants.len(), 5);

        // Verify we can filter to just paragraphs
        let paragraphs: Vec<_> = item.descendants().filter(|d| d.is_paragraph()).collect();
        assert_eq!(paragraphs.len(), 2);
    }

    // ========== Phase 2: Filtering Method Tests ==========

    #[test]
    fn test_find_paragraphs_with_predicate() {
        let doc = Document::with_content(vec![
            ContentItem::Paragraph(Paragraph::from_line("Hello, world!".to_string())),
            ContentItem::Paragraph(Paragraph::from_line("Goodbye, world!".to_string())),
            ContentItem::Paragraph(Paragraph::from_line("Hello again!".to_string())),
        ]);

        // Find paragraphs starting with "Hello"
        let hello_paras = doc.find_paragraphs(|p| p.text().starts_with("Hello"));
        assert_eq!(hello_paras.len(), 2);

        // Find paragraphs containing "Goodbye"
        let goodbye_paras = doc.find_paragraphs(|p| p.text().contains("Goodbye"));
        assert_eq!(goodbye_paras.len(), 1);

        // Find paragraphs with more than 12 characters
        let long_paras = doc.find_paragraphs(|p| p.text().len() > 12);
        assert_eq!(long_paras.len(), 2); // "Hello, world!" (13) and "Goodbye, world!" (15)
    }

    #[test]
    fn test_find_sessions_with_predicate() {
        let mut session1 = Session::with_title("Chapter 1: Introduction".to_string());
        let session2 = Session::with_title("Chapter 2: Advanced".to_string());
        let section = Session::with_title("Section 1.1".to_string());

        session1.children.push(ContentItem::Session(section));

        let doc = Document::with_content(vec![
            ContentItem::Session(session1),
            ContentItem::Session(session2),
        ]);

        // Find sessions with "Chapter" in title
        let chapters = doc.find_sessions(|s| s.title.as_string().contains("Chapter"));
        assert_eq!(chapters.len(), 2);

        // Find sessions with "Section" in title
        let sections = doc.find_sessions(|s| s.title.as_string().contains("Section"));
        assert_eq!(sections.len(), 1);

        // Find sessions with "Advanced" in title
        let advanced = doc.find_sessions(|s| s.title.as_string().contains("Advanced"));
        assert_eq!(advanced.len(), 1);
    }

    #[test]
    fn test_find_nodes_generic_predicate() {
        let mut session = Session::with_title("Test".to_string());
        session
            .children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Child 1".to_string(),
            )));
        session
            .children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Child 2".to_string(),
            )));
        session
            .children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Child 3".to_string(),
            )));

        let doc = Document::with_content(vec![
            ContentItem::Paragraph(Paragraph::from_line("Top".to_string())),
            ContentItem::Session(session),
        ]);

        // Find sessions with more than 2 children
        let big_sessions = doc.find_nodes(|node| {
            matches!(node, ContentItem::Session(_))
                && node.children().map(|c| c.len() > 2).unwrap_or(false)
        });
        assert_eq!(big_sessions.len(), 1);

        // Find all paragraphs (using generic predicate)
        let all_paragraphs = doc.find_nodes(|node| node.is_paragraph());
        assert_eq!(all_paragraphs.len(), 4); // top + 3 children
    }

    #[test]
    fn test_find_nodes_at_depth() {
        // Build nested structure:
        // depth 0: paragraph, session1
        // depth 1: session2 (inside session1)
        // depth 2: paragraph (inside session2)
        let mut session2 = Session::with_title("Inner".to_string());
        session2
            .children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Deep".to_string(),
            )));

        let mut session1 = Session::with_title("Outer".to_string());
        session1.children.push(ContentItem::Session(session2));

        let doc = Document::with_content(vec![
            ContentItem::Paragraph(Paragraph::from_line("Top".to_string())),
            ContentItem::Session(session1),
        ]);

        // Find all depth 0 nodes
        let depth_0 = doc.find_nodes_at_depth(0);
        assert_eq!(depth_0.len(), 2); // paragraph + session1

        // Find all depth 1 nodes (should have session2 at least)
        let depth_1 = doc.find_nodes_at_depth(1);
        assert!(!depth_1.is_empty()); // at minimum session2

        // Find all depth 2 nodes (should have deep paragraph at least)
        let depth_2 = doc.find_nodes_at_depth(2);
        assert!(!depth_2.is_empty()); // at minimum deep paragraph
    }

    #[test]
    fn test_find_sessions_at_depth() {
        let session3 = Session::with_title("Level 2".to_string());
        let mut session2 = Session::with_title("Level 1".to_string());
        session2.children.push(ContentItem::Session(session3));
        let mut session1 = Session::with_title("Level 0".to_string());
        session1.children.push(ContentItem::Session(session2));

        let doc = Document::with_content(vec![ContentItem::Session(session1)]);

        // Find sessions at depth 0
        let level_0 = doc.find_sessions_at_depth(0);
        assert_eq!(level_0.len(), 1);
        assert!(level_0[0].title.as_string().contains("Level 0"));

        // Find sessions at depth 1
        let level_1 = doc.find_sessions_at_depth(1);
        assert_eq!(level_1.len(), 1);
        assert!(level_1[0].title.as_string().contains("Level 1"));

        // Find sessions at depth 2
        let level_2 = doc.find_sessions_at_depth(2);
        assert_eq!(level_2.len(), 1);
        assert!(level_2[0].title.as_string().contains("Level 2"));

        // No sessions at depth 3
        let level_3 = doc.find_sessions_at_depth(3);
        assert_eq!(level_3.len(), 0);
    }

    #[test]
    fn test_find_nodes_in_depth_range() {
        let mut deep_session = Session::with_title("Deep".to_string());
        deep_session
            .children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Very deep".to_string(),
            )));

        let mut mid_session = Session::with_title("Mid".to_string());
        mid_session
            .children
            .push(ContentItem::Session(deep_session));

        let mut top_session = Session::with_title("Top".to_string());
        top_session.children.push(ContentItem::Session(mid_session));

        let doc = Document::with_content(vec![
            ContentItem::Paragraph(Paragraph::from_line("Root".to_string())),
            ContentItem::Session(top_session),
        ]);

        // Find nodes in range 0-1
        let shallow = doc.find_nodes_in_depth_range(0, 1);
        assert!(shallow.len() >= 2);

        // Find nodes in range 1-2
        let mid_range = doc.find_nodes_in_depth_range(1, 2);
        assert!(mid_range.len() >= 2);

        // Find nodes in range 0-10 (should get everything)
        let all = doc.find_nodes_in_depth_range(0, 10);
        assert!(!all.is_empty());
    }

    #[test]
    fn test_find_nodes_with_depth_and_predicate() {
        let mut session = Session::with_title("Test Session".to_string());
        session
            .children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Hello from nested".to_string(),
            )));

        let doc = Document::with_content(vec![
            ContentItem::Paragraph(Paragraph::from_line("Hello from top".to_string())),
            ContentItem::Session(session),
        ]);

        // Find paragraphs at depth 0 containing "Hello"
        let depth_0_hello = doc.find_nodes_with_depth(0, |node| {
            node.as_paragraph()
                .map(|p| p.text().contains("Hello"))
                .unwrap_or(false)
        });
        assert_eq!(depth_0_hello.len(), 1);

        // Find paragraphs at depth 1 containing "Hello"
        let depth_1_hello = doc.find_nodes_with_depth(1, |node| {
            node.as_paragraph()
                .map(|p| p.text().contains("Hello"))
                .unwrap_or(false)
        });
        assert_eq!(depth_1_hello.len(), 1);

        // Find sessions at depth 0
        let depth_0_sessions = doc.find_nodes_with_depth(0, |node| node.is_session());
        assert_eq!(depth_0_sessions.len(), 1);
    }

    #[test]
    fn test_find_paragraphs_at_depth() {
        let mut session = Session::with_title("Section".to_string());
        session
            .children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Nested para".to_string(),
            )));

        let doc = Document::with_content(vec![
            ContentItem::Paragraph(Paragraph::from_line("Top para".to_string())),
            ContentItem::Session(session),
        ]);

        // Find paragraphs at depth 0 (top level)
        let top_paras = doc.find_paragraphs_at_depth(0);
        assert_eq!(top_paras.len(), 1);
        assert!(top_paras[0].text().contains("Top para"));

        // Find paragraphs at depth 1 (nested in session)
        let nested_paras = doc.find_paragraphs_at_depth(1);
        assert_eq!(nested_paras.len(), 1);
        assert!(nested_paras[0].text().contains("Nested para"));
    }

    #[test]
    fn test_phase_2_comprehensive_example() {
        // Build a realistic document to showcase all Phase 2 features
        let mut intro_section = Session::with_title("Introduction".to_string());
        intro_section
            .children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Hello, welcome to the guide.".to_string(),
            )));

        let mut advanced_subsection = Session::with_title("Advanced Topics".to_string());
        advanced_subsection
            .children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "This section covers advanced material.".to_string(),
            )));

        let mut chapter1 = Session::with_title("Chapter 1: Basics".to_string());
        chapter1.children.push(ContentItem::Session(intro_section));

        let mut chapter2 = Session::with_title("Chapter 2: Advanced".to_string());
        chapter2
            .children
            .push(ContentItem::Session(advanced_subsection));

        let doc = Document::with_content(vec![
            ContentItem::Paragraph(Paragraph::from_line("Document preamble.".to_string())),
            ContentItem::Session(chapter1),
            ContentItem::Session(chapter2),
        ]);

        // Example 1: Find paragraphs containing "advanced" (case-insensitive)
        let advanced_paras = doc.find_paragraphs(|p| p.text().to_lowercase().contains("advanced"));
        assert_eq!(advanced_paras.len(), 1);

        // Example 2: Find chapters (sessions at depth 0 with "Chapter" in title)
        let chapters = doc.find_sessions_at_depth(0);
        assert_eq!(chapters.len(), 2);

        // Example 3: Find subsections (sessions at depth 1)
        let subsections = doc.find_sessions_at_depth(1);
        assert_eq!(subsections.len(), 2);

        // Example 4: Find all "Introduction" sections
        let intro_sections = doc.find_sessions(|s| s.title.as_string().contains("Introduction"));
        assert_eq!(intro_sections.len(), 1);

        // Example 5: Find paragraphs with greetings
        let greeting_paras = doc.find_paragraphs(|p| {
            let text = p.text();
            text.contains("Hello") || text.contains("welcome")
        });
        assert_eq!(greeting_paras.len(), 1);

        // Example 6: Complex query - sessions with at least one child
        let non_empty_sessions = doc.find_sessions(|s| !s.children.is_empty());
        assert_eq!(non_empty_sessions.len(), 4); // all sessions have content

        // Example 7: Find nodes in mid-level depth range (1-2)
        let mid_level = doc.find_nodes_in_depth_range(1, 2);
        assert!(!mid_level.is_empty());
    }
}
