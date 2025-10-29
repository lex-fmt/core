use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::*;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::Duration;
use txxt_nano::txxt_nano::ast::lookup::by_position::format_at_position;
use txxt_nano::txxt_nano::parser::{parse_document, Document};

#[derive(Parser)]
#[command(name = "txxtv")]
#[command(about = "A terminal UI viewer for txxt documents")]
struct Args {
    /// Path to the txxt file to open
    file: PathBuf,
}

struct App {
    content: String,
    #[allow(dead_code)]
    file_name: String,
    cursor_row: usize,
    cursor_col: usize,
    document: Option<Document>,
    parse_error: Option<String>,
}

impl App {
    fn new(file_path: PathBuf) -> io::Result<Self> {
        let content = fs::read_to_string(&file_path)?;
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Try to parse the document
        let (document, parse_error) = match parse_document(&content) {
            Ok(doc) => (Some(doc), None),
            Err(_) => (None, Some("Failed to parse txxt document".to_string())),
        };

        Ok(App {
            content,
            file_name,
            cursor_row: 0,
            cursor_col: 0,
            document,
            parse_error,
        })
    }

    fn get_lines(&self) -> Vec<&str> {
        self.content.lines().collect()
    }

    fn max_col_for_row(&self, row: usize) -> usize {
        self.get_lines()
            .get(row)
            .map(|line| line.len())
            .unwrap_or(0)
    }

    fn max_row(&self) -> usize {
        let lines = self.get_lines();
        if lines.is_empty() {
            0
        } else {
            lines.len() - 1
        }
    }

    fn move_cursor_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            // Adjust column if the new row is shorter
            let max_col = self.max_col_for_row(self.cursor_row);
            if self.cursor_col > max_col {
                self.cursor_col = max_col;
            }
        }
    }

    fn move_cursor_down(&mut self) {
        let max_row = self.max_row();
        if self.cursor_row < max_row {
            self.cursor_row += 1;
            // Adjust column if the new row is shorter
            let max_col = self.max_col_for_row(self.cursor_row);
            if self.cursor_col > max_col {
                self.cursor_col = max_col;
            }
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        }
    }

    fn move_cursor_right(&mut self) {
        let max_col = self.max_col_for_row(self.cursor_row);
        if self.cursor_col < max_col {
            self.cursor_col += 1;
        }
    }

    fn render_content_with_cursor(&self) -> Vec<Line<'_>> {
        let lines = self.get_lines();
        lines
            .iter()
            .enumerate()
            .map(|(row_idx, line)| {
                if row_idx == self.cursor_row {
                    // Render this line with cursor highlighting
                    let mut spans = Vec::new();

                    for (col_idx, ch) in line.chars().enumerate() {
                        if col_idx == self.cursor_col {
                            // Render cursor character with inverted colors
                            spans.push(Span::styled(
                                ch.to_string(),
                                Style::default()
                                    .fg(Color::Black)
                                    .bg(Color::White)
                                    .add_modifier(Modifier::REVERSED),
                            ));
                        } else {
                            spans.push(Span::raw(ch.to_string()));
                        }
                    }

                    // If cursor is at the end of the line, show it as a space
                    if self.cursor_col == line.len() && !line.is_empty() {
                        // Already handled above
                    } else if self.cursor_col == line.len() {
                        spans.push(Span::styled(
                            " ",
                            Style::default()
                                .fg(Color::Black)
                                .bg(Color::White)
                                .add_modifier(Modifier::REVERSED),
                        ));
                    }

                    Line::from(spans)
                } else {
                    Line::from(line.to_string())
                }
            })
            .collect()
    }

    fn parse_ast_info(&self, ast_info: &str) -> Vec<String> {
        let mut result = Vec::new();
        let mut current_num = 0;
        let mut current_element: Option<String> = None;
        let mut current_label: Option<String> = None;

        for line in ast_info.lines() {
            // Parse lines like "1. Session (0:0..0:17)"
            if let Some(rest) = line.strip_prefix(&format!("{}. ", current_num + 1)) {
                // Found next element
                if let (Some(elem), Some(label)) = (&current_element, &current_label) {
                    result.push(format!("{}. {}: {}", current_num, elem, label));
                }

                current_num += 1;
                // Extract element type (before the parenthesis)
                if let Some(elem_name) = rest.split('(').next() {
                    current_element = Some(elem_name.trim().to_string());
                }
                current_label = None;
            } else if line.starts_with("   Label:") {
                // Extract label
                let label = line.trim_start_matches("   Label: ");
                current_label = Some(label.to_string());
            }
        }

        // Add the last element
        if let (Some(elem), Some(label)) = (current_element, current_label) {
            result.push(format!("{}. {}: {}", current_num, elem, label));
        }

        result
    }

    fn render_info_panel(&self) -> Vec<Line<'_>> {
        let mut lines = vec![];

        // AST info if available
        if let Some(doc) = &self.document {
            let mut extras = HashMap::new();
            extras.insert(
                "position".to_string(),
                format!("{}:{}", self.cursor_row, self.cursor_col),
            );

            match format_at_position(doc, &extras) {
                Ok(ast_info) => {
                    // Parse and display AST hierarchy
                    let parsed = self.parse_ast_info(&ast_info);
                    for line in parsed {
                        lines.push(Line::from(line));
                    }
                }
                Err(_) => {
                    lines.push(Line::from("(No AST info at cursor)"));
                }
            }
        } else if let Some(error) = &self.parse_error {
            lines.push(Line::from(format!("Parse error: {}", error)));
        }

        lines
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        // Create layout: title bar, file viewer, and info panel
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(11),
            ])
            .split(area);

        // Title bar
        let title_text = format!("txxt:: {}", self.file_name);
        let title = Paragraph::new(title_text).style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
        frame.render_widget(title, chunks[0]);

        // File viewer area with cursor
        let rendered_lines = self.render_content_with_cursor();
        let file_viewer = Paragraph::new(rendered_lines)
            .block(Block::default().borders(Borders::ALL).title("File Content"));
        frame.render_widget(file_viewer, chunks[1]);

        // Info panel at the bottom with position in title
        let info_lines = self.render_info_panel();
        let panel_title = format!("pos: {},{}", self.cursor_row, self.cursor_col);
        let info_panel = Paragraph::new(info_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(panel_title)
                .style(Style::default().fg(Color::Black).bg(Color::Gray)),
        );
        frame.render_widget(info_panel, chunks[2]);
    }
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let app = App::new(args.file)?;

    // Setup terminal
    enable_raw_mode()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // Run the app
    let result = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    terminal.clear()?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        return Err(e);
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|frame| {
            app.draw(frame);
        })?;

        // Poll for events with timeout
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if handle_key_event(key, &mut app) {
                    return Ok(());
                }
            }
        }
    }
}

fn handle_key_event(key: KeyEvent, app: &mut App) -> bool {
    match key.code {
        KeyCode::Char('q') if key.modifiers == KeyModifiers::NONE => true,
        KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => true,
        KeyCode::Up => {
            app.move_cursor_up();
            false
        }
        KeyCode::Down => {
            app.move_cursor_down();
            false
        }
        KeyCode::Left => {
            app.move_cursor_left();
            false
        }
        KeyCode::Right => {
            app.move_cursor_right();
            false
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;

    #[test]
    fn test_app_creation() {
        // Create a temporary test file
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_txxt.txt");
        fs::write(&test_file, "Hello, World!").unwrap();

        let app = App::new(test_file.clone()).unwrap();
        assert_eq!(app.content, "Hello, World!");
        assert_eq!(app.file_name, "test_txxt.txt");
        assert_eq!(app.cursor_row, 0);
        assert_eq!(app.cursor_col, 0);

        // Clean up
        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_render_to_buffer() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_render.txt");
        fs::write(&test_file, "Test Content").unwrap();

        let app = App::new(test_file.clone()).unwrap();

        // Create a test backend with a small terminal size
        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        // Draw to the test backend - this verifies the layout renders correctly
        terminal
            .draw(|frame| {
                app.draw(frame);
            })
            .unwrap();

        // Clean up
        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_cursor_movement() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_cursor.txt");
        fs::write(&test_file, "Line 1\nLine 2\nLine 3").unwrap();

        let mut app = App::new(test_file.clone()).unwrap();

        // Initial position
        assert_eq!(app.cursor_row, 0);
        assert_eq!(app.cursor_col, 0);

        // Move right
        app.move_cursor_right();
        assert_eq!(app.cursor_col, 1);

        // Move down
        app.move_cursor_down();
        assert_eq!(app.cursor_row, 1);
        assert_eq!(app.cursor_col, 1);

        // Move left
        app.move_cursor_left();
        assert_eq!(app.cursor_col, 0);

        // Move up
        app.move_cursor_up();
        assert_eq!(app.cursor_row, 0);

        // Clean up
        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_cursor_bounds() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_bounds.txt");
        fs::write(&test_file, "Hi\nWorld").unwrap();

        let mut app = App::new(test_file.clone()).unwrap();

        // Can't move left from 0
        app.move_cursor_left();
        assert_eq!(app.cursor_col, 0);

        // Can't move up from row 0
        app.move_cursor_up();
        assert_eq!(app.cursor_row, 0);

        // Move to end of line 1
        app.move_cursor_right(); // col 1
        app.move_cursor_right(); // col 2 (at end)
        app.move_cursor_right(); // should stay at 2
        assert_eq!(app.cursor_col, 2);

        // Move down to line 2, column should adjust if needed
        app.move_cursor_down();
        assert_eq!(app.cursor_row, 1);
        assert!(app.cursor_col <= 5); // Line 2 is "World" (5 chars)

        // Can't move down from last line
        app.move_cursor_down();
        assert_eq!(app.cursor_row, 1);

        // Clean up
        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_cursor_rendering() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_cursor_render.txt");
        fs::write(&test_file, "ABC").unwrap();

        let app = App::new(test_file.clone()).unwrap();

        // Create a test backend with a reasonable size
        let backend = TestBackend::new(50, 15);
        let mut terminal = Terminal::new(backend).unwrap();

        // Draw to the test backend
        terminal
            .draw(|frame| {
                app.draw(frame);
            })
            .unwrap();

        // Verify it renders without crashing
        // The cursor styling should be applied

        // Clean up
        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_arrow_key_events() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_keys.txt");
        fs::write(&test_file, "Test").unwrap();

        let mut app = App::new(test_file.clone()).unwrap();

        let up_key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        assert!(!handle_key_event(up_key, &mut app));

        let down_key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        assert!(!handle_key_event(down_key, &mut app));

        let left_key = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
        assert!(!handle_key_event(left_key, &mut app));

        let right_key = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
        assert!(!handle_key_event(right_key, &mut app));
        assert_eq!(app.cursor_col, 1);

        // Quit still works
        let quit_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        assert!(handle_key_event(quit_event, &mut app));

        // Clean up
        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_info_panel_rendering() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_info.txt");
        fs::write(&test_file, "Line 1\nLine 2\nLine 3").unwrap();

        let mut app = App::new(test_file.clone()).unwrap();

        // Move cursor to position 2,3
        app.cursor_row = 2;
        app.cursor_col = 3;

        // Render info panel and verify content
        let info_lines = app.render_info_panel();
        assert!(!info_lines.is_empty(), "Info panel should have content");

        // Verify cursor position is correct
        assert_eq!(app.cursor_row, 2);
        assert_eq!(app.cursor_col, 3);

        // Clean up
        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_full_layout_with_info_panel() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_layout.txt");
        fs::write(&test_file, "Content\nwith\nmultiple\nlines").unwrap();

        let app = App::new(test_file.clone()).unwrap();

        // Create a test backend with a size that accommodates all three sections
        let backend = TestBackend::new(60, 20);
        let mut terminal = Terminal::new(backend).unwrap();

        // Draw to the test backend - should render title, file viewer, and info panel
        terminal
            .draw(|frame| {
                app.draw(frame);
            })
            .unwrap();

        // Clean up
        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_ast_parsing_integration() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_ast.txxt");
        // Create a simple txxt document with a paragraph
        fs::write(&test_file, "This is a simple paragraph").unwrap();

        let app = App::new(test_file.clone()).unwrap();

        // Check that document was parsed successfully
        assert!(app.document.is_some(), "Document should be parsed");
        assert!(app.parse_error.is_none(), "Should have no parse error");

        // Render info panel and verify it includes position info
        let info_lines = app.render_info_panel();
        assert!(!info_lines.is_empty(), "Info panel should have content");

        // Clean up
        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_ast_panel_with_cursor_movement() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_ast_cursor.txxt");
        fs::write(&test_file, "First paragraph\nSecond paragraph").unwrap();

        let mut app = App::new(test_file.clone()).unwrap();

        // Verify cursor position is in app state
        assert_eq!(app.cursor_row, 0);
        assert_eq!(app.cursor_col, 0);

        // Move cursor and verify position updates
        app.move_cursor_right();
        app.move_cursor_right();
        assert_eq!(app.cursor_row, 0);
        assert_eq!(app.cursor_col, 2);

        // Render info panel and verify it has content
        let info_lines = app.render_info_panel();
        assert!(!info_lines.is_empty(), "Info panel should have content");

        // Clean up
        fs::remove_file(test_file).unwrap();
    }
}
