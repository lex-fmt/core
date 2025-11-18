//! Markdown serialization (Lex → Markdown export)
//!
//! Converts Lex documents to CommonMark Markdown.
//! Pipeline: Lex AST → IR → Events → Comrak AST → Markdown string

use crate::error::FormatError;
use crate::ir::events::Event;
use crate::ir::nodes::DocNode;
use crate::mappings::nested_to_flat::tree_to_events;
use comrak::nodes::{Ast, AstNode, ListDelimType, ListType, NodeValue};
use comrak::{format_commonmark, Arena, ComrakOptions};
use lex_parser::lex::ast::Document;
use std::cell::RefCell;

/// Serialize a Lex document to Markdown
pub fn serialize_to_markdown(doc: &Document) -> Result<String, FormatError> {
    // Step 1: Lex AST → IR
    let ir_doc = crate::to_ir(doc);

    // Step 2: IR → Events
    let events = tree_to_events(&DocNode::Document(ir_doc));

    // Step 3: Events → Comrak AST
    let arena = Arena::new();
    let root = build_comrak_ast(&arena, &events)?;

    // Step 4: Comrak AST → Markdown string (using comrak's serializer)
    let mut output = Vec::new();
    let options = ComrakOptions::default();
    format_commonmark(root, &options, &mut output).map_err(|e| {
        FormatError::SerializationError(format!("Comrak serialization failed: {}", e))
    })?;

    String::from_utf8(output)
        .map_err(|e| FormatError::SerializationError(format!("UTF-8 conversion failed: {}", e)))
}

/// Build a Comrak AST from IR events
fn build_comrak_ast<'a>(
    arena: &'a Arena<AstNode<'a>>,
    events: &[Event],
) -> Result<&'a AstNode<'a>, FormatError> {
    // Create document root
    let root = arena.alloc(AstNode::new(RefCell::new(Ast::new(
        NodeValue::Document,
        (0, 0).into(),
    ))));

    let mut current_parent: &'a AstNode<'a> = root;
    let mut parent_stack: Vec<&'a AstNode<'a>> = vec![];

    // State for collecting verbatim content
    let mut in_verbatim = false;
    let mut verbatim_language: Option<String> = None;
    let mut verbatim_content = String::new();

    // State for handling headings (which can only contain inline content)
    let mut current_heading: Option<&'a AstNode<'a>> = None;

    // State for handling list items (need to auto-wrap inline content in paragraphs)
    let mut in_list_item = false;
    let mut list_item_paragraph: Option<&'a AstNode<'a>> = None;

    for event in events {
        match event {
            Event::StartDocument => {
                // Already created root
            }

            Event::EndDocument => {
                // Done
            }

            Event::StartHeading(level) => {
                // Headings can only contain inline content, not block elements
                // Create heading and set it as target for inline content
                let heading_node = arena.alloc(AstNode::new(RefCell::new(Ast::new(
                    NodeValue::Heading(comrak::nodes::NodeHeading {
                        level: (*level as u8).min(6),
                        setext: false,
                    }),
                    (0, 0).into(),
                ))));
                current_parent.append(heading_node);
                current_heading = Some(heading_node);
                // Note: We do NOT change current_parent or push to parent_stack
                // Block content after this heading will be siblings at document level
            }

            Event::EndHeading(_) => {
                // Close heading - block content goes back to document level
                current_heading = None;
            }

            Event::StartParagraph => {
                let para_node = arena.alloc(AstNode::new(RefCell::new(Ast::new(
                    NodeValue::Paragraph,
                    (0, 0).into(),
                ))));
                current_parent.append(para_node);
                parent_stack.push(current_parent);
                current_parent = para_node;
                // If we're in a list item, this explicit paragraph replaces any auto-created one
                if in_list_item {
                    list_item_paragraph = None;
                }
            }

            Event::EndParagraph => {
                current_parent = parent_stack.pop().ok_or_else(|| {
                    FormatError::SerializationError("Unbalanced paragraph end".to_string())
                })?;
            }

            Event::StartList => {
                let list_node = arena.alloc(AstNode::new(RefCell::new(Ast::new(
                    NodeValue::List(comrak::nodes::NodeList {
                        list_type: ListType::Bullet,
                        marker_offset: 0,
                        padding: 0,
                        start: 1,
                        delimiter: ListDelimType::Period,
                        bullet_char: b'-',
                        tight: false,
                    }),
                    (0, 0).into(),
                ))));
                current_parent.append(list_node);
                parent_stack.push(current_parent);
                current_parent = list_node;
            }

            Event::EndList => {
                current_parent = parent_stack.pop().ok_or_else(|| {
                    FormatError::SerializationError("Unbalanced list end".to_string())
                })?;
            }

            Event::StartListItem => {
                let item_node = arena.alloc(AstNode::new(RefCell::new(Ast::new(
                    NodeValue::Item(comrak::nodes::NodeList {
                        list_type: ListType::Bullet,
                        marker_offset: 0,
                        padding: 0,
                        start: 1,
                        delimiter: ListDelimType::Period,
                        bullet_char: b'-',
                        tight: false,
                    }),
                    (0, 0).into(),
                ))));
                current_parent.append(item_node);
                parent_stack.push(current_parent);
                current_parent = item_node;
                in_list_item = true;
                list_item_paragraph = None;
            }

            Event::EndListItem => {
                current_parent = parent_stack.pop().ok_or_else(|| {
                    FormatError::SerializationError("Unbalanced list item end".to_string())
                })?;
                in_list_item = false;
                list_item_paragraph = None;
            }

            Event::StartVerbatim(language) => {
                in_verbatim = true;
                verbatim_language = language.clone();
                verbatim_content.clear();
            }

            Event::EndVerbatim => {
                // Create code block with accumulated content
                let code_node = arena.alloc(AstNode::new(RefCell::new(Ast::new(
                    NodeValue::CodeBlock(comrak::nodes::NodeCodeBlock {
                        fenced: true,
                        fence_char: b'`',
                        fence_length: 3,
                        fence_offset: 0,
                        info: verbatim_language.take().unwrap_or_default(),
                        literal: verbatim_content.clone(),
                    }),
                    (0, 0).into(),
                ))));
                current_parent.append(code_node);
                in_verbatim = false;
                verbatim_content.clear();
            }

            Event::Inline(inline_content) => {
                if in_verbatim {
                    // Accumulate verbatim content
                    if let crate::ir::nodes::InlineContent::Text(text) = inline_content {
                        verbatim_content.push_str(text);
                    }
                } else if let Some(heading) = current_heading {
                    // Add to heading (headings can have inline content directly)
                    add_inline_to_node(arena, heading, inline_content)?;
                } else if in_list_item {
                    // List items need inline content wrapped in a paragraph
                    if list_item_paragraph.is_none() {
                        // Create paragraph for this list item's inline content
                        let para = arena.alloc(AstNode::new(RefCell::new(Ast::new(
                            NodeValue::Paragraph,
                            (0, 0).into(),
                        ))));
                        current_parent.append(para);
                        list_item_paragraph = Some(para);
                    }
                    // Add inline content to the paragraph
                    add_inline_to_node(arena, list_item_paragraph.unwrap(), inline_content)?;
                } else {
                    // Regular inline content added to current_parent
                    add_inline_to_node(arena, current_parent, inline_content)?;
                }
            }

            Event::StartAnnotation { label, parameters } => {
                // Emit as HTML comment
                let mut comment = format!("<!-- lex:{}", label);
                for (key, value) in parameters {
                    comment.push_str(&format!(" {}={}", key, value));
                }
                comment.push_str(" -->");

                let html_node = arena.alloc(AstNode::new(RefCell::new(Ast::new(
                    NodeValue::HtmlBlock(comrak::nodes::NodeHtmlBlock {
                        block_type: 0,
                        literal: comment,
                    }),
                    (0, 0).into(),
                ))));
                current_parent.append(html_node);
            }

            Event::EndAnnotation => {
                // Closing annotation comment
                let html_node = arena.alloc(AstNode::new(RefCell::new(Ast::new(
                    NodeValue::HtmlBlock(comrak::nodes::NodeHtmlBlock {
                        block_type: 0,
                        literal: "<!-- /lex -->".to_string(),
                    }),
                    (0, 0).into(),
                ))));
                current_parent.append(html_node);
            }

            Event::StartDefinition => {
                // Definitions in Markdown: Term paragraph followed by description content
                // Don't create wrapper, let content be siblings at document level
            }

            Event::EndDefinition => {
                // Nothing needed
            }

            Event::StartDefinitionTerm => {
                // Create paragraph for the term with bold styling
                let para_node = arena.alloc(AstNode::new(RefCell::new(Ast::new(
                    NodeValue::Paragraph,
                    (0, 0).into(),
                ))));
                current_parent.append(para_node);
                parent_stack.push(current_parent);
                current_parent = para_node;

                // Add bold wrapper for term text
                let strong_node = arena.alloc(AstNode::new(RefCell::new(Ast::new(
                    NodeValue::Strong,
                    (0, 0).into(),
                ))));
                current_parent.append(strong_node);
                parent_stack.push(current_parent);
                current_parent = strong_node;
            }

            Event::EndDefinitionTerm => {
                // Close bold
                current_parent = parent_stack.pop().ok_or_else(|| {
                    FormatError::SerializationError("Unbalanced definition term end".to_string())
                })?;

                // Add colon after term
                let colon_node = arena.alloc(AstNode::new(RefCell::new(Ast::new(
                    NodeValue::Text(":".to_string()),
                    (0, 0).into(),
                ))));
                current_parent.append(colon_node);

                // Close term paragraph
                current_parent = parent_stack.pop().ok_or_else(|| {
                    FormatError::SerializationError(
                        "Unbalanced definition term paragraph".to_string(),
                    )
                })?;
            }

            Event::StartDefinitionDescription => {
                // Description content will be siblings at document level
                // No wrapper needed
            }

            Event::EndDefinitionDescription => {
                // Nothing needed
            }
        }
    }

    Ok(root)
}

/// Add inline content to a comrak node
fn add_inline_to_node<'a>(
    arena: &'a Arena<AstNode<'a>>,
    parent: &'a AstNode<'a>,
    inline: &crate::ir::nodes::InlineContent,
) -> Result<(), FormatError> {
    use crate::ir::nodes::InlineContent;

    match inline {
        InlineContent::Text(text) => {
            let text_node = arena.alloc(AstNode::new(RefCell::new(Ast::new(
                NodeValue::Text(text.clone()),
                (0, 0).into(),
            ))));
            parent.append(text_node);
        }

        InlineContent::Bold(children) => {
            let strong_node = arena.alloc(AstNode::new(RefCell::new(Ast::new(
                NodeValue::Strong,
                (0, 0).into(),
            ))));
            parent.append(strong_node);
            for child in children {
                add_inline_to_node(arena, strong_node, child)?;
            }
        }

        InlineContent::Italic(children) => {
            let emph_node = arena.alloc(AstNode::new(RefCell::new(Ast::new(
                NodeValue::Emph,
                (0, 0).into(),
            ))));
            parent.append(emph_node);
            for child in children {
                add_inline_to_node(arena, emph_node, child)?;
            }
        }

        InlineContent::Code(code_text) => {
            let code_node = arena.alloc(AstNode::new(RefCell::new(Ast::new(
                NodeValue::Code(comrak::nodes::NodeCode {
                    num_backticks: 1,
                    literal: code_text.clone(),
                }),
                (0, 0).into(),
            ))));
            parent.append(code_node);
        }

        InlineContent::Reference(url) => {
            let link_node = arena.alloc(AstNode::new(RefCell::new(Ast::new(
                NodeValue::Link(comrak::nodes::NodeLink {
                    url: url.clone(),
                    title: String::new(),
                }),
                (0, 0).into(),
            ))));
            parent.append(link_node);
            // Add link text as child
            let text_node = arena.alloc(AstNode::new(RefCell::new(Ast::new(
                NodeValue::Text(url.clone()),
                (0, 0).into(),
            ))));
            link_node.append(text_node);
        }

        InlineContent::Math(math_text) => {
            // Math not supported in CommonMark, render as $...$
            let dollar_open = arena.alloc(AstNode::new(RefCell::new(Ast::new(
                NodeValue::Text("$".to_string()),
                (0, 0).into(),
            ))));
            parent.append(dollar_open);

            let math_node = arena.alloc(AstNode::new(RefCell::new(Ast::new(
                NodeValue::Text(math_text.clone()),
                (0, 0).into(),
            ))));
            parent.append(math_node);

            let dollar_close = arena.alloc(AstNode::new(RefCell::new(Ast::new(
                NodeValue::Text("$".to_string()),
                (0, 0).into(),
            ))));
            parent.append(dollar_close);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use comrak::{parse_document, ComrakOptions};
    use lex_parser::lex::transforms::standard::STRING_TO_AST;

    #[test]
    fn test_simple_paragraph_ast() {
        let lex_src = "This is a simple paragraph.\n";
        let lex_doc = STRING_TO_AST.run(lex_src.to_string()).unwrap();

        // Convert to markdown
        let md = serialize_to_markdown(&lex_doc).unwrap();

        // Parse back to comrak AST to verify structure
        let arena = Arena::new();
        let options = ComrakOptions::default();
        let root = parse_document(&arena, &md, &options);

        // Verify we have a paragraph
        let mut found_paragraph = false;
        for child in root.children() {
            if matches!(child.data.borrow().value, NodeValue::Paragraph) {
                found_paragraph = true;

                // Check inline text content
                for _inline in child.children() {
                    if let NodeValue::Text(ref text) = child.data.borrow().value {
                        assert!(text.contains("simple paragraph"));
                    }
                }
            }
        }
        assert!(found_paragraph, "Should have a paragraph node");
    }

    #[test]
    fn test_heading_ast() {
        let lex_src = "1. Introduction\n\n    Content here.\n";
        let lex_doc = STRING_TO_AST.run(lex_src.to_string()).unwrap();

        let md = serialize_to_markdown(&lex_doc).unwrap();

        // Parse and verify AST structure
        let arena = Arena::new();
        let options = ComrakOptions::default();
        let root = parse_document(&arena, &md, &options);

        let mut found_heading = false;
        for child in root.children() {
            if let NodeValue::Heading(ref heading) = child.data.borrow().value {
                assert_eq!(heading.level, 1);
                found_heading = true;
            }
        }
        assert!(found_heading, "Should have a heading node");
    }
}
