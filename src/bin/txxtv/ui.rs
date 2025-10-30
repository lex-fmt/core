//! UI rendering logic
//!
//! Handles layout and rendering of the application using Ratatui.
//! Layout structure:
//! - Title bar (1 line, fixed)
//! - Middle section (responsive height):
//!   - Tree viewer (30 chars, fixed width)
//!   - File viewer (remaining space)
//! - Info panel (11 lines, fixed)

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
/// Height of the info panel
const INFO_PANEL_HEIGHT: u16 = 11;

/// Render the entire UI
pub fn render(frame: &mut Frame, app: &App, file_name: &str) {
    let size = frame.area();

    // Check minimum width
    if size.width < MIN_TERMINAL_WIDTH {
        render_error_too_narrow(frame, size, &app.theme);
        return;
    }

    // Split layout vertically: title, middle (with tree|file), info panel
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),                 // Title bar
            Constraint::Min(3),                    // Middle (tree|file)
            Constraint::Length(INFO_PANEL_HEIGHT), // Info panel
        ])
        .split(size);

    render_title_bar(frame, chunks[0], file_name, &app.theme);
    render_middle_section(frame, chunks[1], app);
    render_info_panel(frame, chunks[2], app);
}

fn render_error_too_narrow(frame: &mut Frame, area: Rect, theme: &Theme) {
    let msg = format!(
        "Terminal too narrow: {} < {} chars",
        area.width, MIN_TERMINAL_WIDTH
    );
    let paragraph = Paragraph::new(msg).style(theme.error_style);
    frame.render_widget(paragraph, area);
}

fn render_title_bar(frame: &mut Frame, area: Rect, file_name: &str, theme: &Theme) {
    let title = format!("txxt:: {}", file_name);
    let paragraph = Paragraph::new(title).style(theme.title_bar_style);
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

fn render_info_panel(frame: &mut Frame, area: Rect, app: &App) {
    use crate::model::Selection;
    use ratatui::text::Span;

    let title = "Info";
    let theme = &app.theme;

    // Build info content based on current selection
    let mut spans = Vec::new();

    match app.model.selection() {
        Selection::TreeSelection(node_id) => {
            spans.push(Span::styled("MODE: ", theme.info_panel_mode_tree_style));
            spans.push(Span::raw("Tree Selection\n"));

            let path = node_id.path();
            if path.is_empty() {
                spans.push(Span::styled("Selection: ", theme.info_panel_label_style));
                spans.push(Span::raw("Root (Document)\n"));
            } else {
                spans.push(Span::styled("Path: ", theme.info_panel_label_style));
                let path_str = path
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(" → ");
                spans.push(Span::raw(format!("[{}]\n", path_str)));

                // Show if node is expanded
                let is_expanded = app.model.is_node_expanded(node_id);
                spans.push(Span::styled("State: ", theme.info_panel_label_style));
                spans.push(Span::raw(if is_expanded {
                    "Expanded\n"
                } else {
                    "Collapsed\n"
                }));
            }
        }
        Selection::TextSelection(row, col) => {
            spans.push(Span::styled("MODE: ", theme.info_panel_mode_text_style));
            spans.push(Span::raw("Text Selection\n"));

            spans.push(Span::styled("Cursor: ", theme.info_panel_label_style));
            spans.push(Span::raw(format!("line {}, col {}\n", row, col)));

            if let Some(node_id) = app.model.get_node_at_position(row, col) {
                let path = node_id.path();
                spans.push(Span::styled("Node: ", theme.info_panel_label_style));

                if path.is_empty() {
                    spans.push(Span::raw("Root (Document)\n"));
                } else {
                    let path_str = path
                        .iter()
                        .map(|i| i.to_string())
                        .collect::<Vec<_>>()
                        .join(" → ");
                    spans.push(Span::raw(format!("[{}]\n", path_str)));
                }

                // Show if node is expanded
                let is_expanded = app.model.is_node_expanded(node_id);
                spans.push(Span::styled("State: ", theme.info_panel_label_style));
                spans.push(Span::raw(if is_expanded {
                    "Expanded"
                } else {
                    "Collapsed"
                }));
            }
        }
    }

    let paragraph = Paragraph::new(ratatui::text::Line::from(spans))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_type(ratatui::widgets::BorderType::Rounded),
        )
        .style(theme.info_panel_bg_style);

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
    fn test_info_panel_height_constant() {
        assert_eq!(INFO_PANEL_HEIGHT, 11);
    }

    #[test]
    fn test_min_terminal_width() {
        assert_eq!(MIN_TERMINAL_WIDTH, 50);
    }
}
