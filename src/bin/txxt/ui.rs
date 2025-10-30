//! UI rendering logic
//!
//! Handles layout and rendering of the application using Ratatui.
//! Layout structure:
//! - Title bar (1 line, fixed)
//! - Middle section (responsive height):
//!   - Tree viewer (30 chars, fixed width)
//!   - File viewer (remaining space)
//! - Status line (1 line, fixed)

use crate::app::App;
use crate::model::Focus;
use crate::theme::Theme;
use crate::viewer::Viewer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

/// Minimum terminal width required for the UI
const MIN_TERMINAL_WIDTH: u16 = 50;
/// Width allocated to the tree viewer
const TREE_VIEWER_WIDTH: u16 = 30;
/// Height of the status line
const STATUS_LINE_HEIGHT: u16 = 1;

/// Render the entire UI
pub fn render(frame: &mut Frame, app: &App, file_name: &str) {
    let size = frame.area();

    // Check minimum width
    if size.width < MIN_TERMINAL_WIDTH {
        render_error_too_narrow(frame, size, &app.theme);
        return;
    }

    // Split layout vertically: title, middle (with tree|file), status line
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),                  // Title bar
            Constraint::Min(3),                     // Middle (tree|file)
            Constraint::Length(STATUS_LINE_HEIGHT), // Status line
        ])
        .split(size);

    render_title_bar(frame, chunks[0], file_name, &app.theme);
    render_middle_section(frame, chunks[1], app);
    render_status_line(frame, chunks[2], app);
}

fn render_error_too_narrow(frame: &mut Frame, area: Rect, theme: &Theme) {
    let msg = format!(
        "Terminal too narrow: {} < {} chars",
        area.width, MIN_TERMINAL_WIDTH
    );
    let paragraph = Paragraph::new(msg).style(theme.error_message());
    frame.render_widget(paragraph, area);
}

fn render_title_bar(frame: &mut Frame, area: Rect, file_name: &str, theme: &Theme) {
    let title = format!("txxt:: {}", file_name);
    let paragraph = Paragraph::new(title).style(theme.title_bar());
    frame.render_widget(paragraph, area);
}

fn render_middle_section(frame: &mut Frame, area: Rect, app: &App) {
    // Split horizontally: tree viewer and file viewer
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(TREE_VIEWER_WIDTH), // Tree viewer
            Constraint::Min(1),                    // File viewer
        ])
        .split(area);

    render_tree_viewer(frame, chunks[0], app);
    render_file_viewer(frame, chunks[1], app);
}

fn render_tree_viewer(frame: &mut Frame, area: Rect, app: &App) {
    let focus_indicator = if app.focus == Focus::TreeViewer {
        " [FOCUSED]"
    } else {
        ""
    };

    let title = format!("Tree{}", focus_indicator);
    let block = Block::default().borders(Borders::ALL).title(title);

    // Get inner area for content (inside the border)
    let inner_area = block.inner(area);

    // Render the border
    frame.render_widget(block, area);

    // Render the tree viewer's content
    app.tree_viewer
        .render(frame, inner_area, &app.model, &app.theme);
}

fn render_file_viewer(frame: &mut Frame, area: Rect, app: &App) {
    let focus_indicator = if app.focus == Focus::FileViewer {
        " [FOCUSED]"
    } else {
        ""
    };

    let title = format!("File{}", focus_indicator);
    let block = Block::default().borders(Borders::ALL).title(title);

    // Get inner area for content (inside the border)
    let inner_area = block.inner(area);

    // Render the border
    frame.render_widget(block, area);

    // Render the file viewer's content
    app.file_viewer
        .render(frame, inner_area, &app.model, &app.theme);
}

fn render_status_line(frame: &mut Frame, area: Rect, app: &App) {
    use ratatui::text::Span;

    let theme = &app.theme;

    // Get cursor position and node at that position
    let (row, col) = app.model.get_selected_position().unwrap_or((0, 0));
    let mut status_text = format!("cursor: {},{}", row, col);

    // Get the breadcrumb path for the element at cursor position
    if let Some(node_id) = app.model.get_node_at_position(row, col) {
        let path = node_id.path();

        // Build breadcrumb: element type > element type > ... > label
        let mut breadcrumb = String::new();
        let flattened = app.model.flattened_tree();

        // Walk up the tree to build the path
        for (depth, idx) in path.iter().enumerate() {
            if !breadcrumb.is_empty() {
                breadcrumb.push_str(" > ");
            }

            // Find the label for this node in the flattened tree
            if let Some(node) = flattened
                .iter()
                .find(|n| n.node_id.path().len() == depth + 1 && n.node_id.path()[depth] == *idx)
            {
                breadcrumb.push_str(&node.label);
            } else if depth == 0 && path.is_empty() {
                breadcrumb.push_str("Document");
            }
        }

        if !breadcrumb.is_empty() {
            status_text.push(' ');
            status_text.push_str(&breadcrumb);
        }
    }

    // Truncate if too long for the display area
    let max_width = area.width as usize;
    if status_text.len() > max_width {
        status_text.truncate(max_width.saturating_sub(3));
        status_text.push_str("...");
    }

    let spans = vec![Span::styled(status_text, theme.info_panel_bg())];
    let line = ratatui::text::Line::from(spans);
    let paragraph = Paragraph::new(line);

    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_viewer_width_constant() {
        assert_eq!(TREE_VIEWER_WIDTH, 30);
    }

    #[test]
    fn test_status_line_height_constant() {
        assert_eq!(STATUS_LINE_HEIGHT, 1);
    }

    #[test]
    fn test_min_terminal_width() {
        assert_eq!(MIN_TERMINAL_WIDTH, 50);
    }
}
