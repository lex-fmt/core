//! Viewer trait and implementations
//!
//! The Viewer trait defines a common interface for UI components that:
//! - Render themselves given a model and area
//! - Handle keyboard input and return events
//!
//! This abstraction allows different viewers (FileViewer, TreeViewer) to
//! be treated uniformly by the main App.

use crate::model::{Model, NodeId};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use txxt_nano::txxt_nano::formats::treeviz::to_treeviz_str;

/// Events that can be emitted by viewers
///
/// These represent model changes that should be applied after handling input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewerEvent {
    /// Select a tree node
    SelectNode(NodeId),
    /// Select a text position
    SelectPosition(usize, usize),
    /// Toggle whether a node is expanded
    ToggleNodeExpansion(NodeId),
    /// No change to model
    NoChange,
}

/// Trait for UI viewers
///
/// A viewer is a component that:
/// - Knows how to render itself given a model
/// - Knows how to interpret keyboard input
/// - Emits ViewerEvents when user interactions require model changes
pub trait Viewer {
    /// Render this viewer to the given area
    fn render(&self, frame: &mut Frame, area: Rect, model: &Model);

    /// Handle a keyboard event and return the resulting event
    fn handle_key(&mut self, key: KeyEvent, model: &Model) -> Option<ViewerEvent>;
}

/// File viewer - displays and navigates the text content
///
/// The file viewer shows the raw text of the document with a cursor
/// that can be moved with arrow keys. When the cursor moves, it emits
/// a SelectPosition event so the model can track which AST node is selected.
#[derive(Debug)]
pub struct FileViewer {
    /// The raw file content
    content: String,
    /// Current cursor row (0-indexed)
    cursor_row: usize,
    /// Current cursor column (0-indexed)
    cursor_col: usize,
    /// How many lines are scrolled off the top of the viewport
    scroll_offset: usize,
}

impl FileViewer {
    /// Create a new file viewer with content
    pub fn new(content: String) -> Self {
        FileViewer {
            content,
            cursor_row: 0,
            cursor_col: 0,
            scroll_offset: 0,
        }
    }

    /// Get the current cursor position
    pub fn cursor_position(&self) -> (usize, usize) {
        (self.cursor_row, self.cursor_col)
    }

    /// Get the scroll offset
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Move cursor up
    fn move_cursor_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            // Adjust column if necessary for shorter lines
            self.clamp_cursor_column();
            self.ensure_cursor_visible();
        }
    }

    /// Move cursor down
    fn move_cursor_down(&mut self) {
        let lines: Vec<&str> = self.content.lines().collect();
        if self.cursor_row < lines.len().saturating_sub(1) {
            self.cursor_row += 1;
            // Adjust column if necessary for shorter lines
            self.clamp_cursor_column();
            self.ensure_cursor_visible();
        }
    }

    /// Move cursor left
    fn move_cursor_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        }
    }

    /// Move cursor right
    fn move_cursor_right(&mut self) {
        let lines: Vec<&str> = self.content.lines().collect();
        if self.cursor_row < lines.len() {
            let line_len = lines[self.cursor_row].len();
            if self.cursor_col < line_len {
                self.cursor_col += 1;
            }
        }
    }

    /// Clamp cursor column to valid range for current line
    fn clamp_cursor_column(&mut self) {
        let lines: Vec<&str> = self.content.lines().collect();
        if self.cursor_row < lines.len() {
            let line_len = lines[self.cursor_row].len();
            if self.cursor_col > line_len {
                self.cursor_col = line_len;
            }
        }
    }

    /// Ensure cursor is visible in the viewport
    fn ensure_cursor_visible(&mut self) {
        // Simple scrolling: keep cursor within viewport
        // For now, assume viewport is large enough
        // TODO: implement proper scrolling logic
    }
}

impl Default for FileViewer {
    fn default() -> Self {
        Self::new()
    }
}

impl Viewer for FileViewer {
    fn render(&self, frame: &mut Frame, area: Rect, _model: &Model) {
        // Display the file content line by line
        let lines: Vec<Line> = self
            .content
            .lines()
            .enumerate()
            .map(|(row, line_text)| {
                // For now, just display the text
                // Cursor highlighting will be added in a future step
                Line::from(line_text.to_string())
            })
            .collect();

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, area);
    }

    fn handle_key(&mut self, key: KeyEvent, _model: &Model) -> Option<ViewerEvent> {
        match key.code {
            KeyCode::Up => {
                self.move_cursor_up();
                Some(ViewerEvent::SelectPosition(self.cursor_row, self.cursor_col))
            }
            KeyCode::Down => {
                self.move_cursor_down();
                Some(ViewerEvent::SelectPosition(self.cursor_row, self.cursor_col))
            }
            KeyCode::Left => {
                self.move_cursor_left();
                Some(ViewerEvent::SelectPosition(self.cursor_row, self.cursor_col))
            }
            KeyCode::Right => {
                self.move_cursor_right();
                Some(ViewerEvent::SelectPosition(self.cursor_row, self.cursor_col))
            }
            _ => Some(ViewerEvent::NoChange),
        }
    }
}

/// Tree viewer - displays and navigates the AST structure
///
/// The tree viewer shows the document as a tree of nodes.
/// Users can navigate with arrow keys and expand/collapse nodes.
#[derive(Debug)]
pub struct TreeViewer {
    /// Currently selected node (by index in flattened tree)
    selected_index: usize,
    /// How many items are scrolled off the top
    scroll_offset: usize,
}

impl TreeViewer {
    /// Create a new tree viewer
    pub fn new() -> Self {
        TreeViewer {
            selected_index: 0,
            scroll_offset: 0,
        }
    }

    /// Get the currently selected node index
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Get the scroll offset
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }
}

impl Default for TreeViewer {
    fn default() -> Self {
        Self::new()
    }
}

impl Viewer for TreeViewer {
    fn render(&self, frame: &mut Frame, area: Rect, model: &Model) {
        // Use treeviz to render the document as an ASCII tree
        let tree_str = to_treeviz_str(&model.document);

        // Convert tree string to lines for rendering
        let lines: Vec<Line> = tree_str
            .lines()
            .map(|line_text| Line::from(line_text.to_string()))
            .collect();

        // Create a paragraph widget to display the tree
        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, area);
    }

    fn handle_key(&mut self, key: KeyEvent, _model: &Model) -> Option<ViewerEvent> {
        match key.code {
            KeyCode::Up => {
                // TODO: implement navigation through tree
                Some(ViewerEvent::NoChange)
            }
            KeyCode::Down => {
                // TODO: implement navigation through tree
                Some(ViewerEvent::NoChange)
            }
            KeyCode::Left => {
                // TODO: implement collapse
                Some(ViewerEvent::NoChange)
            }
            KeyCode::Right => {
                // TODO: implement expand
                Some(ViewerEvent::NoChange)
            }
            _ => Some(ViewerEvent::NoChange),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_viewer_creation() {
        let viewer = FileViewer::new();
        assert_eq!(viewer.cursor_position(), (0, 0));
        assert_eq!(viewer.scroll_offset(), 0);
    }

    #[test]
    fn test_tree_viewer_creation() {
        let viewer = TreeViewer::new();
        assert_eq!(viewer.selected_index(), 0);
        assert_eq!(viewer.scroll_offset(), 0);
    }
}
