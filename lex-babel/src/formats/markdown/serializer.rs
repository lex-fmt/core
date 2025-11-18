//! Markdown serialization (Lex → Markdown export)
//!
//! Converts Lex documents to CommonMark Markdown format.
//! Pipeline: Lex AST → IR → Events → Markdown string

use crate::error::FormatError;
use crate::ir::events::Event;
use crate::ir::nodes::{DocNode, InlineContent};
use crate::mappings::nested_to_flat::tree_to_events;

/// Serialize a Lex document to Markdown
pub fn serialize_to_markdown(doc: &lex_parser::lex::ast::Document) -> Result<String, FormatError> {
    // Step 1: Lex AST → IR
    let ir_doc = crate::to_ir(doc);

    // Step 2: IR → Events
    let events = tree_to_events(&DocNode::Document(ir_doc));

    // Step 3: Events → Markdown string (direct conversion, no intermediate AST)
    events_to_markdown(&events)
}

/// Convert IR events to Markdown string
fn events_to_markdown(events: &[Event]) -> Result<String, FormatError> {
    let mut output = String::new();
    let mut list_depth: usize = 0;
    let mut in_heading = false;
    let mut heading_inline_done = false;
    let mut in_list_item = false;
    let mut list_item_first_inline = false;

    for event in events {
        match event {
            Event::StartDocument => {
                // Nothing to emit for document start
            }
            Event::EndDocument => {
                // Ensure trailing newline
                if !output.is_empty() && !output.ends_with('\n') {
                    output.push('\n');
                }
            }

            Event::StartHeading(level) => {
                // Clamp to markdown's max heading level (h6)
                let md_level = (*level).min(6);
                output.push_str(&"#".repeat(md_level));
                output.push(' ');
                in_heading = true;
                heading_inline_done = false;
            }
            Event::EndHeading(_) => {
                // If we haven't closed the heading line yet (no nested content), close it now
                if !heading_inline_done {
                    output.push_str("\n\n");
                } else {
                    // We had nested content, add a blank line after the heading section
                    output.push('\n');
                }
                in_heading = false;
                heading_inline_done = false;
            }

            Event::StartParagraph => {
                // If we're in a heading and haven't closed the heading line yet, do so now
                if in_heading && !heading_inline_done {
                    output.push_str("\n\n");
                    heading_inline_done = true;
                }
            }
            Event::EndParagraph => {
                // Don't add double newline if we're inside a heading (will be added by EndHeading)
                if !in_heading {
                    output.push('\n');
                    output.push('\n'); // Double newline after paragraph
                } else {
                    output.push('\n');
                }
            }

            Event::StartList => {
                list_depth += 1;
            }
            Event::EndList => {
                list_depth -= 1;
                if list_depth == 0 {
                    output.push('\n'); // Blank line after list
                }
            }

            Event::StartListItem => {
                // Indent based on depth
                let indent = "  ".repeat(list_depth.saturating_sub(1));
                output.push_str(&indent);
                output.push_str("- ");
                in_list_item = true;
                list_item_first_inline = true;
            }
            Event::EndListItem => {
                // List items in Lex include trailing newline, so just add one more for spacing
                if !output.ends_with('\n') {
                    output.push('\n');
                }
                in_list_item = false;
                list_item_first_inline = false;
            }

            Event::StartVerbatim(language) => {
                output.push_str("```");
                if let Some(lang) = language {
                    output.push_str(lang);
                }
                output.push('\n');
            }
            Event::EndVerbatim => {
                output.push_str("```\n\n");
            }

            Event::StartDefinition => {
                // Definitions will be handled specially via inline content
            }
            Event::EndDefinition => {
                output.push('\n');
                output.push('\n');
            }
            Event::StartDefinitionTerm => {
                output.push_str("**");
            }
            Event::EndDefinitionTerm => {
                output.push_str("**:");
            }
            Event::StartDefinitionDescription => {
                output.push(' ');
            }
            Event::EndDefinitionDescription => {
                // Nothing needed
            }

            Event::StartAnnotation { label, parameters } => {
                // Convert to HTML comment
                output.push_str("<!-- lex:");
                output.push_str(label);
                for (key, value) in parameters {
                    output.push(' ');
                    output.push_str(key);
                    output.push('=');
                    output.push_str(value);
                }
                output.push_str(" -->\n");
            }
            Event::EndAnnotation => {
                output.push_str("<!-- /lex -->\n\n");
            }

            Event::Inline(inline) => {
                if in_list_item && list_item_first_inline {
                    // Strip leading list marker from first inline in list item
                    emit_inline_stripped(inline, &mut output);
                    list_item_first_inline = false;
                } else {
                    emit_inline(inline, &mut output);
                }
            }
        }
    }

    Ok(output)
}

/// Emit inline content to the output
fn emit_inline(inline: &InlineContent, output: &mut String) {
    match inline {
        InlineContent::Text(text) => {
            output.push_str(text);
        }
        InlineContent::Bold(children) => {
            output.push_str("**");
            for child in children {
                emit_inline(child, output);
            }
            output.push_str("**");
        }
        InlineContent::Italic(children) => {
            output.push('*');
            for child in children {
                emit_inline(child, output);
            }
            output.push('*');
        }
        InlineContent::Code(text) => {
            output.push('`');
            output.push_str(text);
            output.push('`');
        }
        InlineContent::Math(text) => {
            output.push('$');
            output.push_str(text);
            output.push('$');
        }
        InlineContent::Reference(ref_text) => {
            // Simple reference format: [ref]
            output.push('[');
            output.push_str(ref_text);
            output.push(']');
        }
    }
}

/// Emit inline content with leading list marker stripped
fn emit_inline_stripped(inline: &InlineContent, output: &mut String) {
    match inline {
        InlineContent::Text(text) => {
            // Strip leading "- ", "* ", "+ " or numbered markers
            let stripped = text
                .trim_start_matches("- ")
                .trim_start_matches("* ")
                .trim_start_matches("+ ");

            // Also handle numbered lists like "1. ", "2) ", etc.
            let stripped = if let Some(pos) = stripped.find(|c: char| c == '.' || c == ')') {
                if stripped[..pos].chars().all(|c| c.is_ascii_digit()) {
                    stripped[pos + 1..].trim_start()
                } else {
                    stripped
                }
            } else {
                stripped
            };

            output.push_str(stripped);
        }
        // For other inline types, emit normally (they shouldn't have list markers)
        _ => emit_inline(inline, output),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lex_parser::lex::ast::elements::{ContentItem, Document, Paragraph};
    use lex_parser::lex::transforms::standard::STRING_TO_AST;

    #[test]
    fn test_simple_paragraph() {
        let doc = Document::with_content(vec![ContentItem::Paragraph(Paragraph::from_line(
            "Hello world".to_string(),
        ))]);

        let result = serialize_to_markdown(&doc);
        assert!(result.is_ok());
        let md = result.unwrap();
        assert!(md.contains("Hello world"));
    }

    #[test]
    fn test_paragraph_from_spec() {
        // Test with actual spec file
        let lex_src = "This is a simple paragraph.\n";
        let lex_doc = STRING_TO_AST.run(lex_src.to_string()).unwrap();

        let result = serialize_to_markdown(&lex_doc);
        assert!(result.is_ok());
        let md = result.unwrap();
        assert!(md.contains("This is a simple paragraph"));
    }

    #[test]
    fn test_simple_trifecta() {
        let lex_src = r#"Paragraphs and Single Session Test

This document tests the combination of paragraphs and a single session at the root level.

1. Introduction

    This is the content of the session.

This paragraph comes after the session.
"#;
        let lex_doc = STRING_TO_AST.run(lex_src.to_string()).unwrap();

        // Debug: Check IR events
        let ir_doc = crate::to_ir(&lex_doc);
        let events = crate::mappings::nested_to_flat::tree_to_events(&DocNode::Document(ir_doc));
        println!("Events ({} total):", events.len());
        for (i, event) in events.iter().enumerate() {
            println!("  [{}] {:?}", i, event);
        }

        let result = serialize_to_markdown(&lex_doc);
        assert!(result.is_ok());
        let md = result.unwrap();

        println!("\nGenerated Markdown:\n{}", md);

        // Check for paragraph content
        assert!(md.contains("Paragraphs and Single Session Test"));
        assert!(md.contains("This document tests"));

        // Check for heading (Lex sessions include the numbering in the title)
        assert!(md.contains("# 1. Introduction"));

        // Check for nested paragraph
        assert!(md.contains("This is the content of the session"));

        // Check for paragraph after session
        assert!(md.contains("This paragraph comes after the session"));
    }
}
