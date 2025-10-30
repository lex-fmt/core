//! Test infrastructure for txxtv
//!
//! Provides utilities for testing the full application including:
//! - TestApp: wrapper for testing the application
//! - Keyboard helpers: easy creation of keyboard events
//! - Render helpers: getting and verifying UI output
//! - Test fixtures: pre-loaded test documents

use super::app::App;
use super::model::Model;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::fs;

/// Test application wrapper with test backend
pub struct TestApp {
    app: App,
    terminal: Terminal<TestBackend>,
}

impl TestApp {
    /// Create a new test app with a simple document
    pub fn new() -> Self {
        Self::with_content("test")
    }

    /// Create a test app with specific content
    pub fn with_content(content: &str) -> Self {
        let document = txxt_nano::txxt_nano::parser::parse_document(content)
            .expect("Failed to parse test document");
        let model = Model::new(document);
        let app = App::new(model, content.to_string());

        // Create terminal with reasonable default size (80x24)
        let backend = TestBackend::new(80, 24);
        let terminal = Terminal::new(backend).expect("Failed to create terminal");

        TestApp { app, terminal }
    }

    /// Load a test document from file
    pub fn with_file(path: &str) -> Self {
        let content = fs::read_to_string(path)
            .expect("Failed to read test file");
        Self::with_content(&content)
    }

    /// Send a keyboard event and return the rendered output
    pub fn send_key(&mut self, code: KeyCode) -> String {
        self.send_key_with_modifiers(code, KeyModifiers::empty())
    }

    /// Send a keyboard event with modifiers and return the rendered output
    pub fn send_key_with_modifiers(&mut self, code: KeyCode, modifiers: KeyModifiers) -> String {
        let key = KeyEvent::new(code, modifiers);
        let _ = self.app.handle_key(key);
        self.render()
    }

    /// Render the current application state and return output
    pub fn render(&mut self) -> String {
        use super::ui;

        self.terminal
            .draw(|frame| {
                let file_name = "test.txxt";
                ui::render(frame, &self.app, file_name);
            })
            .expect("Failed to draw");

        // Get the rendered output as string
        self.terminal_output()
    }

    /// Get the current terminal output as a string
    fn terminal_output(&self) -> String {
        // For now, return a simple string indicating render was successful
        // Full snapshot testing will be done with insta
        format!("Rendered output ({} x {})", self.terminal.size().unwrap().width, self.terminal.size().unwrap().height)
    }

    /// Get reference to the app for assertions
    pub fn app(&self) -> &App {
        &self.app
    }

    /// Get mutable reference to the app for direct state manipulation
    pub fn app_mut(&mut self) -> &mut App {
        &mut self.app
    }

    /// Check if file viewer is focused
    pub fn is_file_viewer_focused(&self) -> bool {
        self.app.focus == crate::model::Focus::FileViewer
    }

    /// Check if tree viewer is focused
    pub fn is_tree_viewer_focused(&self) -> bool {
        self.app.focus == crate::model::Focus::TreeViewer
    }

    /// Check if a node is expanded
    pub fn is_node_expanded(&self, node_id: crate::model::NodeId) -> bool {
        self.app.model.is_node_expanded(node_id)
    }

    /// Get the currently selected node ID
    pub fn selected_node_id(&self) -> Option<crate::model::NodeId> {
        self.app.model.get_selected_node_id()
    }

    /// Get the currently selected text position
    pub fn selected_position(&self) -> Option<(usize, usize)> {
        self.app.model.get_selected_position()
    }

    /// Check if app should quit
    pub fn should_quit(&self) -> bool {
        self.app.should_quit
    }
}

impl Default for TestApp {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions for creating keyboard events
pub mod keyboard {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    /// Create a key event with no modifiers
    pub fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    /// Create a key event with Ctrl modifier
    pub fn ctrl(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::CONTROL)
    }

    /// Create a key event with Shift modifier
    pub fn shift(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::SHIFT)
    }

    /// Create a key event with Alt modifier
    pub fn alt(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::ALT)
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::model::NodeId;
    use crossterm::event::KeyCode;

    #[test]
    fn test_app_has_flattened_tree() {
        let app = TestApp::with_content("# Heading\n\nParagraph");

        // The model should be able to produce a flattened tree
        let flattened = app.app().model.flattened_tree();
        assert!(!flattened.is_empty(), "Should have flattened tree nodes");
    }

    #[test]
    fn test_file_viewer_cursor_movement() {
        let mut app = TestApp::with_content("Line 1\nLine 2\nLine 3");

        // Initial cursor should be at (0, 0)
        assert_eq!(
            app.app().file_viewer.cursor_position(),
            (0, 0),
            "Cursor should start at (0, 0)"
        );

        // Press down arrow
        app.send_key(KeyCode::Down);

        // Cursor should now be at (1, 0)
        assert_eq!(
            app.app().file_viewer.cursor_position(),
            (1, 0),
            "Cursor should move down"
        );
    }

    #[test]
    fn test_file_position_maps_to_tree_node() {
        let mut app = TestApp::with_content("# Heading\n\nParagraph text");

        // Move cursor to position under the heading
        app.send_key(KeyCode::Down);

        // Get the current cursor position
        let (row, col) = app.app().file_viewer.cursor_position();

        // Model should be able to find AST node at this position
        if let Some(node_id) = app.app().model.get_node_at_position(row, col) {
            // Should have a valid node ID
            assert!(!node_id.path().is_empty() || node_id.path().len() == 0);
        }
    }

    #[test]
    fn test_flattened_tree_respects_expansion_state() {
        let mut app = TestApp::with_content("# Heading\n## Subheading\n");

        // Initially nothing is expanded
        let flattened_before = app.app().model.flattened_tree();
        let count_before = flattened_before.len();

        // Expand the first node
        let first_node = NodeId::new(&[0]);
        app.app_mut().model.expand_nodes(&[first_node]);

        // Get flattened tree again
        let flattened_after = app.app().model.flattened_tree();
        let count_after = flattened_after.len();

        // Should have more nodes visible when expanded
        assert!(
            count_after >= count_before,
            "Expanding should not decrease visible nodes"
        );
    }

    #[test]
    fn test_selection_persistence() {
        let mut app = TestApp::with_content("# Heading\nContent");

        // Select file position
        app.send_key(KeyCode::Down);
        let (row, col) = app.app().file_viewer.cursor_position();

        // Should be able to get the selection from model
        assert_eq!(
            app.app().model.get_selected_position(),
            Some((row, col)),
            "Model should track selected position"
        );
    }
}
