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

use crate::txxt_nano::ast::{Container, ContentItem, Document};

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
            // <list><item>text<children>...</children></item>...</list>
            output.push_str(&format!("{}<list>\n", indent));
            for item in &l.items {
                let item_indent = "  ".repeat(indent_level + 1);
                output.push_str(&format!("{}<item>", item_indent));
                output.push_str(&escape_xml(item.text()));
                
                if item.content.is_empty() {
                    // No nested content
                    output.push_str("</item>\n");
                } else {
                    // Has nested content (paragraphs, nested lists, etc.)
                    output.push_str("<children>\n");
                    for child in &item.content {
                        serialize_content_item(child, indent_level + 2, output);
                    }
                    output.push_str(&format!("{}</children></item>\n", item_indent));
                }
            }
            output.push_str(&format!("{}</list>\n", indent));
        }
        ContentItem::Definition(d) => {
            // <definition>subject<content>...</content></definition>
            output.push_str(&format!("{}<definition>", indent));
            output.push_str(&escape_xml(d.subject.as_string()));

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
            output.push_str(&escape_xml(fb.subject.as_string()));

            if fb.content.is_empty() {
                // Marker form - no content
                output.push_str("<content></content>");
            } else {
                // Block form - raw content
                output.push_str("<content>");
                output.push_str(&escape_xml(fb.content.as_string()));
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
    use crate::txxt_nano::ast::{Paragraph, Session, TextContent};

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
            TextContent::from_string("Introduction".to_string(), None),
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

    #[test]
    fn test_serialize_simple_list() {
        use crate::txxt_nano::ast::{List, ListItem};

        let doc = Document::with_content(vec![ContentItem::List(List::new(vec![
            ListItem::new("- First item".to_string()),
            ListItem::new("- Second item".to_string()),
        ]))]);

        let result = serialize_document(&doc);
        assert!(result.contains("<list>"));
        assert!(result.contains("<item>- First item</item>"));
        assert!(result.contains("<item>- Second item</item>"));
        assert!(result.contains("</list>"));
        // Simple items should not have <children>
        assert!(!result.contains("<children>"));
    }

    #[test]
    fn test_serialize_nested_list() {
        use crate::txxt_nano::ast::{List, ListItem};

        // Create a nested list structure:
        // - Outer item one
        //   - Inner item one
        //   - Inner item two
        // - Outer item two
        let inner_list = List::new(vec![
            ListItem::new("- Inner item one".to_string()),
            ListItem::new("- Inner item two".to_string()),
        ]);

        let outer_list = List::new(vec![
            ListItem::with_content(
                "- Outer item one".to_string(),
                vec![ContentItem::List(inner_list)],
            ),
            ListItem::new("- Outer item two".to_string()),
        ]);

        let doc = Document::with_content(vec![ContentItem::List(outer_list)]);

        let result = serialize_document(&doc);
        
        // Verify outer list structure
        assert!(result.contains("<list>"));
        assert!(result.contains("- Outer item one<children>"));
        assert!(result.contains("</children></item>"));
        
        // Verify nested list structure
        assert!(result.contains("- Inner item one</item>"));
        assert!(result.contains("- Inner item two</item>"));
        
        // Verify second outer item (no children)
        assert!(result.contains("<item>- Outer item two</item>"));
    }

    #[test]
    fn test_serialize_list_with_paragraph_content() {
        use crate::txxt_nano::ast::{List, ListItem, Paragraph};

        // Create a list with paragraph content:
        // - First item
        //   This is a nested paragraph.
        let list = List::new(vec![ListItem::with_content(
            "- First item".to_string(),
            vec![ContentItem::Paragraph(Paragraph::from_line(
                "This is a nested paragraph.".to_string(),
            ))],
        )]);

        let doc = Document::with_content(vec![ContentItem::List(list)]);

        let result = serialize_document(&doc);
        
        assert!(result.contains("- First item<children>"));
        assert!(result.contains("<paragraph>This is a nested paragraph.</paragraph>"));
        assert!(result.contains("</children></item>"));
    }

    #[test]
    fn test_serialize_list_with_mixed_content() {
        use crate::txxt_nano::ast::{List, ListItem, Paragraph};

        // Create a list with mixed content:
        // 1. First item
        //    Paragraph explaining the item.
        //    - Nested list item one
        //    - Nested list item two
        //    Another paragraph.
        let nested_list = List::new(vec![
            ListItem::new("- Nested list item one".to_string()),
            ListItem::new("- Nested list item two".to_string()),
        ]);

        let list = List::new(vec![ListItem::with_content(
            "1. First item".to_string(),
            vec![
                ContentItem::Paragraph(Paragraph::from_line("Paragraph explaining the item.".to_string())),
                ContentItem::List(nested_list),
                ContentItem::Paragraph(Paragraph::from_line("Another paragraph.".to_string())),
            ],
        )]);

        let doc = Document::with_content(vec![ContentItem::List(list)]);

        let result = serialize_document(&doc);
        
        // Verify the structure
        assert!(result.contains("1. First item<children>"));
        assert!(result.contains("<paragraph>Paragraph explaining the item.</paragraph>"));
        assert!(result.contains("- Nested list item one</item>"));
        assert!(result.contains("- Nested list item two</item>"));
        assert!(result.contains("<paragraph>Another paragraph.</paragraph>"));
        assert!(result.contains("</children></item>"));
    }

    #[test]
    fn test_serialize_deeply_nested_lists() {
        use crate::txxt_nano::ast::{List, ListItem};

        // Create deeply nested structure:
        // - Outer item
        //   - Middle item
        //     - Inner item one
        //     - Inner item two
        let inner_list = List::new(vec![
            ListItem::new("- Inner item one".to_string()),
            ListItem::new("- Inner item two".to_string()),
        ]);

        let middle_list = List::new(vec![ListItem::with_content(
            "- Middle item".to_string(),
            vec![ContentItem::List(inner_list)],
        )]);

        let outer_list = List::new(vec![ListItem::with_content(
            "- Outer item".to_string(),
            vec![ContentItem::List(middle_list)],
        )]);

        let doc = Document::with_content(vec![ContentItem::List(outer_list)]);

        let result = serialize_document(&doc);
        
        // Verify three levels of nesting
        assert!(result.contains("- Outer item<children>"));
        assert!(result.contains("- Middle item<children>"));
        assert!(result.contains("- Inner item one</item>"));
        assert!(result.contains("- Inner item two</item>"));
        
        // Count closing tags to verify proper nesting
        let children_open = result.matches("<children>").count();
        let children_close = result.matches("</children>").count();
        assert_eq!(children_open, children_close, "Mismatched <children> tags");
    }

    #[test]
    fn test_serialize_nested_lists_from_sample_file() {
        // Integration test: Parse a sample file and verify nested lists are serialized correctly
        use crate::txxt_nano::lexer::lex_with_spans;
        use crate::txxt_nano::parser::api::parse_with_source;
        use crate::txxt_nano::processor::txxt_sources::TxxtSources;

        let source = TxxtSources::get_string("070-nested-lists-simple.txxt")
            .expect("Failed to load sample file");
        let tokens = lex_with_spans(&source);
        let doc = parse_with_source(tokens, &source).expect("Failed to parse");

        let result = serialize_document(&doc);

        // Verify the structure has nested content
        assert!(result.contains("<item>- First outer item {{list-item}}<children>"));
        assert!(result.contains("<item>- First nested item {{list-item}}</item>"));
        assert!(result.contains("<item>- Second nested item {{list-item}}</item>"));
        assert!(result.contains("</children></item>"));
        
        // Verify second outer item also has nested content
        assert!(result.contains("<item>- Second outer item {{list-item}}<children>"));
        assert!(result.contains("<item>- Another nested item {{list-item}}</item>"));
        
        // Verify numbered list with nested dashed list
        assert!(result.contains("<item>1. First numbered item {{list-item}}<children>"));
        assert!(result.contains("<item>- Nested dash item one {{list-item}}</item>"));
    }

    #[test]
    fn test_serialize_mixed_content_lists_from_sample_file() {
        // Integration test: Verify lists with paragraphs and nested lists are serialized correctly
        use crate::txxt_nano::lexer::lex_with_spans;
        use crate::txxt_nano::parser::api::parse_with_source;
        use crate::txxt_nano::processor::txxt_sources::TxxtSources;

        let source = TxxtSources::get_string("080-nested-lists-mixed-content.txxt")
            .expect("Failed to load sample file");
        let tokens = lex_with_spans(&source);
        let doc = parse_with_source(tokens, &source).expect("Failed to parse");

        let result = serialize_document(&doc);

        // Verify list items with paragraph content
        assert!(result.contains("<item>- First item with nested paragraph {{list-item}}<children>"));
        assert!(result.contains("<paragraph>This is a paragraph nested inside the first list item"));
        
        // Verify list items with multiple paragraphs
        assert!(result.contains("<item>- Second item with multiple paragraphs {{list-item}}<children>"));
        assert!(result.contains("<paragraph>This is the first paragraph in the second item"));
        assert!(result.contains("<paragraph>This is a second paragraph"));
        
        // Verify mixed content (para + list + para)
        assert!(result.contains("<item>1. First complex item {{list-item}}<children>"));
        assert!(result.contains("<paragraph>This is a paragraph explaining the first item"));
        assert!(result.contains("<item>- Nested list item one {{list-item}}</item>"));
        assert!(result.contains("<paragraph>Another paragraph after the nested list"));
        
        // Verify deeply nested structure (list > list > list)
        assert!(result.contains("<item>- Outer item one {{list-item}}<children>"));
        assert!(result.contains("<item>- Middle item one {{list-item}}<children>"));
        assert!(result.contains("<item>- Inner item one {{list-item}}</item>"));
    }
}
