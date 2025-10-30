//! Tree viewer - displays and navigates the AST structure
//!
//! The tree viewer shows the document as a tree of nodes.
//! Users can navigate with arrow keys and expand/collapse nodes.

use super::model::{Model, NodeId, Selection};
use super::viewer::{Viewer, ViewerEvent};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

/// Map node types to their icons based on treeviz.rs comments
fn get_node_icon(node_type: &str) -> &'static str {
    match node_type {
        "Document" => "⧉",
        "Session" => "§",
        "SessionTitle" => "⊤",
        "Annotation" => "\"",
        "Paragraph" => "¶",
        "List" => "☰",
        "ListItem" => "•",
        "Foreign" => "𝒱",
        "ForeignLine" => "℣",
        "Definition" => "≔",
        "SessionContainer" => "Ψ",
        "ContentContainer" => "➔",
        "Content" => "⊤",
        "Text" => "◦",
        "TextLine" => "↵",
        "Italic" => "𝐼",
        "Bold" => "𝐁",
        "Code" => "ƒ",
        "Math" => "√",
        "Reference" => "⊕",
        "ReferenceFile" => "/",
        "ReferenceCitation" => "†",
        "ReferenceCitationAuthor" => "@",
        "ReferenceCitationPage" => "◫",
        "ReferenceToCome" => "⋯",
        "ReferenceUnknown" => "∅",
        "ReferenceFootnote" => "³",
        "ReferenceSession" => "#",
        _ => "◦", // Default fallback
    }
}

/// Tree viewer - displays and navigates the AST structure
///
/// The tree viewer shows the document as a tree of nodes.
/// Users can navigate with arrow keys and expand/collapse nodes.
#[derive(Debug)]
#[allow(dead_code)]
pub struct TreeViewer {
    /// Currently selected node ID
    selected_node_id: Option<NodeId>,
    /// How many items are scrolled off the top
    scroll_offset: usize,
}

#[allow(dead_code)]
impl TreeViewer {
    /// Create a new tree viewer
    pub fn new() -> Self {
        TreeViewer {
            selected_node_id: None,
            scroll_offset: 0,
        }
    }

    /// Get the currently selected node ID
    pub fn selected_node_id(&self) -> Option<NodeId> {
        self.selected_node_id
    }

    /// Get the scroll offset
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Get the next visible node in the flattened tree
    ///
    /// Returns the NodeId of the next node considering only visible nodes
    /// (respecting expansion state). Returns None if already at the last node.
    pub fn get_next_visible_node(&self, current_node_id: NodeId, model: &Model) -> Option<NodeId> {
        let flattened = model.flattened_tree();

        // Find the current node in the flattened tree
        if let Some(current_index) = flattened.iter().position(|n| n.node_id == current_node_id) {
            // Return the next node if it exists
            if current_index < flattened.len() - 1 {
                return Some(flattened[current_index + 1].node_id);
            }
        }

        None
    }

    /// Get the previous visible node in the flattened tree
    ///
    /// Returns the NodeId of the previous node considering only visible nodes
    /// (respecting expansion state). Returns None if already at the first node.
    pub fn get_previous_visible_node(
        &self,
        current_node_id: NodeId,
        model: &Model,
    ) -> Option<NodeId> {
        let flattened = model.flattened_tree();

        // Find the current node in the flattened tree
        if let Some(current_index) = flattened.iter().position(|n| n.node_id == current_node_id) {
            // Return the previous node if it exists
            if current_index > 0 {
                return Some(flattened[current_index - 1].node_id);
            }
        }

        None
    }
}

impl Default for TreeViewer {
    fn default() -> Self {
        Self::new()
    }
}

impl Viewer for TreeViewer {
    fn render(&self, frame: &mut Frame, area: Rect, model: &Model) {
        // Get the flattened tree for rendering
        let flattened = model.flattened_tree();

        // Determine which node should be highlighted
        // This handles both TreeSelection and TextSelection types
        let highlighted_node_id = match model.selection() {
            Selection::TreeSelection(node_id) => Some(node_id),
            Selection::TextSelection(row, col) => model.get_node_at_position(row, col),
        };

        // Build lines from the flattened tree
        let lines: Vec<Line> = flattened
            .iter()
            .map(|node| {
                // Build indentation based on depth (2 spaces per level)
                let indent = "  ".repeat(node.depth);

                // Get the icon for this node type
                let icon = get_node_icon(node.node_type);

                let text = format!("{}{} {}", indent, icon, node.label);

                // Style the line based on selection and expansion state
                let is_collapsed = !node.is_expanded && node.has_children;
                let is_highlighted = Some(node.node_id) == highlighted_node_id;

                if is_highlighted {
                    // Highlighted node: blue background, but muted text if collapsed
                    let text_color = if is_collapsed {
                        Color::Gray
                    } else {
                        Color::White
                    };

                    Line::from(text).style(
                        Style::default()
                            .bg(Color::Blue)
                            .fg(text_color)
                            .add_modifier(Modifier::BOLD),
                    )
                } else if is_collapsed {
                    // Collapsed node (not highlighted): muted gray text
                    Line::from(text)
                        .style(Style::default().fg(Color::Gray).add_modifier(Modifier::DIM))
                } else {
                    // Normal node: default styling
                    Line::from(text)
                }
            })
            .collect();

        // Create a paragraph widget to display the tree
        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, area);
    }

    fn handle_key(&mut self, key: KeyEvent, model: &Model) -> Option<ViewerEvent> {
        // Initialize selection to first visible node if not yet selected
        if self.selected_node_id.is_none() {
            let flattened = model.flattened_tree();
            if !flattened.is_empty() {
                self.selected_node_id = Some(flattened[0].node_id);
            }
        }

        let Some(current_node_id) = self.selected_node_id else {
            return Some(ViewerEvent::NoChange);
        };

        match key.code {
            KeyCode::Up => {
                // Move to previous visible node
                if let Some(prev_node_id) = self.get_previous_visible_node(current_node_id, model) {
                    self.selected_node_id = Some(prev_node_id);
                    Some(ViewerEvent::SelectNode(prev_node_id))
                } else {
                    Some(ViewerEvent::NoChange)
                }
            }
            KeyCode::Down => {
                // Move to next visible node
                if let Some(next_node_id) = self.get_next_visible_node(current_node_id, model) {
                    self.selected_node_id = Some(next_node_id);
                    Some(ViewerEvent::SelectNode(next_node_id))
                } else {
                    Some(ViewerEvent::NoChange)
                }
            }
            KeyCode::Left => {
                // Toggle collapse for the currently selected node
                Some(ViewerEvent::ToggleNodeExpansion(current_node_id))
            }
            KeyCode::Right => {
                // Toggle expand for the currently selected node
                Some(ViewerEvent::ToggleNodeExpansion(current_node_id))
            }
            _ => Some(ViewerEvent::NoChange),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_viewer_creation() {
        let viewer = TreeViewer::new();
        assert_eq!(viewer.selected_node_id(), None);
        assert_eq!(viewer.scroll_offset(), 0);
    }
}
