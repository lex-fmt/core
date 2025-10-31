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

use crate::txxt::ast::{Container, ContentItem, Document, ListItem};

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

/// Generic helper to serialize children of a container node
///
/// This function provides a uniform way to serialize nested content for all
/// container types (Session, Definition, Annotation, ListItem) using the Container trait.
fn serialize_children(
    children: &[ContentItem],
    indent_level: usize,
    output: &mut String,
    wrapper_tag: &str,
    parent_indent: &str,
) {
    if children.is_empty() {
        return;
    }

    output.push_str(&format!("<{wrapper_tag}>\n"));
    for child in children {
        serialize_content_item(child, indent_level, output);
    }
    output.push_str(&format!("{}</{wrapper_tag}>", parent_indent));
}

/// Serialize a list item with its nested content
fn serialize_list_item(item: &ListItem, indent_level: usize, output: &mut String) {
    let indent = "  ".repeat(indent_level);
    output.push_str(&format!("{}<item>", indent));
    output.push_str(&escape_xml(item.label())); // Uses Container trait

    if item.children().is_empty() {
        // No nested content
        output.push_str("</item>\n");
    } else {
        // Has nested content - use generic serialization
        serialize_children(
            item.children(),
            indent_level + 1,
            output,
            "children",
            &indent,
        );
        output.push_str("</item>\n");
    }
}

/// Serialize the first child of a definition, removing {{definition}} marker if present
fn serialize_definition_first_child(item: &ContentItem, indent_level: usize, output: &mut String) {
    let indent = "  ".repeat(indent_level);

    match item {
        ContentItem::Paragraph(p) => {
            // Extract and remove {{definition}} marker from text
            let text = p.text();
            let (cleaned_text, _) = extract_definition_marker(&text);
            output.push_str(&format!(
                "{}<paragraph>{}</paragraph>\n",
                indent,
                escape_xml(&cleaned_text)
            ));
        }
        ContentItem::List(l) => {
            // For lists as first child, serialize normally but clean first item
            output.push_str(&format!("{}<list>\n", indent));
            for (i, item) in l.content.iter().enumerate() {
                if let ContentItem::ListItem(list_item) = item {
                    if i == 0 {
                        // First item - remove {{definition}} marker if present
                        let item_text = list_item.label();
                        let (cleaned_text, _) = extract_list_item_definition_marker(item_text);
                        let temp_indent = "  ".repeat(indent_level + 1);
                        output.push_str(&format!(
                            "{}<item>{}",
                            temp_indent,
                            escape_xml(&cleaned_text)
                        ));

                        if list_item.children().is_empty() {
                            output.push_str("</item>\n");
                        } else {
                            serialize_children(
                                list_item.children(),
                                indent_level + 2,
                                output,
                                "children",
                                &temp_indent,
                            );
                            output.push_str("</item>\n");
                        }
                    } else {
                        serialize_list_item(list_item, indent_level + 1, output);
                    }
                }
            }
            output.push_str(&format!("{}</list>\n", indent));
        }
        _ => {
            // For other types, serialize normally
            serialize_content_item(item, indent_level, output);
        }
    }
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
            // Uses Container trait for generic child serialization
            output.push_str(&format!("{}<session>", indent));
            output.push_str(&escape_xml(s.label())); // Uses Container trait

            if s.children().is_empty() {
                output.push_str("</session>\n");
            } else {
                serialize_children(s.children(), indent_level + 1, output, "children", &indent);
                output.push_str("</session>\n");
            }
        }
        ContentItem::List(l) => {
            // <list><item>...</item>...</list>
            // Each item uses Container trait for nested content
            output.push_str(&format!("{}<list>\n", indent));
            for item in &l.content {
                if let ContentItem::ListItem(li) = item {
                    serialize_list_item(li, indent_level + 1, output);
                }
            }
            output.push_str(&format!("{}</list>\n", indent));
        }
        ContentItem::Definition(d) => {
            // <definition>subject {{definition}}<content>...</content></definition>
            // Extract {{definition}} marker from first child if present
            output.push_str(&format!("{}<definition>", indent));
            output.push_str(&escape_xml(d.subject.as_string()));

            // Check if first child has {{definition}} marker
            let mut definition_marker = None;
            if !d.children().is_empty() {
                // Look for {{definition}} in first paragraph child
                if let ContentItem::Paragraph(p) = &d.children()[0] {
                    let text = p.text();
                    let (_, marker) = extract_definition_marker(&text);
                    definition_marker = marker;
                } else if let ContentItem::List(l) = &d.children()[0] {
                    // Check if first list item has {{definition}} marker
                    if !l.content.is_empty() {
                        if let ContentItem::ListItem(li) = &l.content[0] {
                            let item_text = li.label();
                            let (_, marker) = extract_list_item_definition_marker(item_text);
                            definition_marker = marker;
                        }
                    }
                }
            }

            // Add marker to subject line if found
            if let Some(marker) = definition_marker {
                output.push(' ');
                output.push_str(&escape_xml(&marker));
            }

            if d.children().is_empty() {
                output.push_str("</definition>\n");
            } else {
                // Serialize children with modified first child if marker was extracted
                output.push_str("<content>\n");
                for (i, child) in d.children().iter().enumerate() {
                    if i == 0 {
                        // First child - may need to remove {{definition}} marker
                        serialize_definition_first_child(child, indent_level + 1, output);
                    } else {
                        serialize_content_item(child, indent_level + 1, output);
                    }
                }
                output.push_str(&format!("{}</content>", indent));
                output.push_str("</definition>\n");
            }
        }
        ContentItem::Annotation(a) => {
            // <annotation>label<parameters>...</parameters><content>...</content></annotation>
            // Uses Container trait for generic child serialization
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
                output.push_str("</annotation>\n");
            } else {
                serialize_children(a.children(), indent_level + 1, output, "content", &indent);
                output.push_str("</annotation>\n");
            }
        }
        ContentItem::ListItem(li) => {
            // ListItems should be serialized within List context using serialize_list_item
            // But handle it here for completeness
            serialize_list_item(li, indent_level, output);
        }
        ContentItem::ForeignBlock(fb) => {
            // <foreign-block>subject<content>raw content</content><closing-annotation>...</closing-annotation></foreign-block>
            // ForeignBlock has raw content, not structured children, so no Container trait usage
            output.push_str(&format!("{}<foreign-block>", indent));
            output.push_str(&escape_xml(fb.subject.as_string()));

            if fb.content.is_empty() {
                output.push_str("<content></content>");
            } else {
                output.push_str("<content>");
                output.push_str(&escape_xml(fb.content.as_string()));
                output.push_str("</content>");
            }

            output.push_str("<closing-annotation>");
            output.push_str(&escape_xml(&fb.closing_annotation.label.value));

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

/// Extract {{definition}} marker from text if present at the end
/// Returns (text_without_marker, marker_if_found)
fn extract_definition_marker(text: &str) -> (String, Option<String>) {
    // Look for {{definition}} at the end of the text (possibly with other markers like {{paragraph}})
    if let Some(pos) = text.rfind("{{definition}}") {
        // Found {{definition}} marker
        let mut cleaned_text = text[..pos].to_string();
        let mut marker = "{{definition}}".to_string();

        // Remove trailing whitespace after removing the marker
        cleaned_text = cleaned_text.trim_end().to_string();

        // Check if there's also a {{paragraph}} marker right before {{definition}}
        // and include it in the marker if found
        if cleaned_text.ends_with("{{paragraph}}") {
            let para_pos = cleaned_text.rfind("{{paragraph}}").unwrap();
            marker = format!("{{{{paragraph}}}} {}", marker);
            cleaned_text = cleaned_text[..para_pos].trim_end().to_string();
        }

        return (cleaned_text, Some(marker));
    }
    (text.to_string(), None)
}

/// Extract {{list-item}} and {{definition}} markers from text if present at the end
/// Returns (text_without_markers, markers_if_found)
fn extract_list_item_definition_marker(text: &str) -> (String, Option<String>) {
    // Look for {{list-item}} {{definition}} at the end
    if text.ends_with("{{list-item}} {{definition}}") {
        let pos = text.rfind("{{list-item}} {{definition}}").unwrap();
        let cleaned_text = text[..pos].trim_end().to_string();
        return (cleaned_text, Some("{{definition}}".to_string()));
    }
    (text.to_string(), None)
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
    use crate::txxt::ast::{Paragraph, Session, TextContent};

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
        use crate::txxt::ast::{List, ListItem};

        let doc = Document::with_content(vec![ContentItem::List(List::new(vec![
            ContentItem::ListItem(ListItem::new("- First item".to_string())),
            ContentItem::ListItem(ListItem::new("- Second item".to_string())),
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
        use crate::txxt::ast::{List, ListItem};

        // Create a nested list structure:
        // - Outer item one
        //   - Inner item one
        //   - Inner item two
        // - Outer item two
        let inner_list = List::new(vec![
            ContentItem::ListItem(ListItem::new("- Inner item one".to_string())),
            ContentItem::ListItem(ListItem::new("- Inner item two".to_string())),
        ]);

        let outer_list = List::new(vec![
            ContentItem::ListItem(ListItem::with_content(
                "- Outer item one".to_string(),
                vec![ContentItem::List(inner_list)],
            )),
            ContentItem::ListItem(ListItem::new("- Outer item two".to_string())),
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
        use crate::txxt::ast::{List, ListItem, Paragraph};

        // Create a list with paragraph content:
        // - First item
        //   This is a nested paragraph.
        let list = List::new(vec![ContentItem::ListItem(ListItem::with_content(
            "- First item".to_string(),
            vec![ContentItem::Paragraph(Paragraph::from_line(
                "This is a nested paragraph.".to_string(),
            ))],
        ))]);

        let doc = Document::with_content(vec![ContentItem::List(list)]);

        let result = serialize_document(&doc);

        assert!(result.contains("- First item<children>"));
        assert!(result.contains("<paragraph>This is a nested paragraph.</paragraph>"));
        assert!(result.contains("</children></item>"));
    }

    #[test]
    fn test_serialize_list_with_mixed_content() {
        use crate::txxt::ast::{List, ListItem, Paragraph};

        // Create a list with mixed content:
        // 1. First item
        //    Paragraph explaining the item.
        //    - Nested list item one
        //    - Nested list item two
        //    Another paragraph.
        let nested_list = List::new(vec![
            ContentItem::ListItem(ListItem::new("- Nested list item one".to_string())),
            ContentItem::ListItem(ListItem::new("- Nested list item two".to_string())),
        ]);

        let list = List::new(vec![ContentItem::ListItem(ListItem::with_content(
            "1. First item".to_string(),
            vec![
                ContentItem::Paragraph(Paragraph::from_line(
                    "Paragraph explaining the item.".to_string(),
                )),
                ContentItem::List(nested_list),
                ContentItem::Paragraph(Paragraph::from_line("Another paragraph.".to_string())),
            ],
        ))]);

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
        use crate::txxt::ast::{List, ListItem};

        // Create deeply nested structure:
        // - Outer item
        //   - Middle item
        //     - Inner item one
        //     - Inner item two
        let inner_list = List::new(vec![
            ContentItem::ListItem(ListItem::new("- Inner item one".to_string())),
            ContentItem::ListItem(ListItem::new("- Inner item two".to_string())),
        ]);

        let middle_list = List::new(vec![ContentItem::ListItem(ListItem::with_content(
            "- Middle item".to_string(),
            vec![ContentItem::List(inner_list)],
        ))]);

        let outer_list = List::new(vec![ContentItem::ListItem(ListItem::with_content(
            "- Outer item".to_string(),
            vec![ContentItem::List(middle_list)],
        ))]);

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
        use crate::txxt::lexer::lex_with_locations;
        use crate::txxt::parser::api::parse_with_source;
        use crate::txxt::processor::txxt_sources::TxxtSources;

        let source = TxxtSources::get_string("070-nested-lists-simple.txxt")
            .expect("Failed to load sample file");
        let tokens = lex_with_locations(&source);
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
        use crate::txxt::lexer::lex_with_locations;
        use crate::txxt::parser::api::parse_with_source;
        use crate::txxt::processor::txxt_sources::TxxtSources;

        let source = TxxtSources::get_string("080-nested-lists-mixed-content.txxt")
            .expect("Failed to load sample file");
        let tokens = lex_with_locations(&source);
        let doc = parse_with_source(tokens, &source).expect("Failed to parse");

        let result = serialize_document(&doc);

        // Verify list items with paragraph content
        assert!(result.contains("<item>- First item with nested paragraph {{list-item}}<children>"));
        assert!(result.contains("<paragraph>This is a paragraph nested inside the first list item"));

        // Verify list items with multiple paragraphs
        assert!(
            result.contains("<item>- Second item with multiple paragraphs {{list-item}}<children>")
        );
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

    #[test]
    fn test_all_nestable_elements_use_container_trait() {
        // Comprehensive test: Verify that all nestable elements (Session, Annotation, ListItem)
        // properly serialize their nested content using the generic Container trait approach
        use crate::txxt::ast::elements::label::Label;
        use crate::txxt::ast::{
            Annotation, Definition, List, ListItem, Paragraph, Session, TextContent,
        };

        // Create a complex nested structure with all element types
        let nested_paragraph =
            ContentItem::Paragraph(Paragraph::from_line("Nested paragraph".to_string()));
        let nested_list = ContentItem::List(List::new(vec![
            ContentItem::ListItem(ListItem::new("- Item one".to_string())),
            ContentItem::ListItem(ListItem::new("- Item two".to_string())),
        ]));

        // Test Session with nested content
        let session = Session::new(
            TextContent::from_string("Test Session".to_string(), None),
            vec![nested_paragraph.clone(), nested_list.clone()],
        );

        // Test Definition with nested content
        let definition = Definition::new(
            TextContent::from_string("Test Definition".to_string(), None),
            vec![nested_paragraph.clone(), nested_list.clone()],
        );

        // Test Annotation with nested content
        let annotation = Annotation::new(
            Label::new("test".to_string()),
            vec![],
            vec![nested_paragraph.clone(), nested_list.clone()],
        );

        // Test ListItem with nested content (list in list)
        let outer_list = List::new(vec![ContentItem::ListItem(ListItem::with_content(
            "Outer item".to_string(),
            vec![nested_paragraph.clone(), nested_list.clone()],
        ))]);

        let doc = Document::with_content(vec![
            ContentItem::Session(session),
            ContentItem::Definition(definition),
            ContentItem::Annotation(annotation),
            ContentItem::List(outer_list),
        ]);

        let result = serialize_document(&doc);

        // Verify Session uses <children> wrapper
        assert!(result.contains("<session>Test Session<children>"));
        assert!(result.contains("</children></session>"));

        // Verify Definition uses <content> wrapper
        assert!(result.contains("<definition>Test Definition<content>"));
        assert!(result.contains("</content></definition>"));

        // Verify Annotation uses <content> wrapper
        assert!(result.contains("<annotation>test<content>"));
        assert!(result.contains("</content></annotation>"));

        // Verify ListItem uses <children> wrapper
        assert!(result.contains("<item>Outer item<children>"));
        assert!(result.contains("</children></item>"));

        // Verify all have nested paragraph
        let para_count = result
            .matches("<paragraph>Nested paragraph</paragraph>")
            .count();
        assert_eq!(
            para_count, 4,
            "Should have 4 nested paragraphs (one in each container)"
        );

        // Verify all have nested lists
        let list_open_count = result.matches("<list>").count();
        // We have: 1 in session, 1 in definition, 1 in annotation, 1 outer + 1 nested in outer = 5 total
        assert_eq!(list_open_count, 5, "Should have 5 lists total");
    }
}
