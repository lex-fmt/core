//! AST Snapshot - a normalized intermediate representation of the AST tree
//!
//! This module provides a canonical, format-agnostic representation of the AST
//! suitable for serialization to any output format (JSON, YAML, treeviz, tag, etc.)
//!
//! The snapshot captures the complete tree structure with node types, labels,
//! attributes, and children - allowing each serializer to focus solely on
//! presentation without reimplementing AST traversal logic.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A snapshot of an AST node in a normalized, serializable form
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AstSnapshot {
    /// The type of node (e.g., "Paragraph", "Session", "List")
    pub node_type: String,

    /// The primary label or text content of the node
    pub label: String,

    /// Additional attributes specific to the node type
    pub attributes: HashMap<String, String>,

    /// Child nodes in the tree
    pub children: Vec<AstSnapshot>,
}

impl AstSnapshot {
    /// Create a new snapshot with the given node type and label
    pub fn new(node_type: String, label: String) -> Self {
        Self {
            node_type,
            label,
            attributes: HashMap::new(),
            children: Vec::new(),
        }
    }

    /// Add an attribute to this snapshot
    pub fn with_attribute(mut self, key: String, value: String) -> Self {
        self.attributes.insert(key, value);
        self
    }

    /// Add a child snapshot
    pub fn with_child(mut self, child: AstSnapshot) -> Self {
        self.children.push(child);
        self
    }

    /// Add multiple children
    pub fn with_children(mut self, children: Vec<AstSnapshot>) -> Self {
        self.children.extend(children);
        self
    }
}
