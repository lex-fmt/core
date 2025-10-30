//! Test infrastructure for txxtv
//!
//! This module provides helper utilities and fixtures for testing txxtv.
//! The TestApp struct allows us to:
//! - Load a test document
//! - Render using Ratatui's TestBackend (headless)
//! - Make assertions on model state
//! - Simulate keyboard input
//!
//! Test Document:
//! We use docs/specs/v1/samples/030-paragraphs-sessions-nested-multiple.txxt
//! This file is ideal because it:
//! - Parses cleanly
//! - Has nested sessions (2 levels deep in most cases, 3 in one case)
//! - Has multiple paragraphs at various levels
//! - Is small (35 lines) so tests can reason about the entire structure
//! - Tests both flat and nested navigation

use crate::model::{Model, NodeId, Selection};
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::fs;

/// Test application harness
pub struct TestApp {
    pub model: Model,
    pub terminal: Terminal<TestBackend>,
}

impl TestApp {
    /// Create a new test app with the standard test document
    pub fn new() -> Self {
        let content =
            fs::read_to_string("docs/specs/v1/samples/030-paragraphs-sessions-nested-multiple.txxt")
                .expect("Failed to load test document");

        let document = txxt_nano::txxt_nano::parser::parse_document(&content)
            .expect("Failed to parse test document");

        let model = Model::new(document);

        // Create a test terminal with reasonable dimensions
        // Standard terminal is ~80x24, we use 80x30 to give enough space
        let backend = TestBackend::new(80, 30);
        let terminal = Terminal::new(backend).expect("Failed to create test terminal");

        TestApp { model, terminal }
    }

    /// Create a NodeId from a path for testing
    pub fn node_id(path: &[usize]) -> NodeId {
        NodeId::new(path)
    }

    /// Get the current selection
    pub fn selection(&self) -> Selection {
        self.model.selection()
    }

    /// Get the selected position if in text mode
    pub fn selected_position(&self) -> Option<(usize, usize)> {
        self.model.get_selected_position()
    }

    /// Get the selected node ID if in tree mode
    pub fn selected_node_id(&self) -> Option<NodeId> {
        self.model.get_selected_node_id()
    }

    /// Assert that the selected position matches expected
    pub fn assert_selected_position(&self, row: usize, col: usize) {
        assert_eq!(
            self.selected_position(),
            Some((row, col)),
            "Expected selection at ({}, {}), got {:?}",
            row,
            col,
            self.selected_position()
        );
    }

    /// Assert that the selected node matches expected
    pub fn assert_selected_node_id(&self, expected: NodeId) {
        assert_eq!(
            self.selected_node_id(),
            Some(expected),
            "Expected selection of node {:?}, got {:?}",
            expected.path(),
            self.selected_node_id().map(|id| id.path())
        );
    }

    /// Assert that a node is expanded
    pub fn assert_node_expanded(&self, node_id: NodeId) {
        assert!(
            self.model.is_node_expanded(node_id),
            "Expected node {:?} to be expanded",
            node_id.path()
        );
    }

    /// Assert that a node is collapsed
    pub fn assert_node_collapsed(&self, node_id: NodeId) {
        assert!(
            !self.model.is_node_expanded(node_id),
            "Expected node {:?} to be collapsed",
            node_id.path()
        );
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let app = TestApp::new();
        // Verify that the test document was loaded
        assert!(app.model.document.items.len() > 0);
    }

    #[test]
    fn test_initial_selection() {
        let app = TestApp::new();
        // Initial selection should be text position (0, 0)
        app.assert_selected_position(0, 0);
    }

    #[test]
    fn test_select_position() {
        let mut app = TestApp::new();
        app.model.select_position(5, 10);
        app.assert_selected_position(5, 10);
    }

    #[test]
    fn test_select_node() {
        let mut app = TestApp::new();
        let node_id = TestApp::node_id(&[0, 1]);
        app.model.select_node(node_id);
        app.assert_selected_node_id(node_id);
    }

    #[test]
    fn test_toggle_expansion() {
        let mut app = TestApp::new();
        let node_id = TestApp::node_id(&[0]);

        // Initially collapsed
        app.assert_node_collapsed(node_id);

        // Expand
        app.model.toggle_node_expansion(node_id);
        app.assert_node_expanded(node_id);

        // Collapse again
        app.model.toggle_node_expansion(node_id);
        app.assert_node_collapsed(node_id);
    }

    #[test]
    fn test_expand_multiple_nodes() {
        let mut app = TestApp::new();
        let nodes = [
            TestApp::node_id(&[0]),
            TestApp::node_id(&[0, 1]),
            TestApp::node_id(&[0, 1, 2]),
        ];

        app.model.expand_nodes(&nodes);

        for &node in &nodes {
            app.assert_node_expanded(node);
        }
    }

    #[test]
    fn test_node_id_operations() {
        let parent = TestApp::node_id(&[0, 1]);
        let child = parent.child(3);

        assert_eq!(child.path(), &[0, 1, 3]);

        let back_to_parent = child.parent();
        assert_eq!(back_to_parent, Some(parent));
    }
}
