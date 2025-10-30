//! Theme configuration for txxtv UI
//!
//! Implements a three-layer theming architecture for clean separation of concerns:
//!
//! **Presentation Layer:** Actual Style values organized by semantic role
//! - Contains concrete style definitions used throughout the application
//! - Example: `active`, `normal`, `label`, `error`, etc.
//! - This is where you tweak colors and styling
//!
//! **Semantic Layer:** Implicit - defined by which styles multiple UI elements share
//! - Example: Both "file_viewer_cursor" and "tree_viewer_selected" map to "active"
//! - This ensures related UI elements look visually consistent
//! - Multiple application-layer elements → same presentation style
//!
//! **Application Layer:** Methods named after UI element locations
//! - Example: `file_viewer_cursor()`, `tree_viewer_selected()`, `info_panel_label()`
//! - Application code uses these stable, semantic-aware method names
//! - Easy to understand what style each UI element gets
//!
//! Benefits:
//! - Changing presentation.active updates all "active" elements consistently
//! - Adding new UI elements is simple: just add a new method that uses existing styles
//! - Application code remains stable even when tweaking the visual design
//! - Clear intent: methods show which elements should look the same

use ratatui::style::{Color, Modifier, Style};

/// Presentation layer: Actual Style values
///
/// Contains all the concrete style definitions used by the application.
/// Styles are grouped by semantic role (e.g., active elements, normal content, labels).
#[derive(Debug, Clone)]
pub struct PresentationStyles {
    /// Active/selected elements (cursors, highlighted nodes, selections)
    pub active: Style,

    /// Normal content (regular text, unselected nodes)
    pub normal: Style,

    /// Label text (field names in info panel)
    pub label: Style,

    /// Mode indicators specific to tree selection mode
    pub mode_tree: Style,

    /// Mode indicators specific to text selection mode
    pub mode_text: Style,

    /// Error and important messages
    pub error: Style,

    /// Title/header bar
    pub title: Style,

    /// Info panel background
    pub panel_bg: Style,

    /// Borders
    #[allow(dead_code)]
    pub border: Style,
}

/// Application and Semantic Layers
///
/// This struct provides the public interface for styling UI elements.
/// Methods are named after UI locations (application layer),
/// and implicitly group them semantically by which presentation style they use.
#[derive(Debug, Clone)]
pub struct Theme {
    presentation: PresentationStyles,
}

impl Theme {
    // ============================================================================
    // APPLICATION LAYER: Methods named after UI element locations
    // ============================================================================
    // These methods map application-layer names to the semantic layer
    // (implicit in which presentation style they use) and finally to actual styles.

    /// Styling for the file viewer text cursor
    /// Semantic: active element
    pub fn file_viewer_cursor(&self) -> Style {
        self.presentation.active
    }

    /// Styling for normal text in the file viewer
    /// Semantic: normal element
    pub fn file_viewer_text(&self) -> Style {
        self.presentation.normal
    }

    /// Styling for selected nodes in the tree viewer
    /// Semantic: active element (same as file_viewer_cursor)
    pub fn tree_viewer_selected(&self) -> Style {
        self.presentation.active
    }

    /// Styling for normal nodes in the tree viewer
    /// Semantic: normal element (same as file_viewer_text)
    pub fn tree_viewer_normal(&self) -> Style {
        self.presentation.normal
    }

    /// Styling for labels in the info panel
    /// Semantic: label text
    pub fn info_panel_label(&self) -> Style {
        self.presentation.label
    }

    /// Styling for the tree mode indicator in info panel
    /// Semantic: tree mode indicator (distinct from other modes)
    pub fn info_panel_mode_tree(&self) -> Style {
        self.presentation.mode_tree
    }

    /// Styling for the text mode indicator in info panel
    /// Semantic: text mode indicator (distinct from other modes)
    pub fn info_panel_mode_text(&self) -> Style {
        self.presentation.mode_text
    }

    /// Styling for the info panel background
    /// Semantic: container background
    pub fn info_panel_bg(&self) -> Style {
        self.presentation.panel_bg
    }

    /// Styling for error messages
    /// Semantic: error/important content
    pub fn error_message(&self) -> Style {
        self.presentation.error
    }

    /// Styling for the title bar
    /// Semantic: title/header
    pub fn title_bar(&self) -> Style {
        self.presentation.title
    }

    /// Styling for borders
    /// Semantic: border element
    #[allow(dead_code)]
    pub fn border(&self) -> Style {
        self.presentation.border
    }

    // ============================================================================
    // CONSTRUCTORS: Creating themes with custom presentation styles
    // ============================================================================

    /// Create a theme with a custom presentation layer
    #[allow(dead_code)]
    pub fn with_presentation(presentation: PresentationStyles) -> Self {
        Theme { presentation }
    }

    // ============================================================================
    // INTERNAL: Access to presentation layer for advanced customization
    // ============================================================================

    /// Get a mutable reference to presentation styles for customization
    #[allow(dead_code)]
    pub fn presentation_mut(&mut self) -> &mut PresentationStyles {
        &mut self.presentation
    }
}

impl Default for Theme {
    /// Create the default theme with sensible colors and styling
    fn default() -> Self {
        Theme {
            presentation: PresentationStyles {
                // SEMANTIC: Active/selected elements - Blue background, white bold
                // Includes: file_viewer_cursor, tree_viewer_selected
                active: Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),

                // SEMANTIC: Normal content - Default style
                // Includes: file_viewer_text, tree_viewer_normal
                normal: Style::default(),

                // SEMANTIC: Label text - Yellow text
                // Includes: info_panel_label
                label: Style::default().fg(Color::Yellow),

                // SEMANTIC: Tree mode indicator - Cyan, bold
                // Includes: info_panel_mode_tree
                mode_tree: Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),

                // SEMANTIC: Text mode indicator - Green, bold
                // Includes: info_panel_mode_text
                mode_text: Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),

                // SEMANTIC: Error/important - Red, bold
                // Includes: error_message
                error: Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),

                // SEMANTIC: Title/header - Black text on Cyan background, bold
                // Includes: title_bar
                title: Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),

                // SEMANTIC: Container background - Black background, white text
                // Includes: info_panel_bg
                panel_bg: Style::default().bg(Color::Black).fg(Color::White),

                // SEMANTIC: Border - Default style
                // Includes: border
                border: Style::default(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_semantic_grouping() {
        let theme = Theme::default();
        // Verify that semantically related elements use the same style
        assert_eq!(
            theme.file_viewer_cursor(),
            theme.tree_viewer_selected(),
            "Active elements should share the same style"
        );
        assert_eq!(
            theme.file_viewer_text(),
            theme.tree_viewer_normal(),
            "Normal elements should share the same style"
        );
    }

    #[test]
    fn test_theme_default_colors() {
        let theme = Theme::default();
        let active = theme.file_viewer_cursor();
        let normal = theme.file_viewer_text();
        let label = theme.info_panel_label();

        // Verify active style has blue background
        assert_eq!(active.bg, Some(Color::Blue));
        // Verify normal style is default
        assert_eq!(normal.bg, None);
        // Verify label style has yellow foreground
        assert_eq!(label.fg, Some(Color::Yellow));
    }
}
