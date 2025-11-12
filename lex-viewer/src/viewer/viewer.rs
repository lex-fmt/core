//! Viewer module - trait, events, and main entry point
//!
//! The Viewer trait defines a common interface for UI components that:
//! - Render themselves given a model and area
//! - Handle keyboard input and return events
//!
//! This module also contains the main viewer application entry point.

use super::app::App;
use super::model::Model;
use super::ui;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use lex_parser::lex::parsing::parse_document;
use ratatui::layout::Rect;
use ratatui::prelude::{CrosstermBackend, Terminal};
use ratatui::Frame;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use super::model::NodeId;

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

/// Run the viewer for the given file path
pub fn run_viewer(file_path: PathBuf) -> io::Result<()> {
    // Load the file
    let content = fs::read_to_string(&file_path)?;
    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Parse the document
    let document = parse_document(&content).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to parse lex document: {:?}", e),
        )
    })?;

    // Create the modular App
    let model = Model::new(document);
    let mut app = App::new(model, content);

    // Setup terminal
    enable_raw_mode()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // Run the app
    let result = run_app(&mut terminal, &mut app, &file_name);

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

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    file_name: &str,
) -> io::Result<()> {
    loop {
        // Render the full UI every frame
        terminal.draw(|frame| {
            ui::render(frame, app, file_name);
        })?;

        // Poll for events with timeout
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    if handle_key_event(key, app) {
                        return Ok(());
                    }
                }
                // On terminal resize, the next loop iteration will re-render with new dimensions
                Event::Resize(_, _) => {
                    // Terminal resize event - the next draw() call will use the new dimensions
                    // No explicit action needed, just continue the loop
                }
                _ => {
                    // Ignore other events (mouse, focus, etc.)
                }
            }
        }
    }
}

fn handle_key_event(key: KeyEvent, app: &mut App) -> bool {
    match key.code {
        KeyCode::Char('q') if key.modifiers.is_empty() => true,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => true,
        KeyCode::Tab => {
            app.toggle_focus();
            false
        }
        _ => {
            // Delegate to app's key handler
            let _ = app.handle_key(key);
            false
        }
    }
}
