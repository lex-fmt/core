//! Export tests for Markdown format (Lex → Markdown)
//!
//! These tests verify that Lex documents are correctly converted to Markdown
//! by checking the resulting Comrak AST structure.

use comrak::nodes::NodeValue;
use comrak::{parse_document, Arena, ComrakOptions};
use insta::assert_snapshot;
use lex_babel::format::Format;
use lex_babel::formats::markdown::MarkdownFormat;
use lex_parser::lex::transforms::standard::STRING_TO_AST;

/// Helper to convert Lex source to Markdown and parse to Comrak AST
fn lex_to_comrak_ast<'a>(
    lex_src: &str,
    arena: &'a Arena<comrak::nodes::AstNode<'a>>,
) -> &'a comrak::nodes::AstNode<'a> {
    let lex_doc = STRING_TO_AST.run(lex_src.to_string()).unwrap();
    let md = MarkdownFormat.serialize(&lex_doc).unwrap();
    let options = ComrakOptions::default();
    parse_document(arena, &md, &options)
}

/// Helper to recursively collect node types from a Comrak AST
fn collect_node_types<'a>(
    node: &'a comrak::nodes::AstNode<'a>,
    types: &mut std::collections::HashSet<String>,
) {
    let value = &node.data.borrow().value;
    let type_name = match value {
        NodeValue::Document => "Document",
        NodeValue::Paragraph => "Paragraph",
        NodeValue::Heading(_) => "Heading",
        NodeValue::List(_) => "List",
        NodeValue::Item(_) => "Item",
        NodeValue::CodeBlock(_) => "CodeBlock",
        NodeValue::Strong => "Strong",
        NodeValue::Emph => "Emph",
        NodeValue::Code(_) => "Code",
        NodeValue::Link(_) => "Link",
        _ => "Other",
    };
    types.insert(type_name.to_string());

    for child in node.children() {
        collect_node_types(child, types);
    }
}

#[test]
fn test_paragraph_simple() {
    let lex_src = "This is a simple paragraph.\n";

    let arena = Arena::new();
    let root = lex_to_comrak_ast(lex_src, &arena);

    // Verify structure: Document → Paragraph → Text
    let mut found_paragraph = false;
    for child in root.children() {
        if matches!(child.data.borrow().value, NodeValue::Paragraph) {
            found_paragraph = true;
        }
    }
    assert!(found_paragraph, "Should have a paragraph node");
}

#[test]
fn test_heading_and_paragraph_separation() {
    // Session title followed by paragraph content should not be merged into the heading
    let lex_src = "1. Title\n\n    Body text.\n";

    let arena = Arena::new();
    let root = lex_to_comrak_ast(lex_src, &arena);

    let mut heading_text = String::new();
    let mut paragraph_text = String::new();

    for child in root.children() {
        match &child.data.borrow().value {
            NodeValue::Heading(_) => {
                for inline in child.children() {
                    if let NodeValue::Text(t) = &inline.data.borrow().value {
                        heading_text.push_str(t);
                    }
                }
            }
            NodeValue::Paragraph => {
                for inline in child.children() {
                    if let NodeValue::Text(t) = &inline.data.borrow().value {
                        paragraph_text.push_str(t);
                    }
                }
            }
            _ => {}
        }
    }

    assert_eq!(heading_text.trim(), "(1) Title");
    assert!(paragraph_text.contains("Body text."));
}

// ============================================================================
// TRIFECTA TESTS - Document Structure
// ============================================================================

#[test]
fn test_trifecta_010_paragraphs_sessions_flat_single() {
    let lex_src =
        std::fs::read_to_string("../specs/v1/trifecta/010-paragraphs-sessions-flat-single.lex")
            .expect("trifecta 010 file should exist");

    let arena = Arena::new();
    let root = lex_to_comrak_ast(&lex_src, &arena);

    // Collect all top-level node types
    let mut paragraphs = 0;
    let mut headings = 0;
    let mut heading_levels = Vec::new();

    for child in root.children() {
        match &child.data.borrow().value {
            NodeValue::Paragraph => paragraphs += 1,
            NodeValue::Heading(h) => {
                headings += 1;
                heading_levels.push(h.level);
            }
            _ => {}
        }
    }

    // Should have:
    // - Initial paragraph ("Paragraphs and Single Session Test")
    // - Another paragraph ("This document tests...")
    // - Heading for "1. Introduction"
    // - Paragraphs that were nested in the session (now at document level)
    // - Paragraph after session
    // - Heading for "Another Session"
    // - Paragraph nested in that session
    // - Final paragraph

    assert!(paragraphs > 0, "Should have paragraphs");
    assert!(
        headings >= 2,
        "Should have at least 2 headings (sessions), found {}",
        headings
    );

    // All sessions at root level should be h1
    for level in &heading_levels {
        assert_eq!(*level, 1, "Root-level sessions should be h1");
    }

    println!(
        "Trifecta 010: {} paragraphs, {} headings",
        paragraphs, headings
    );
}

#[test]
fn test_trifecta_020_paragraphs_sessions_flat_multiple() {
    let lex_src =
        std::fs::read_to_string("../specs/v1/trifecta/020-paragraphs-sessions-flat-multiple.lex")
            .expect("trifecta 020 file should exist");

    let arena = Arena::new();
    let root = lex_to_comrak_ast(&lex_src, &arena);

    // Count headings (sessions)
    let heading_count = root
        .children()
        .filter(|child| matches!(child.data.borrow().value, NodeValue::Heading(_)))
        .count();

    // Should have multiple sessions including nested ones
    // Root: "First Session", "Second Session", "Third Session", "Session Without Numbering"
    // Nested: "Session titles don't require numbering markers." (nested in "Session Without Numbering")
    assert!(
        heading_count >= 4,
        "Should have at least 4 headings (sessions), found {}",
        heading_count
    );

    // Collect heading levels to see if we have nesting
    let mut has_h1 = false;
    let mut heading_levels_vec = Vec::new();
    for child in root.children() {
        if let NodeValue::Heading(h) = &child.data.borrow().value {
            heading_levels_vec.push(h.level);
            if h.level == 1 {
                has_h1 = true;
            }
        }
    }

    assert!(has_h1, "Should have at least one h1 (root session)");
    println!("Trifecta 020: Heading levels: {:?}", heading_levels_vec);

    println!("Trifecta 020: {} headings", heading_count);
}

#[test]
fn test_trifecta_060_nesting() {
    let lex_src = std::fs::read_to_string("../specs/v1/trifecta/060-trifecta-nesting.lex")
        .expect("trifecta 060 file should exist");

    let arena = Arena::new();
    let root = lex_to_comrak_ast(&lex_src, &arena);

    // Collect heading levels to verify nesting structure
    let mut heading_levels = Vec::new();
    let mut has_paragraphs = false;
    let mut has_lists = false;

    for child in root.children() {
        match &child.data.borrow().value {
            NodeValue::Heading(h) => heading_levels.push(h.level),
            NodeValue::Paragraph => has_paragraphs = true,
            NodeValue::List(_) => has_lists = true,
            _ => {}
        }
    }

    // Verify we have multiple heading levels (representing nested sessions)
    let min_level = heading_levels.iter().min().copied().unwrap_or(0);
    let max_level = heading_levels.iter().max().copied().unwrap_or(0);

    assert!(
        min_level >= 1 && max_level >= 2,
        "Should have nested heading levels (h1, h2, h3...), found range {}..{}",
        min_level,
        max_level
    );

    // Should have all three element types
    assert!(has_paragraphs, "Should have paragraphs");
    assert!(has_lists, "Should have lists");
    assert!(!heading_levels.is_empty(), "Should have headings");

    // Verify heading sequence makes sense
    // Session 1.2.1 should generate h3, 1.1 should generate h2, etc.
    assert!(
        heading_levels.contains(&1),
        "Should have h1 for root sessions"
    );
    assert!(
        heading_levels.contains(&2) || heading_levels.contains(&3),
        "Should have h2 or h3 for nested sessions"
    );

    println!(
        "Trifecta 060: {} headings with levels {:?}, {} paragraphs, {} lists",
        heading_levels.len(),
        heading_levels,
        if has_paragraphs { "some" } else { "no" },
        if has_lists { "some" } else { "no" }
    );
}

// ============================================================================
// BENCHMARK TESTS
// ============================================================================

#[test]
fn test_kitchensink() {
    let lex_src = std::fs::read_to_string("../specs/v1/benchmark/010-kitchensink.lex")
        .expect("kitchensink file should exist");

    let arena = Arena::new();
    let root = lex_to_comrak_ast(&lex_src, &arena);

    // Kitchensink should have variety of nodes
    let mut node_types = std::collections::HashSet::new();
    collect_node_types(root, &mut node_types);

    // Kitchensink should exercise multiple element types
    assert!(
        node_types.len() >= 5,
        "Kitchensink should have at least 5 different node types, found: {:?}",
        node_types
    );

    println!("Kitchensink node types: {:?}", node_types);
}

#[test]
fn test_kitchensink_snapshot() {
    let lex_src = std::fs::read_to_string("../specs/v1/benchmark/010-kitchensink.lex")
        .expect("kitchensink file should exist");

    let lex_doc = STRING_TO_AST.run(lex_src.to_string()).unwrap();
    let md = MarkdownFormat.serialize(&lex_doc).unwrap();

    assert_snapshot!("kitchensink_markdown", md);
}
