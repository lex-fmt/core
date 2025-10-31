//! XML-like AST tag serialization
//!
//! Serializes AST nodes to an XML-like format that directly reflects the AST structure.
//! Uses the Visitor pattern for uniform serialization across all node types.
//!
//! ## Format
//!
//! - Node type → tag name
//! - Label (title/subject/item_line) → text content
//! - Children → nested tags (no wrapper)
//! - Leaf text → text content
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

use crate::txxt::ast::elements::paragraph::TextLine;
use crate::txxt::ast::{
    traits::visit_children, Annotation, AstNode, Container, Definition, Document, ForeignBlock,
    List, ListItem, Paragraph, Session, Visitor,
};

/// Tag serializer using the Visitor pattern
struct TagSerializer {
    output: String,
    indent_level: usize,
}

impl TagSerializer {
    fn indent(&self) -> String {
        "  ".repeat(self.indent_level)
    }

    fn push_indent(&mut self, s: &str) {
        self.output.push_str(&self.indent());
        self.output.push_str(s);
    }

    fn open_tag(&mut self, tag: &str) {
        self.push_indent(&format!("<{tag}>"));
    }

    fn tag_with_text(&mut self, tag: &str, text: &str) {
        self.push_indent(&format!("<{tag}>{}</{}>\n", escape_xml(text), tag));
    }

    fn close_tag_inline(&mut self, tag: &str) {
        self.output.push_str(&format!("</{tag}>"));
    }
}

impl Visitor for TagSerializer {
    fn visit_paragraph(&mut self, para: &Paragraph) {
        self.open_tag("paragraph");

        if para.lines.is_empty() {
            self.close_tag_inline("paragraph");
        } else {
            self.output.push('\n');
            self.indent_level += 1;
            visit_children(self, &para.lines);
            self.indent_level -= 1;
            self.push_indent("");
            self.close_tag_inline("paragraph");
        }
        self.output.push('\n');
    }

    fn visit_text_line(&mut self, tl: &TextLine) {
        let text = tl.text();
        self.tag_with_text("text-line", text);
    }

    fn visit_session(&mut self, session: &Session) {
        self.open_tag("session");
        self.output.push_str(&escape_xml(session.label()));

        if !session.children().is_empty() {
            self.output.push('\n');
            self.indent_level += 1;
            visit_children(self, session.children());
            self.indent_level -= 1;
            self.push_indent("");
        }

        self.close_tag_inline("session");
        self.output.push('\n');
    }

    fn visit_list(&mut self, list: &List) {
        self.open_tag("list");
        self.output.push('\n');
        self.indent_level += 1;
        visit_children(self, &list.content);
        self.indent_level -= 1;
        self.push_indent("");
        self.close_tag_inline("list");
        self.output.push('\n');
    }

    fn visit_list_item(&mut self, item: &ListItem) {
        self.open_tag("list-item");
        self.output.push_str(&escape_xml(item.label()));

        if !item.children().is_empty() {
            self.output.push('\n');
            self.indent_level += 1;
            visit_children(self, item.children());
            self.indent_level -= 1;
            self.push_indent("");
        }

        self.close_tag_inline("list-item");
        self.output.push('\n');
    }

    fn visit_definition(&mut self, def: &Definition) {
        self.open_tag("definition");
        self.output.push_str(&escape_xml(def.label()));

        if !def.children().is_empty() {
            self.output.push('\n');
            self.indent_level += 1;
            visit_children(self, def.children());
            self.indent_level -= 1;
            self.push_indent("");
        }

        self.close_tag_inline("definition");
        self.output.push('\n');
    }

    fn visit_foreign_block(&mut self, fb: &ForeignBlock) {
        self.open_tag("foreign-block");
        self.output.push_str(&escape_xml(fb.subject.as_string()));

        if !fb.content.as_string().is_empty() {
            self.output.push('\n');
            self.indent_level += 1;
            self.push_indent(&format!(
                "<content>{}</content>\n",
                escape_xml(fb.content.as_string())
            ));
            self.indent_level -= 1;
        }

        self.close_tag_inline("foreign-block");
        self.output.push('\n');
    }

    fn visit_annotation(&mut self, ann: &Annotation) {
        self.open_tag("annotation");
        self.output.push_str(&escape_xml(&ann.label.value));

        // Add parameters if present
        if !ann.parameters.is_empty() {
            self.output.push('[');
            for (i, param) in ann.parameters.iter().enumerate() {
                if i > 0 {
                    self.output.push(',');
                }
                self.output.push_str(&escape_xml(&param.key));
                if let Some(value) = &param.value {
                    self.output.push('=');
                    self.output.push_str(&escape_xml(value));
                }
            }
            self.output.push(']');
        }

        if !ann.children().is_empty() {
            self.output.push('\n');
            self.indent_level += 1;
            visit_children(self, ann.children());
            self.indent_level -= 1;
            self.push_indent("");
        }

        self.close_tag_inline("annotation");
        self.output.push('\n');
    }
}

/// Serialize a document to AST tag format
pub fn serialize_document(doc: &Document) -> String {
    let mut result = String::new();
    result.push_str("<document>\n");

    let mut serializer = TagSerializer {
        output: String::new(),
        indent_level: 1,
    };

    for item in &doc.content {
        item.accept(&mut serializer);
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
        .replace('"', "&quot;")
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
        assert!(result.contains("<text-line>Hello world</text-line>"));
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
        assert!(result.contains("<session>Introduction"));
        assert!(result.contains("<paragraph>"));
        assert!(result.contains("<text-line>Welcome</text-line>"));
        assert!(result.contains("</paragraph>"));
        assert!(result.contains("</session>"));
        assert!(!result.contains("<children>"));
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
        assert!(result.contains("<text-line>Para 1</text-line>"));
        assert!(result.contains("<session>Nested"));
        assert!(result.contains("<text-line>Nested para</text-line>"));
        assert!(!result.contains("<children>"));
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
        assert!(!result.contains("<children>"));
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
        // No wrapper tags with new visitor format
        assert!(!result.contains("<children>"));
    }
}
