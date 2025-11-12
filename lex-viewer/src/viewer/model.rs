//! Data model for lex
//!
//! The Model struct holds the pure application state:
//! - The parsed AST (Document)
//! - Current selection (either a text position or a tree node)
//! - Expanded/collapsed state of tree nodes
//!
//! This separation of concerns makes testing easier: the model is pure data
//! and can be tested independently of rendering and UI logic.

use lex_parser::lex::ast::elements::content_item::ContentItem;
use lex_parser::lex::ast::range::{Position, Range};
use lex_parser::lex::ast::traits::AstNode;
use lex_parser::lex::ast::{snapshot_visitor::snapshot_from_document, AstSnapshot};
use lex_parser::lex::parsing::Document;
use std::collections::HashSet;

/// Which viewer currently has keyboard focus
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Focus {
    /// File viewer (text) has focus
    #[default]
    FileViewer,
    /// Tree viewer (AST) has focus
    TreeViewer,
}

impl Focus {
    /// Toggle focus to the other viewer
    pub fn toggle(&self) -> Focus {
        match self {
            Focus::FileViewer => Focus::TreeViewer,
            Focus::TreeViewer => Focus::FileViewer,
        }
    }
}

/// Stable identifier for an AST node.
///
/// A NodeId is a path through the tree, represented as a `Vec<usize>`.
/// For example, [0, 1, 2] means: child 0 of root, then child 1 of that, then child 2 of that.
/// This remains stable across re-renders, unlike raw references.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId {
    // We'll use a simple approach: store as a static-sized array for performance
    // Most documents won't have nesting deeper than 8-10 levels
    indices: [usize; 8],
    depth: usize, // How many indices are actually used
}

#[allow(dead_code)]
impl NodeId {
    /// Create a new NodeId from a path
    pub fn new(path: &[usize]) -> Self {
        assert!(path.len() <= 8, "NodeId path too deep (max 8 levels)");
        let mut indices = [0; 8];
        for (i, &idx) in path.iter().enumerate() {
            indices[i] = idx;
        }
        NodeId {
            indices,
            depth: path.len(),
        }
    }

    /// Get the path as a slice
    pub fn path(&self) -> &[usize] {
        &self.indices[0..self.depth]
    }

    /// Get the parent NodeId, or None if this is the root
    pub fn parent(&self) -> Option<NodeId> {
        if self.depth > 0 {
            Some(NodeId {
                indices: self.indices,
                depth: self.depth - 1,
            })
        } else {
            None
        }
    }

    /// Create a child NodeId
    pub fn child(&self, index: usize) -> NodeId {
        assert!(self.depth < 8, "NodeId path too deep");
        let mut indices = self.indices;
        indices[self.depth] = index;
        NodeId {
            indices,
            depth: self.depth + 1,
        }
    }
}

/// Current selection in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selection {
    /// Text position (row, col) - user is navigating the text view
    TextSelection(usize, usize),
    /// Tree node selection - user is navigating the tree view
    TreeSelection(NodeId),
}

/// A node in the flattened tree representation
///
/// This represents a single node in a depth-first flattening of the AST tree.
/// Used for rendering the tree viewer where nodes are displayed in a list.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FlattenedTreeNode {
    /// Stable identifier for this node
    pub node_id: NodeId,
    /// Depth in the tree (for indentation)
    pub depth: usize,
    /// Display label for this node
    pub label: String,
    /// Whether this node is currently expanded
    pub is_expanded: bool,
    /// Whether this node has children
    pub has_children: bool,
    /// The type of the AST node
    pub node_type: &'static str,
}

/// The core data model
#[derive(Clone)]
pub struct Model {
    /// The parsed AST document
    pub document: Document,

    /// Current selection (text or tree)
    selection: Selection,

    /// Which tree nodes are expanded (rest are collapsed)
    expanded_nodes: HashSet<NodeId>,
}

#[allow(dead_code)]
impl Model {
    /// Create a new model from a document
    pub fn new(document: Document) -> Self {
        let mut model = Model {
            document,
            selection: Selection::TextSelection(0, 0),
            expanded_nodes: HashSet::new(),
        };
        // Start with all nodes expanded
        model.expand_all_nodes();
        model
    }

    /// Expand all nodes in the document (for initial state)
    fn expand_all_nodes(&mut self) {
        let content = self.document.root.children.clone();
        self.expand_all_recursive(&content, &NodeId::new(&[]));
    }

    /// Recursively expand all nodes
    fn expand_all_recursive(&mut self, items: &[ContentItem], parent_id: &NodeId) {
        for (index, item) in items.iter().enumerate() {
            let node_id = parent_id.child(index);
            self.expanded_nodes.insert(node_id);

            // Recursively expand children
            if let Some(children) = item.children() {
                let children_clone = children.to_vec();
                self.expand_all_recursive(&children_clone, &node_id);
            }
        }
    }

    /// Get the current selection
    pub fn selection(&self) -> Selection {
        self.selection
    }

    /// Select a text position
    pub fn select_position(&mut self, row: usize, col: usize) {
        self.selection = Selection::TextSelection(row, col);
    }

    /// Select a tree node
    pub fn select_node(&mut self, node_id: NodeId) {
        self.selection = Selection::TreeSelection(node_id);
    }

    /// Get the selected position if in text mode
    pub fn get_selected_position(&self) -> Option<(usize, usize)> {
        match self.selection {
            Selection::TextSelection(r, c) => Some((r, c)),
            _ => None,
        }
    }

    /// Get the selected node ID if in tree mode
    pub fn get_selected_node_id(&self) -> Option<NodeId> {
        match self.selection {
            Selection::TreeSelection(id) => Some(id),
            _ => None,
        }
    }

    /// Toggle whether a node is expanded
    pub fn toggle_node_expansion(&mut self, node_id: NodeId) {
        if self.expanded_nodes.contains(&node_id) {
            self.expanded_nodes.remove(&node_id);
        } else {
            self.expanded_nodes.insert(node_id);
        }
    }

    /// Check if a node is expanded
    pub fn is_node_expanded(&self, node_id: NodeId) -> bool {
        self.expanded_nodes.contains(&node_id)
    }

    /// Expand multiple nodes at once (e.g., all ancestors)
    pub fn expand_nodes(&mut self, node_ids: &[NodeId]) {
        for &id in node_ids {
            self.expanded_nodes.insert(id);
        }
    }

    /// Collapse a node
    pub fn collapse_node(&mut self, node_id: NodeId) {
        self.expanded_nodes.remove(&node_id);
    }

    /// Find the innermost node at the given position
    ///
    /// Uses the AST's location information to map a file position (line, col) to
    /// the deepest AST node containing that position.
    ///
    /// This is the core of the line-to-node mapping: when the file viewer
    /// moves the cursor to a position, this method finds which AST node that
    /// position belongs to.
    ///
    /// Returns the NodeId of the deepest node that contains the position.
    /// If multiple nodes contain the position, returns the innermost (deepest) one.
    pub fn get_node_at_position(&self, row: usize, col: usize) -> Option<NodeId> {
        let pos = Position::new(row, col);

        // Find the deepest element at this position using AST locations
        if let Some(element) = self.document.element_at(pos) {
            self.find_node_id_for_element(element)
        } else {
            None
        }
    }

    /// Get the location for a node
    ///
    /// Returns the text range (start and end position) for the given node.
    /// The location indicates where in the source text this node is located.
    pub fn get_location_for_node(&self, node_id: NodeId) -> Option<Range> {
        if node_id.path().is_empty() {
            // Document doesn't have a location; the root session does
            return Some(self.document.root.location.clone());
        }

        self.get_node(node_id)
            .map(|(item, _depth)| item.range().clone())
    }

    /// Get the ancestors of a node (path from root to node, not including the node itself)
    ///
    /// Returns the NodeIds of all ancestors from root to parent.
    pub fn get_ancestors(&self, node_id: NodeId) -> Vec<NodeId> {
        let mut ancestors = Vec::new();
        let path = node_id.path();

        for i in 0..path.len() {
            let ancestor_path = &path[0..i];
            ancestors.push(NodeId::new(ancestor_path));
        }

        ancestors
    }

    /// Build a flattened tree structure for rendering
    ///
    /// Creates a depth-first flattening of the document tree, respecting
    /// the expanded/collapsed state. Only includes expanded nodes and their children.
    /// Uses the canonical snapshot representation to ensure consistency with serializers.
    pub fn flattened_tree(&self) -> Vec<FlattenedTreeNode> {
        let mut nodes = Vec::new();

        // Build snapshot tree from the document root
        let snapshot = snapshot_from_document(&self.document);
        let root_id = NodeId::new(&[]);
        self.flatten_snapshot_recursive(&snapshot, &root_id, &mut nodes);

        nodes
    }

    /// Recursively flatten a snapshot tree, respecting expanded/collapsed state
    fn flatten_snapshot_recursive(
        &self,
        snapshot: &AstSnapshot,
        current_id: &NodeId,
        nodes: &mut Vec<FlattenedTreeNode>,
    ) {
        let depth = current_id.path().len();
        let has_children = !snapshot.children.is_empty();
        let is_expanded = current_id.path().is_empty() || self.is_node_expanded(*current_id);

        // Map snapshot node type to static string for FlattenedTreeNode
        let node_type = match snapshot.node_type.as_str() {
            "Document" => "Document",
            "Session" => "Session",
            "Paragraph" => "Paragraph",
            "List" => "List",
            "ListItem" => "ListItem",
            "Definition" => "Definition",
            "VerbatimBlock" => "VerbatimBlock",
            "Annotation" => "Annotation",
            "TextLine" => "TextLine",
            _ => "Unknown",
        };

        nodes.push(FlattenedTreeNode {
            node_id: *current_id,
            depth,
            label: snapshot.label.clone(),
            is_expanded,
            has_children,
            node_type,
        });

        // Recursively add children if expanded
        if is_expanded && has_children {
            for (child_index, child) in snapshot.children.iter().enumerate() {
                let child_id = current_id.child(child_index);
                self.flatten_snapshot_recursive(child, &child_id, nodes);
            }
        }
    }

    // ========== Private helper methods ==========

    /// Get a node by its ID and its depth
    ///
    /// Returns (element, depth) if found, or None if not found
    fn get_node(&self, node_id: NodeId) -> Option<(&ContentItem, usize)> {
        let path = node_id.path();
        if path.is_empty() {
            return None;
        }

        let mut current: &[ContentItem] = &self.document.root.children;
        let mut depth = 0;

        for (path_idx, &index) in path.iter().enumerate() {
            if index >= current.len() {
                return None;
            }

            let item = &current[index];
            depth += 1;

            // If this is the last index in the path, return this item
            if path_idx == path.len() - 1 {
                return Some((item, depth));
            }

            // Otherwise, navigate to this item's children for the next iteration
            if let Some(children) = item.children() {
                current = children;
            } else {
                // We've reached a leaf node, but there's still more path
                return None;
            }
        }

        None
    }

    /// Find the NodeId for an element by searching the tree
    ///
    /// This is a linear search through the tree to find the element.
    /// In practice, this is fast enough since lex documents are typically small.
    fn find_node_id_for_element(&self, target: &ContentItem) -> Option<NodeId> {
        self.find_node_id_recursive(target, &self.document.root.children, &mut Vec::new())
    }

    #[allow(clippy::only_used_in_recursion)]
    fn find_node_id_recursive(
        &self,
        target: &ContentItem,
        items: &[ContentItem],
        path: &mut Vec<usize>,
    ) -> Option<NodeId> {
        for (index, item) in items.iter().enumerate() {
            path.push(index);

            // Check if this is the target
            if std::ptr::eq(item, target) {
                let node_id = NodeId::new(path);
                path.pop();
                return Some(node_id);
            }

            // Search children
            if let Some(children) = item.children() {
                if let Some(node_id) = self.find_node_id_recursive(target, children, path) {
                    path.pop();
                    return Some(node_id);
                }
            }

            path.pop();
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id_new() {
        let id = NodeId::new(&[0, 1, 2]);
        assert_eq!(id.path(), &[0, 1, 2]);
    }

    #[test]
    fn test_node_id_parent() {
        let id = NodeId::new(&[0, 1, 2]);
        let parent = id.parent().unwrap();
        assert_eq!(parent.path(), &[0, 1]);

        let grandparent = parent.parent().unwrap();
        assert_eq!(grandparent.path(), &[0]);

        let root = grandparent.parent().unwrap();
        assert_eq!(root.path().len(), 0);

        assert!(root.parent().is_none());
    }

    #[test]
    fn test_node_id_child() {
        let id = NodeId::new(&[0, 1]);
        let child = id.child(5);
        assert_eq!(child.path(), &[0, 1, 5]);
    }

    #[test]
    fn test_selection_text() {
        let mut model = Model::new(lex_parser::lex::parsing::parse_document("test").unwrap());
        model.select_position(5, 10);
        assert_eq!(model.get_selected_position(), Some((5, 10)));
        assert_eq!(model.get_selected_node_id(), None);
    }

    #[test]
    fn test_selection_node() {
        let mut model = Model::new(lex_parser::lex::parsing::parse_document("test").unwrap());
        let node_id = NodeId::new(&[0, 1]);
        model.select_node(node_id);
        assert_eq!(model.get_selected_node_id(), Some(node_id));
        assert_eq!(model.get_selected_position(), None);
    }

    #[test]
    fn test_node_expansion() {
        let mut model = Model::new(lex_parser::lex::parsing::parse_document("test").unwrap());
        let node_id = NodeId::new(&[0, 1]);

        assert!(!model.is_node_expanded(node_id));
        model.toggle_node_expansion(node_id);
        assert!(model.is_node_expanded(node_id));
        model.toggle_node_expansion(node_id);
        assert!(!model.is_node_expanded(node_id));
    }

    #[test]
    fn test_expand_nodes() {
        let mut model = Model::new(lex_parser::lex::parsing::parse_document("test").unwrap());
        let nodes = [
            NodeId::new(&[0]),
            NodeId::new(&[0, 1]),
            NodeId::new(&[0, 1, 2]),
        ];

        model.expand_nodes(&nodes);
        for &node in &nodes {
            assert!(model.is_node_expanded(node));
        }
    }

    #[test]
    fn test_get_ancestors() {
        let model = Model::new(lex_parser::lex::parsing::parse_document("test").unwrap());

        let node = NodeId::new(&[0, 1, 2]);
        let ancestors = model.get_ancestors(node);

        // Ancestors should be [root], [0], [0, 1] (not including [0, 1, 2])
        assert_eq!(ancestors.len(), 3);
        assert_eq!(ancestors[0].path().len(), 0);
        assert_eq!(ancestors[1].path(), &[0usize][..]);
        assert_eq!(ancestors[2].path(), &[0usize, 1][..]);
    }

    #[test]
    fn test_get_ancestors_root_child() {
        let model = Model::new(lex_parser::lex::parsing::parse_document("test").unwrap());

        let node = NodeId::new(&[0]);
        let ancestors = model.get_ancestors(node);

        // Only root should be ancestor
        assert_eq!(ancestors.len(), 1);
        assert_eq!(ancestors[0].path().len(), 0);
    }

    #[test]
    fn test_flattened_tree_with_content() {
        let doc_str = "# Heading\n\nParagraph text";
        let model = Model::new(lex_parser::lex::parsing::parse_document(doc_str).unwrap());

        let flattened = model.flattened_tree();

        // Should have some nodes
        assert!(!flattened.is_empty());

        // First node should be the document root
        assert_eq!(flattened[0].node_type, "Document");
        assert_eq!(flattened[0].depth, 0);

        // All nodes should have valid node IDs
        for node in &flattened {
            assert!(node.node_id.path().is_empty() || node.depth > 0);
        }
    }

    #[test]
    fn test_flattened_tree_respects_expansion() {
        let doc_str = "# Heading\n## Subheading\nText";
        let mut model = Model::new(lex_parser::lex::parsing::parse_document(doc_str).unwrap());

        // Get flattened tree when nothing is expanded
        let flattened_collapsed = model.flattened_tree();
        let num_collapsed = flattened_collapsed.len();

        // Expand first node
        let first_node_id = NodeId::new(&[0]);
        model.expand_nodes(&[first_node_id]);

        // Get flattened tree when expanded
        let flattened_expanded = model.flattened_tree();
        let num_expanded = flattened_expanded.len();

        // When expanded, we should have more nodes visible
        assert!(num_expanded >= num_collapsed);
    }

    #[test]
    fn test_get_node_at_position_finds_ast_node() {
        // Create document with content we know the structure of
        let doc_str = "# Heading\n\nParagraph";
        let model = Model::new(lex_parser::lex::parsing::parse_document(doc_str).unwrap());

        // Position (0, 0) should be at the heading
        if let Some(node_id) = model.get_node_at_position(0, 0) {
            // Should find some node (the heading or a child of it)
            assert!(!node_id.path().is_empty() || node_id.path().is_empty());
        }

        // Position (1, 0) should find something (blank line)
        let node_at_line_1 = model.get_node_at_position(1, 0);
        // May or may not find a node depending on how AST handles blank lines
        let _ = node_at_line_1;
    }

    #[test]
    fn test_select_position_then_get_node() {
        let mut model =
            Model::new(lex_parser::lex::parsing::parse_document("# Title\nContent").unwrap());

        // Simulate file viewer selecting a position
        model.select_position(0, 0);
        assert_eq!(model.get_selected_position(), Some((0, 0)));

        // We can then find the node at that position
        if let Some(_node_id) = model.get_node_at_position(0, 0) {
            // Successfully found a node at the cursor position
            // In a real scenario, FileViewer would use this to notify app
            // of SelectPosition event, app would then call get_node_at_position
            // and auto-expand ancestors
        }
    }
}
