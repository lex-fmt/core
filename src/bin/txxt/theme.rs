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
//!
//! **YAML Configuration:** Load themes from YAML files
//! - Define colors and styles in YAML for easy customization
//! - Use `Theme::from_yaml()` to load a theme from a string
//! - Use `Theme::from_yaml_file()` to load a theme from a file

use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};
use std::fs;

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

    /// Muted/disabled elements (collapsed nodes, inactive content)
    pub muted: Style,

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

    /// Styling for collapsed/muted nodes in the tree viewer
    /// Semantic: muted/disabled element
    pub fn tree_viewer_collapsed(&self) -> Style {
        self.presentation.muted
    }

    /// Styling for labels in the info panel
    /// Semantic: label text
    #[allow(dead_code)]
    pub fn info_panel_label(&self) -> Style {
        self.presentation.label
    }

    /// Styling for the tree mode indicator in info panel
    /// Semantic: tree mode indicator (distinct from other modes)
    #[allow(dead_code)]
    pub fn info_panel_mode_tree(&self) -> Style {
        self.presentation.mode_tree
    }

    /// Styling for the text mode indicator in info panel
    /// Semantic: text mode indicator (distinct from other modes)
    #[allow(dead_code)]
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

    // ============================================================================
    // YAML LOADING: Load themes from YAML configuration files
    // ============================================================================

    /// Load a theme from a YAML string
    pub fn from_yaml(yaml_str: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config: ThemeConfig = serde_yaml::from_str(yaml_str)?;
        Ok(config.into_theme())
    }

    /// Load a theme from a YAML file
    pub fn from_yaml_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        Self::from_yaml(&content)
    }
}

/// YAML-deserializable theme configuration
///
/// Represents all themeable properties that can be loaded from YAML.
/// Each style can specify foreground, background, and modifiers.
/// Colors can be specified by name (e.g., "red", "blue") or as RGB (e.g., "#FF0000").
/// Modifiers can be: "bold", "dim", "italic", "underline", "slow_blink", "rapid_blink", "reverse", "hidden", "crossed_out"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Active/selected elements (cursors, highlighted nodes, selections)
    pub active: Option<StyleConfig>,

    /// Normal content (regular text, unselected nodes)
    pub normal: Option<StyleConfig>,

    /// Muted/disabled elements (collapsed nodes, inactive content)
    pub muted: Option<StyleConfig>,

    /// Label text (field names in info panel)
    pub label: Option<StyleConfig>,

    /// Mode indicators specific to tree selection mode
    pub mode_tree: Option<StyleConfig>,

    /// Mode indicators specific to text selection mode
    pub mode_text: Option<StyleConfig>,

    /// Error and important messages
    pub error: Option<StyleConfig>,

    /// Title/header bar
    pub title: Option<StyleConfig>,

    /// Info panel background
    pub panel_bg: Option<StyleConfig>,

    /// Borders
    pub border: Option<StyleConfig>,
}

/// Individual style configuration from YAML
///
/// Represents the customizable aspects of a ratatui Style.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleConfig {
    /// Foreground color (e.g., "red", "blue", "#FF0000")
    pub fg: Option<String>,

    /// Background color (e.g., "red", "blue", "#FF0000")
    pub bg: Option<String>,

    /// Text modifiers (e.g., "bold", "italic", "underline")
    pub modifiers: Option<Vec<String>>,
}

impl ThemeConfig {
    /// Convert a ThemeConfig (from YAML) into a Theme (with actual styles)
    pub fn into_theme(self) -> Theme {
        let default = Theme::default();
        let default_presentation = default.presentation.clone();

        Theme {
            presentation: PresentationStyles {
                active: self
                    .active
                    .as_ref()
                    .map(|s| s.to_style())
                    .unwrap_or(default_presentation.active),
                normal: self
                    .normal
                    .as_ref()
                    .map(|s| s.to_style())
                    .unwrap_or(default_presentation.normal),
                muted: self
                    .muted
                    .as_ref()
                    .map(|s| s.to_style())
                    .unwrap_or(default_presentation.muted),
                label: self
                    .label
                    .as_ref()
                    .map(|s| s.to_style())
                    .unwrap_or(default_presentation.label),
                mode_tree: self
                    .mode_tree
                    .as_ref()
                    .map(|s| s.to_style())
                    .unwrap_or(default_presentation.mode_tree),
                mode_text: self
                    .mode_text
                    .as_ref()
                    .map(|s| s.to_style())
                    .unwrap_or(default_presentation.mode_text),
                error: self
                    .error
                    .as_ref()
                    .map(|s| s.to_style())
                    .unwrap_or(default_presentation.error),
                title: self
                    .title
                    .as_ref()
                    .map(|s| s.to_style())
                    .unwrap_or(default_presentation.title),
                panel_bg: self
                    .panel_bg
                    .as_ref()
                    .map(|s| s.to_style())
                    .unwrap_or(default_presentation.panel_bg),
                border: self
                    .border
                    .as_ref()
                    .map(|s| s.to_style())
                    .unwrap_or(default_presentation.border),
            },
        }
    }
}

impl StyleConfig {
    /// Convert a StyleConfig into a ratatui Style
    fn to_style(&self) -> Style {
        let mut style = Style::default();

        // Apply foreground color
        if let Some(fg) = &self.fg {
            if let Some(color) = parse_color(fg) {
                style = style.fg(color);
            }
        }

        // Apply background color
        if let Some(bg) = &self.bg {
            if let Some(color) = parse_color(bg) {
                style = style.bg(color);
            }
        }

        // Apply modifiers
        if let Some(modifiers) = &self.modifiers {
            for modifier_str in modifiers {
                if let Some(modifier) = parse_modifier(modifier_str) {
                    style = style.add_modifier(modifier);
                }
            }
        }

        style
    }
}

/// Parse a color name or hex code into a ratatui Color
///
/// Supports:
/// - Named colors: "black", "red", "green", "yellow", "blue", "magenta", "cyan", "white", "gray", "dark_gray"
/// - Indexed colors: "color0" through "color255"
/// - RGB hex: "#FF0000" or "#fff"
fn parse_color(color_str: &str) -> Option<Color> {
    let lower = color_str.to_lowercase();

    // Named colors
    match lower.as_str() {
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "white" => Some(Color::White),
        "gray" => Some(Color::Gray),
        "dark_gray" => Some(Color::DarkGray),
        _ => {
            // Try indexed color (color0-color255)
            if let Some(color_idx) = lower.strip_prefix("color") {
                if let Ok(idx) = color_idx.parse::<u8>() {
                    return Some(Color::Indexed(idx));
                }
            }

            // Try RGB hex
            if let Some(hex) = lower.strip_prefix('#') {
                if hex.len() == 6 {
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        u8::from_str_radix(&hex[0..2], 16),
                        u8::from_str_radix(&hex[2..4], 16),
                        u8::from_str_radix(&hex[4..6], 16),
                    ) {
                        return Some(Color::Rgb(r, g, b));
                    }
                } else if hex.len() == 3 {
                    // Short form: #fff -> #ffffff
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        u8::from_str_radix(&format!("{}{}", &hex[0..1], &hex[0..1]), 16),
                        u8::from_str_radix(&format!("{}{}", &hex[1..2], &hex[1..2]), 16),
                        u8::from_str_radix(&format!("{}{}", &hex[2..3], &hex[2..3]), 16),
                    ) {
                        return Some(Color::Rgb(r, g, b));
                    }
                }
            }

            None
        }
    }
}

/// Parse a modifier name into a ratatui Modifier
fn parse_modifier(modifier_str: &str) -> Option<Modifier> {
    match modifier_str.to_lowercase().as_str() {
        "bold" => Some(Modifier::BOLD),
        "dim" => Some(Modifier::DIM),
        "italic" => Some(Modifier::ITALIC),
        "underline" => Some(Modifier::UNDERLINED),
        "slow_blink" => Some(Modifier::SLOW_BLINK),
        "rapid_blink" => Some(Modifier::RAPID_BLINK),
        "reverse" => Some(Modifier::REVERSED),
        "hidden" => Some(Modifier::HIDDEN),
        "crossed_out" => Some(Modifier::CROSSED_OUT),
        _ => None,
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

                // SEMANTIC: Muted/disabled elements - Dim gray text
                // Includes: tree_viewer_collapsed
                muted: Style::default().fg(Color::Gray).add_modifier(Modifier::DIM),

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

    #[test]
    fn test_parse_color_named() {
        // Test named colors
        assert_eq!(parse_color("red"), Some(Color::Red));
        assert_eq!(parse_color("blue"), Some(Color::Blue));
        assert_eq!(parse_color("RED"), Some(Color::Red)); // Case insensitive
        assert_eq!(parse_color("CyAn"), Some(Color::Cyan));
        assert_eq!(parse_color("dark_gray"), Some(Color::DarkGray));
    }

    #[test]
    fn test_parse_color_hex() {
        // Test hex colors (6-digit)
        assert_eq!(parse_color("#FF0000"), Some(Color::Rgb(255, 0, 0)));
        assert_eq!(parse_color("#00FF00"), Some(Color::Rgb(0, 255, 0)));
        assert_eq!(parse_color("#ff00ff"), Some(Color::Rgb(255, 0, 255))); // Case insensitive

        // Test hex colors (3-digit short form)
        assert_eq!(parse_color("#f00"), Some(Color::Rgb(255, 0, 0)));
        assert_eq!(parse_color("#0f0"), Some(Color::Rgb(0, 255, 0)));
        assert_eq!(parse_color("#00f"), Some(Color::Rgb(0, 0, 255)));
    }

    #[test]
    fn test_parse_color_indexed() {
        // Test indexed colors
        assert_eq!(parse_color("color0"), Some(Color::Indexed(0)));
        assert_eq!(parse_color("color255"), Some(Color::Indexed(255)));
        assert_eq!(parse_color("color128"), Some(Color::Indexed(128)));
    }

    #[test]
    fn test_parse_modifier() {
        assert_eq!(parse_modifier("bold"), Some(Modifier::BOLD));
        assert_eq!(parse_modifier("italic"), Some(Modifier::ITALIC));
        assert_eq!(parse_modifier("underline"), Some(Modifier::UNDERLINED));
        assert_eq!(parse_modifier("BOLD"), Some(Modifier::BOLD)); // Case insensitive
        assert_eq!(parse_modifier("invalid"), None);
    }

    #[test]
    fn test_yaml_minimal_theme() {
        let yaml = r#"
active:
  fg: red
  bg: blue
"#;
        let theme = Theme::from_yaml(yaml).expect("Should parse valid YAML");
        assert_eq!(theme.file_viewer_cursor().fg, Some(Color::Red));
        assert_eq!(theme.file_viewer_cursor().bg, Some(Color::Blue));
        // Other elements should use defaults
        assert_eq!(
            theme.file_viewer_text(),
            Theme::default().file_viewer_text()
        );
    }

    #[test]
    fn test_yaml_with_modifiers() {
        let yaml = r#"
label:
  fg: yellow
  modifiers: [bold, underline]
"#;
        let theme = Theme::from_yaml(yaml).expect("Should parse valid YAML");
        let label_style = theme.info_panel_label();
        assert_eq!(label_style.fg, Some(Color::Yellow));
        assert!(label_style.add_modifier.contains(Modifier::BOLD));
        assert!(label_style.add_modifier.contains(Modifier::UNDERLINED));
    }

    #[test]
    fn test_yaml_with_hex_colors() {
        let yaml = "active:\n  fg: \"#FF5500\"\n  bg: \"#001100\"\n";
        let theme = Theme::from_yaml(yaml).expect("Should parse valid YAML");
        assert_eq!(theme.file_viewer_cursor().fg, Some(Color::Rgb(255, 85, 0)));
        assert_eq!(theme.file_viewer_cursor().bg, Some(Color::Rgb(0, 17, 0)));
    }

    #[test]
    fn test_yaml_partial_override() {
        let yaml = r#"
title:
  fg: green
mode_text:
  fg: red
  modifiers: [bold]
"#;
        let theme = Theme::from_yaml(yaml).expect("Should parse valid YAML");
        // Override elements
        assert_eq!(theme.title_bar().fg, Some(Color::Green));
        assert_eq!(theme.info_panel_mode_text().fg, Some(Color::Red));
        assert!(theme
            .info_panel_mode_text()
            .add_modifier
            .contains(Modifier::BOLD));
        // Non-override elements use defaults
        assert_eq!(
            theme.file_viewer_cursor(),
            Theme::default().file_viewer_cursor()
        );
    }

    #[test]
    fn test_yaml_invalid_syntax() {
        let yaml = "{ invalid: yaml: content";
        assert!(Theme::from_yaml(yaml).is_err());
    }

    #[test]
    fn test_style_config_to_style() {
        let config = StyleConfig {
            fg: Some("blue".to_string()),
            bg: Some("yellow".to_string()),
            modifiers: Some(vec!["bold".to_string(), "italic".to_string()]),
        };
        let style = config.to_style();
        assert_eq!(style.fg, Some(Color::Blue));
        assert_eq!(style.bg, Some(Color::Yellow));
        assert!(style.add_modifier.contains(Modifier::BOLD));
        assert!(style.add_modifier.contains(Modifier::ITALIC));
    }
}
