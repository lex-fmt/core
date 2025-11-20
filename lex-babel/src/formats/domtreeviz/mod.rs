//! DOM Tree Visualization - Source-first tree representation
//!
//! This format provides 1:1 correspondence with source lines by iterating
//! through the source and showing both block context and line content inline.
//!
//! ## Concept
//!
//! For each source line, show:
//! - The containing **block element** (Session, Definition, List, etc.)
//! - The **line node** itself (TextLine, ListItem, etc.)
//! - On the same output line, using indentation to show nesting
//!
//! ## Example
//!
//! ```text
//! ↵ This is a paragraph.
//! ≔  ↵ This definition contains...
//! ≔  • Item 1 in definition
//! § 1. Primary Session
//! §   ↵ Content in session
//! §   • List item in session
//! §   §  1.1. Nested Session
//! ```
//!
//! ## Visual Language
//!
//! - Indentation shows block nesting depth
//! - First icon shows block element (≔ Definition, § Session, etc.)
//! - Second icon shows line type (↵ TextLine, • ListItem, etc.)
//! - Text is the line content
//!
//! ## Traits
//!
//! - `VisualLine`: Nodes that correspond to source lines
//! - `BlockElement`: Container/structural elements

use crate::error::FormatError;
use crate::format::Format;
use lex_parser::lex::ast::{AstNode, ContentItem, Document, Position};

/// Marker trait for nodes that correspond to source lines
pub trait VisualLine {
    fn is_visual_line(&self) -> bool {
        true
    }
}

/// Marker trait for block/container elements
pub trait BlockElement {
    fn is_block_element(&self) -> bool {
        true
    }
}

/// Get icon for a node type
fn get_icon(node_type: &str) -> &'static str {
    match node_type {
        "Document" => "⧉",
        "Session" => "§",
        "Paragraph" => "¶",
        "TextLine" => "↵",
        "List" => "☰",
        "ListItem" => "•",
        "Definition" => "≔",
        "VerbatimBlock" => "𝒱",
        "VerbatimLine" => "↵",
        "Annotation" => "\"",
        "BlankLineGroup" => "○",
        _ => "○",
    }
}

/// Truncate text to max length
fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() > max_chars {
        let mut truncated = s.chars().take(max_chars).collect::<String>();
        truncated.push_str("...");
        truncated
    } else {
        s.to_string()
    }
}

/// Check if a node type is a meaningful block element (structural containers, not collections)
fn is_meaningful_block(node_type: &str) -> bool {
    matches!(
        node_type,
        "Session" | "Definition" | "Annotation" | "VerbatimBlock"
    )
}

/// Check if a node type is a visual line
fn is_visual_line(node_type: &str) -> bool {
    matches!(
        node_type,
        "TextLine"
            | "ListItem"
            | "VerbatimLine"
            | "BlankLineGroup"
            | "Session"
            | "Definition"
            | "Annotation"
            | "VerbatimBlock"
    )
}

/// Find the full path of ancestor nodes from root to the node at the given position
fn find_path_to_position(document: &Document, position: Position) -> Vec<String> {
    let mut path = vec!["Document".to_string()];

    // Check document-level annotations
    for ann in &document.annotations {
        if ann.range().contains(position) {
            path.push("Annotation".to_string());
            // Even if position is not in children (e.g., on marker lines), keep Annotation in path
            find_path_in_content_items_with_container(&ann.children, position, &mut path);
            return path;
        }
    }

    // Check root content
    find_path_in_content_items_with_container(&document.root.children, position, &mut path);
    path
}

/// Recursively find path in content items, checking both container ranges and children
fn find_path_in_content_items_with_container(
    items: &[ContentItem],
    position: Position,
    path: &mut Vec<String>,
) -> bool {
    for item in items {
        if !item.range().contains(position) {
            continue;
        }

        path.push(item.node_type().to_string());

        match item {
            ContentItem::Session(session) => {
                // Check session annotations first
                for ann in &session.annotations {
                    if ann.range().contains(position) {
                        path.push("Annotation".to_string());
                        find_path_in_content_items_with_container(&ann.children, position, path);
                        return true;
                    }
                }
                // Then check session content
                find_path_in_content_items_with_container(&session.children, position, path);
                return true;
            }
            ContentItem::Annotation(ann) => {
                // For annotations, even marker lines should show annotation context
                find_path_in_content_items_with_container(&ann.children, position, path);
                return true;
            }
            ContentItem::Definition(def) => {
                // Check definition annotations first
                for ann in &def.annotations {
                    if ann.range().contains(position) {
                        path.push("Annotation".to_string());
                        find_path_in_content_items_with_container(&ann.children, position, path);
                        return true;
                    }
                }
                // Then check definition content
                find_path_in_content_items_with_container(&def.children, position, path);
                return true;
            }
            ContentItem::VerbatimBlock(verb) => {
                // Check verbatim annotations first
                for ann in &verb.annotations {
                    if ann.range().contains(position) {
                        path.push("Annotation".to_string());
                        find_path_in_content_items_with_container(&ann.children, position, path);
                        return true;
                    }
                }
                // Then check verbatim content
                find_path_in_content_items_with_container(&verb.children, position, path);
                return true;
            }
            ContentItem::List(list) => {
                find_path_in_content_items_with_container(&list.items, position, path);
                return true;
            }
            ContentItem::ListItem(list_item) => {
                // Check list item annotations first
                for ann in &list_item.annotations {
                    if ann.range().contains(position) {
                        path.push("Annotation".to_string());
                        find_path_in_content_items_with_container(&ann.children, position, path);
                        return true;
                    }
                }
                // Then check list item content
                find_path_in_content_items_with_container(&list_item.children, position, path);
                return true;
            }
            ContentItem::Paragraph(para) => {
                find_path_in_content_items_with_container(&para.lines, position, path);
                return true;
            }
            _ => {
                // Leaf items (TextLine, VerbatimLine, BlankLineGroup)
                return true;
            }
        }
    }

    false
}

/// Format a single source line with its AST context
fn format_source_line(line_num: usize, line_text: &str, document: &Document) -> Option<String> {
    let first_char_col = line_text
        .chars()
        .position(|c| !c.is_whitespace())
        .unwrap_or(0);

    let pos = Position::new(line_num, first_char_col);

    // Get the full path of node types from root to leaf
    let path = find_path_to_position(document, pos);

    if path.len() <= 1 {
        // Only Document in path, no content
        return None;
    }

    // Find the visual line node (deepest in path that's a visual line)
    let line_node_type = path.iter().rev().find(|t| is_visual_line(t))?;

    // If the line node is itself a meaningful block (Session/Definition/etc),
    // we should NOT show a separate block icon - just show the line once
    let line_is_meaningful_block = is_meaningful_block(line_node_type);

    // Find the meaningful block ancestor (skip Paragraph/List, show Session/Definition/etc)
    // But only if the line itself is not a meaningful block
    let line_idx = path.iter().rposition(|t| t == line_node_type)?;
    let block_node_type = if !line_is_meaningful_block {
        path[..line_idx]
            .iter()
            .rev()
            .find(|t| is_meaningful_block(t))
    } else {
        None
    };

    // Calculate indentation: count meaningful blocks in path before the line
    let meaningful_blocks_before_line: Vec<_> = path[..line_idx]
        .iter()
        .filter(|t| is_meaningful_block(t))
        .collect();

    let depth = if block_node_type.is_some() {
        // If showing a block icon, indent by remaining blocks
        meaningful_blocks_before_line.len().saturating_sub(1)
    } else {
        // If line is a block itself, indent by all blocks before it
        meaningful_blocks_before_line.len()
    };
    let indent = "  ".repeat(depth);

    // Build output line
    let mut output = String::new();
    output.push_str(&indent);

    // If there's a block element (and line is not itself a block), show its icon
    if let Some(block_type) = block_node_type {
        let block_icon = get_icon(block_type);
        output.push_str(block_icon);
        output.push(' ');
    }

    // Show line node icon
    let line_icon = get_icon(line_node_type);
    output.push_str(line_icon);
    output.push(' ');

    // Show actual source line content (not AST node label)
    // This ensures 1:1 correspondence with source
    let label = truncate(line_text.trim(), 60);
    output.push_str(&label);

    Some(output)
}

/// Convert a document to domtreeviz format
pub fn to_domtreeviz_str(doc: &Document, source: &str) -> String {
    let mut output = String::new();

    for (line_num, line_text) in source.lines().enumerate() {
        if let Some(formatted) = format_source_line(line_num, line_text, doc) {
            output.push_str(&formatted);
            output.push('\n');
        }
    }

    output
}

/// Format implementation for DOM tree visualization
pub struct DomTreevizFormat;

impl Format for DomTreevizFormat {
    fn name(&self) -> &str {
        "domtreeviz"
    }

    fn description(&self) -> &str {
        "DOM-like tree with inline block+line display (1:1 source correspondence)"
    }

    fn file_extensions(&self) -> &[&str] {
        &["domtree"]
    }

    fn supports_serialization(&self) -> bool {
        true
    }

    fn supports_parsing(&self) -> bool {
        false
    }

    fn serialize(&self, _doc: &Document) -> Result<String, FormatError> {
        Err(FormatError::NotSupported(
            "domtreeviz requires source text - use serialize_with_source()".to_string(),
        ))
    }
}

impl DomTreevizFormat {
    /// Serialize with source text (required for this format)
    pub fn serialize_with_source(
        &self,
        doc: &Document,
        source: &str,
    ) -> Result<String, FormatError> {
        Ok(to_domtreeviz_str(doc, source))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_mapping() {
        assert_eq!(get_icon("Session"), "§");
        assert_eq!(get_icon("TextLine"), "↵");
        assert_eq!(get_icon("ListItem"), "•");
        assert_eq!(get_icon("Definition"), "≔");
    }

    #[test]
    fn test_is_visual_line() {
        assert!(is_visual_line("TextLine"));
        assert!(is_visual_line("ListItem"));
        assert!(is_visual_line("Session"));
        assert!(!is_visual_line("List"));
        assert!(!is_visual_line("Paragraph"));
    }

    #[test]
    fn test_is_meaningful_block() {
        assert!(is_meaningful_block("Session"));
        assert!(is_meaningful_block("Definition"));
        assert!(is_meaningful_block("Annotation"));
        assert!(is_meaningful_block("VerbatimBlock"));
        assert!(!is_meaningful_block("List"));
        assert!(!is_meaningful_block("Paragraph"));
        assert!(!is_meaningful_block("TextLine"));
        assert!(!is_meaningful_block("ListItem"));
    }
}
