//! Main application state and event handling
//!
//! The App struct brings together:
//! - Model (the application state)
//! - FileViewer and TreeViewer (the UI components)
//! - Focus management (which viewer has keyboard focus)
//! - Global key handling (quit, focus switching, delegating to viewers)

use super::fileviewer::FileViewer;
use super::model::{Focus, Model};
use super::treeviewer::TreeViewer;
use super::viewer::{Viewer, ViewerEvent};
use crossterm::event::KeyEvent;

/// The main application
pub struct App {
    /// The model holding document and selection state
    pub model: Model,

    /// File viewer (shows text content)
    pub file_viewer: FileViewer,

    /// Tree viewer (shows AST structure)
    pub tree_viewer: TreeViewer,

    /// Which viewer currently has focus
    pub focus: Focus,

    /// Whether the app should quit
    pub should_quit: bool,

    /// The raw file content
    #[allow(dead_code)]
    pub file_content: String,
}

impl App {
    /// Create a new application with a model and file content
    pub fn new(model: Model, content: String) -> Self {
        App {
            model,
            file_viewer: FileViewer::new(content.clone()),
            tree_viewer: TreeViewer::new(),
            focus: Focus::default(),
            should_quit: false,
            file_content: content,
        }
    }

    /// Toggle focus between viewers
    pub fn toggle_focus(&mut self) {
        self.focus = self.focus.toggle();
    }

    /// Handle a keyboard event
    ///
    /// Returns whether the state changed (needed for re-rendering)
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        use crossterm::event::{KeyCode, KeyModifiers};

        // Check for quit key (q or Ctrl+C)
        match key.code {
            KeyCode::Char('q') if key.modifiers.is_empty() => {
                self.should_quit = true;
                return true;
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
                return true;
            }
            KeyCode::Tab => {
                self.toggle_focus();
                return true;
            }
            _ => {}
        }

        // Delegate to focused viewer
        let event = match self.focus {
            Focus::FileViewer => self.file_viewer.handle_key(key, &self.model),
            Focus::TreeViewer => self.tree_viewer.handle_key(key, &self.model),
        };

        // Process the event if there is one
        if let Some(event) = event {
            self.process_viewer_event(event)
        } else {
            false
        }
    }

    /// Process a viewer event and update the model
    fn process_viewer_event(&mut self, event: ViewerEvent) -> bool {
        match event {
            ViewerEvent::SelectNode(node_id) => {
                self.model.select_node(node_id);
                // Auto-expand ancestors so the node is visible in the tree
                let ancestors = self.model.get_ancestors(node_id);
                self.model.expand_nodes(&ancestors);
                // Sync: update file viewer cursor to node's location if available
                self.sync_viewers_after_selection();
                true
            }
            ViewerEvent::SelectPosition(row, col) => {
                self.model.select_position(row, col);
                // Find which node is at this position and auto-expand its ancestors
                if let Some(node_id) = self.model.get_node_at_position(row, col) {
                    let ancestors = self.model.get_ancestors(node_id);
                    self.model.expand_nodes(&ancestors);
                }
                // Sync: update file viewer cursor to reflect selection
                self.sync_viewers_after_selection();
                true
            }
            ViewerEvent::ToggleNodeExpansion(node_id) => {
                self.model.toggle_node_expansion(node_id);
                true
            }
            ViewerEvent::NoChange => false,
        }
    }

    /// Synchronize viewer states after a selection change
    ///
    /// This ensures that when the model selection changes, the viewers are updated accordingly.
    /// Each viewer reads the model.selection and syncs its UI state to reflect it.
    fn sync_viewers_after_selection(&mut self) {
        use super::model::Selection;

        match self.model.selection() {
            Selection::TreeSelection(node_id) => {
                // User selected a node in tree
                // FileViewer's cursor should move to the node's text location
                if let Some(span) = self.model.get_span_for_node(node_id) {
                    self.file_viewer
                        .sync_cursor_to_position(span.start.line, span.start.column);
                }
            }
            Selection::TextSelection(row, col) => {
                // User moved cursor in text
                // FileViewer's cursor is already at this position (it moved itself)
                // Just ensure consistency
                self.file_viewer.sync_cursor_to_position(row, col);
            }
        }

        // TreeViewer doesn't need explicit sync - it highlights based on model selection
        // during render() by calling model.get_selected_node_id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let content = "test".to_string();
        let doc = txxt_nano::txxt_nano::parser::parse_document(&content).unwrap();
        let model = Model::new(doc);
        let app = App::new(model, content);

        assert_eq!(app.focus, Focus::FileViewer);
        assert!(!app.should_quit);
    }

    #[test]
    fn test_focus_toggle() {
        let content = "test".to_string();
        let doc = txxt_nano::txxt_nano::parser::parse_document(&content).unwrap();
        let model = Model::new(doc);
        let mut app = App::new(model, content);

        assert_eq!(app.focus, Focus::FileViewer);

        app.toggle_focus();
        assert_eq!(app.focus, Focus::TreeViewer);

        app.toggle_focus();
        assert_eq!(app.focus, Focus::FileViewer);
    }

    #[test]
    fn test_focus_enum_toggle() {
        let focus = Focus::FileViewer;
        assert_eq!(focus.toggle(), Focus::TreeViewer);

        let focus = Focus::TreeViewer;
        assert_eq!(focus.toggle(), Focus::FileViewer);
    }
}
