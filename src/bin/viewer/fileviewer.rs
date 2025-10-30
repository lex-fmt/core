//! File viewer - displays and navigates the text content
//!
//! The file viewer shows the raw text of the document with a cursor
//! that can be moved with arrow keys. When the cursor moves, it emits
//! a SelectPosition event so the model can track which AST node is selected.

use super::model::Model;
use super::viewer::{Viewer, ViewerEvent};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

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
    }
}

impl Viewer for FileViewer {
    fn render(&self, frame: &mut Frame, area: Rect, _model: &Model) {
        use ratatui::style::{Color, Modifier, Style};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_viewer_creation() {
        let viewer = FileViewer::new("test content".to_string());
        assert_eq!(viewer.cursor_position(), (0, 0));
        assert_eq!(viewer.scroll_offset(), 0);
    }
}
