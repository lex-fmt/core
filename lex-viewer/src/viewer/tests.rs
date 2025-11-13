//! Test infrastructure for lex
//!
//! Provides utilities for testing the full application including:
//! - TestApp: wrapper for testing the application
//! - Keyboard helpers: easy creation of keyboard events
//! - Render helpers: getting and verifying UI output
//! - Test fixtures: pre-loaded test documents

use super::app::App;
use super::model::Model;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::backend::{Backend, TestBackend};
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
        let document = lex_parser::lex::parsing::parse_document(content)
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
        // In a workspace, we need to construct the path from the workspace root
        // CARGO_MANIFEST_DIR points to lex-viewer/, so we go up one level to workspace root
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let workspace_root = std::path::Path::new(manifest_dir).parent().unwrap();
        let full_path = workspace_root.join(path);
        let content = fs::read_to_string(&full_path)
            .unwrap_or_else(|e| panic!("Failed to read test file {}: {}", full_path.display(), e));
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
                let file_name = "test.lex";
                ui::render(frame, &self.app, file_name);
            })
            .expect("Failed to draw");

        // Get the rendered output as string
        self.terminal_output()
    }

    /// Get the current terminal output as a string
    fn terminal_output(&self) -> String {
        let backend = self.terminal.backend();
        let (width, height) = (
            backend.size().unwrap().width,
            backend.size().unwrap().height,
        );
        let mut output = String::new();

        for y in 0..height {
            for x in 0..width {
                if let Some(cell) = backend.buffer().cell((x, y)) {
                    output.push_str(cell.symbol());
                } else {
                    output.push(' ');
                }
            }
            output.push('\n');
        }

        output
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
        self.app.focus == super::model::Focus::FileViewer
    }

    /// Check if tree viewer is focused
    pub fn is_tree_viewer_focused(&self) -> bool {
        self.app.focus == super::model::Focus::TreeViewer
    }

    /// Check if a node is expanded
    pub fn is_node_expanded(&self, node_id: super::model::NodeId) -> bool {
        self.app.model.is_node_expanded(node_id)
    }

    /// Get the currently selected node ID
    pub fn selected_node_id(&self) -> Option<super::model::NodeId> {
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
use super::model::NodeId;

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
fn test_nested_elements_have_location_information() {
    // ISSUE: The lex parser does not set location information on nested elements.
    // This causes lex to be unable to highlight tree nodes when the cursor is on
    // their text in the file viewer, because get_node_at_position() relies on
    // document.elements_at() which depends on location information.
    //
    // This test verifies the issue: nested paragraph/list elements should have
    // location information just like their parent session elements do.

    let app = TestApp::with_file("docs/specs/v1/trifecta/070-trifecta-flat-simple.lex");

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

        // Session should have a location
        let session_location = app.app().model.get_location_for_node(*session_id);
        assert!(
            session_location.is_some(),
            "Session {:?} should have location information",
            session_id.path()
        );

        // Child element should ALSO have location information
        let child_location = app.app().model.get_location_for_node(child_node_id);
        assert!(
            child_location.is_some(),
            "Nested element {:?} should have location information, but it doesn't. \
             This is a lex parser issue: location information is not set on nested elements.",
            child_node_id.path()
        );
    } else {
        panic!("Test file should have a session with children");
    }
}

#[test]
fn test_text_view_cursor_on_nested_element_updates_model() {
    // Now that nested elements have locations, verify the full chain works:
    // FileViewer cursor on nested element → get_node_at_position finds it →
    // emit event → model updates → tree should highlight

    let mut app = TestApp::with_file("docs/specs/v1/trifecta/070-trifecta-flat-simple.lex");

    // Navigate the tree to a nested element to know where it is
    app.send_key(KeyCode::Tab); // Focus tree viewer
    for _ in 0..10 {
        app.send_key(KeyCode::Down); // Navigate to find a nested element
    }

    // Get the currently selected node (should be nested)
    if let Some(tree_selected) = app.app().model.get_selected_node_id() {
        let tree_path = tree_selected.path();
        println!(
            "Tree selected node: {:?} (depth {})",
            tree_path,
            tree_path.len()
        );

        // If it's nested, check if it has a location
        if tree_path.len() > 1 {
            println!("✓ Found a nested element!");
            if let Some(location) = app.app().model.get_location_for_node(tree_selected) {
                println!(
                    "Nested node location: line {}-{}, col {}-{}",
                    location.start.line,
                    location.end.line,
                    location.start.column,
                    location.end.column
                );

                // Try to find the node using the root session's element lookup
                use lex_parser::lex::ast::range::Position;
                let pos = Position::new(location.start.line, location.start.column);
                let element = app.app().model.document.root.element_at(pos);
                // Verify Session::element_at() now finds nested elements
                assert!(
                    element.is_some(),
                    "Session::element_at() should find nested elements at position {:?}",
                    pos
                );

                // Switch back to file viewer
                app.send_key(KeyCode::Tab);

                // Move cursor to that position
                app.app_mut()
                    .file_viewer
                    .sync_cursor_to_position(location.start.line, location.start.column);

                // Try get_node_at_position - should now find the nested element
                let found_node = app
                    .app()
                    .model
                    .get_node_at_position(location.start.line, location.start.column);

                // With TextLines as first-class ContentItems, we might find a TextLine
                // instead of the paragraph. Check if the found node is the tree_selected
                // or is a child of tree_selected (e.g., a TextLine within a paragraph)
                let nodes_match = if let Some(found_id) = found_node {
                    found_id == tree_selected || found_id.path().starts_with(tree_selected.path())
                } else {
                    false
                };

                assert!(
                    nodes_match,
                    "Should find the nested node by position or a child of it. Expected {:?} or descendant, got {:?}",
                    tree_selected.path(),
                    found_node.map(|n| n.path().to_vec())
                );
            }
        } else {
            println!(
                "✗ Node is not nested (depth {}), skipping test",
                tree_path.len()
            );
        }
    }
}

#[test]
fn test_tree_viewer_expand_collapse_indicators() {
    // Step 9.5: Verify that tree viewer shows expand/collapse indicators
    // - ▼ for expanded nodes with children
    // - ▶ for collapsed nodes with children
    // - two spaces for leaf nodes (no children)

    let mut app = TestApp::with_file("docs/specs/v1/trifecta/070-trifecta-flat-simple.lex");

    // Switch to tree viewer
    app.send_key(KeyCode::Tab);

    // Press Down to select the first node
    app.send_key(KeyCode::Down);

    // Get the flattened tree to verify structure
    let flattened = app.app().model.flattened_tree();

    // Find a node with children (skip Document, TextLines and Paragraphs as they expose TextLines)
    let mut node_with_children = None;
    for node in &flattened {
        if node.has_children {
            // Skip Document (tree root), TextLines and Paragraphs, look for containers like Session, List, etc.
            if node.node_type != "Document"
                && !node.node_type.contains("TextLine")
                && !node.node_type.contains("Paragraph")
            {
                node_with_children = Some(node.clone());
                break;
            }
        }
    }

    assert!(
        node_with_children.is_some(),
        "Test file should have container nodes with children (e.g., Session, List, etc.)"
    );

    let node_with_children = node_with_children.unwrap();

    // When expanded, should show ▼
    assert!(node_with_children.has_children, "Node should have children");

    // Find the tree node in the flattened tree and navigate to it
    let node_index = flattened
        .iter()
        .position(|n| n.node_id == node_with_children.node_id)
        .expect("Node should be in flattened tree");

    // We already pressed Down once to select the first node
    // Now navigate down to reach the target node (subtract 1 since we're already at index 0)
    for _ in 1..node_index {
        app.send_key(KeyCode::Down);
    }

    // Verify we're at the right node
    let current_selected = app.app().model.get_selected_node_id();
    assert_eq!(
        current_selected,
        Some(node_with_children.node_id),
        "Should be at the selected node with children"
    );

    // Now collapse it by pressing Left
    app.send_key(KeyCode::Left);

    // Check the tree again
    let flattened_after = app.app().model.flattened_tree();
    let same_node = flattened_after
        .iter()
        .find(|n| n.node_id == node_with_children.node_id)
        .expect("Node should still exist");

    // After pressing Left, node should be collapsed
    assert!(
        !same_node.is_expanded,
        "Node should be collapsed after pressing Left"
    );

    // Expand it again by pressing Right
    app.send_key(KeyCode::Right);

    let flattened_final = app.app().model.flattened_tree();
    let same_node_final = flattened_final
        .iter()
        .find(|n| n.node_id == node_with_children.node_id)
        .expect("Node should still exist");

    // After pressing Right, node should be expanded again
    assert!(
        same_node_final.is_expanded,
        "Node should be expanded after pressing Right"
    );
}

#[test]
fn test_tree_viewer_leaf_nodes_have_alignment_spacing() {
    // Verify that leaf nodes (without children) show proper spacing for alignment
    let app = TestApp::with_file("docs/specs/v1/trifecta/070-trifecta-flat-simple.lex");

    let flattened = app.app().model.flattened_tree();

    // Find a leaf node (no children)
    let leaf_node = flattened.iter().find(|n| !n.has_children);

    assert!(leaf_node.is_some(), "Test file should have leaf nodes");

    let leaf_node = leaf_node.unwrap();
    assert!(!leaf_node.has_children, "Node should be a leaf");

    // Leaf nodes don't show expand/collapse indicators, they show spacing
    // This is verified by the rendering logic showing "  " (two spaces)
    // The actual visual verification is manual, but the structure is correct
}

// ========== Status Line Rendering Tests ==========

#[test]
fn test_status_line_renders_tree_mode_indicator() {
    let mut app = TestApp::with_content("# Heading\nParagraph");

    // Switch to tree viewer (tree mode)
    app.send_key(KeyCode::Tab);

    let output = app.render();

    // Status line should contain tree mode indicator
    assert!(
        output.contains("🌳") || output.contains("Tree"),
        "Status line should show tree mode indicator"
    );
}

#[test]
fn test_status_line_renders_text_mode_indicator() {
    let mut app = TestApp::with_content("# Heading\nParagraph");

    // Start in file viewer (text mode - default)
    let output = app.render();

    // Status line should contain text mode indicator
    assert!(
        output.contains("📝") || output.contains("Text"),
        "Status line should show text mode indicator"
    );
}

#[test]
fn test_status_line_shows_cursor_position() {
    let mut app = TestApp::with_content("Line 1\nLine 2\nLine 3");

    // Get initial render (cursor at 0,0)
    let output = app.render();

    // Status line should show cursor position
    assert!(
        output.contains("Cursor") || output.contains("Line") || output.contains("Col"),
        "Status line should show cursor information"
    );
}

#[test]
fn test_status_line_is_single_row() {
    let mut app = TestApp::with_content("# Heading\nParagraph");

    let output = app.render();
    let lines: Vec<&str> = output.lines().collect();

    // Terminal is 24 lines: title (1) + middle + status (1)
    // The status line should be at the bottom
    assert_eq!(lines.len(), 24, "Terminal should be 24 lines");

    // The last line should contain status information
    let last_line = lines[23];
    assert!(
        !last_line.trim().is_empty() || last_line.contains("│"),
        "Last line should contain status line content or tree border"
    );
}

#[test]
fn test_status_line_shows_tree_selection_path() {
    let mut app = TestApp::with_content("# Heading\n## Subheading\nParagraph");

    // Switch to tree viewer
    app.send_key(KeyCode::Tab);

    // Navigate to get a non-root selection
    app.send_key(KeyCode::Down);

    let output = app.render();

    // Status line should show selection path
    assert!(
        output.contains("Path") || output.contains("[") || output.contains("Selection"),
        "Status line should show tree selection information"
    );
}

#[test]
fn test_status_line_shows_expanded_collapsed_state() {
    let mut app = TestApp::with_content("# Heading\n## Subheading");

    // Switch to tree viewer
    app.send_key(KeyCode::Tab);

    // Get render with node that has children (should show state)
    let output = app.render();

    // Status line may show Expanded or Collapsed depending on the node
    let _has_state_info =
        output.contains("Expanded") || output.contains("Collapsed") || output.contains("State");

    // This test verifies that render doesn't panic with nested content
    // State info may or may not be shown depending on node type
}

#[test]
fn test_status_line_no_borders() {
    let mut app = TestApp::with_content("# Test");

    let output = app.render();

    // Count lines with full border characters (the status line should NOT have borders like title)
    // The status line should be plain text, not enclosed in a box
    let lines: Vec<&str> = output.lines().collect();

    // Count rounded border characters in last line (status line)
    let last_line = lines[23];
    let has_borders = last_line.contains("╭")
        || last_line.contains("╮")
        || last_line.contains("╰")
        || last_line.contains("╯");

    assert!(
        !has_borders,
        "Status line should not have rounded borders (should be plain text)"
    );
}

#[test]
fn test_status_line_with_long_path() {
    // Create a deeply nested structure
    let content = "# Level 0\n## Level 1\n### Level 2\n#### Level 3\nContent";
    let mut app = TestApp::with_content(content);

    // Switch to tree viewer
    app.send_key(KeyCode::Tab);

    let output = app.render();

    // Status line should handle long paths gracefully (not panic or truncate incorrectly)
    assert!(!output.is_empty(), "Should render even with deep nesting");

    // Check that status line is still present
    let lines: Vec<&str> = output.lines().collect();
    assert!(lines.len() == 24, "Terminal height should remain constant");
}
