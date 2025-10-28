//! Treeviz formatter for AST nodes

use crate::txxt_nano::parser::ast::{AstNode, Container, ContentItem, Document, ListItem};

fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() > max_chars {
        let mut truncated = s.chars().take(max_chars).collect::<String>();
        truncated.push_str("...");
        truncated
    } else {
        s.to_string()
    }
}

pub fn to_treeviz_str(doc: &Document) -> String {
    let mut result = String::new();
    for (i, item) in doc.content.iter().enumerate() {
        let is_last = i == doc.content.len() - 1;
        append_content_item(&mut result, item, "", is_last);
    }
    result
}

fn append_content_item(result: &mut String, item: &ContentItem, prefix: &str, is_last: bool) {
    let connector = if is_last { "└─" } else { "├─" };
    let node_type = item.node_type();

    let display_label = truncate(&item.display_label(), 30);

    result.push_str(&format!(
        "{}{} {}: {}\n",
        prefix, connector, node_type, display_label
    ));

    let new_prefix = format!("{}{}", prefix, if is_last { "  " } else { "│ " });

    match item {
        ContentItem::Session(session) => {
            append_children(result, session.children(), &new_prefix);
        }
        ContentItem::Definition(definition) => {
            append_children(result, definition.children(), &new_prefix);
        }
        ContentItem::Annotation(annotation) => {
            append_children(result, annotation.children(), &new_prefix);
        }
        ContentItem::List(list) => {
            append_list_items(result, &list.items, &new_prefix);
        }
        ContentItem::Paragraph(_) => {}
        ContentItem::ForeignBlock(_) => {} // Foreign blocks don't have children
    }
}

fn append_children(result: &mut String, children: &[ContentItem], prefix: &str) {
    for (i, child) in children.iter().enumerate() {
        let is_last = i == children.len() - 1;
        append_content_item(result, child, prefix, is_last);
    }
}

fn append_list_items(result: &mut String, items: &[ListItem], prefix: &str) {
    for (i, item) in items.iter().enumerate() {
        let is_last = i == items.len() - 1;
        let connector = if is_last { "└─" } else { "├─" };
        let node_type = item.node_type();
        let display_label = truncate(item.label(), 30);
        result.push_str(&format!(
            "{}{} {}: {}\n",
            prefix, connector, node_type, display_label
        ));

        let new_prefix = format!("{}{}", prefix, if is_last { "  " } else { "│ " });
        append_children(result, item.children(), &new_prefix);
    }
}
