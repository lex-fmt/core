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
use lex_parser::lex::ast::traits::{AstNode, Container, VisualStructure};
use lex_parser::lex::ast::{ContentItem, Document};
use std::collections::HashMap;

/// Format a single ContentItem node with collapsing logic
fn format_content_item(
    item: &ContentItem,
    prefix: &str,
    child_index: usize,
    child_count: usize,
    show_linum: bool,
) -> String {
    let mut output = String::new();
    let is_last = child_index == child_count - 1;
    let connector = if is_last { "└─" } else { "├─" };

    // Check if this node collapses with its children using the VisualStructure trait
    let collapses = match item {
        ContentItem::Paragraph(p) => p.collapses_with_children(),
        ContentItem::List(l) => l.collapses_with_children(),
        ContentItem::Session(s) => s.collapses_with_children(),
        ContentItem::Definition(d) => d.collapses_with_children(),
        ContentItem::Annotation(a) => a.collapses_with_children(),
        ContentItem::VerbatimBlock(v) => v.collapses_with_children(),
        _ => false,
    };

    if collapses {
        // Get parent info
        let parent_icon = get_icon(item.node_type());
        let continuation_prefix = format!("{}{}", prefix, if is_last { "  " } else { "│ " });
        let children: Vec<&ContentItem> = match item {
            ContentItem::Paragraph(p) => p.lines.iter().collect(),
            ContentItem::List(l) => l.items.iter().collect(),
            _ => Vec::new(),
        };

        // Show children with combined parent+child icons, using the parent's connector
        for (i, child) in children.iter().enumerate() {
            let child_is_last = i == children.len() - 1;
            let child_icon = get_icon(child.node_type());

            // For the first child, use the parent's connector; for subsequent children get indented
            if i == 0 {
                let linum_prefix = if show_linum {
                    format!("{:02} ", child.range().start.line + 1)
                } else {
                    String::new()
                };

                output.push_str(&format!(
                    "{}{}{} {} {} {}\n",
                    linum_prefix,
                    prefix,
                    connector,
                    parent_icon,
                    child_icon,
                    child.display_label()
                ));
            } else {
                // Subsequent children get indented with the parent's continuation
                let child_connector = if child_is_last { "└─" } else { "├─" };
                let linum_prefix = if show_linum {
                    format!("{:02} ", child.range().start.line + 1)
                } else {
                    String::new()
                };

                output.push_str(&format!(
                    "{}{}{} {} {} {}\n",
                    linum_prefix,
                    continuation_prefix,
                    child_connector,
                    parent_icon,
                    child_icon,
                    child.display_label()
                ));
            }

            append_descendants(
                child,
                &continuation_prefix,
                child_is_last,
                show_linum,
                &mut output,
            );
        }
    } else {
        // Normal node - show as usual
        let icon = get_icon(item.node_type());
        let linum_prefix = if show_linum {
            format!("{:02} ", item.range().start.line + 1)
        } else {
            String::new()
        };

        output.push_str(&format!(
            "{}{}{} {} {}\n",
            linum_prefix,
            prefix,
            connector,
            icon,
            item.display_label()
        ));

        // Process children
        let children = match item {
            ContentItem::Session(s) => s.children(),
            ContentItem::Definition(d) => d.children(),
            ContentItem::ListItem(li) => li.children(),
            ContentItem::Annotation(a) => a.children(),
            ContentItem::VerbatimBlock(v) => v.children(),
            _ => &[],
        };

        if !children.is_empty() {
            let child_prefix = format!("{}{}", prefix, if is_last { "  " } else { "│ " });
            render_children(
                item.range().start.line + 1,
                children,
                &child_prefix,
                show_linum,
                &mut output,
            );
        }
    }

    output
}

/// Convert a document to linetreeviz string
pub fn to_linetreeviz_str(doc: &Document) -> String {
    to_linetreeviz_str_with_params(doc, &HashMap::new())
}

/// Convert a document to linetreeviz string with optional parameters
///
/// # Parameters
///
/// - `"ast-full"`: When set to `"true"`, includes all AST node properties
///   Note: Currently this parameter is not fully implemented for linetreeviz
pub fn to_linetreeviz_str_with_params(doc: &Document, params: &HashMap<String, String>) -> String {
    let show_linum = params
        .get("show-linum")
        .map(|v| v != "false")
        .unwrap_or(false);

    let icon = get_icon("Document");
    let mut output = format!(
        "{} Document ({} annotations, {} items)\n",
        icon,
        doc.annotations.len(),
        doc.root.children.len()
    );

    let children = &doc.root.children;
    render_children(0, children, "", show_linum, &mut output);

    output
}

fn render_children(
    parent_line: usize,
    children: &[ContentItem],
    child_prefix: &str,
    show_linum: bool,
    output: &mut String,
) {
    if children.is_empty() {
        return;
    }

    let mut last_consumed_line = parent_line;
    for (i, child) in children.iter().enumerate() {
        let child_start_line = child.range().start.line + 1;
        if child_start_line > last_consumed_line + 1 {
            insert_blank_line_entries(
                last_consumed_line + 1,
                child_start_line,
                child_prefix,
                i,
                children.len(),
                show_linum,
                output,
            );
        }
        last_consumed_line = child.range().end.line + 1;

        output.push_str(&format_content_item(
            child,
            child_prefix,
            i,
            children.len(),
            show_linum,
        ));
    }
}

fn insert_blank_line_entries(
    start_line: usize,
    end_line: usize,
    child_prefix: &str,
    next_child_index: usize,
    total_children: usize,
    show_linum: bool,
    output: &mut String,
) {
    if start_line >= end_line {
        return;
    }

    for missing_line in start_line..end_line {
        let linum_prefix = if show_linum {
            format!("{:02} ", missing_line)
        } else {
            String::new()
        };
        let is_last_sibling = next_child_index == total_children && missing_line == end_line - 1;
        let connector = if is_last_sibling { "└─" } else { "├─" };

        output.push_str(&format!(
            "{}{}{} {} 1 blank line\n",
            linum_prefix,
            child_prefix,
            connector,
            get_icon("BlankLineGroup")
        ));
    }
}

fn append_descendants(
    child: &ContentItem,
    parent_prefix: &str,
    child_is_last: bool,
    show_linum: bool,
    output: &mut String,
) {
    let children = match child {
        ContentItem::Session(s) => s.children(),
        ContentItem::Definition(d) => d.children(),
        ContentItem::ListItem(li) => li.children(),
        ContentItem::Annotation(a) => a.children(),
        ContentItem::VerbatimBlock(v) => v.children(),
        _ => &[],
    };

    if children.is_empty() {
        return;
    }

    let child_prefix = format!(
        "{}{}",
        parent_prefix,
        if child_is_last { "  " } else { "│ " }
    );
    render_children(
        child.range().start.line + 1,
        children,
        &child_prefix,
        show_linum,
        output,
    );
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
    use lex_parser::lex::loader::DocumentLoader;

    #[test]
    fn test_icon_mapping() {
        assert_eq!(get_icon("Session"), "§");
        assert_eq!(get_icon("TextLine"), "↵");
        assert_eq!(get_icon("ListItem"), "•");
        assert_eq!(get_icon("Definition"), "≔");
        assert_eq!(get_icon("Paragraph"), "¶");
        assert_eq!(get_icon("List"), "☰");
    }

    #[test]
    fn linetreeviz_renders_nested_list_children_per_line() {
        let source = r#"Session Root {{session}}

    - Parent list item. {{list-item}}
        Nested paragraph in list. {{paragraph}}
    - Second list item. {{list-item}}
"#;

        let doc = DocumentLoader::from_string(source)
            .parse()
            .expect("document to parse");
        let mut params = HashMap::new();
        params.insert("show-linum".to_string(), "true".to_string());

        let output = to_linetreeviz_str_with_params(&doc, &params);
        let nested_line = output
            .lines()
            .find(|line| line.contains("Nested paragraph in list."))
            .unwrap_or_else(|| panic!("Missing nested paragraph line in output:\n{output}"));

        assert!(
            nested_line.starts_with("04 "),
            "Expected nested paragraph to retain source line number, got:\n{}\nFull output:\n{}",
            nested_line,
            output
        );
    }
}
