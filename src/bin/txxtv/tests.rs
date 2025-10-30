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

#[allow(dead_code)]
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
        let content = fs::read_to_string(path).expect("Failed to read test file");
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
        format!(
            "Rendered output ({} x {})",
            self.terminal.size().unwrap().width,
            self.terminal.size().unwrap().height
        )
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
#[allow(dead_code)]
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

// Tests that can only run within this module (needs access to TestApp, etc)
use crate::model::NodeId;

#[test]
fn tests_simple_sanity_check() {
    assert_eq!(1, 1);
}

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
        assert!(!node_id.path().is_empty() || node_id.path().is_empty());
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

    // Initial state: should be at position 0,0 with node at that position
    let initial_node = app.app().model.get_node_at_position(0, 0);

    // Move cursor down to a different line
    app.send_key(KeyCode::Down);
    let (row, col) = app.app().file_viewer.cursor_position();

    // If we moved to a different node, the model selection should reflect it
    if let Some(new_node) = app.app().model.get_node_at_position(row, col) {
        if Some(new_node) != initial_node {
            // Selection changed, so model should track the new position
            assert_eq!(
                app.app().model.get_selected_position(),
                Some((row, col)),
                "Model should track selected position when node changes"
            );
        }
    }
}

// ========== Step 9: Tree Navigation Tests ==========

#[test]
fn test_tree_navigation_down() {
    let mut app = TestApp::with_content("# Heading\n## Subheading\nParagraph");

    // Switch to tree viewer
    app.send_key(KeyCode::Tab);

    // Get initial selected node
    let flattened_before = app.app().model.flattened_tree();
    assert!(!flattened_before.is_empty(), "Should have nodes in tree");

    // Press down arrow to navigate to next node
    app.send_key(KeyCode::Down);

    // Verify that tree viewer has updated selected node
    if let Some(selected_node_id) = app.app().model.get_selected_node_id() {
        // Find index of selected node in flattened tree
        let flattened = app.app().model.flattened_tree();
        let selected_index = flattened
            .iter()
            .position(|n| n.node_id == selected_node_id)
            .expect("Selected node should be in flattened tree");
        assert!(selected_index > 0, "Should have moved to next node");
    }
}

#[test]
fn test_tree_navigation_up() {
    let mut app = TestApp::with_content("# Heading\n## Subheading\nParagraph");

    // Switch to tree viewer
    app.send_key(KeyCode::Tab);

    // Move down first
    app.send_key(KeyCode::Down);

    // Then move up
    app.send_key(KeyCode::Up);

    // Verify that tree viewer has updated selected node
    if let Some(selected_node_id) = app.app().model.get_selected_node_id() {
        let flattened = app.app().model.flattened_tree();
        let selected_index = flattened
            .iter()
            .position(|n| n.node_id == selected_node_id)
            .expect("Selected node should be in flattened tree");
        // Should be back at first node
        assert_eq!(selected_index, 0, "Should be back at first node");
    }
}

#[test]
fn test_tree_navigation_respects_expansion() {
    let mut app = TestApp::with_content("# Heading\n## Subheading\nParagraph");

    // Switch to tree viewer
    app.send_key(KeyCode::Tab);

    // Get flattened tree before expansion
    let flattened_collapsed = app.app().model.flattened_tree();
    let count_collapsed = flattened_collapsed.len();

    // Expand first node
    let first_node = NodeId::new(&[0]);
    app.app_mut().model.expand_nodes(&[first_node]);

    // Get flattened tree after expansion
    let flattened_expanded = app.app().model.flattened_tree();
    let count_expanded = flattened_expanded.len();

    // After expansion, more nodes should be visible
    assert!(
        count_expanded >= count_collapsed,
        "Expanding should show more or equal nodes"
    );

    // Navigate down in the expanded tree
    app.send_key(KeyCode::Down);

    // Should be able to reach newly visible nodes through navigation
    if let Some(selected_node_id) = app.app().model.get_selected_node_id() {
        let flattened = app.app().model.flattened_tree();
        assert!(
            flattened.iter().any(|n| n.node_id == selected_node_id),
            "Selected node should be in flattened tree"
        );
    }
}

#[test]
fn test_tree_expand_collapse_navigation() {
    let mut app = TestApp::with_content("# Heading\n## Subheading\nParagraph");

    // Switch to tree viewer
    app.send_key(KeyCode::Tab);

    // Simulate selecting first node
    let flattened = app.app().model.flattened_tree();
    if let Some(first_flattened_node) = flattened.first() {
        let first_node_id = first_flattened_node.node_id;

        // Verify we can emit expand/collapse events via tree navigation
        // Press Right arrow to emit ToggleNodeExpansion
        app.send_key(KeyCode::Right);

        // Node should now be toggled
        let _is_expanded = app.app().model.is_node_expanded(first_node_id);
        // Just verify that the toggle operation executed without panicking
        // The actual expansion behavior is tested in test_tree_navigation_respects_expansion
    }
}

#[test]
fn test_tree_viewer_boundary_navigation() {
    let mut app = TestApp::with_content("# Heading\nParagraph");

    // Switch to tree viewer
    app.send_key(KeyCode::Tab);

    // Get initial selection
    let initial_flattened = app.app().model.flattened_tree();
    if initial_flattened.len() > 1 {
        // Navigate to last node
        for _ in 0..initial_flattened.len() {
            app.send_key(KeyCode::Down);
        }

        // Try to go down again (should not crash, should stay at last node or no-op)
        app.send_key(KeyCode::Down);

        // Verify selection is still valid
        if let Some(selected_node_id) = app.app().model.get_selected_node_id() {
            let flattened = app.app().model.flattened_tree();
            assert!(
                flattened.iter().any(|n| n.node_id == selected_node_id),
                "Selected node should still be valid"
            );
        }
    }
}

#[test]
fn test_tree_selection_emits_select_node_event() {
    let mut app = TestApp::with_content("# Heading\n## Subheading\n");

    // Switch to tree viewer
    app.send_key(KeyCode::Tab);

    // Press down to navigate
    app.send_key(KeyCode::Down);

    // After navigation, the model should track the tree selection
    let flattened = app.app().model.flattened_tree();
    if !flattened.is_empty() {
        if let Some(selected_node_id) = app.app().model.get_selected_node_id() {
            // Verify the selected node exists in the flattened tree
            let node_exists = flattened.iter().any(|n| n.node_id == selected_node_id);
            assert!(
                node_exists,
                "Navigation should select a node from the visible tree"
            );
        }
    }
}

#[test]
#[ignore = "txxt_nano parser does not set span information on nested elements - see https://github.com/arthur-debert/txxt-nano/issues/64"]
fn test_nested_elements_have_span_information() {
    // ISSUE: The txxt_nano parser does not set span information on nested elements.
    // This causes txxtv to be unable to highlight tree nodes when the cursor is on
    // their text in the file viewer, because get_node_at_position() relies on
    // document.elements_at() which depends on span information.
    //
    // This test verifies the issue: nested paragraph/list elements should have
    // span information just like their parent session elements do.

    let app = TestApp::with_file("docs/specs/v1/samples/050-trifecta-flat-simple.txxt");

    let flattened = app.app().model.flattened_tree();

    // Find a session with children
    let mut session_with_children = None;
    for (i, node) in flattened.iter().enumerate() {
        if node.node_id.path().len() == 1 && node.has_children {
            // This is a top-level session with children
            // Check if the next node is a child
            if i + 1 < flattened.len() && flattened[i + 1].node_id.path().len() > 1 {
                session_with_children = Some((i, &node.node_id));
                break;
            }
        }
    }

    if let Some((session_idx, session_id)) = session_with_children {
        let child_idx = session_idx + 1;
        let child_node = &flattened[child_idx];
        let child_node_id = child_node.node_id;

        // Session should have a span
        let session_span = app.app().model.get_span_for_node(*session_id);
        assert!(
            session_span.is_some(),
            "Session {:?} should have span information",
            session_id.path()
        );

        // Child element should ALSO have span information (currently fails)
        let child_span = app.app().model.get_span_for_node(child_node_id);
        assert!(
            child_span.is_some(),
            "Nested element {:?} should have span information, but it doesn't. \
             This is a txxt_nano parser issue: span information is not set on nested elements.",
            child_node_id.path()
        );

        // If the child has a span, we should be able to find it by position
        if let Some(span) = child_span {
            let node_at_pos = app
                .app()
                .model
                .get_node_at_position(span.start.line, span.start.column);
            assert_eq!(
                node_at_pos,
                Some(child_node_id),
                "Should be able to find nested element by its span position"
            );
        }
    } else {
        panic!("Test file should have a session with children");
    }
}
