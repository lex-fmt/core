//! Session element
//!
//! A session is the main structural element of lex documents. Sessions can be arbitrarily nested
//! and contain required titles and content.
//!
//! Sessions establish hierarchy within a document via their title e and nested content, like all
//! major elements in lex.
//!
//! Structure:
//! - Title: short text identifying the session
//! - Content: any elements allowed in the body
//!
//! The title can be any text content, and is often decorated with an ordering indicator, just like lists,
//! and in lex all the numerical, alphabetical, and roman numeral indicators are supported.
//!
//! Examples:
//!
//! Welcome to The Lex format
//!
//!     Lex is a plain text document format. ...
//!
//! 1.4 The Finale
//!
//!     Here is where we stop.
//!
use super::super::range::{Position, Range};
use super::super::text_content::TextContent;
use super::super::traits::{AstNode, Container, Visitor};
use super::annotation::Annotation;
use super::container::SessionContainer;
use super::content_item::ContentItem;
use super::definition::Definition;
use super::list::{List, ListItem};
use super::paragraph::Paragraph;
use super::typed_content::SessionContent;
use super::verbatim::Verbatim;
use std::fmt;

// ============================================================================
// MACROS FOR GENERATING REPETITIVE ITERATOR/FINDER METHODS
// ============================================================================

/// Macro to generate recursive iterator methods for different AST node types
macro_rules! impl_recursive_iterator {
    ($method_name:ident, $type:ty, $as_method:ident, $doc:expr) => {
        #[doc = $doc]
        pub fn $method_name(&self) -> Box<dyn Iterator<Item = &$type> + '_> {
            Box::new(self.iter_all_nodes().filter_map(|item| item.$as_method()))
        }
    };
}

/// Macro to generate "first" convenience methods
macro_rules! impl_first_method {
    ($method_name:ident, $type:ty, $iter_method:ident, $doc:expr) => {
        #[doc = $doc]
        pub fn $method_name(&self) -> Option<&$type> {
            self.$iter_method().next()
        }
    };
}

/// Macro to generate predicate-based finder methods
macro_rules! impl_find_method {
    ($method_name:ident, $type:ty, $iter_method:ident, $doc:expr) => {
        #[doc = $doc]
        pub fn $method_name<F>(&self, predicate: F) -> Vec<&$type>
        where
            F: Fn(&$type) -> bool,
        {
            self.$iter_method().filter(|x| predicate(x)).collect()
        }
    };
}

/// A session represents a hierarchical container with a title
#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    pub title: TextContent,
    pub children: SessionContainer,
    pub annotations: Vec<Annotation>,
    pub location: Range,
}

impl Session {
    fn default_location() -> Range {
        Range::new(0..0, Position::new(0, 0), Position::new(0, 0))
    }
    pub fn new(title: TextContent, children: Vec<SessionContent>) -> Self {
        Self {
            title,
            children: SessionContainer::from_typed(children),
            annotations: Vec::new(),
            location: Self::default_location(),
        }
    }
    pub fn with_title(title: String) -> Self {
        Self {
            title: TextContent::from_string(title, None),
            children: SessionContainer::empty(),
            annotations: Vec::new(),
            location: Self::default_location(),
        }
    }

    /// Preferred builder
    pub fn at(mut self, location: Range) -> Self {
        self.location = location;
        self
    }

    /// Annotations attached to this session header/content block.
    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    /// Range covering only the session title line, if available.
    pub fn header_location(&self) -> Option<&Range> {
        self.title.location.as_ref()
    }

    /// Bounding range covering only the session's children.
    pub fn body_location(&self) -> Option<Range> {
        Range::bounding_box(self.children.iter().map(|item| item.range()))
    }

    /// Mutable access to session annotations.
    pub fn annotations_mut(&mut self) -> &mut Vec<Annotation> {
        &mut self.annotations
    }

    /// Iterate over annotation blocks in source order.
    pub fn iter_annotations(&self) -> std::slice::Iter<'_, Annotation> {
        self.annotations.iter()
    }

    /// Iterate over all content items nested inside attached annotations.
    pub fn iter_annotation_contents(&self) -> impl Iterator<Item = &ContentItem> {
        self.annotations
            .iter()
            .flat_map(|annotation| annotation.children())
    }

    pub fn iter_items(&self) -> impl Iterator<Item = &ContentItem> {
        self.children.iter()
    }

    pub fn iter_paragraphs(&self) -> impl Iterator<Item = &Paragraph> {
        self.children.iter().filter_map(|item| item.as_paragraph())
    }

    pub fn iter_sessions(&self) -> impl Iterator<Item = &Session> {
        self.children.iter().filter_map(|item| item.as_session())
    }

    pub fn iter_lists(&self) -> impl Iterator<Item = &List> {
        self.children.iter().filter_map(|item| item.as_list())
    }

    pub fn iter_verbatim_blocks(&self) -> impl Iterator<Item = &Verbatim> {
        self.children
            .iter()
            .filter_map(|item| item.as_verbatim_block())
    }

    /// Iterate all nodes in the session tree (depth-first pre-order traversal)
    pub fn iter_all_nodes(&self) -> Box<dyn Iterator<Item = &ContentItem> + '_> {
        Box::new(
            self.children
                .iter()
                .flat_map(|item| std::iter::once(item).chain(item.descendants())),
        )
    }

    /// Iterate all nodes with their depth (0 = immediate children)
    pub fn iter_all_nodes_with_depth(
        &self,
    ) -> Box<dyn Iterator<Item = (&ContentItem, usize)> + '_> {
        Box::new(
            self.children
                .iter()
                .flat_map(|item| std::iter::once((item, 0)).chain(item.descendants_with_depth(1))),
        )
    }

    impl_recursive_iterator!(
        iter_paragraphs_recursive,
        Paragraph,
        as_paragraph,
        "Recursively iterate all paragraphs at any depth in the session"
    );
    impl_recursive_iterator!(
        iter_sessions_recursive,
        Session,
        as_session,
        "Recursively iterate all sessions at any depth in the session"
    );
    impl_recursive_iterator!(
        iter_lists_recursive,
        List,
        as_list,
        "Recursively iterate all lists at any depth in the session"
    );
    impl_recursive_iterator!(
        iter_verbatim_blocks_recursive,
        Verbatim,
        as_verbatim_block,
        "Recursively iterate all verbatim blocks at any depth in the session"
    );
    impl_recursive_iterator!(
        iter_list_items_recursive,
        ListItem,
        as_list_item,
        "Recursively iterate all list items at any depth in the session"
    );
    impl_recursive_iterator!(
        iter_definitions_recursive,
        Definition,
        as_definition,
        "Recursively iterate all definitions at any depth in the session"
    );
    impl_recursive_iterator!(
        iter_annotations_recursive,
        Annotation,
        as_annotation,
        "Recursively iterate all annotations at any depth in the session"
    );

    impl_first_method!(
        first_paragraph,
        Paragraph,
        iter_paragraphs_recursive,
        "Get the first paragraph in the session (returns None if not found)"
    );
    impl_first_method!(
        first_session,
        Session,
        iter_sessions_recursive,
        "Get the first session in the session tree (returns None if not found)"
    );
    impl_first_method!(
        first_list,
        List,
        iter_lists_recursive,
        "Get the first list in the session (returns None if not found)"
    );
    impl_first_method!(
        first_definition,
        Definition,
        iter_definitions_recursive,
        "Get the first definition in the session (returns None if not found)"
    );
    impl_first_method!(
        first_annotation,
        Annotation,
        iter_annotations_recursive,
        "Get the first annotation in the session (returns None if not found)"
    );
    impl_first_method!(
        first_verbatim,
        Verbatim,
        iter_verbatim_blocks_recursive,
        "Get the first verbatim block in the session (returns None if not found)"
    );

    pub fn expect_paragraph(&self) -> &Paragraph {
        self.first_paragraph()
            .expect("No paragraph found in session")
    }

    pub fn expect_session(&self) -> &Session {
        self.first_session()
            .expect("No session found in session tree")
    }

    pub fn expect_list(&self) -> &List {
        self.first_list().expect("No list found in session")
    }

    pub fn expect_definition(&self) -> &Definition {
        self.first_definition()
            .expect("No definition found in session")
    }

    pub fn expect_annotation(&self) -> &Annotation {
        self.first_annotation()
            .expect("No annotation found in session")
    }

    pub fn expect_verbatim(&self) -> &Verbatim {
        self.first_verbatim()
            .expect("No verbatim block found in session")
    }

    impl_find_method!(
        find_paragraphs,
        Paragraph,
        iter_paragraphs_recursive,
        "Find all paragraphs matching a predicate"
    );
    impl_find_method!(
        find_sessions,
        Session,
        iter_sessions_recursive,
        "Find all sessions matching a predicate"
    );
    impl_find_method!(
        find_lists,
        List,
        iter_lists_recursive,
        "Find all lists matching a predicate"
    );
    impl_find_method!(
        find_definitions,
        Definition,
        iter_definitions_recursive,
        "Find all definitions matching a predicate"
    );
    impl_find_method!(
        find_annotations,
        Annotation,
        iter_annotations_recursive,
        "Find all annotations matching a predicate"
    );

    pub fn find_nodes<F>(&self, predicate: F) -> Vec<&ContentItem>
    where
        F: Fn(&ContentItem) -> bool,
    {
        self.iter_all_nodes().filter(|n| predicate(n)).collect()
    }

    pub fn find_nodes_at_depth(&self, target_depth: usize) -> Vec<&ContentItem> {
        self.iter_all_nodes_with_depth()
            .filter(|(_, depth)| *depth == target_depth)
            .map(|(node, _)| node)
            .collect()
    }

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

    pub fn find_nodes_with_depth<F>(&self, target_depth: usize, predicate: F) -> Vec<&ContentItem>
    where
        F: Fn(&ContentItem) -> bool,
    {
        self.iter_all_nodes_with_depth()
            .filter(|(node, depth)| *depth == target_depth && predicate(node))
            .map(|(node, _)| node)
            .collect()
    }

    pub fn count_by_type(&self) -> (usize, usize, usize, usize) {
        let paragraphs = self.iter_paragraphs().count();
        let sessions = self.iter_sessions().count();
        let lists = self.iter_lists().count();
        let verbatim_blocks = self.iter_verbatim_blocks().count();
        (paragraphs, sessions, lists, verbatim_blocks)
    }

    /// Returns the deepest (most nested) element that contains the position
    pub fn element_at(&self, pos: Position) -> Option<&ContentItem> {
        for item in &self.children {
            if let Some(result) = item.element_at(pos) {
                return Some(result);
            }
        }
        None
    }

    /// Returns the deepest AST node at the given position, if any.
    pub fn find_nodes_at_position(&self, position: Position) -> Vec<&dyn AstNode> {
        if let Some(item) = self.element_at(position) {
            vec![item as &dyn AstNode]
        } else {
            Vec::new()
        }
    }

    /// Formats information about nodes located at a given position.
    pub fn format_at_position(&self, position: Position) -> String {
        let nodes = self.find_nodes_at_position(position);
        if nodes.is_empty() {
            "No AST nodes at this position".to_string()
        } else {
            nodes
                .iter()
                .map(|node| format!("- {}: {}", node.node_type(), node.display_label()))
                .collect::<Vec<_>>()
                .join("\n")
        }
    }
}

impl AstNode for Session {
    fn node_type(&self) -> &'static str {
        "Session"
    }
    fn display_label(&self) -> String {
        self.title.as_string().to_string()
    }
    fn range(&self) -> &Range {
        &self.location
    }

    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_session(self);
        super::super::traits::visit_children(visitor, &self.children);
    }
}

impl Container for Session {
    fn label(&self) -> &str {
        self.title.as_string()
    }
    fn children(&self) -> &[ContentItem] {
        &self.children
    }
    fn children_mut(&mut self) -> &mut Vec<ContentItem> {
        &mut self.children
    }
}

impl fmt::Display for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Session('{}', {} items)",
            self.title.as_string(),
            self.children.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::paragraph::Paragraph;
    use super::*;

    #[test]
    fn test_session_creation() {
        let mut session = Session::with_title("Introduction".to_string());
        session
            .children_mut()
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Content".to_string(),
            )));
        assert_eq!(session.label(), "Introduction");
        assert_eq!(session.children.len(), 1);
    }

    #[test]
    fn test_session() {
        let location = super::super::super::range::Range::new(
            0..0,
            super::super::super::range::Position::new(1, 0),
            super::super::super::range::Position::new(1, 10),
        );
        let session = Session::with_title("Title".to_string()).at(location.clone());
        assert_eq!(session.location, location);
    }

    #[test]
    fn test_iter_paragraphs_recursive() {
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

        let mut root = Session::with_title("Root".to_string());
        root.children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Top".to_string(),
            )));
        root.children.push(ContentItem::Session(outer_session));

        assert_eq!(root.iter_paragraphs().count(), 1);
        let paragraphs: Vec<_> = root.iter_paragraphs_recursive().collect();
        assert_eq!(paragraphs.len(), 3);
    }

    #[test]
    fn test_iter_sessions_recursive() {
        let inner_session = Session::with_title("Inner".to_string());
        let mut outer_session = Session::with_title("Outer".to_string());
        outer_session
            .children
            .push(ContentItem::Session(inner_session));

        let mut root = Session::with_title("Root".to_string());
        root.children.push(ContentItem::Session(outer_session));

        assert_eq!(root.iter_sessions().count(), 1);
        assert_eq!(root.iter_sessions_recursive().count(), 2);
    }

    #[test]
    fn test_iter_all_nodes_with_depth() {
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

        let mut root = Session::with_title("Root".to_string());
        root.children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Top".to_string(),
            )));
        root.children.push(ContentItem::Session(outer_session));

        let nodes_with_depth: Vec<_> = root.iter_all_nodes_with_depth().collect();
        assert_eq!(nodes_with_depth.len(), 6);
        assert_eq!(nodes_with_depth[0].1, 0);
        assert!(nodes_with_depth[0].0.is_paragraph());
        assert_eq!(nodes_with_depth[1].1, 1);
        assert!(nodes_with_depth[1].0.is_text_line());
    }

    #[test]
    fn test_query_api_example_on_session() {
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

        let mut root = Session::with_title("Root".to_string());
        root.children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Preamble".to_string(),
            )));
        root.children.push(ContentItem::Session(chapter1));
        root.children.push(ContentItem::Session(chapter2));

        assert_eq!(root.iter_paragraphs_recursive().count(), 4);
        assert_eq!(root.iter_sessions_recursive().count(), 3);

        let hello_paragraphs: Vec<_> = root
            .iter_paragraphs_recursive()
            .filter(|p| p.text().contains("Hello"))
            .collect();
        assert_eq!(hello_paragraphs.len(), 1);

        let nested_sessions: Vec<_> = root
            .iter_all_nodes_with_depth()
            .filter(|(node, depth)| node.is_session() && *depth >= 1)
            .collect();
        assert_eq!(nested_sessions.len(), 1);
    }

    #[test]
    fn test_find_paragraphs_with_predicate() {
        let mut root = Session::with_title("Root".to_string());
        root.children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Hello, world!".to_string(),
            )));
        root.children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Goodbye, world!".to_string(),
            )));
        root.children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Hello again!".to_string(),
            )));

        let hello_paras = root.find_paragraphs(|p| p.text().starts_with("Hello"));
        assert_eq!(hello_paras.len(), 2);

        let goodbye_paras = root.find_paragraphs(|p| p.text().contains("Goodbye"));
        assert_eq!(goodbye_paras.len(), 1);
    }

    #[test]
    fn test_find_sessions_with_predicate() {
        let mut session1 = Session::with_title("Chapter 1: Introduction".to_string());
        session1
            .children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Intro".to_string(),
            )));
        let session2 = Session::with_title("Chapter 2: Advanced".to_string());
        let section = Session::with_title("Section 1.1".to_string());
        session1.children.push(ContentItem::Session(section));

        let mut root = Session::with_title("Root".to_string());
        root.children.push(ContentItem::Session(session1));
        root.children.push(ContentItem::Session(session2));

        let chapters = root.find_sessions(|s| s.title.as_string().contains("Chapter"));
        assert_eq!(chapters.len(), 2);
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

        let mut root = Session::with_title("Root".to_string());
        root.children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Top".to_string(),
            )));
        root.children.push(ContentItem::Session(session));

        let big_sessions = root.find_nodes(|node| {
            matches!(node, ContentItem::Session(_))
                && node.children().map(|c| c.len() > 2).unwrap_or(false)
        });
        assert_eq!(big_sessions.len(), 1);
    }

    #[test]
    fn test_find_nodes_at_depth() {
        let mut inner = Session::with_title("Inner".to_string());
        inner
            .children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Deep".to_string(),
            )));
        let mut outer = Session::with_title("Outer".to_string());
        outer.children.push(ContentItem::Session(inner));

        let mut root = Session::with_title("Root".to_string());
        root.children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Top".to_string(),
            )));
        root.children.push(ContentItem::Session(outer));

        assert_eq!(root.find_nodes_at_depth(0).len(), 2);
        assert!(!root.find_nodes_at_depth(1).is_empty());
    }

    #[test]
    fn test_find_sessions_at_depth() {
        let mut level2 = Session::with_title("Level 2".to_string());
        level2
            .children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Leaf".to_string(),
            )));
        let mut level1 = Session::with_title("Level 1".to_string());
        level1.children.push(ContentItem::Session(level2));
        let mut root = Session::with_title("Level 0".to_string());
        root.children.push(ContentItem::Session(level1));

        let level_0: Vec<_> = root
            .find_nodes_at_depth(0)
            .into_iter()
            .filter_map(|n| n.as_session())
            .collect();
        assert_eq!(level_0.len(), 1);

        let level_1: Vec<_> = root
            .find_nodes_at_depth(1)
            .into_iter()
            .filter_map(|n| n.as_session())
            .collect();
        assert_eq!(level_1.len(), 1);
    }

    #[test]
    fn test_find_nodes_in_depth_range() {
        let mut deep = Session::with_title("Deep".to_string());
        deep.children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Very deep".to_string(),
            )));
        let mut mid = Session::with_title("Mid".to_string());
        mid.children.push(ContentItem::Session(deep));
        let mut root = Session::with_title("Top".to_string());
        root.children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Root".to_string(),
            )));
        root.children.push(ContentItem::Session(mid));

        assert!(!root.find_nodes_in_depth_range(0, 1).is_empty());
        assert!(!root.find_nodes_in_depth_range(1, 2).is_empty());
    }

    #[test]
    fn test_find_nodes_with_depth_and_predicate() {
        let mut session = Session::with_title("Test Session".to_string());
        session
            .children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Hello from nested".to_string(),
            )));

        let mut root = Session::with_title("Root".to_string());
        root.children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Hello from top".to_string(),
            )));
        root.children.push(ContentItem::Session(session));

        let depth_0_hello = root.find_nodes_with_depth(0, |node| {
            node.as_paragraph()
                .map(|p| p.text().contains("Hello"))
                .unwrap_or(false)
        });
        assert_eq!(depth_0_hello.len(), 1);
    }

    #[test]
    fn test_find_paragraphs_at_depth() {
        let mut session = Session::with_title("Section".to_string());
        session
            .children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Nested para".to_string(),
            )));

        let mut root = Session::with_title("Root".to_string());
        root.children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Top".to_string(),
            )));
        root.children.push(ContentItem::Session(session));

        let top_paras: Vec<_> = root
            .find_nodes_at_depth(0)
            .into_iter()
            .filter_map(|n| n.as_paragraph())
            .collect();
        assert_eq!(top_paras.len(), 1);
    }

    #[test]
    fn test_phase_2_comprehensive_example() {
        let mut session1 = Session::with_title("Chapter 1".to_string());
        session1
            .children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Intro".to_string(),
            )));

        let session2 = Session::with_title("Chapter 2".to_string());

        let mut root = Session::with_title("Root".to_string());
        root.children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Preface".to_string(),
            )));
        root.children.push(ContentItem::Session(session1));
        root.children.push(ContentItem::Session(session2));

        let all_paragraphs: Vec<_> = root.iter_paragraphs_recursive().collect();
        assert_eq!(all_paragraphs.len(), 2);
        let all_sessions: Vec<_> = root.iter_sessions_recursive().collect();
        assert_eq!(all_sessions.len(), 2);
    }

    #[test]
    fn test_session_header_and_body_locations() {
        let title_range = Range::new(0..5, Position::new(0, 0), Position::new(0, 5));
        let child_range = Range::new(10..20, Position::new(1, 0), Position::new(2, 0));
        let title = TextContent::from_string("Title".to_string(), Some(title_range.clone()));
        let child = Paragraph::from_line("Child".to_string()).at(child_range.clone());
        let child_item = ContentItem::Paragraph(child);
        let session = Session::new(title, vec![SessionContent::from(child_item)]).at(Range::new(
            0..25,
            Position::new(0, 0),
            Position::new(2, 0),
        ));

        assert_eq!(session.header_location(), Some(&title_range));
        assert_eq!(session.body_location().unwrap().span, child_range.span);
    }
}
