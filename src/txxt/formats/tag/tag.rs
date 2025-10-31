//! XML-like AST tag serialization
//!
//! Serializes AST snapshots to an XML-like format.
//! Consumes the normalized AST snapshot representation and applies
//! XML/tag-specific formatting.
//!
//! ## Format
//!
//! - Node type → tag name (snake-case)
//! - Label → text content
//! - Children → nested tags (no wrapper)
//!
//! ## Example
//!
//! ```text
//! <document>
//!   <session>Introduction
//!     <paragraph>
//!       <text-line>Welcome to the guide</text-line>
//!     </paragraph>
//!   </session>
//! </document>
//! ```

use crate::txxt::ast::{AstSnapshot, Document};

/// Tag serializer that converts AstSnapshot to XML-like format
struct TagSerializer {
    output: String,
    indent_level: usize,
}

impl TagSerializer {
    fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
        }
    }

    fn indent(&self) -> String {
        "  ".repeat(self.indent_level)
    }

    fn push_indent(&mut self, s: &str) {
        self.output.push_str(&self.indent());
        self.output.push_str(s);
    }

    fn serialize_snapshot(&mut self, snapshot: &AstSnapshot) {
        let tag = to_tag_name(&snapshot.node_type);

        self.push_indent(&format!("<{}>", tag));
        self.output.push_str(&escape_xml(&snapshot.label));

        if snapshot.children.is_empty() {
            self.output.push_str(&format!("</{}>", tag));
            self.output.push('\n');
        } else {
            self.output.push('\n');
            self.indent_level += 1;
            for child in &snapshot.children {
                self.serialize_snapshot(child);
            }
            self.indent_level -= 1;
            self.push_indent(&format!("</{}>", tag));
            self.output.push('\n');
        }
    }
}

/// Convert a node type name to a tag name (e.g., "TextLine" → "text-line")
fn to_tag_name(node_type: &str) -> String {
    let mut tag = String::new();
    for (i, c) in node_type.chars().enumerate() {
        if i > 0 && c.is_uppercase() {
            tag.push('-');
        }
        tag.push(c.to_lowercase().next().unwrap());
    }
    tag
}

/// Serialize a document to AST tag format
pub fn serialize_document(doc: &Document) -> String {
    let mut result = String::new();
    result.push_str("<document>\n");

    let mut serializer = TagSerializer::new();
    serializer.indent_level = 1;

    for item in &doc.content {
        let snapshot = crate::txxt::ast::snapshot_visitor::snapshot_from_content(item);
        eprintln!("SNAPSHOT: {:?}", snapshot);
        serializer.serialize_snapshot(&snapshot);
    }

    result.push_str(&serializer.output);
    result.push_str("</document>");
    result
}

/// Escape XML special characters
fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::txxt::ast::{ContentItem, Paragraph, Session, TextContent};

    #[test]
    fn test_serialize_simple_paragraph() {
        let doc = Document::with_content(vec![ContentItem::Paragraph(Paragraph::from_line(
            "Hello world".to_string(),
        ))]);

        let result = serialize_document(&doc);
        assert!(result.contains("<document>"));
        assert!(result.contains("<paragraph>"));
        assert!(result.contains("Hello world"));
        assert!(result.contains("</paragraph>"));
        assert!(result.contains("</document>"));
    }

    #[test]
    fn test_serialize_session_with_paragraph() {
        let doc = Document::with_content(vec![ContentItem::Session(Session::new(
            TextContent::from_string("Introduction".to_string(), None),
            vec![ContentItem::Paragraph(Paragraph::from_line(
                "Welcome".to_string(),
            ))],
        ))]);

        let result = serialize_document(&doc);
        println!("RESULT:\n{}", result);
        assert!(result.contains("<session>Introduction"));
        assert!(result.contains("<paragraph>"));
        assert!(result.contains("Welcome"));
        assert!(result.contains("</paragraph>"));
        assert!(result.contains("</session>"));
    }

    #[test]
    fn test_serialize_nested_sessions() {
        let doc = Document::with_content(vec![ContentItem::Session(Session::new(
            TextContent::from_string("Root".to_string(), None),
            vec![
                ContentItem::Paragraph(Paragraph::from_line("Para 1".to_string())),
                ContentItem::Session(Session::new(
                    TextContent::from_string("Nested".to_string(), None),
                    vec![ContentItem::Paragraph(Paragraph::from_line(
                        "Nested para".to_string(),
                    ))],
                )),
            ],
        ))]);

        let result = serialize_document(&doc);
        assert!(result.contains("<session>Root"));
        assert!(result.contains("<paragraph>"));
        assert!(result.contains("Para 1"));
        assert!(result.contains("<session>Nested"));
        assert!(result.contains("Nested para"));
    }

    #[test]
    fn test_xml_escaping() {
        let doc = Document::with_content(vec![ContentItem::Paragraph(Paragraph::from_line(
            "Text with <special> & \"chars\"".to_string(),
        ))]);

        let result = serialize_document(&doc);
        assert!(result.contains("&lt;special&gt;"));
        assert!(result.contains("&amp;"));
        assert!(result.contains("&quot;"));
    }

    #[test]
    fn test_empty_session() {
        let doc = Document::with_content(vec![ContentItem::Session(Session::with_title(
            "Empty".to_string(),
        ))]);

        let result = serialize_document(&doc);
        assert!(result.contains("<session>Empty</session>"));
    }

    #[test]
    fn test_serialize_simple_list() {
        use crate::txxt::ast::{List, ListItem};

        let doc = Document::with_content(vec![ContentItem::List(List::new(vec![
            ContentItem::ListItem(ListItem::new("- First item".to_string())),
            ContentItem::ListItem(ListItem::new("- Second item".to_string())),
        ]))]);

        let result = serialize_document(&doc);
        assert!(result.contains("<list>"));
        assert!(result.contains("<list-item>- First item</list-item>"));
        assert!(result.contains("<list-item>- Second item</list-item>"));
        assert!(result.contains("</list>"));
    }
}
