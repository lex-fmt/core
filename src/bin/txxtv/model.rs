//! Data model for txxtv
//!
//! The Model struct holds the pure application state:
//! - The parsed AST (Document)
//! - Current selection (either a text position or a tree node)
//! - Expanded/collapsed state of tree nodes
//!
//! This separation of concerns makes testing easier: the model is pure data
//! and can be tested independently of rendering and UI logic.

use std::collections::HashSet;
use txxt_nano::txxt_nano::ast::elements::content_item::ContentItem;
use txxt_nano::txxt_nano::ast::span::{Position, Span};
use txxt_nano::txxt_nano::parser::Document;

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
/// A NodeId is a path through the tree, represented as a Vec<usize>.
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
        Model {
            document,
            selection: Selection::TextSelection(0, 0),
            expanded_nodes: HashSet::new(),
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
    /// Returns the NodeId of the deepest node that contains the position.
    /// If multiple nodes contain the position, returns the innermost (deepest) one.
    pub fn get_node_at_position(&self, row: usize, col: usize) -> Option<NodeId> {
        let pos = Position::new(row, col);

        // Find all elements at this position (deepest first)
        let elements = self.document.elements_at(pos);

        // Return the first (deepest) element's NodeId
        if let Some(element) = elements.first() {
            self.find_node_id_for_element(element)
        } else {
            None
        }
    }

    /// Get the span for a node
    ///
    /// Returns the text range (start and end position) for the given node.
    #[allow(dead_code)]
    pub fn get_span_for_node(&self, _node_id: NodeId) -> Option<Span> {
        // TODO: Implement span lookup for ContentItem enum variants
        None
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

    // ========== Private helper methods ==========

    /// Get a node by its ID and its depth
    ///
    /// Returns (element, depth) if found, or None if not found
    fn get_node(&self, node_id: NodeId) -> Option<(&ContentItem, usize)> {
        let path = node_id.path();
        if path.is_empty() {
            return None;
        }

        let mut current: &[ContentItem] = &self.document.content;
        let mut _depth = 0;

        for &index in path {
            if index >= current.len() {
                return None;
            }

            let item = &current[index];
            let _ = _depth + 1; // Unused but kept for future use

            if let Some(children) = item.children() {
                current = children;
            } else {
                // We've reached a leaf node, but there's still more path
                return None;
            }
        }

        // Return the last item we navigated to
        // We need to backtrack one step
        if !path.is_empty() {
            let parent_path = &path[0..path.len() - 1];
            if parent_path.is_empty() {
                // Direct child of root
                let last_index = path[0];
                if last_index < self.document.content.len() {
                    return Some((&self.document.content[last_index], 1));
                }
            } else if let Some((parent, parent_depth)) = self.get_node(NodeId::new(parent_path)) {
                if let Some(children) = parent.children() {
                    let child_index = path[path.len() - 1];
                    if child_index < children.len() {
                        return Some((&children[child_index], parent_depth + 1));
                    }
                }
            }
        }

        None
    }

    /// Find the NodeId for an element by searching the tree
    ///
    /// This is a linear search through the tree to find the element.
    /// In practice, this is fast enough since txxt documents are typically small.
    fn find_node_id_for_element(&self, target: &ContentItem) -> Option<NodeId> {
        self.find_node_id_recursive(target, &self.document.content, &mut Vec::new())
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
        let mut model = Model::new(txxt_nano::txxt_nano::parser::parse_document("test").unwrap());
        model.select_position(5, 10);
        assert_eq!(model.get_selected_position(), Some((5, 10)));
        assert_eq!(model.get_selected_node_id(), None);
    }

    #[test]
    fn test_selection_node() {
        let mut model = Model::new(txxt_nano::txxt_nano::parser::parse_document("test").unwrap());
        let node_id = NodeId::new(&[0, 1]);
        model.select_node(node_id);
        assert_eq!(model.get_selected_node_id(), Some(node_id));
        assert_eq!(model.get_selected_position(), None);
    }

    #[test]
    fn test_node_expansion() {
        let mut model = Model::new(txxt_nano::txxt_nano::parser::parse_document("test").unwrap());
        let node_id = NodeId::new(&[0, 1]);

        assert!(!model.is_node_expanded(node_id));
        model.toggle_node_expansion(node_id);
        assert!(model.is_node_expanded(node_id));
        model.toggle_node_expansion(node_id);
        assert!(!model.is_node_expanded(node_id));
    }

    #[test]
    fn test_expand_nodes() {
        let mut model = Model::new(txxt_nano::txxt_nano::parser::parse_document("test").unwrap());
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
        let model = Model::new(txxt_nano::txxt_nano::parser::parse_document("test").unwrap());

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
        let model = Model::new(txxt_nano::txxt_nano::parser::parse_document("test").unwrap());

        let node = NodeId::new(&[0]);
        let ancestors = model.get_ancestors(node);

        // Only root should be ancestor
        assert_eq!(ancestors.len(), 1);
        assert_eq!(ancestors[0].path().len(), 0);
    }
}
