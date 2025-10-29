use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

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
}

impl App {
    fn new(file_path: PathBuf) -> io::Result<Self> {
        let content = fs::read_to_string(&file_path)?;
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(App { content, file_name })
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        // Main block
        let block = Block::default().borders(Borders::ALL).title("txxtv");
        let inner = block.inner(area);

        frame.render_widget(block, area);

        // Display the file content
        let paragraph = Paragraph::new(self.content.as_str());
        frame.render_widget(paragraph, inner);
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
    let result = run_app(&mut terminal, &app);

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

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &App) -> io::Result<()> {
    loop {
        terminal.draw(|frame| {
            app.draw(frame);
        })?;

        // Poll for events with timeout
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if handle_key_event(key) {
                    return Ok(());
                }
            }
        }
    }
}

fn handle_key_event(key: KeyEvent) -> bool {
    matches!(
        (key.code, key.modifiers),
        (KeyCode::Char('q'), KeyModifiers::NONE) | (KeyCode::Char('c'), KeyModifiers::CONTROL)
    )
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

        // Draw to the test backend
        terminal
            .draw(|frame| {
                app.draw(frame);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let rendered = buffer.content();
        assert!(!rendered.is_empty());

        // Clean up
        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_quit_key_event() {
        let quit_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        assert!(handle_key_event(quit_event));

        let ctrl_c_event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert!(handle_key_event(ctrl_c_event));

        let other_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(!handle_key_event(other_event));
    }
}
