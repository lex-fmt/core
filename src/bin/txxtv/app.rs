//! Main application state and event handling
//!
//! The App struct brings together:
//! - Model (the application state)
//! - FileViewer and TreeViewer (the UI components)
//! - Focus management (which viewer has keyboard focus)
//! - Global key handling (quit, focus switching, delegating to viewers)

use crate::model::{Focus, Model};
use crate::viewer::{FileViewer, TreeViewer, Viewer, ViewerEvent};
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
                true
            }
            ViewerEvent::SelectPosition(row, col) => {
                self.model.select_position(row, col);
                // Find which node is at this position and auto-expand its ancestors
                if let Some(node_id) = self.model.get_node_at_position(row, col) {
                    let ancestors = self.model.get_ancestors(node_id);
                    self.model.expand_nodes(&ancestors);
                }
                true
            }
            ViewerEvent::ToggleNodeExpansion(node_id) => {
                self.model.toggle_node_expansion(node_id);
                true
            }
            ViewerEvent::NoChange => false,
        }
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
