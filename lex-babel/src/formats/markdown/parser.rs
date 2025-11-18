//! Markdown parsing (Markdown → Lex import)
//!
//! Converts CommonMark Markdown to Lex documents.
//! Pipeline: Markdown string → Comrak AST → Events → IR → Lex AST

use crate::error::FormatError;
use crate::ir::events::Event;
use crate::ir::nodes::InlineContent;
use crate::mappings::flat_to_nested::events_to_tree;
use comrak::nodes::{AstNode, NodeValue};
use comrak::{parse_document, Arena, ComrakOptions};
use lex_parser::lex::ast::Document;

/// Parse Markdown string to Lex document
pub fn parse_from_markdown(source: &str) -> Result<Document, FormatError> {
    // Step 1: Parse Markdown string to Comrak AST
    let arena = Arena::new();
    let options = ComrakOptions::default();
    let root = parse_document(&arena, source, &options);

    // Step 2: Convert Comrak AST to IR events
    let events = comrak_ast_to_events(root)?;

    // Step 3: Convert events to IR tree
    let ir_doc = events_to_tree(&events).map_err(|e| {
        FormatError::ParseError(format!("Failed to build IR tree from events: {}", e))
    })?;

    // Step 4: Convert IR to Lex AST
    let lex_doc = crate::from_ir(&ir_doc);
    Ok(lex_doc)
}

/// Convert Comrak AST to IR events
fn comrak_ast_to_events<'a>(root: &'a AstNode<'a>) -> Result<Vec<Event>, FormatError> {
    let mut events = vec![Event::StartDocument];

    // Track heading levels to build session hierarchy
    let mut heading_stack: Vec<usize> = vec![];

    for child in root.children() {
        collect_events_from_node(child, &mut events, &mut heading_stack)?;
    }

    // Close any remaining open headings
    while let Some(level) = heading_stack.pop() {
        events.push(Event::EndHeading(level));
    }

    events.push(Event::EndDocument);
    Ok(events)
}

/// Recursively collect events from a Comrak AST node
fn collect_events_from_node<'a>(
    node: &'a AstNode<'a>,
    events: &mut Vec<Event>,
    heading_stack: &mut Vec<usize>,
) -> Result<(), FormatError> {
    let node_data = node.data.borrow();

    match &node_data.value {
        NodeValue::Document => {
            // Skip document wrapper, process children
            for child in node.children() {
                collect_events_from_node(child, events, heading_stack)?;
            }
        }

        NodeValue::Heading(heading) => {
            let level = heading.level as usize;

            // Close any open headings at same or deeper level
            while let Some(&stack_level) = heading_stack.last() {
                if stack_level >= level {
                    events.push(Event::EndHeading(stack_level));
                    heading_stack.pop();
                } else {
                    break;
                }
            }

            // Start new heading (becomes Session in Lex)
            events.push(Event::StartHeading(level));
            heading_stack.push(level);

            // Process heading text (inline content)
            for child in node.children() {
                collect_inline_events(child, events)?;
            }

            // Note: We don't emit EndHeading here - it will be emitted when we encounter
            // the next heading at same/higher level, or at end of document
        }

        NodeValue::Paragraph => {
            events.push(Event::StartParagraph);

            // Process inline content
            for child in node.children() {
                collect_inline_events(child, events)?;
            }

            events.push(Event::EndParagraph);
        }

        NodeValue::List(_) => {
            events.push(Event::StartList);

            // Process list items
            for child in node.children() {
                collect_events_from_node(child, events, heading_stack)?;
            }

            events.push(Event::EndList);
        }

        NodeValue::Item(_) => {
            events.push(Event::StartListItem);

            // Process list item content
            for child in node.children() {
                collect_events_from_node(child, events, heading_stack)?;
            }

            events.push(Event::EndListItem);
        }

        NodeValue::CodeBlock(code_block) => {
            let language = if code_block.info.is_empty() {
                None
            } else {
                Some(code_block.info.clone())
            };

            events.push(Event::StartVerbatim(language));
            events.push(Event::Inline(InlineContent::Text(
                code_block.literal.clone(),
            )));
            events.push(Event::EndVerbatim);
        }

        NodeValue::HtmlBlock(html) => {
            // Try to parse as Lex annotation
            if let Some((label, parameters)) = parse_lex_annotation(&html.literal) {
                events.push(Event::StartAnnotation { label, parameters });
                // Note: Closing annotation will be found when parsing closing comment
            } else if html.literal.trim() == "<!-- /lex -->" {
                events.push(Event::EndAnnotation);
            }
            // Otherwise skip HTML blocks
        }

        NodeValue::ThematicBreak => {
            // Thematic breaks (---) don't have direct Lex equivalent, skip
        }

        NodeValue::BlockQuote => {
            // Block quotes don't have direct Lex equivalent
            // Process children as regular content
            for child in node.children() {
                collect_events_from_node(child, events, heading_stack)?;
            }
        }

        _ => {
            // Unknown block type, skip
        }
    }

    Ok(())
}

/// Collect inline events from a Comrak node
fn collect_inline_events<'a>(
    node: &'a AstNode<'a>,
    events: &mut Vec<Event>,
) -> Result<(), FormatError> {
    let node_data = node.data.borrow();

    match &node_data.value {
        NodeValue::Text(text) => {
            events.push(Event::Inline(InlineContent::Text(text.clone())));
        }

        NodeValue::Strong => {
            // Collect children as bold content
            let mut children = vec![];
            for child in node.children() {
                collect_inline_content(child, &mut children)?;
            }
            events.push(Event::Inline(InlineContent::Bold(children)));
        }

        NodeValue::Emph => {
            // Collect children as italic content
            let mut children = vec![];
            for child in node.children() {
                collect_inline_content(child, &mut children)?;
            }
            events.push(Event::Inline(InlineContent::Italic(children)));
        }

        NodeValue::Code(code) => {
            events.push(Event::Inline(InlineContent::Code(code.literal.clone())));
        }

        NodeValue::Link(link) => {
            events.push(Event::Inline(InlineContent::Reference(link.url.clone())));
        }

        NodeValue::SoftBreak | NodeValue::LineBreak => {
            events.push(Event::Inline(InlineContent::Text(" ".to_string())));
        }

        _ => {
            // Skip unknown inline types
        }
    }

    Ok(())
}

/// Recursively collect inline content (for nested inlines like bold/italic)
fn collect_inline_content<'a>(
    node: &'a AstNode<'a>,
    content: &mut Vec<InlineContent>,
) -> Result<(), FormatError> {
    let node_data = node.data.borrow();

    match &node_data.value {
        NodeValue::Text(text) => {
            content.push(InlineContent::Text(text.clone()));
        }

        NodeValue::Strong => {
            let mut children = vec![];
            for child in node.children() {
                collect_inline_content(child, &mut children)?;
            }
            content.push(InlineContent::Bold(children));
        }

        NodeValue::Emph => {
            let mut children = vec![];
            for child in node.children() {
                collect_inline_content(child, &mut children)?;
            }
            content.push(InlineContent::Italic(children));
        }

        NodeValue::Code(code) => {
            content.push(InlineContent::Code(code.literal.clone()));
        }

        NodeValue::Link(link) => {
            content.push(InlineContent::Reference(link.url.clone()));
        }

        NodeValue::SoftBreak | NodeValue::LineBreak => {
            content.push(InlineContent::Text(" ".to_string()));
        }

        _ => {}
    }

    Ok(())
}

/// Parse Lex annotation from HTML comment
/// Format: <!-- lex:label key1=val1 key2=val2 -->
fn parse_lex_annotation(html: &str) -> Option<(String, Vec<(String, String)>)> {
    let trimmed = html.trim();
    if !trimmed.starts_with("<!-- lex:") || !trimmed.ends_with("-->") {
        return None;
    }

    // Remove <!-- lex: prefix and --> suffix
    let content = trimmed
        .strip_prefix("<!-- lex:")?
        .strip_suffix("-->")?
        .trim();

    // Split on whitespace
    let parts: Vec<&str> = content.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    let label = parts[0].to_string();
    let mut parameters = vec![];

    for part in &parts[1..] {
        if let Some((key, value)) = part.split_once('=') {
            parameters.push((key.to_string(), value.to_string()));
        }
    }

    Some((label, parameters))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_paragraph() {
        let md = "This is a simple paragraph.\n";
        let doc = parse_from_markdown(md).unwrap();

        // Verify we got a Lex document with a root session
        assert!(!doc.root.children.is_empty());
    }

    #[test]
    fn test_heading_to_session() {
        let md = "# Introduction\n\nSome content.\n";
        let doc = parse_from_markdown(md).unwrap();

        // Should have session with content
        assert!(!doc.root.children.is_empty());
    }

    #[test]
    fn test_code_block_to_verbatim() {
        let md = "```rust\nfn main() {}\n```\n";
        let doc = parse_from_markdown(md).unwrap();

        assert!(!doc.root.children.is_empty());
    }
}
