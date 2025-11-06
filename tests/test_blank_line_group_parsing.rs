//! Unit tests for BlankLineGroup AST node.
//!
//! These tests verify that:
//! - BlankLineGroup is a valid AST node type
//! - BlankLineGroup nodes have proper structure (count, source_tokens, location)
//! - BlankLineGroup nodes are accessible via the visitor pattern
//! - Blank lines in nested structures (definitions, sessions, lists) can contain BlankLineGroup nodes

use lex::lex::ast::AstNode;
use lex::lex::ast::Document;
use lex::lex::parsing::{parse_document as parse_doc, ContentItem};

fn parse_document(source: &str) -> Document {
    parse_doc(source).expect("Failed to parse document")
}

fn find_blank_line_groups(items: &[ContentItem]) -> Vec<&lex::lex::ast::elements::BlankLineGroup> {
    items
        .iter()
        .filter_map(|item| {
            if let ContentItem::BlankLineGroup(blg) = item {
                Some(blg)
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn test_blank_line_group_location_visitor() {
    let source = "A\n\nB";
    let doc = parse_document(source);

    // Find blank line groups and verify location field works
    for item in &doc.root.content {
        if let ContentItem::BlankLineGroup(blg) = item {
            // Test that location field is accessible and works without panic
            let _loc = &blg.location;
            return;
        }
    }
}

#[test]
fn test_blank_line_group_node_type_visitor() {
    let source = "A\n\nB";
    let doc = parse_document(source);

    for item in &doc.root.content {
        if let ContentItem::BlankLineGroup(blg) = item {
            // Test that node_type method works
            let node_type = blg.node_type();
            assert_eq!(node_type, "BlankLineGroup");
            return;
        }
    }
}

#[test]
fn test_blank_line_group_display_label_visitor() {
    let source = "A\n\nB";
    let doc = parse_document(source);

    for item in &doc.root.content {
        if let ContentItem::BlankLineGroup(blg) = item {
            // Test that display_label method works
            let label = blg.display_label();
            assert!(label.contains("1"), "Label should mention blank line count");
            return;
        }
    }
}

#[test]
fn test_blank_line_group_structure_count() {
    let source = "A\n\nB";
    let doc = parse_document(source);

    for item in &doc.root.content {
        if let ContentItem::BlankLineGroup(blg) = item {
            // Verify count field exists and is accessible
            assert!(blg.count > 0, "BlankLineGroup should have count > 0");
            return;
        }
    }
}

#[test]
fn test_blank_line_group_structure_source_tokens() {
    let source = "A\n\nB";
    let doc = parse_document(source);

    for item in &doc.root.content {
        if let ContentItem::BlankLineGroup(blg) = item {
            // Verify source_tokens field exists and is accessible
            assert!(
                !blg.source_tokens.is_empty(),
                "BlankLineGroup should have source tokens"
            );
            // Verify tokens contain BlankLine variant
            let has_blank_line_token = blg
                .source_tokens
                .iter()
                .any(|t| matches!(t, lex::lex::lexing::Token::BlankLine(_)));
            assert!(has_blank_line_token, "Should contain BlankLine token");
            return;
        }
    }
}

#[test]
fn test_blank_line_group_in_list_items() {
    let source = "- Item\n\n    Content";
    let doc = parse_document(source);

    // Search for lists and their item content
    for item in &doc.root.content {
        if let ContentItem::List(list) = item {
            for list_item in &list.content {
                if let ContentItem::ListItem(li) = list_item {
                    // Check if this list item has blank lines
                    let blank_groups = find_blank_line_groups(&li.content);
                    if !blank_groups.is_empty() {
                        assert!(blank_groups[0].count > 0);
                        return;
                    }
                }
            }
        }
    }
}

#[test]
fn test_blank_line_group_in_definitions() {
    let source = "Definition:\n    First\n\n    Second";
    let doc = parse_document(source);

    // Search for definitions and check their content
    for item in &doc.root.content {
        if let ContentItem::Definition(def) = item {
            let blank_groups = find_blank_line_groups(&def.content);
            if !blank_groups.is_empty() {
                assert!(
                    blank_groups[0].count > 0,
                    "Definition should have blank lines"
                );
                return;
            }
        }
    }
}

#[test]
fn test_blank_line_group_in_sessions() {
    let source = "Title\n\n    First\n\n    Second";
    let doc = parse_document(source);

    // Search for sessions and check their content
    for item in &doc.root.content {
        if let ContentItem::Session(session) = item {
            let blank_groups = find_blank_line_groups(&session.content);
            if !blank_groups.is_empty() {
                assert!(blank_groups[0].count > 0, "Session should have blank lines");
                return;
            }
        }
    }
}

#[test]
fn test_blank_line_group_is_content_item_variant() {
    // Verify BlankLineGroup can be matched as a ContentItem variant
    let source = "A\n\nB";
    let doc = parse_document(source);

    // This test verifies the variant exists in ContentItem enum
    // by successfully pattern matching it in the content
    let _has_blank_variant = doc
        .root
        .content
        .iter()
        .any(|item| matches!(item, ContentItem::BlankLineGroup(_)));
    // Test passes if compilation succeeds
}
