//! XML-like AST tag serialization
//!
//! Serializes AST nodes to an XML-like format that directly reflects the AST structure.
//! Uses the AST traits (Container, TextNode) to provide uniform serialization across node types.
//!
//! ## Format
//!
//! - Node type → tag name
//! - Label (title/subject/item_line) → text content
//! - Children → nested in `<children>` tag
//! - Other fields → attributes (future)
//!
//! ## Example
//!
//! ```text
//! <session>Introduction<children>
//!   <paragraph>Welcome to the guide</paragraph>
//!   <session>Getting Started<children>
//!     <paragraph>First, install the software</paragraph>
//!   </children></session>
//! </children></session>
//! ```

use crate::txxt_nano::parser::ast::{Container, ContentItem, Document};

/// Serialize a document to AST tag format
pub fn serialize_document(doc: &Document) -> String {
    let mut result = String::new();
    result.push_str("<document>\n");
    for item in &doc.content {
        serialize_content_item(item, 1, &mut result);
    }
    result.push_str("</document>");
    result
}

/// Serialize a content item (recursive)
fn serialize_content_item(item: &ContentItem, indent_level: usize, output: &mut String) {
    let indent = "  ".repeat(indent_level);

    match item {
        ContentItem::Paragraph(p) => {
            // <paragraph>text content</paragraph>
            let text = p.text();
            output.push_str(&format!(
                "{}<paragraph>{}</paragraph>\n",
                indent,
                escape_xml(&text)
            ));
        }
        ContentItem::Session(s) => {
            // <session>label<children>...</children></session>
            output.push_str(&format!("{}<session>", indent));
            output.push_str(&escape_xml(s.label()));

            if s.children().is_empty() {
                // Empty session
                output.push_str("</session>\n");
            } else {
                // Session with children
                output.push_str("<children>\n");
                for child in s.children() {
                    serialize_content_item(child, indent_level + 1, output);
                }
                output.push_str(&format!("{}</children></session>\n", indent));
            }
        }
        ContentItem::List(l) => {
            // <list><item>text</item><item>text</item>...</list>
            output.push_str(&format!("{}<list>\n", indent));
            for item in &l.items {
                output.push_str(&format!(
                    "{}<item>{}</item>\n",
                    "  ".repeat(indent_level + 1),
                    escape_xml(item.text())
                ));
            }
            output.push_str(&format!("{}</list>\n", indent));
        }
        ContentItem::Definition(d) => {
            // <definition>subject<content>...</content></definition>
            output.push_str(&format!("{}<definition>", indent));
            output.push_str(&escape_xml(&d.subject));

            if d.children().is_empty() {
                // Empty definition
                output.push_str("</definition>\n");
            } else {
                // Definition with children
                output.push_str("<content>\n");
                for child in d.children() {
                    serialize_content_item(child, indent_level + 1, output);
                }
                output.push_str(&format!("{}</content></definition>\n", indent));
            }
        }
        ContentItem::Annotation(a) => {
            // <annotation>label<parameters>...</parameters><content>...</content></annotation>
            output.push_str(&format!("{}<annotation>", indent));
            output.push_str(&escape_xml(&a.label.value));

            // Add parameters if present
            if !a.parameters.is_empty() {
                output.push_str("<parameters>");
                for (i, param) in a.parameters.iter().enumerate() {
                    if i > 0 {
                        output.push(',');
                    }
                    output.push_str(&escape_xml(&param.key));
                    if let Some(value) = &param.value {
                        output.push('=');
                        output.push_str(&escape_xml(value));
                    }
                }
                output.push_str("</parameters>");
            }

            if a.children().is_empty() {
                // Empty annotation (marker or single-line form)
                output.push_str("</annotation>\n");
            } else {
                // Annotation with content (block form)
                output.push_str("<content>\n");
                for child in a.children() {
                    serialize_content_item(child, indent_level + 1, output);
                }
                output.push_str(&format!("{}</content></annotation>\n", indent));
            }
        }
        ContentItem::ForeignBlock(fb) => {
            // <foreign-block>subject<content>raw content</content><closing-annotation>...</closing-annotation></foreign-block>
            output.push_str(&format!("{}<foreign-block>", indent));
            output.push_str(&escape_xml(&fb.subject));

            if fb.content.is_empty() {
                // Marker form - no content
                output.push_str("<content></content>");
            } else {
                // Block form - raw content
                output.push_str("<content>");
                output.push_str(&escape_xml(&fb.content));
                output.push_str("</content>");
            }

            // Add closing annotation
            output.push_str("<closing-annotation>");
            output.push_str(&escape_xml(&fb.closing_annotation.label.value));

            // Add parameters if present
            if !fb.closing_annotation.parameters.is_empty() {
                output.push_str("<parameters>");
                for (i, param) in fb.closing_annotation.parameters.iter().enumerate() {
                    if i > 0 {
                        output.push(',');
                    }
                    output.push_str(&escape_xml(&param.key));
                    if let Some(value) = &param.value {
                        output.push('=');
                        output.push_str(&escape_xml(value));
                    }
                }
                output.push_str("</parameters>");
            }

            output.push_str("</closing-annotation></foreign-block>\n");
        }
    }
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
    use crate::txxt_nano::parser::ast::{Paragraph, Session};

    #[test]
    fn test_serialize_simple_paragraph() {
        let doc = Document::with_content(vec![ContentItem::Paragraph(Paragraph::from_line(
            "Hello world".to_string(),
        ))]);

        let result = serialize_document(&doc);
        assert!(result.contains("<document>"));
        assert!(result.contains("<paragraph>Hello world</paragraph>"));
        assert!(result.contains("</document>"));
    }

    #[test]
    fn test_serialize_session_with_paragraph() {
        let doc = Document::with_content(vec![ContentItem::Session(Session::new(
            "Introduction".to_string(),
            vec![ContentItem::Paragraph(Paragraph::from_line(
                "Welcome".to_string(),
            ))],
        ))]);

        let result = serialize_document(&doc);
        assert!(result.contains("<session>Introduction<children>"));
        assert!(result.contains("<paragraph>Welcome</paragraph>"));
        assert!(result.contains("</children></session>"));
    }

    #[test]
    fn test_serialize_nested_sessions() {
        let doc = Document::with_content(vec![ContentItem::Session(Session::new(
            "Root".to_string(),
            vec![
                ContentItem::Paragraph(Paragraph::from_line("Para 1".to_string())),
                ContentItem::Session(Session::new(
                    "Nested".to_string(),
                    vec![ContentItem::Paragraph(Paragraph::from_line(
                        "Nested para".to_string(),
                    ))],
                )),
            ],
        ))]);

        let result = serialize_document(&doc);
        assert!(result.contains("<session>Root<children>"));
        assert!(result.contains("<paragraph>Para 1</paragraph>"));
        assert!(result.contains("<session>Nested<children>"));
        assert!(result.contains("<paragraph>Nested para</paragraph>"));
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
}
