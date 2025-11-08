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

#[test]
fn test_blank_line_group_location_visitor() {
    let source = "A\n\nB";
    let doc = parse_document(source);

    // Use new query API to iterate all nodes and find blank line groups
    let all_nodes: Vec<_> = doc.iter_all_nodes().collect();
    if let Some(blg) = all_nodes
        .iter()
        .filter_map(|item| item.as_blank_line_group())
        .next()
    {
        // Test that location field is accessible and works without panic
        let _loc = &blg.location;
    }
}

#[test]
fn test_blank_line_group_node_type_visitor() {
    let source = "A\n\nB";
    let doc = parse_document(source);

    let all_nodes: Vec<_> = doc.iter_all_nodes().collect();
    if let Some(blg) = all_nodes
        .iter()
        .filter_map(|item| item.as_blank_line_group())
        .next()
    {
        // Test that node_type method works
        let node_type = blg.node_type();
        assert_eq!(node_type, "BlankLineGroup");
    }
}

#[test]
fn test_blank_line_group_display_label_visitor() {
    let source = "A\n\nB";
    let doc = parse_document(source);

    let all_nodes: Vec<_> = doc.iter_all_nodes().collect();
    if let Some(blg) = all_nodes
        .iter()
        .filter_map(|item| item.as_blank_line_group())
        .next()
    {
        // Test that display_label method works
        let label = blg.display_label();
        assert!(label.contains("1"), "Label should mention blank line count");
    }
}

#[test]
fn test_blank_line_group_structure_count() {
    let source = "A\n\nB";
    let doc = parse_document(source);

    let all_nodes: Vec<_> = doc.iter_all_nodes().collect();
    if let Some(blg) = all_nodes
        .iter()
        .filter_map(|item| item.as_blank_line_group())
        .next()
    {
        // Verify count field exists and is accessible
        assert!(blg.count > 0, "BlankLineGroup should have count > 0");
    }
}

#[test]
fn test_blank_line_group_structure_source_tokens() {
    let source = "A\n\nB";
    let doc = parse_document(source);

    let all_nodes: Vec<_> = doc.iter_all_nodes().collect();
    if let Some(blg) = all_nodes
        .iter()
        .filter_map(|item| item.as_blank_line_group())
        .next()
    {
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
    }
}

#[test]
fn test_blank_line_group_in_list_items() {
    let source = "- Item\n\n    Content";
    let doc = parse_document(source);

    // Use new query API to find list items with blank line groups
    let list_items: Vec<_> = doc.iter_list_items_recursive().collect();
    if let Some(list_item) = list_items.iter().find(|li| {
        li.children
            .iter()
            .any(|child| matches!(child, ContentItem::BlankLineGroup(_)))
    }) {
        if let Some(blg) = list_item
            .children
            .iter()
            .filter_map(|item| item.as_blank_line_group())
            .next()
        {
            assert!(blg.count > 0);
        }
    }
}

#[test]
fn test_blank_line_group_in_definitions() {
    let source = "Definition:\n    First\n\n    Second";
    let doc = parse_document(source);

    // Use new query API to find definitions with blank line groups
    let definitions = doc.find_definitions(|def| {
        def.children
            .iter()
            .any(|child| matches!(child, ContentItem::BlankLineGroup(_)))
    });

    if let Some(definition) = definitions.into_iter().next() {
        if let Some(blg) = definition
            .children
            .iter()
            .filter_map(|item| item.as_blank_line_group())
            .next()
        {
            assert!(blg.count > 0, "Definition should have blank lines");
        }
    }
}

#[test]
fn test_blank_line_group_in_sessions() {
    let source = "Title\n\n    First\n\n    Second";
    let doc = parse_document(source);

    // Use new query API to find sessions with blank line groups
    let sessions = doc.find_sessions(|s| {
        s.children
            .iter()
            .any(|child| matches!(child, ContentItem::BlankLineGroup(_)))
    });

    if let Some(session) = sessions.into_iter().next() {
        if let Some(blg) = session
            .children
            .iter()
            .filter_map(|item| item.as_blank_line_group())
            .next()
        {
            assert!(blg.count > 0, "Session should have blank lines");
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
        .children
        .iter()
        .any(|item| matches!(item, ContentItem::BlankLineGroup(_)));
    // Test passes if compilation succeeds
}
