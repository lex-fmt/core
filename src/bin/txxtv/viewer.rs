//! Viewer trait and implementations
//!
//! The Viewer trait defines a common interface for UI components that:
//! - Render themselves given a model and area
//! - Handle keyboard input and return events
//!
//! This abstraction allows different viewers (FileViewer, TreeViewer) to
//! be treated uniformly by the main App.

use crate::model::{Model, NodeId};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

/// Events that can be emitted by viewers
///
/// These represent model changes that should be applied after handling input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
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
#[allow(dead_code)]
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

#[allow(dead_code)]
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

    /// Sync cursor to a specific position (called when model selection changes)
    pub fn sync_cursor_to_position(&mut self, row: usize, col: usize) {
        self.cursor_row = row;
        self.cursor_col = col;
        self.clamp_cursor_column();
        self.ensure_cursor_visible();
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

impl Viewer for FileViewer {
    fn render(&self, frame: &mut Frame, area: Rect, _model: &Model) {
        use ratatui::style::{Color, Modifier, Style};
        use ratatui::text::Span;

        // Display the file content line by line, highlighting cursor position
        let lines: Vec<Line> = self
            .content
            .lines()
            .enumerate()
            .map(|(row_idx, line_text)| {
                if row_idx == self.cursor_row {
                    // This is the row with the cursor - render with cursor highlight
                    let mut spans = Vec::new();
                    let chars: Vec<char> = line_text.chars().collect();

                    for (col_idx, ch) in chars.iter().enumerate() {
                        if col_idx == self.cursor_col {
                            // Highlight the cursor character
                            spans.push(Span::styled(
                                ch.to_string(),
                                Style::default()
                                    .bg(Color::Yellow)
                                    .fg(Color::Black)
                                    .add_modifier(Modifier::BOLD),
                            ));
                        } else {
                            spans.push(Span::raw(ch.to_string()));
                        }
                    }

                    // Handle case where cursor is at end of line
                    if self.cursor_col >= chars.len() {
                        spans.push(Span::styled(
                            " ",
                            Style::default().bg(Color::Yellow).fg(Color::Black),
                        ));
                    }

                    Line::from(spans)
                } else {
                    // Regular line without cursor
                    Line::from(line_text.to_string())
                }
            })
            .collect();

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, area);
    }

    fn handle_key(&mut self, key: KeyEvent, model: &Model) -> Option<ViewerEvent> {
        // Get the node at the current position before moving
        let old_node = model.get_node_at_position(self.cursor_row, self.cursor_col);

        match key.code {
            KeyCode::Up => {
                self.move_cursor_up();
            }
            KeyCode::Down => {
                self.move_cursor_down();
            }
            KeyCode::Left => {
                self.move_cursor_left();
            }
            KeyCode::Right => {
                self.move_cursor_right();
            }
            _ => return Some(ViewerEvent::NoChange),
        }

        // Get the node at the new position after moving
        let new_node = model.get_node_at_position(self.cursor_row, self.cursor_col);

        // Only emit event if the node actually changed
        if old_node != new_node {
            Some(ViewerEvent::SelectPosition(
                self.cursor_row,
                self.cursor_col,
            ))
        } else {
            Some(ViewerEvent::NoChange)
        }
    }
}

/// Tree viewer - displays and navigates the AST structure
///
/// The tree viewer shows the document as a tree of nodes.
/// Users can navigate with arrow keys and expand/collapse nodes.
#[derive(Debug)]
#[allow(dead_code)]
pub struct TreeViewer {
    /// Currently selected node ID
    selected_node_id: Option<NodeId>,
    /// How many items are scrolled off the top
    scroll_offset: usize,
}

#[allow(dead_code)]
impl TreeViewer {
    /// Create a new tree viewer
    pub fn new() -> Self {
        TreeViewer {
            selected_node_id: None,
            scroll_offset: 0,
        }
    }

    /// Get the currently selected node ID
    pub fn selected_node_id(&self) -> Option<NodeId> {
        self.selected_node_id
    }

    /// Get the scroll offset
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Get the next visible node in the flattened tree
    ///
    /// Returns the NodeId of the next node considering only visible nodes
    /// (respecting expansion state). Returns None if already at the last node.
    pub fn get_next_visible_node(&self, current_node_id: NodeId, model: &Model) -> Option<NodeId> {
        let flattened = model.flattened_tree();

        // Find the current node in the flattened tree
        if let Some(current_index) = flattened.iter().position(|n| n.node_id == current_node_id) {
            // Return the next node if it exists
            if current_index < flattened.len() - 1 {
                return Some(flattened[current_index + 1].node_id);
            }
        }

        None
    }

    /// Get the previous visible node in the flattened tree
    ///
    /// Returns the NodeId of the previous node considering only visible nodes
    /// (respecting expansion state). Returns None if already at the first node.
    pub fn get_previous_visible_node(
        &self,
        current_node_id: NodeId,
        model: &Model,
    ) -> Option<NodeId> {
        let flattened = model.flattened_tree();

        // Find the current node in the flattened tree
        if let Some(current_index) = flattened.iter().position(|n| n.node_id == current_node_id) {
            // Return the previous node if it exists
            if current_index > 0 {
                return Some(flattened[current_index - 1].node_id);
            }
        }

        None
    }
}

impl Default for TreeViewer {
    fn default() -> Self {
        Self::new()
    }
}

impl Viewer for TreeViewer {
    fn render(&self, frame: &mut Frame, area: Rect, model: &Model) {
        use crate::model::Selection;
        use ratatui::style::{Color, Modifier, Style};

        // Get the flattened tree for rendering
        let flattened = model.flattened_tree();

        // Determine which node should be highlighted
        // This handles both TreeSelection and TextSelection types
        let highlighted_node_id = match model.selection() {
            Selection::TreeSelection(node_id) => Some(node_id),
            Selection::TextSelection(row, col) => model.get_node_at_position(row, col),
        };

        // Build lines from the flattened tree
        let lines: Vec<Line> = flattened
            .iter()
            .map(|node| {
                // Build indentation based on depth
                let indent = "  ".repeat(node.depth);

                // Build the node label with tree characters
                // Note: In Step 9.5, this will be refined to show different prefixes
                // based on expansion state (e.g., "▼ " vs "▶ " for expanded/collapsed)
                let prefix = if node.has_children {
                    "├─ "
                } else {
                    "└─ "
                };

                let text = format!("{}{}{}", indent, prefix, node.label);

                // Style the line - highlight if it matches the current selection
                if Some(node.node_id) == highlighted_node_id {
                    Line::from(text).style(
                        Style::default()
                            .bg(Color::Blue)
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    Line::from(text)
                }
            })
            .collect();

        // Create a paragraph widget to display the tree
        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, area);
    }

    fn handle_key(&mut self, key: KeyEvent, model: &Model) -> Option<ViewerEvent> {
        // Initialize selection to first visible node if not yet selected
        if self.selected_node_id.is_none() {
            let flattened = model.flattened_tree();
            if !flattened.is_empty() {
                self.selected_node_id = Some(flattened[0].node_id);
            }
        }

        let Some(current_node_id) = self.selected_node_id else {
            return Some(ViewerEvent::NoChange);
        };

        match key.code {
            KeyCode::Up => {
                // Move to previous visible node
                if let Some(prev_node_id) = self.get_previous_visible_node(current_node_id, model) {
                    self.selected_node_id = Some(prev_node_id);
                    Some(ViewerEvent::SelectNode(prev_node_id))
                } else {
                    Some(ViewerEvent::NoChange)
                }
            }
            KeyCode::Down => {
                // Move to next visible node
                if let Some(next_node_id) = self.get_next_visible_node(current_node_id, model) {
                    self.selected_node_id = Some(next_node_id);
                    Some(ViewerEvent::SelectNode(next_node_id))
                } else {
                    Some(ViewerEvent::NoChange)
                }
            }
            KeyCode::Left => {
                // Toggle collapse for the currently selected node
                Some(ViewerEvent::ToggleNodeExpansion(current_node_id))
            }
            KeyCode::Right => {
                // Toggle expand for the currently selected node
                Some(ViewerEvent::ToggleNodeExpansion(current_node_id))
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
        let viewer = FileViewer::new("test content".to_string());
        assert_eq!(viewer.cursor_position(), (0, 0));
        assert_eq!(viewer.scroll_offset(), 0);
    }

    #[test]
    fn test_tree_viewer_creation() {
        let viewer = TreeViewer::new();
        assert_eq!(viewer.selected_node_id(), None);
        assert_eq!(viewer.scroll_offset(), 0);
    }
}
