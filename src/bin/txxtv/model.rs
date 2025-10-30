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
use txxt_nano::txxt_nano::parser::Document;

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
pub struct Model {
    /// The parsed AST document
    pub document: Document,

    /// Current selection (text or tree)
    selection: Selection,

    /// Which tree nodes are expanded (rest are collapsed)
    expanded_nodes: HashSet<NodeId>,
}

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

        let root = parent.parent().unwrap();
        assert_eq!(root.path(), &[0]);

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
        let nodes = [NodeId::new(&[0]), NodeId::new(&[0, 1]), NodeId::new(&[0, 1, 2])];

        model.expand_nodes(&nodes);
        for &node in &nodes {
            assert!(model.is_node_expanded(node));
        }
    }
}
