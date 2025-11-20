//! Line Tree Visualization - Collapsed tree representation
//!
//! This format provides the same visual tree structure as treeviz but collapses
//! homogeneous container nodes (Paragraph, List) with their children by showing
//! combined parent+child icons (e.g., ¶ ↵ for Paragraph/TextLine, ☰ • for List/ListItem).
//!
//! ## Example
//!
//! ```text
//! ⧉ Document (0 annotations, 2 items)
//! ├─ § Session Title
//! │ ├─ ¶ ↵ First line of paragraph
//! │ └─ ¶ ↵ Second line of paragraph
//! └─ ☰ • List item 1
//!   └─ ☰ • List item 2
//! ```
//!
//! ## Key Differences from treeviz
//!
//! - Collapses Paragraph containers with TextLine children (shows `¶ ↵` not separate nodes)
//! - Collapses List containers with ListItem children (shows `☰ •` not separate nodes)
//! - Uses VisualStructure trait to identify collapsible containers
//! - Shares icon mapping with treeviz

use super::icons::get_icon;
use crate::error::FormatError;
use crate::format::Format;
use lex_parser::lex::ast::{
    snapshot_from_document, snapshot_from_document_with_options, AstSnapshot, Document,
};
use std::collections::HashMap;

/// Check if a node type represents a collapsible container
fn is_collapsible_container(node_type: &str) -> bool {
    matches!(node_type, "Paragraph" | "List")
}

/// Build treeviz output from an AstSnapshot, collapsing container nodes
fn format_snapshot(
    snapshot: &AstSnapshot,
    prefix: &str,
    child_index: usize,
    child_count: usize,
) -> String {
    let mut output = String::new();

    let is_last = child_index == child_count - 1;
    let connector = if is_last { "└─" } else { "├─" };

    // Check if this is a collapsible container
    if is_collapsible_container(&snapshot.node_type) && !snapshot.children.is_empty() {
        // Don't show the container itself, show children with combined icons
        let parent_icon = get_icon(&snapshot.node_type);

        for (i, child) in snapshot.children.iter().enumerate() {
            let child_is_last = i == snapshot.children.len() - 1;
            let child_connector = if child_is_last { "└─" } else { "├─" };
            let child_icon = get_icon(&child.node_type);

            // Show combined parent+child icon
            output.push_str(&format!(
                "{}{} {} {} {}\n",
                prefix, child_connector, parent_icon, child_icon, child.label
            ));

            // Process grandchildren with adjusted prefix
            if !child.children.is_empty() {
                let grandchild_prefix =
                    format!("{}{}", prefix, if child_is_last { "  " } else { "│ " });
                let grandchild_count = child.children.len();

                for (j, grandchild) in child.children.iter().enumerate() {
                    output.push_str(&format_snapshot(
                        grandchild,
                        &grandchild_prefix,
                        j,
                        grandchild_count,
                    ));
                }
            }
        }
    } else {
        // Normal node - show as usual
        let icon = get_icon(&snapshot.node_type);

        output.push_str(&format!(
            "{}{} {} {}\n",
            prefix, connector, icon, snapshot.label
        ));

        // Process children if any
        if !snapshot.children.is_empty() {
            let child_prefix = format!("{}{}", prefix, if is_last { "  " } else { "│ " });
            let child_count = snapshot.children.len();

            for (i, child) in snapshot.children.iter().enumerate() {
                output.push_str(&format_snapshot(child, &child_prefix, i, child_count));
            }
        }
    }

    output
}

fn format_document_snapshot(snapshot: &AstSnapshot) -> String {
    let icon = get_icon(&snapshot.node_type);
    let mut output = format!("{} {}\n", icon, snapshot.label);

    if !snapshot.children.is_empty() {
        let child_count = snapshot.children.len();
        for (i, child) in snapshot.children.iter().enumerate() {
            output.push_str(&format_snapshot(child, "", i, child_count));
        }
    }

    output
}

pub fn to_linetreeviz_str(doc: &Document) -> String {
    to_linetreeviz_str_with_params(doc, &HashMap::new())
}

/// Convert a document to linetreeviz string with optional parameters
///
/// # Parameters
///
/// - `"ast-full"`: When set to `"true"`, includes all AST node properties
///   (same as treeviz)
pub fn to_linetreeviz_str_with_params(doc: &Document, params: &HashMap<String, String>) -> String {
    // Check if ast-full parameter is set to true
    let include_all = params
        .get("ast-full")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);

    let snapshot = if include_all {
        snapshot_from_document_with_options(doc, true)
    } else {
        snapshot_from_document(doc)
    };

    format_document_snapshot(&snapshot)
}

/// Format implementation for line tree visualization
pub struct LinetreevizFormat;

impl Format for LinetreevizFormat {
    fn name(&self) -> &str {
        "linetreeviz"
    }

    fn description(&self) -> &str {
        "Tree visualization with collapsed containers (Paragraph/List)"
    }

    fn file_extensions(&self) -> &[&str] {
        &["linetree"]
    }

    fn supports_serialization(&self) -> bool {
        true
    }

    fn supports_parsing(&self) -> bool {
        false
    }

    fn serialize(&self, doc: &Document) -> Result<String, FormatError> {
        Ok(to_linetreeviz_str(doc))
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
        assert_eq!(get_icon("Paragraph"), "¶");
        assert_eq!(get_icon("List"), "☰");
    }
}
