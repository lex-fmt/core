//! Markdown parsing (Markdown → Lex import)
//!
//! Converts CommonMark Markdown to Lex documents.
//! Pipeline: Markdown string → Comrak AST → Events → IR → Lex AST

use crate::common::flat_to_nested::events_to_tree;
use crate::error::FormatError;
use crate::ir::events::Event;
use crate::ir::nodes::InlineContent;
use comrak::nodes::{AstNode, NodeValue};
use comrak::{parse_document, Arena, ComrakOptions};
use lex_parser::lex::ast::Document;

/// Parse Markdown string to Lex document
pub fn parse_from_markdown(source: &str) -> Result<Document, FormatError> {
    // Step 1: Parse Markdown string to Comrak AST
    let arena = Arena::new();
    let options = default_comrak_options();
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

fn default_comrak_options() -> ComrakOptions<'static> {
    let mut options = ComrakOptions::default();
    options.extension.table = true;
    options.extension.strikethrough = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.superscript = true;
    options
}

type DefinitionPieces = Option<(Vec<InlineContent>, Vec<InlineContent>)>;

/// Convert Comrak AST to IR events
fn comrak_ast_to_events<'a>(root: &'a AstNode<'a>) -> Result<Vec<Event>, FormatError> {
    let mut events = vec![Event::StartDocument];

    collect_children_with_definitions(root.children(), &mut events)?;

    events.push(Event::EndDocument);
    Ok(events)
}

/// Recursively collect events from a Comrak AST node
fn collect_events_from_node<'a>(
    node: &'a AstNode<'a>,
    events: &mut Vec<Event>,
) -> Result<(), FormatError> {
    let node_data = node.data.borrow();

    match &node_data.value {
        NodeValue::Document => {
            // Skip document wrapper, process children
            collect_children_with_definitions(node.children(), events)?;
        }

        NodeValue::Heading(heading) => {
            let level = heading.level as usize;

            // Just emit StartHeading - flat_to_nested will auto-close headings
            events.push(Event::StartHeading(level));

            // Process heading text (inline content)
            for child in node.children() {
                collect_inline_events(child, events)?;
            }

            // No EndHeading needed - the generic flat_to_nested converter
            // automatically closes headings when it sees a new heading at same/higher level
        }

        NodeValue::Paragraph => {
            events.push(Event::StartParagraph);

            // Process inline content
            for child in node.children() {
                collect_inline_events(child, events)?;
            }

            events.push(Event::EndParagraph);
        }

        NodeValue::List(list) => {
            let ordered = matches!(list.list_type, comrak::nodes::ListType::Ordered);
            events.push(Event::StartList { ordered });

            // Process list items
            for child in node.children() {
                collect_events_from_node(child, events)?;
            }

            events.push(Event::EndList);
        }

        NodeValue::Item(_) => {
            events.push(Event::StartListItem);

            // Process list item content
            collect_children_with_definitions(node.children(), events)?;

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
                collect_events_from_node(child, events)?;
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

/// Determine if a node is a heading (used to know when to stop collecting
/// definition description siblings).
fn is_heading_node(node: &AstNode<'_>) -> bool {
    matches!(node.data.borrow().value, NodeValue::Heading(_))
}

/// Attempt to parse a paragraph as a definition term of the form
/// `**Term**: Description...` returning the term inline content and any
/// inline description that appears in the same paragraph after the colon.
fn try_parse_definition_term<'a>(node: &'a AstNode<'a>) -> Result<DefinitionPieces, FormatError> {
    if !matches!(node.data.borrow().value, NodeValue::Paragraph) {
        return Ok(None);
    }

    let mut children = node.children();
    let first = match children.next() {
        Some(child) => child,
        None => return Ok(None),
    };

    if !matches!(first.data.borrow().value, NodeValue::Strong) {
        return Ok(None);
    }

    // Gather the term content from the strong node
    let mut term_inlines = Vec::new();
    for child in first.children() {
        collect_inline_content(child, &mut term_inlines)?;
    }

    let mut description_inlines = Vec::new();
    let mut saw_colon = false;

    for child in children {
        let child_data = child.data.borrow();
        match &child_data.value {
            NodeValue::Text(text) => {
                if !saw_colon {
                    let trimmed = text.trim_start();
                    if let Some(rest) = trimmed.strip_prefix(':') {
                        saw_colon = true;
                        let rest = rest.trim_start();
                        if !rest.is_empty() {
                            description_inlines.push(InlineContent::Text(rest.to_string()));
                        }
                    } else {
                        // Text before a colon means this is not a definition pattern
                        return Ok(None);
                    }
                } else if !text.is_empty() {
                    description_inlines.push(InlineContent::Text(text.clone()));
                }
            }
            // Only collect additional inline nodes after we have seen the colon
            NodeValue::Strong
            | NodeValue::Emph
            | NodeValue::Code(_)
            | NodeValue::Link(_)
            | NodeValue::SoftBreak
            | NodeValue::LineBreak => {
                if !saw_colon {
                    return Ok(None);
                }
                collect_inline_content(child, &mut description_inlines)?;
            }
            _ => {
                if !saw_colon {
                    return Ok(None);
                }
            }
        }
    }

    if !saw_colon {
        return Ok(None);
    }

    Ok(Some((term_inlines, description_inlines)))
}

/// Collect sibling nodes, treating definition term patterns as Definition IR
/// and consuming subsequent siblings as the description until a heading or
/// another definition term is encountered.
fn collect_children_with_definitions<'a, I>(
    children: I,
    events: &mut Vec<Event>,
) -> Result<(), FormatError>
where
    I: Iterator<Item = &'a AstNode<'a>>,
{
    let mut iter = children.peekable();

    while let Some(node) = iter.next() {
        if let Some((term_inlines, inline_description)) = try_parse_definition_term(node)? {
            events.push(Event::StartDefinition);
            events.push(Event::StartDefinitionTerm);
            for inline in term_inlines {
                events.push(Event::Inline(inline));
            }
            events.push(Event::EndDefinitionTerm);

            events.push(Event::StartDefinitionDescription);
            if !inline_description.is_empty() {
                events.push(Event::StartParagraph);
                for inline in inline_description {
                    events.push(Event::Inline(inline));
                }
                events.push(Event::EndParagraph);
            }

            // Consume subsequent siblings as the description body until we hit
            // a heading or another definition term.
            while let Some(peek) = iter.peek() {
                if is_heading_node(peek) {
                    break;
                }

                // Stop if the next paragraph is another definition term
                let should_stop = try_parse_definition_term(peek)?.is_some();
                if should_stop {
                    break;
                }

                let next = iter.next().expect("peek yielded a node");
                collect_events_from_node(next, events)?;
            }

            events.push(Event::EndDefinitionDescription);
            events.push(Event::EndDefinition);
        } else {
            collect_events_from_node(node, events)?;
        }
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
