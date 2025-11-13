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

use super::super::range::Range;
use super::super::traits::{AstNode, Visitor};
use super::annotation::Annotation;
use super::content_item::ContentItem;
use super::session::Session;
use super::typed_content;
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
        let session_content = typed_content::into_session_contents(content);
        root.children = super::container::SessionContainer::from_typed(session_content);
        Self {
            metadata: Vec::new(),
            root,
        }
    }

    pub fn with_metadata_and_content(metadata: Vec<Annotation>, content: Vec<ContentItem>) -> Self {
        let mut root = Session::with_title(String::new());
        let session_content = typed_content::into_session_contents(content);
        root.children = super::container::SessionContainer::from_typed(session_content);
        Self { metadata, root }
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

        let result = doc.root.element_at(Position::new(1, 3));
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
        use crate::lex::ast::traits::AstNode;

        let doc = Document::with_content(vec![ContentItem::Paragraph(Paragraph::from_line(
            "Line".to_string(),
        ))]);

        assert_eq!(doc.node_type(), "Document");
        assert_eq!(doc.display_label(), "Document (0 metadata, 1 items)");
        assert_eq!(doc.root.children.len(), 1);
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
        assert_eq!(doc.root.iter_paragraphs().count(), 1);

        // Recursive iteration should find all 3 paragraphs
        let paragraphs: Vec<_> = doc.root.iter_paragraphs_recursive().collect();
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
        assert_eq!(doc.root.iter_sessions().count(), 1);

        // Recursive iteration finds both sessions
        assert_eq!(doc.root.iter_sessions_recursive().count(), 2);
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

        let nodes_with_depth: Vec<_> = doc.root.iter_all_nodes_with_depth().collect();

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
        let all_paragraphs: Vec<_> = doc.root.iter_paragraphs_recursive().collect();
        assert_eq!(all_paragraphs.len(), 4); // preamble, intro, nested, advanced

        // Example 2: Find all sessions recursively
        let all_sessions: Vec<_> = doc.root.iter_sessions_recursive().collect();
        assert_eq!(all_sessions.len(), 3); // chapter1, section1.1, chapter2

        // Example 3: Filter paragraphs by content using iterator combinators
        let hello_paragraphs: Vec<_> = doc
            .root
            .iter_paragraphs_recursive()
            .filter(|p| p.text().contains("Hello"))
            .collect();
        assert_eq!(hello_paragraphs.len(), 1);

        // Example 4: Find deeply nested sessions (depth >= 1)
        let nested_sessions: Vec<_> = doc
            .root
            .iter_all_nodes_with_depth()
            .filter(|(node, depth)| node.is_session() && *depth >= 1)
            .collect();
        assert_eq!(nested_sessions.len(), 1); // only Section 1.1

        // Example 5: Count nodes at each depth level
        let depth_0_count = doc
            .root
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
        let hello_paras = doc.root.find_paragraphs(|p| p.text().starts_with("Hello"));
        assert_eq!(hello_paras.len(), 2);

        // Find paragraphs containing "Goodbye"
        let goodbye_paras = doc.root.find_paragraphs(|p| p.text().contains("Goodbye"));
        assert_eq!(goodbye_paras.len(), 1);

        // Find paragraphs with more than 12 characters
        let long_paras = doc.root.find_paragraphs(|p| p.text().len() > 12);
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
        let chapters = doc
            .root
            .find_sessions(|s| s.title.as_string().contains("Chapter"));
        assert_eq!(chapters.len(), 2);

        // Find sessions with "Section" in title
        let sections = doc
            .root
            .find_sessions(|s| s.title.as_string().contains("Section"));
        assert_eq!(sections.len(), 1);

        // Find sessions with "Advanced" in title
        let advanced = doc
            .root
            .find_sessions(|s| s.title.as_string().contains("Advanced"));
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
        let big_sessions = doc.root.find_nodes(|node| {
            matches!(node, ContentItem::Session(_))
                && node.children().map(|c| c.len() > 2).unwrap_or(false)
        });
        assert_eq!(big_sessions.len(), 1);

        // Find all paragraphs (using generic predicate)
        let all_paragraphs = doc.root.find_nodes(|node| node.is_paragraph());
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
        let depth_0 = doc.root.find_nodes_at_depth(0);
        assert_eq!(depth_0.len(), 2); // paragraph + session1

        // Find all depth 1 nodes (should have session2 at least)
        let depth_1 = doc.root.find_nodes_at_depth(1);
        assert!(!depth_1.is_empty()); // at minimum session2

        // Find all depth 2 nodes (should have deep paragraph at least)
        let depth_2 = doc.root.find_nodes_at_depth(2);
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

        // Find sessions at depth 0 using generic method + filter
        let level_0: Vec<_> = doc
            .root
            .find_nodes_at_depth(0)
            .into_iter()
            .filter_map(|n| n.as_session())
            .collect();
        assert_eq!(level_0.len(), 1);
        assert!(level_0[0].title.as_string().contains("Level 0"));

        // Find sessions at depth 1
        let level_1: Vec<_> = doc
            .root
            .find_nodes_at_depth(1)
            .into_iter()
            .filter_map(|n| n.as_session())
            .collect();
        assert_eq!(level_1.len(), 1);
        assert!(level_1[0].title.as_string().contains("Level 1"));

        // Find sessions at depth 2
        let level_2: Vec<_> = doc
            .root
            .find_nodes_at_depth(2)
            .into_iter()
            .filter_map(|n| n.as_session())
            .collect();
        assert_eq!(level_2.len(), 1);
        assert!(level_2[0].title.as_string().contains("Level 2"));

        // No sessions at depth 3
        let level_3: Vec<_> = doc
            .root
            .find_nodes_at_depth(3)
            .into_iter()
            .filter_map(|n| n.as_session())
            .collect();
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
        let shallow = doc.root.find_nodes_in_depth_range(0, 1);
        assert!(shallow.len() >= 2);

        // Find nodes in range 1-2
        let mid_range = doc.root.find_nodes_in_depth_range(1, 2);
        assert!(mid_range.len() >= 2);

        // Find nodes in range 0-10 (should get everything)
        let all = doc.root.find_nodes_in_depth_range(0, 10);
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
        let depth_0_hello = doc.root.find_nodes_with_depth(0, |node| {
            node.as_paragraph()
                .map(|p| p.text().contains("Hello"))
                .unwrap_or(false)
        });
        assert_eq!(depth_0_hello.len(), 1);

        // Find paragraphs at depth 1 containing "Hello"
        let depth_1_hello = doc.root.find_nodes_with_depth(1, |node| {
            node.as_paragraph()
                .map(|p| p.text().contains("Hello"))
                .unwrap_or(false)
        });
        assert_eq!(depth_1_hello.len(), 1);

        // Find sessions at depth 0
        let depth_0_sessions = doc.root.find_nodes_with_depth(0, |node| node.is_session());
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

        // Find paragraphs at depth 0 (top level) using generic method + filter
        let top_paras: Vec<_> = doc
            .root
            .find_nodes_at_depth(0)
            .into_iter()
            .filter_map(|n| n.as_paragraph())
            .collect();
        assert_eq!(top_paras.len(), 1);
        assert!(top_paras[0].text().contains("Top para"));

        // Find paragraphs at depth 1 (nested in session)
        let nested_paras: Vec<_> = doc
            .root
            .find_nodes_at_depth(1)
            .into_iter()
            .filter_map(|n| n.as_paragraph())
            .collect();
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
        let advanced_paras = doc
            .root
            .find_paragraphs(|p| p.text().to_lowercase().contains("advanced"));
        assert_eq!(advanced_paras.len(), 1);

        // Example 2: Find chapters (sessions at depth 0 with "Chapter" in title)
        let chapters: Vec<_> = doc
            .root
            .find_nodes_at_depth(0)
            .into_iter()
            .filter_map(|n| n.as_session())
            .collect();
        assert_eq!(chapters.len(), 2);

        // Example 3: Find subsections (sessions at depth 1)
        let subsections: Vec<_> = doc
            .root
            .find_nodes_at_depth(1)
            .into_iter()
            .filter_map(|n| n.as_session())
            .collect();
        assert_eq!(subsections.len(), 2);

        // Example 4: Find all "Introduction" sections
        let intro_sections = doc
            .root
            .find_sessions(|s| s.title.as_string().contains("Introduction"));
        assert_eq!(intro_sections.len(), 1);

        // Example 5: Find paragraphs with greetings
        let greeting_paras = doc.root.find_paragraphs(|p| {
            let text = p.text();
            text.contains("Hello") || text.contains("welcome")
        });
        assert_eq!(greeting_paras.len(), 1);

        // Example 6: Complex query - sessions with at least one child
        let non_empty_sessions = doc.root.find_sessions(|s| !s.children.is_empty());
        assert_eq!(non_empty_sessions.len(), 4); // all sessions have content

        // Example 7: Find nodes in mid-level depth range (1-2)
        let mid_level = doc.root.find_nodes_in_depth_range(1, 2);
        assert!(!mid_level.is_empty());
    }
}
