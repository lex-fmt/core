//! Theme configuration for txxtv UI
//!
//! Defines all visual styling for the application including colors,
//! modifiers (bold, underline, etc.), and style combinations for different UI elements.

use ratatui::style::{Color, Modifier, Style};

/// Complete theme configuration for the TUI application
///
/// Defines styles for all UI elements in a centralized location.
/// To change the appearance, modify the fields in the default theme
/// or create a new Theme struct with custom values.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Theme {
    // Title bar styling
    pub title_bar_style: Style,

    // File viewer styling
    pub file_viewer_cursor_style: Style,
    pub file_viewer_normal_style: Style,

    // Tree viewer styling
    pub tree_normal_style: Style,
    pub tree_selected_style: Style,

    // Info panel styling
    pub info_panel_bg_style: Style,
    pub info_panel_label_style: Style,
    pub info_panel_mode_tree_style: Style,
    pub info_panel_mode_text_style: Style,

    // Error styling
    pub error_style: Style,

    // Border styling
    pub border_style: Style,
}

impl Theme {
    /// Create a new theme with custom values
    #[allow(dead_code)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        title_bar_style: Style,
        file_viewer_cursor_style: Style,
        file_viewer_normal_style: Style,
        tree_normal_style: Style,
        tree_selected_style: Style,
        info_panel_bg_style: Style,
        info_panel_label_style: Style,
        info_panel_mode_tree_style: Style,
        info_panel_mode_text_style: Style,
        error_style: Style,
        border_style: Style,
    ) -> Self {
        Theme {
            title_bar_style,
            file_viewer_cursor_style,
            file_viewer_normal_style,
            tree_normal_style,
            tree_selected_style,
            info_panel_bg_style,
            info_panel_label_style,
            info_panel_mode_tree_style,
            info_panel_mode_text_style,
            error_style,
            border_style,
        }
    }
}

impl Default for Theme {
    /// Create the default theme with sensible colors and styling
    fn default() -> Self {
        Theme {
            // Title bar: Black text on Cyan background, bold
            title_bar_style: Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),

            // File viewer cursor: Yellow background, black text, bold
            file_viewer_cursor_style: Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),

            // File viewer normal text: default style
            file_viewer_normal_style: Style::default(),

            // Tree viewer normal node: default style
            tree_normal_style: Style::default(),

            // Tree viewer selected node: Blue background, white text, bold
            tree_selected_style: Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),

            // Info panel background: Black background, white text
            info_panel_bg_style: Style::default().bg(Color::Black).fg(Color::White),

            // Info panel field labels: Yellow text
            info_panel_label_style: Style::default().fg(Color::Yellow),

            // Info panel Tree selection mode label: Cyan text, bold
            info_panel_mode_tree_style: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),

            // Info panel Text selection mode label: Green text, bold
            info_panel_mode_text_style: Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),

            // Error styling: Red text, bold
            error_style: Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),

            // Border styling: default style (can be customized)
            border_style: Style::default(),
        }
    }
}
