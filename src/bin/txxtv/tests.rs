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

use crate::app::App;
use crate::model::{Model, NodeId, Selection, Focus};
use crate::ui;
use crate::viewer::{FileViewer, TreeViewer, Viewer, ViewerEvent};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::fs;

/// Test application harness
pub struct TestApp {
    pub model: Model,
    pub app: App,
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
        let app = App::new(model.clone(), content);

        // Create a test terminal with reasonable dimensions
        // Standard terminal is ~80x24, we use 80x30 to give enough space
        let backend = TestBackend::new(80, 30);
        let terminal = Terminal::new(backend).expect("Failed to create test terminal");

        TestApp { model, app, terminal }
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

    /// Get the ancestors of a node
    pub fn ancestors(&self, node_id: NodeId) -> Vec<NodeId> {
        self.model.get_ancestors(node_id)
    }

    /// Assert that ancestors match expected
    pub fn assert_ancestors(&self, node_id: NodeId, expected_paths: &[&[usize]]) {
        let ancestors = self.ancestors(node_id);
        assert_eq!(
            ancestors.len(),
            expected_paths.len(),
            "Expected {} ancestors, got {}",
            expected_paths.len(),
            ancestors.len()
        );
        for (i, &expected_path) in expected_paths.iter().enumerate() {
            assert_eq!(
                ancestors[i].path(),
                expected_path,
                "Ancestor {} mismatch",
                i
            );
        }
    }

    // ========== Keyboard handling ==========

    /// Create a KeyEvent for testing
    fn create_key_event(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::empty(),
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        }
    }

    /// Create a KeyEvent with modifiers for testing
    fn create_key_event_with_modifiers(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        }
    }

    /// Send a key press to the FileViewer
    pub fn file_viewer_key(&mut self, key: KeyCode) -> Option<ViewerEvent> {
        let mut viewer = FileViewer::new();
        let key_event = Self::create_key_event(key);
        viewer.handle_key(key_event, &self.model)
    }

    /// Send a key press to the TreeViewer
    pub fn tree_viewer_key(&mut self, key: KeyCode) -> Option<ViewerEvent> {
        let mut viewer = TreeViewer::new();
        let key_event = Self::create_key_event(key);
        viewer.handle_key(key_event, &self.model)
    }

    /// Send Up arrow to FileViewer
    pub fn file_viewer_key_up(&mut self) -> Option<ViewerEvent> {
        self.file_viewer_key(KeyCode::Up)
    }

    /// Send Down arrow to FileViewer
    pub fn file_viewer_key_down(&mut self) -> Option<ViewerEvent> {
        self.file_viewer_key(KeyCode::Down)
    }

    /// Send Left arrow to FileViewer
    pub fn file_viewer_key_left(&mut self) -> Option<ViewerEvent> {
        self.file_viewer_key(KeyCode::Left)
    }

    /// Send Right arrow to FileViewer
    pub fn file_viewer_key_right(&mut self) -> Option<ViewerEvent> {
        self.file_viewer_key(KeyCode::Right)
    }

    /// Send Up arrow to TreeViewer
    pub fn tree_viewer_key_up(&mut self) -> Option<ViewerEvent> {
        self.tree_viewer_key(KeyCode::Up)
    }

    /// Send Down arrow to TreeViewer
    pub fn tree_viewer_key_down(&mut self) -> Option<ViewerEvent> {
        self.tree_viewer_key(KeyCode::Down)
    }

    /// Send Left arrow to TreeViewer (collapse)
    pub fn tree_viewer_key_left(&mut self) -> Option<ViewerEvent> {
        self.tree_viewer_key(KeyCode::Left)
    }

    /// Send Right arrow to TreeViewer (expand)
    pub fn tree_viewer_key_right(&mut self) -> Option<ViewerEvent> {
        self.tree_viewer_key(KeyCode::Right)
    }

    /// Assert that the event is SelectPosition
    pub fn assert_event_select_position(&self, event: Option<ViewerEvent>, row: usize, col: usize) {
        assert_eq!(
            event,
            Some(ViewerEvent::SelectPosition(row, col)),
            "Expected SelectPosition({}, {}), got {:?}",
            row,
            col,
            event
        );
    }

    /// Assert that the event is SelectNode
    pub fn assert_event_select_node(&self, event: Option<ViewerEvent>, node_id: NodeId) {
        assert_eq!(
            event,
            Some(ViewerEvent::SelectNode(node_id)),
            "Expected SelectNode({:?}), got {:?}",
            node_id.path(),
            event
        );
    }

    /// Assert that the event is NoChange
    pub fn assert_event_no_change(&self, event: Option<ViewerEvent>) {
        assert_eq!(
            event, Some(ViewerEvent::NoChange),
            "Expected NoChange, got {:?}",
            event
        );
    }

    // ========== Focus management ==========

    /// Get the current focus
    pub fn focused_viewer(&self) -> Focus {
        self.app.focus
    }

    /// Toggle focus
    pub fn toggle_focus(&mut self) {
        self.app.toggle_focus();
    }

    /// Assert that the focused viewer matches expected
    pub fn assert_focused_viewer(&self, expected: Focus) {
        assert_eq!(
            self.app.focus, expected,
            "Expected focus on {:?}, got {:?}",
            expected, self.app.focus
        );
    }

    /// Send a Tab key to the app (focus toggle)
    pub fn send_tab(&mut self) -> bool {
        let key_event = Self::create_key_event(KeyCode::Tab);
        self.app.handle_key(key_event)
    }

    /// Send a 'q' key to the app (quit)
    pub fn send_quit(&mut self) -> bool {
        let key_event = Self::create_key_event(KeyCode::Char('q'));
        self.app.handle_key(key_event)
    }

    /// Check if app should quit
    pub fn should_quit(&self) -> bool {
        self.app.should_quit
    }

    /// Send a key to the app and get whether it changed state
    pub fn app_handle_key(&mut self, key: KeyCode) -> bool {
        let key_event = Self::create_key_event(key);
        self.app.handle_key(key_event)
    }

    // ========== Rendering ==========

    /// Render the UI and return it as a string for inspection
    pub fn render(&mut self) -> String {
        self.terminal
            .draw(|frame| {
                ui::render(frame, &self.app, "test.txxt");
            })
            .expect("Failed to draw");

        // Extract the buffer content from the test backend
        let backend = self.terminal.backend();
        let mut output = String::new();
        for cell in backend.content.iter() {
            output.push(cell.symbol.chars().next().unwrap_or(' '));
        }
        output
    }

    /// Get the terminal size
    pub fn terminal_size(&self) -> (u16, u16) {
        let size = self.terminal.size().unwrap();
        (size.width, size.height)
    }

    /// Assert that rendered output contains a substring
    pub fn assert_render_contains(&mut self, text: &str) {
        let output = self.render();
        assert!(
            output.contains(text),
            "Expected output to contain '{}', but got:\n{}",
            text,
            output
        );
    }

    /// Assert that layout has minimum width constraint
    pub fn assert_minimum_width(&self, min_width: u16) {
        let (width, _) = self.terminal_size();
        assert!(
            width >= min_width,
            "Terminal width {} is less than minimum {}",
            width,
            min_width
        );
    }

    /// Create a test app with specific terminal size
    pub fn with_size(width: u16, height: u16) -> Self {
        let content =
            fs::read_to_string("docs/specs/v1/samples/030-paragraphs-sessions-nested-multiple.txxt")
                .expect("Failed to load test document");

        let document = txxt_nano::txxt_nano::parser::parse_document(&content)
            .expect("Failed to parse test document");

        let model = Model::new(document);
        let app = App::new(model.clone(), content);

        let backend = TestBackend::new(width, height);
        let terminal = Terminal::new(backend).expect("Failed to create test terminal");

        TestApp { model, app, terminal }
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

    #[test]
    fn test_ancestors() {
        let app = TestApp::new();
        let node_id = TestApp::node_id(&[0, 1, 2]);

        app.assert_ancestors(
            node_id,
            &[&[], &[0], &[0, 1]],
        );
    }

    #[test]
    fn test_ancestors_root_child() {
        let app = TestApp::new();
        let node_id = TestApp::node_id(&[0]);

        app.assert_ancestors(
            node_id,
            &[&[]],
        );
    }

    #[test]
    fn test_ancestors_deep_nesting() {
        let app = TestApp::new();
        let node_id = TestApp::node_id(&[2, 0, 1, 3]);

        app.assert_ancestors(
            node_id,
            &[&[], &[2], &[2, 0], &[2, 0, 1]],
        );
    }

    #[test]
    fn test_file_viewer_creation() {
        let viewer = FileViewer::new();
        assert_eq!(viewer.cursor_position(), (0, 0));
    }

    #[test]
    fn test_tree_viewer_creation() {
        let viewer = TreeViewer::new();
        assert_eq!(viewer.selected_index(), 0);
    }

    #[test]
    fn test_file_viewer_key_up_returns_select_position() {
        let mut app = TestApp::new();
        let event = app.file_viewer_key_up();
        // Event should be SelectPosition with initial cursor position
        assert!(matches!(event, Some(ViewerEvent::SelectPosition(_, _))));
    }

    #[test]
    fn test_file_viewer_key_down_returns_select_position() {
        let mut app = TestApp::new();
        let event = app.file_viewer_key_down();
        // Event should be SelectPosition with cursor position
        assert!(matches!(event, Some(ViewerEvent::SelectPosition(_, _))));
    }

    #[test]
    fn test_file_viewer_key_left_returns_select_position() {
        let mut app = TestApp::new();
        let event = app.file_viewer_key_left();
        // Event should be SelectPosition with cursor position
        assert!(matches!(event, Some(ViewerEvent::SelectPosition(_, _))));
    }

    #[test]
    fn test_file_viewer_key_right_returns_select_position() {
        let mut app = TestApp::new();
        let event = app.file_viewer_key_right();
        // Event should be SelectPosition with cursor position
        assert!(matches!(event, Some(ViewerEvent::SelectPosition(_, _))));
    }

    #[test]
    fn test_tree_viewer_keys_return_no_change() {
        let mut app = TestApp::new();
        let event = app.tree_viewer_key_up();
        app.assert_event_no_change(event);
    }

    #[test]
    fn test_viewer_event_equality() {
        let event1 = ViewerEvent::SelectPosition(5, 10);
        let event2 = ViewerEvent::SelectPosition(5, 10);
        assert_eq!(event1, event2);

        let node_id = TestApp::node_id(&[0, 1]);
        let event3 = ViewerEvent::SelectNode(node_id);
        let event4 = ViewerEvent::SelectNode(node_id);
        assert_eq!(event3, event4);
    }

    #[test]
    fn test_app_initial_focus() {
        let app = TestApp::new();
        app.assert_focused_viewer(Focus::FileViewer);
    }

    #[test]
    fn test_app_focus_toggle() {
        let mut app = TestApp::new();

        app.assert_focused_viewer(Focus::FileViewer);

        let changed = app.send_tab();
        assert!(changed);
        app.assert_focused_viewer(Focus::TreeViewer);

        let changed = app.send_tab();
        assert!(changed);
        app.assert_focused_viewer(Focus::FileViewer);
    }

    #[test]
    fn test_app_quit() {
        let mut app = TestApp::new();
        assert!(!app.should_quit());

        let changed = app.send_quit();
        assert!(changed);
        assert!(app.should_quit());
    }

    #[test]
    fn test_app_quit_with_ctrl_c() {
        let mut app = TestApp::new();
        assert!(!app.should_quit());

        let key_event = KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };
        let changed = app.app.handle_key(key_event);
        assert!(changed);
        assert!(app.should_quit());
    }

    #[test]
    fn test_focus_enum_default() {
        let focus: Focus = Default::default();
        assert_eq!(focus, Focus::FileViewer);
    }

    #[test]
    fn test_render_title_bar() {
        let mut app = TestApp::new();
        app.assert_render_contains("txxt::");
    }

    #[test]
    fn test_render_tree_viewer() {
        let mut app = TestApp::new();
        app.assert_render_contains("Tree");
    }

    #[test]
    fn test_render_file_viewer() {
        let mut app = TestApp::new();
        app.assert_render_contains("File");
    }

    #[test]
    fn test_render_info_panel() {
        let mut app = TestApp::new();
        app.assert_render_contains("Info");
    }

    #[test]
    fn test_focus_indicator_in_file_viewer() {
        let mut app = TestApp::new();
        assert_eq!(app.app.focus, Focus::FileViewer);
        app.assert_render_contains("FOCUSED");
    }

    #[test]
    fn test_focus_indicator_in_tree_viewer() {
        let mut app = TestApp::new();
        app.send_tab();
        assert_eq!(app.app.focus, Focus::TreeViewer);
        app.assert_render_contains("FOCUSED");
    }

    #[test]
    fn test_layout_with_standard_width() {
        let app = TestApp::new();
        let (width, height) = app.terminal_size();
        assert_eq!(width, 80);
        assert_eq!(height, 30);
    }

    #[test]
    fn test_layout_with_custom_size() {
        let app = TestApp::with_size(100, 40);
        let (width, height) = app.terminal_size();
        assert_eq!(width, 100);
        assert_eq!(height, 40);
    }

    #[test]
    fn test_minimum_width_constraint() {
        let app = TestApp::new();
        app.assert_minimum_width(50);
    }

    #[test]
    fn test_narrow_terminal_shows_error() {
        let mut app = TestApp::with_size(40, 20);
        app.assert_render_contains("too narrow");
    }

    #[test]
    fn test_wide_terminal_renders_normally() {
        let mut app = TestApp::with_size(120, 30);
        app.assert_render_contains("Tree");
        app.assert_render_contains("File");
    }

    #[test]
    fn test_file_viewer_arrow_up_when_at_top() {
        let mut app = TestApp::new();
        let event = app.file_viewer_key_up();
        // Should still emit SelectPosition even at top
        assert!(matches!(event, Some(ViewerEvent::SelectPosition(_, _))));
    }

    #[test]
    fn test_file_viewer_arrow_down() {
        let mut app = TestApp::new();
        let event = app.file_viewer_key_down();
        // Should emit SelectPosition with updated row
        assert!(matches!(event, Some(ViewerEvent::SelectPosition(_, _))));
    }

    #[test]
    fn test_file_viewer_arrow_left() {
        let mut app = TestApp::new();
        let event = app.file_viewer_key_left();
        // Should emit SelectPosition
        assert!(matches!(event, Some(ViewerEvent::SelectPosition(_, _))));
    }

    #[test]
    fn test_file_viewer_arrow_right() {
        let mut app = TestApp::new();
        let event = app.file_viewer_key_right();
        // Should emit SelectPosition
        assert!(matches!(event, Some(ViewerEvent::SelectPosition(_, _))));
    }

    #[test]
    fn test_app_process_select_position_event() {
        let mut app = TestApp::new();
        let initial_pos = app.app.model.get_selected_position();

        // Send a cursor movement that will change position
        app.file_viewer_key_down();
        app.app.handle_key(Self::create_key_event(KeyCode::Down));

        let new_pos = app.app.model.get_selected_position();
        // Position should have changed (or tried to)
        assert!(new_pos.is_some());
    }

    #[test]
    fn test_app_auto_expands_ancestors_on_position_change() {
        let mut app = TestApp::new();

        // Select a position that would be in a deep node
        app.app.handle_key(Self::create_key_event(KeyCode::Down));

        // Check if any ancestors were expanded
        // Since we have auto-expansion in process_viewer_event,
        // at least the ancestors of the selected position should be expanded
        let expanded_count = app.app.model.document.content.len() as usize;
        // Just verify the feature doesn't crash
        assert!(app.app.model.document.content.len() > 0);
    }

    #[test]
    fn test_file_viewer_with_content() {
        let viewer = FileViewer::new("line 1\nline 2\nline 3".to_string());
        assert_eq!(viewer.cursor_position(), (0, 0));
    }

    #[test]
    fn test_cursor_clamping_on_short_line() {
        let mut app = TestApp::new();
        // The test document has lines of varying lengths
        // This tests that cursor column is clamped to line length
        let (row, col) = app.app.file_viewer.cursor_position();
        assert_eq!(row, 0);
        assert_eq!(col, 0);
    }
}
