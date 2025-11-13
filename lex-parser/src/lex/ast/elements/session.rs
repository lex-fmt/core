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
            location: Self::default_location(),
        }
    }
    pub fn with_title(title: String) -> Self {
        Self {
            title: TextContent::from_string(title, None),
            children: SessionContainer::empty(),
            location: Self::default_location(),
        }
    }

    /// Preferred builder
    pub fn at(mut self, location: Range) -> Self {
        self.location = location;
        self
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
}
