//! Position-based lookup for AST nodes
//!
//! This module provides functionality to find and display AST nodes at a given
//! position (line:column) in the source code.

use super::ast::{AstNode, ContentItem, Document, Position, Span};
use crate::txxt_nano::processor::ProcessingError;
use std::collections::HashMap;

/// Format the element at the given position
///
/// Required extras:
/// - position: "<line>:<column>" where line and column are 0-indexed
pub fn format_at_position(
    doc: &Document,
    extras: &HashMap<String, String>,
) -> Result<String, ProcessingError> {
    // Validate and extract position from extras
    let position_str = extras.get("position").ok_or_else(|| {
        ProcessingError::InvalidFormatType(
            "ast-position format requires --extras-position=<line>:<column>".to_string(),
        )
    })?;

    // Parse the position string
    let (line, column) = parse_position(position_str).map_err(|e| {
        ProcessingError::InvalidFormatType(format!(
            "Invalid position format '{}': {}. Expected format: <line>:<column>",
            position_str, e
        ))
    })?;

    let target_pos = Position::new(line, column);

    // Find the element at the position
    find_and_format_at_position(doc, target_pos)
}

/// Parse a position string in the format "line:column"
fn parse_position(position_str: &str) -> Result<(usize, usize), String> {
    let parts: Vec<&str> = position_str.split(':').collect();

    if parts.len() != 2 {
        return Err("Expected format line:column".to_string());
    }

    let line = parts[0]
        .parse::<usize>()
        .map_err(|_| format!("Invalid line number: {}", parts[0]))?;

    let column = parts[1]
        .parse::<usize>()
        .map_err(|_| format!("Invalid column number: {}", parts[1]))?;

    Ok((line, column))
}

/// Find and format the element at the given position
fn find_and_format_at_position(
    doc: &Document,
    target_pos: Position,
) -> Result<String, ProcessingError> {
    // Find all elements in the hierarchy containing this position
    let mut elements = Vec::new();
    find_elements_at_position(&doc.content, target_pos, &mut elements);

    if elements.is_empty() {
        return Err(ProcessingError::InvalidFormatType(format!(
            "No element found at position {}:{}",
            target_pos.line, target_pos.column
        )));
    }

    // Format all elements in the hierarchy
    let mut result = String::new();
    for (i, item) in elements.iter().enumerate() {
        if i > 0 {
            result.push('\n');
        }
        result.push_str(&format!("{}. {}\n", i + 1, item.node_type()));

        if let Some(label) = get_content_item_label(item) {
            result.push_str(&format!("   Label: {}\n", label));
        }

        if let Some(span) = get_content_item_span(item) {
            result.push_str(&format!("   Span: {}\n", span));
        }
    }

    Ok(result)
}

/// Recursively find all elements in the hierarchy containing the target position
fn find_elements_at_position<'a>(
    items: &'a [ContentItem],
    target_pos: Position,
    path: &mut Vec<&'a ContentItem>,
) -> bool {
    for item in items {
        // Check if this item contains the position
        if let Some(span) = get_content_item_span(item) {
            if span.contains(target_pos) {
                // Found it - add to path and return
                path.push(item);
                return true;
            }
        }

        // Always check children of container nodes for nested content
        // even if the container's span doesn't contain the target position
        match item {
            ContentItem::Session(s) => {
                if find_elements_at_position(&s.content, target_pos, path) {
                    path.insert(0, item);
                    return true;
                }
            }
            ContentItem::Definition(d) => {
                if find_elements_at_position(&d.content, target_pos, path) {
                    path.insert(0, item);
                    return true;
                }
            }
            ContentItem::Annotation(a) => {
                if find_elements_at_position(&a.content, target_pos, path) {
                    path.insert(0, item);
                    return true;
                }
            }
            _ => {}
        }
    }

    false
}

/// Get the span of a content item
fn get_content_item_span(item: &ContentItem) -> Option<Span> {
    match item {
        ContentItem::Paragraph(p) => p.span,
        ContentItem::Session(s) => s.span,
        ContentItem::List(l) => l.span,
        ContentItem::Definition(d) => d.span,
        ContentItem::Annotation(a) => a.span,
        ContentItem::ForeignBlock(f) => f.span,
    }
}

/// Get the label of a content item
fn get_content_item_label(item: &ContentItem) -> Option<String> {
    match item {
        ContentItem::Session(s) => Some(s.title.clone()),
        ContentItem::Definition(d) => Some(d.subject.clone()),
        ContentItem::Annotation(a) => Some(a.label.value.clone()),
        ContentItem::Paragraph(p) => Some(p.display_label()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::txxt_nano::parser::{Document as DocumentType, Paragraph};

    #[test]
    fn test_parse_position_valid() {
        assert_eq!(parse_position("0:0").unwrap(), (0, 0));
        assert_eq!(parse_position("5:10").unwrap(), (5, 10));
        assert_eq!(parse_position("100:200").unwrap(), (100, 200));
    }

    #[test]
    fn test_parse_position_invalid() {
        assert!(parse_position("0").is_err());
        assert!(parse_position("0:a").is_err());
        assert!(parse_position("a:0").is_err());
        assert!(parse_position("0:0:0").is_err());
    }

    #[test]
    fn test_format_at_position_missing_position() {
        let doc = DocumentType {
            metadata: vec![],
            content: vec![],
            span: None,
        };

        let result = format_at_position(&doc, &HashMap::new());
        assert!(result.is_err());

        if let Err(ProcessingError::InvalidFormatType(msg)) = result {
            assert!(msg.contains("requires --extras-position"));
        }
    }

    #[test]
    fn test_format_at_position_invalid_position_format() {
        let doc = DocumentType {
            metadata: vec![],
            content: vec![],
            span: None,
        };

        let mut extras = HashMap::new();
        extras.insert("position".to_string(), "invalid".to_string());

        let result = format_at_position(&doc, &extras);
        assert!(result.is_err());

        if let Err(ProcessingError::InvalidFormatType(msg)) = result {
            assert!(msg.contains("Invalid position format"));
        }
    }

    #[test]
    fn test_find_position_with_span() {
        let para = Paragraph::new(vec!["Test paragraph".to_string()])
            .with_span(Some(Span::new(Position::new(0, 0), Position::new(0, 14))));

        let doc = DocumentType::with_content(vec![ContentItem::Paragraph(para)]);

        let mut extras = HashMap::new();
        extras.insert("position".to_string(), "0:5".to_string());

        let result = format_at_position(&doc, &extras);
        assert!(result.is_ok());

        if let Ok(output) = result {
            assert!(output.contains("Paragraph"));
        }
    }

    #[test]
    fn test_find_position_with_actual_parsing() {
        use crate::txxt_nano::lexer::lex_with_spans;
        use crate::txxt_nano::parser::parse_with_source_positions;

        let content = "Simple paragraph\nAnother paragraph";
        let tokens = lex_with_spans(content);
        let doc = parse_with_source_positions(tokens, content).unwrap();

        // Try to find element at position 0:0
        let mut extras = HashMap::new();
        extras.insert("position".to_string(), "0:0".to_string());

        let result = format_at_position(&doc, &extras);

        // Print debug info
        if result.is_err() {
            eprintln!("Error: {:?}", result);

            // Print all elements and their spans
            for (i, item) in doc.content.iter().enumerate() {
                eprintln!(
                    "Item {}: {:?}, span: {:?}",
                    i,
                    item.node_type(),
                    match item {
                        ContentItem::Paragraph(p) => p.span,
                        ContentItem::Session(s) => s.span,
                        ContentItem::List(l) => l.span,
                        ContentItem::Definition(d) => d.span,
                        ContentItem::Annotation(a) => a.span,
                        ContentItem::ForeignBlock(f) => f.span,
                    }
                );
            }
        }

        assert!(result.is_ok(), "Should find element at position 0:0");
    }
}
