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
    /// Intended cursor column for vertical navigation (0-indexed)
    /// When moving up/down, the cursor tries to stay as close to this column as possible
    intended_cursor_col: usize,
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
            intended_cursor_col: 0,
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
        self.intended_cursor_col = col;
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
            self.intended_cursor_col = self.cursor_col;
        }
    }

    /// Move cursor right
    fn move_cursor_right(&mut self) {
        let lines: Vec<&str> = self.content.lines().collect();
        if self.cursor_row < lines.len() {
            let line_len = lines[self.cursor_row].len();
            if self.cursor_col < line_len {
                self.cursor_col += 1;
                self.intended_cursor_col = self.cursor_col;
            }
        }
    }

    /// Move cursor to start of next word (w key)
    fn move_to_next_word(&mut self) {
        let lines: Vec<&str> = self.content.lines().collect();
        if self.cursor_row >= lines.len() {
            return;
        }

        let current_line = lines[self.cursor_row];
        let chars: Vec<char> = current_line.chars().collect();

        // Start from current position + 1
        let mut pos = self.cursor_col;

        // Skip current word (alphanumeric or punctuation)
        while pos < chars.len() && !chars[pos].is_whitespace() {
            pos += 1;
        }

        // Skip whitespace
        while pos < chars.len() && chars[pos].is_whitespace() {
            pos += 1;
        }

        // If we reached end of line, try moving to next line
        if pos >= chars.len() && self.cursor_row + 1 < lines.len() {
            self.cursor_row += 1;
            self.cursor_col = 0;
            self.intended_cursor_col = 0;
        } else if pos < chars.len() {
            self.cursor_col = pos;
            self.intended_cursor_col = pos;
        }
    }

    /// Move cursor to start of previous word (b key)
    fn move_to_previous_word(&mut self) {
        let lines: Vec<&str> = self.content.lines().collect();
        if self.cursor_row >= lines.len() {
            return;
        }

        let current_line = lines[self.cursor_row];
        let chars: Vec<char> = current_line.chars().collect();

        // If at start of line, move to end of previous line
        if self.cursor_col == 0 {
            if self.cursor_row > 0 {
                self.cursor_row -= 1;
                let prev_line = lines[self.cursor_row];
                self.cursor_col = prev_line.len();
                self.intended_cursor_col = self.cursor_col;
                // Now find the start of the last word on this line
                self.move_to_previous_word();
            }
            return;
        }

        let mut pos = self.cursor_col.saturating_sub(1);

        // Skip whitespace backwards
        while pos > 0 && chars[pos].is_whitespace() {
            pos = pos.saturating_sub(1);
        }

        // Skip current word backwards
        while pos > 0 && !chars[pos].is_whitespace() {
            pos = pos.saturating_sub(1);
        }

        // If we stopped at whitespace, move one forward
        if (pos > 0 || (pos == 0 && chars[0].is_whitespace())) && chars[pos].is_whitespace() {
            pos += 1;
        }

        self.cursor_col = pos;
        self.intended_cursor_col = pos;
    }

    /// Move cursor to start of next element/paragraph (} key)
    fn move_to_next_element(&mut self, model: &Model) {
        // Get current node
        let current_node = model.get_node_at_position(self.cursor_row, self.cursor_col);

        // Get flattened tree
        let flattened = model.flattened_tree();

        // Find next node in flattened tree
        let next_node = if let Some(current) = current_node {
            // Find current node index
            flattened
                .iter()
                .position(|n| n.node_id == current)
                .and_then(|idx| flattened.get(idx + 1))
        } else {
            // No current node, just get first
            flattened.first()
        };

        // Move to start of next node's location
        if let Some(node) = next_node {
            if let Some(range) = model.get_location_for_node(node.node_id) {
                self.cursor_row = range.start.line;
                self.cursor_col = range.start.column;
                self.intended_cursor_col = self.cursor_col;
            }
        }
    }

    /// Move cursor to start of previous element/paragraph ({ key)
    fn move_to_previous_element(&mut self, model: &Model) {
        // Get current node
        let current_node = model.get_node_at_position(self.cursor_row, self.cursor_col);

        // Get flattened tree
        let flattened = model.flattened_tree();

        // Find previous node in flattened tree
        let prev_node = if let Some(current) = current_node {
            // Find current node index
            flattened
                .iter()
                .position(|n| n.node_id == current)
                .and_then(|idx| {
                    if idx > 0 {
                        flattened.get(idx - 1)
                    } else {
                        None
                    }
                })
        } else {
            // No current node, just get last
            flattened.last()
        };

        // Move to start of previous node's location
        if let Some(node) = prev_node {
            if let Some(range) = model.get_location_for_node(node.node_id) {
                self.cursor_row = range.start.line;
                self.cursor_col = range.start.column;
                self.intended_cursor_col = self.cursor_col;
            }
        }
    }

    /// Clamp cursor column to valid range for current line
    /// Uses the intended cursor column to maintain horizontal position during vertical movement
    fn clamp_cursor_column(&mut self) {
        let lines: Vec<&str> = self.content.lines().collect();
        if self.cursor_row < lines.len() {
            let line_len = lines[self.cursor_row].len();
            // Set cursor to intended column, but clamp to line length if shorter
            self.cursor_col = self.intended_cursor_col.min(line_len);
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

        // Calculate line number width based on total line count
        let line_count = self.content.lines().count();
        let line_num_width = if line_count == 0 {
            1
        } else {
            line_count.to_string().len()
        };

        // Display the file content line by line, highlighting cursor position
        let lines: Vec<Line> = self
            .content
            .lines()
            .enumerate()
            .map(|(row_idx, line_text)| {
                let line_number = row_idx + 1;
                let line_num_str = format!("{:0width$} ", line_number, width = line_num_width);

                if row_idx == self.cursor_row {
                    // This is the row with the cursor - render with cursor highlight
                    let mut spans = Vec::new();

                    // Add line number with active styling (normal white)
                    spans.push(Span::styled(
                        line_num_str,
                        Style::default().fg(Color::White),
                    ));

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
                    let spans = vec![
                        // Add line number with subdued styling (dark gray)
                        Span::styled(line_num_str, Style::default().fg(Color::DarkGray)),
                        // Add line text
                        Span::raw(line_text.to_string()),
                    ];

                    Line::from(spans)
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
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_cursor_up();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_cursor_down();
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.move_cursor_left();
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.move_cursor_right();
            }
            KeyCode::Char('w') => {
                self.move_to_next_word();
            }
            KeyCode::Char('b') => {
                self.move_to_previous_word();
            }
            KeyCode::Char('}') => {
                self.move_to_next_element(model);
            }
            KeyCode::Char('{') => {
                self.move_to_previous_element(model);
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
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_file_viewer_creation() {
        let viewer = FileViewer::new("test content".to_string());
        assert_eq!(viewer.cursor_position(), (0, 0));
        assert_eq!(viewer.scroll_offset(), 0);
    }

    #[test]
    fn test_intended_column_tracking_through_empty_line() {
        // Test the scenario from the requirements:
        // Line 01: "Hello" (5 chars)
        // Line 02: "" (empty)
        // Line 03: "World" (5 chars)
        //
        // Start at line 01, col 4 (the 'o' in Hello)
        // Move down to line 02 -> should go to col 0 (empty line)
        // Move down to line 03 -> should go back to col 4 (the 'o' in World)

        let content = "Hello\n\nWorld".to_string();
        let mut viewer = FileViewer::new(content);

        // Create a dummy model for handle_key
        let test_doc = "# Test";
        let doc = lex_parser::lex::parsing::parse_document(test_doc).unwrap();
        let model = Model::new(doc);

        // Start at (0, 0)
        assert_eq!(viewer.cursor_position(), (0, 0));

        // Move right 4 times to get to column 4 (the 'o' in "Hello")
        for _ in 0..4 {
            viewer.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE), &model);
        }
        assert_eq!(viewer.cursor_position(), (0, 4));

        // Move down to line 1 (empty line) - should go to column 0 (empty line has no chars)
        viewer.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE), &model);
        assert_eq!(
            viewer.cursor_position(),
            (1, 0),
            "Empty line should clamp to column 0"
        );

        // Move down to line 2 ("World") - should return to column 4
        viewer.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE), &model);
        assert_eq!(
            viewer.cursor_position(),
            (2, 4),
            "Should return to intended column 4"
        );
    }

    #[test]
    fn test_intended_column_tracking_through_shorter_line() {
        // Test with varying line lengths:
        // Line 0: "1234567890" (10 chars)
        // Line 1: "123" (3 chars)
        // Line 2: "1234567890ABCDEF" (16 chars)
        //
        // Start at line 0, col 9 (last char)
        // Move down to line 1 -> should go to col 3 (last char of shorter line)
        // Move down to line 2 -> should return to col 9

        let content = "1234567890\n123\n1234567890ABCDEF".to_string();
        let mut viewer = FileViewer::new(content);

        let test_doc = "# Test";
        let doc = lex_parser::lex::parsing::parse_document(test_doc).unwrap();
        let model = Model::new(doc);

        // Move to column 9 on line 0
        for _ in 0..9 {
            viewer.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE), &model);
        }
        assert_eq!(viewer.cursor_position(), (0, 9));

        // Move down to shorter line - should clamp to column 3
        viewer.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE), &model);
        assert_eq!(
            viewer.cursor_position(),
            (1, 3),
            "Should clamp to shorter line length"
        );

        // Move down to longer line - should return to column 9
        viewer.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE), &model);
        assert_eq!(
            viewer.cursor_position(),
            (2, 9),
            "Should return to intended column 9"
        );
    }

    #[test]
    fn test_horizontal_movement_updates_intended_column() {
        // Test that moving left/right updates the intended column
        let content = "Hello\nWorld\nTest".to_string();
        let mut viewer = FileViewer::new(content);

        let test_doc = "# Test";
        let doc = lex_parser::lex::parsing::parse_document(test_doc).unwrap();
        let model = Model::new(doc);

        // Move right to column 4
        for _ in 0..4 {
            viewer.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE), &model);
        }
        assert_eq!(viewer.cursor_position(), (0, 4));

        // Move down - should maintain column 4
        viewer.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE), &model);
        assert_eq!(viewer.cursor_position(), (1, 4));

        // Move left to column 2
        for _ in 0..2 {
            viewer.handle_key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE), &model);
        }
        assert_eq!(viewer.cursor_position(), (1, 2));

        // Move down - should maintain NEW intended column 2 (not old 4)
        viewer.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE), &model);
        assert_eq!(
            viewer.cursor_position(),
            (2, 2),
            "Horizontal movement should update intended column"
        );
    }

    #[test]
    fn test_sync_cursor_updates_intended_column() {
        // Test that sync_cursor_to_position updates the intended column
        let content = "Hello\n\nWorld".to_string();
        let mut viewer = FileViewer::new(content);

        // Sync to position (0, 4)
        viewer.sync_cursor_to_position(0, 4);
        assert_eq!(viewer.cursor_position(), (0, 4));

        let test_doc = "# Test";
        let doc = lex_parser::lex::parsing::parse_document(test_doc).unwrap();
        let model = Model::new(doc);

        // Move down through empty line to "World"
        viewer.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE), &model);
        viewer.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE), &model);

        // Should be at (2, 4) - maintaining the intended column from sync
        assert_eq!(
            viewer.cursor_position(),
            (2, 4),
            "Sync should set intended column"
        );
    }

    #[test]
    fn test_intended_column_going_up_from_blank_line() {
        // Test moving UP from a blank line back to a line with content
        // The cursor should return to the intended column, not column 0
        let content = "Hello World\n\nTest".to_string();
        let mut viewer = FileViewer::new(content);

        let test_doc = "# Test";
        let doc = lex_parser::lex::parsing::parse_document(test_doc).unwrap();
        let model = Model::new(doc);

        // Start at (0, 0) and move to column 8
        for _ in 0..8 {
            viewer.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE), &model);
        }
        assert_eq!(viewer.cursor_position(), (0, 8));

        // Move down to blank line
        viewer.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE), &model);
        assert_eq!(
            viewer.cursor_position(),
            (1, 0),
            "Blank line should have cursor at 0"
        );

        // Move up back to first line - should go back to column 8
        viewer.handle_key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE), &model);
        assert_eq!(
            viewer.cursor_position(),
            (0, 8),
            "Should return to intended column 8 when moving up from blank line"
        );
    }

    #[test]
    fn test_vim_jkhl_navigation() {
        // Test that Vim j/k/h/l keys work the same as arrow keys
        let content = "Line1\nLine2\nLine3".to_string();
        let mut viewer = FileViewer::new(content);

        let test_doc = "# Test";
        let doc = lex_parser::lex::parsing::parse_document(test_doc).unwrap();
        let model = Model::new(doc);

        // Start at (0, 0)
        assert_eq!(viewer.cursor_position(), (0, 0));

        // Test 'j' (down)
        viewer.handle_key(
            KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
            &model,
        );
        assert_eq!(viewer.cursor_position(), (1, 0), "j should move down");

        // Test 'k' (up)
        viewer.handle_key(
            KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE),
            &model,
        );
        assert_eq!(viewer.cursor_position(), (0, 0), "k should move up");

        // Test 'l' (right)
        for _ in 0..3 {
            viewer.handle_key(
                KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE),
                &model,
            );
        }
        assert_eq!(viewer.cursor_position(), (0, 3), "l should move right");

        // Test 'h' (left)
        viewer.handle_key(
            KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
            &model,
        );
        assert_eq!(viewer.cursor_position(), (0, 2), "h should move left");
    }

    #[test]
    fn test_vim_word_navigation_w() {
        // Test 'w' (next word)
        let content = "hello world foo bar".to_string();
        let mut viewer = FileViewer::new(content);

        let test_doc = "# Test";
        let doc = lex_parser::lex::parsing::parse_document(test_doc).unwrap();
        let model = Model::new(doc);

        // Start at (0, 0) - 'h' in "hello"
        assert_eq!(viewer.cursor_position(), (0, 0));

        // Press 'w' - should jump to 'w' in "world"
        viewer.handle_key(
            KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE),
            &model,
        );
        assert_eq!(
            viewer.cursor_position(),
            (0, 6),
            "w should move to next word"
        );

        // Press 'w' again - should jump to 'f' in "foo"
        viewer.handle_key(
            KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE),
            &model,
        );
        assert_eq!(
            viewer.cursor_position(),
            (0, 12),
            "w should move to next word again"
        );

        // Press 'w' again - should jump to 'b' in "bar"
        viewer.handle_key(
            KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE),
            &model,
        );
        assert_eq!(
            viewer.cursor_position(),
            (0, 16),
            "w should move to last word"
        );
    }

    #[test]
    fn test_vim_word_navigation_b() {
        // Test 'b' (previous word)
        let content = "hello world foo bar".to_string();
        let mut viewer = FileViewer::new(content);

        let test_doc = "# Test";
        let doc = lex_parser::lex::parsing::parse_document(test_doc).unwrap();
        let model = Model::new(doc);

        // Start at end - manually set position
        viewer.sync_cursor_to_position(0, 16); // 'b' in "bar"
        assert_eq!(viewer.cursor_position(), (0, 16));

        // Press 'b' - should jump to 'f' in "foo"
        viewer.handle_key(
            KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE),
            &model,
        );
        assert_eq!(
            viewer.cursor_position(),
            (0, 12),
            "b should move to previous word"
        );

        // Press 'b' again - should jump to 'w' in "world"
        viewer.handle_key(
            KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE),
            &model,
        );
        assert_eq!(
            viewer.cursor_position(),
            (0, 6),
            "b should move to previous word again"
        );

        // Press 'b' again - should jump to 'h' in "hello"
        viewer.handle_key(
            KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE),
            &model,
        );
        assert_eq!(
            viewer.cursor_position(),
            (0, 0),
            "b should move to first word"
        );
    }

    #[test]
    fn test_vim_word_navigation_across_lines() {
        // Test 'w' and 'b' across line boundaries
        let content = "first line\nsecond line".to_string();
        let mut viewer = FileViewer::new(content);

        let test_doc = "# Test";
        let doc = lex_parser::lex::parsing::parse_document(test_doc).unwrap();
        let model = Model::new(doc);

        // Start at 'l' in "line" on first line
        viewer.sync_cursor_to_position(0, 6);
        assert_eq!(viewer.cursor_position(), (0, 6));

        // Press 'w' - should move to start of next line
        viewer.handle_key(
            KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE),
            &model,
        );
        assert_eq!(
            viewer.cursor_position(),
            (1, 0),
            "w should move to next line when at end of line"
        );

        // Press 'b' - should move back to "line" on first line
        viewer.handle_key(
            KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE),
            &model,
        );
        assert_eq!(
            viewer.cursor_position(),
            (0, 6),
            "b should move to previous line"
        );
    }

    #[test]
    fn test_vim_element_navigation_forward() {
        // Test '}' (next element)
        let content = "# Heading\n\nParagraph one.\n\n## Subheading\n\nParagraph two.".to_string();
        let doc = lex_parser::lex::parsing::parse_document(&content).unwrap();
        let model = Model::new(doc);
        let mut viewer = FileViewer::new(content);

        // Start at (0, 0) - on "# Heading"
        assert_eq!(viewer.cursor_position(), (0, 0));

        // Press '}' - should move to next element
        viewer.handle_key(
            KeyEvent::new(KeyCode::Char('}'), KeyModifiers::NONE),
            &model,
        );

        // Should have moved to a different element
        let (row, col) = viewer.cursor_position();
        assert!(
            row > 0 || col > 0,
            "Should have moved to next element from (0,0)"
        );
    }

    #[test]
    fn test_vim_element_navigation_backward() {
        // Test '{' (previous element)
        let content = "# Heading\n\nParagraph one.\n\n## Subheading\n\nParagraph two.".to_string();
        let doc = lex_parser::lex::parsing::parse_document(&content).unwrap();
        let model = Model::new(doc);
        let mut viewer = FileViewer::new(content);

        // Start somewhere in the middle
        viewer.sync_cursor_to_position(4, 0); // "## Subheading"
        assert_eq!(viewer.cursor_position(), (4, 0));

        // Press '{' - should move to previous element
        viewer.handle_key(
            KeyEvent::new(KeyCode::Char('{'), KeyModifiers::NONE),
            &model,
        );

        // Should have moved to a different element
        let (row, _col) = viewer.cursor_position();
        assert!(row < 4, "Should have moved to previous element from (4,0)");
    }

    #[test]
    fn test_vim_element_navigation_round_trip() {
        // Test that we can navigate forward and backward through elements
        let content = "# H1\n\nPara 1\n\n## H2\n\nPara 2".to_string();
        let doc = lex_parser::lex::parsing::parse_document(&content).unwrap();
        let model = Model::new(doc);
        let mut viewer = FileViewer::new(content);

        // Start at beginning
        let start_pos = viewer.cursor_position();
        assert_eq!(start_pos, (0, 0));

        // Navigate forward a few times
        for _ in 0..3 {
            viewer.handle_key(
                KeyEvent::new(KeyCode::Char('}'), KeyModifiers::NONE),
                &model,
            );
        }

        // Should be at a different position
        let middle_pos = viewer.cursor_position();
        assert_ne!(middle_pos, start_pos, "Should have moved forward");

        // Navigate backward the same number of times
        for _ in 0..3 {
            viewer.handle_key(
                KeyEvent::new(KeyCode::Char('{'), KeyModifiers::NONE),
                &model,
            );
        }

        // Should be back at or near the start
        let end_pos = viewer.cursor_position();
        assert_eq!(
            end_pos, start_pos,
            "Should return to start after navigating back"
        );
    }

    #[test]
    fn test_line_number_width_calculation() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;

        // Test with 9 lines (single digit) - should use width 1
        let content_9 = (1..=9)
            .map(|i| format!("Line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        let viewer_9 = FileViewer::new(content_9);
        let doc = lex_parser::lex::parsing::parse_document("# Test").unwrap();
        let model = Model::new(doc);

        let backend = TestBackend::new(80, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let area = frame.area();
                viewer_9.render(frame, area, &model);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let first_line = (0..10)
            .map(|x| buffer.cell((x, 0)).unwrap().symbol())
            .collect::<String>();

        // Should have single-digit line numbers: "1 Line 1"
        assert!(
            first_line.starts_with("1 "),
            "9-line document should use 1-digit line numbers, got: '{}'",
            first_line
        );

        // Test with 99 lines (double digit) - should use width 2
        let content_99 = (1..=99)
            .map(|i| format!("Line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        let viewer_99 = FileViewer::new(content_99);

        terminal
            .draw(|frame| {
                let area = frame.area();
                viewer_99.render(frame, area, &model);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let first_line = (0..10)
            .map(|x| buffer.cell((x, 0)).unwrap().symbol())
            .collect::<String>();

        // Should have zero-padded double-digit line numbers: "01 Line 1"
        assert!(
            first_line.starts_with("01 "),
            "99-line document should use 2-digit zero-padded line numbers, got: '{}'",
            first_line
        );

        // Test with 120 lines (triple digit) - should use width 3
        let content_120 = (1..=120)
            .map(|i| format!("Line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        let viewer_120 = FileViewer::new(content_120);

        terminal
            .draw(|frame| {
                let area = frame.area();
                viewer_120.render(frame, area, &model);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let first_line = (0..10)
            .map(|x| buffer.cell((x, 0)).unwrap().symbol())
            .collect::<String>();

        // Should have zero-padded triple-digit line numbers: "001 Line 1"
        assert!(
            first_line.starts_with("001 "),
            "120-line document should use 3-digit zero-padded line numbers, got: '{}'",
            first_line
        );
    }

    #[test]
    fn test_line_number_styling_active_vs_inactive() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;

        let content = "Line 1\nLine 2\nLine 3".to_string();
        let mut viewer = FileViewer::new(content);
        let doc = lex_parser::lex::parsing::parse_document("# Test").unwrap();
        let model = Model::new(doc);

        // Set cursor to line 1 (second line)
        viewer.sync_cursor_to_position(1, 0);

        let backend = TestBackend::new(80, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let area = frame.area();
                viewer.render(frame, area, &model);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();

        // Line 0 should have DarkGray line number (inactive)
        let line_0_num_cell = buffer.cell((0, 0)).unwrap();
        assert_eq!(
            line_0_num_cell.fg,
            ratatui::style::Color::DarkGray,
            "Inactive line should have DarkGray line number"
        );

        // Line 1 should have White line number (active, cursor is here)
        let line_1_num_cell = buffer.cell((0, 1)).unwrap();
        assert_eq!(
            line_1_num_cell.fg,
            ratatui::style::Color::White,
            "Active line (with cursor) should have White line number"
        );

        // Line 2 should have DarkGray line number (inactive)
        let line_2_num_cell = buffer.cell((0, 2)).unwrap();
        assert_eq!(
            line_2_num_cell.fg,
            ratatui::style::Color::DarkGray,
            "Inactive line should have DarkGray line number"
        );
    }
}
