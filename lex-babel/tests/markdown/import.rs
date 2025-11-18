//! Import tests for Markdown format (Markdown → Lex)
//!
//! These tests verify that Markdown documents are correctly converted to Lex
//! by checking the resulting Lex AST structure.

use comrak::{parse_document, Arena, ComrakOptions};
use lex_babel::format::Format;
use lex_babel::formats::markdown::MarkdownFormat;
use lex_parser::lex::ast::ContentItem;

/// Helper to parse Markdown to Lex AST
fn md_to_lex(md: &str) -> lex_parser::lex::ast::Document {
    MarkdownFormat.parse(md).expect("Should parse markdown")
}

#[test]
fn test_paragraph_simple() {
    let md = "This is a simple paragraph.\n";
    let doc = md_to_lex(md);

    // Should have paragraph in root session
    assert!(!doc.root.children.is_empty());

    // Verify first element is a paragraph
    match &doc.root.children[0] {
        ContentItem::Paragraph(_) => {}
        _ => panic!("Expected paragraph element"),
    }
}

#[test]
fn test_heading_to_session() {
    let md = "# Introduction\n\nSome content here.\n";
    let doc = md_to_lex(md);

    // Should have session with title "Introduction"
    assert!(!doc.root.children.is_empty());

    match &doc.root.children[0] {
        ContentItem::Session(session) => {
            // Check title
            assert!(
                !session.title.is_empty(),
                "Session should have title from heading"
            );

            // Should have content
            assert!(
                !session.children.is_empty(),
                "Session should have paragraph content"
            );
        }
        _ => panic!("Expected session element from heading"),
    }
}

#[test]
fn test_nested_headings() {
    let md = "# Level 1\n\n## Level 2\n\nContent.\n";
    let doc = md_to_lex(md);

    // Should have nested sessions
    assert!(!doc.root.children.is_empty());

    match &doc.root.children[0] {
        ContentItem::Session(session1) => {
            // First session should have nested session
            let has_nested_session = session1
                .children
                .iter()
                .any(|el| matches!(el, ContentItem::Session(_)));
            assert!(has_nested_session, "Should have nested session");
        }
        _ => panic!("Expected session element"),
    }
}

#[test]
fn test_list() {
    let md = "- First item\n- Second item\n- Third item\n";
    let doc = md_to_lex(md);

    assert!(!doc.root.children.is_empty());

    match &doc.root.children[0] {
        ContentItem::List(list) => {
            assert_eq!(list.items.len(), 3, "Should have 3 list items");
        }
        _ => panic!("Expected list element"),
    }
}

#[test]
fn test_code_block_to_verbatim() {
    let md = "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```\n";
    let doc = md_to_lex(md);

    assert!(!doc.root.children.is_empty());

    match &doc.root.children[0] {
        ContentItem::VerbatimBlock(verbatim) => {
            // Should have code content (children contains VerbatimLine items)
            assert!(!verbatim.children.is_empty(), "Should have code content");
        }
        _ => panic!("Expected verbatim element from code block"),
    }
}

#[test]
fn test_inline_formatting() {
    // Test that paragraphs are created and have content
    let md = "This is **bold** and *italic* and `code` text.\n";
    let doc = md_to_lex(md);

    match &doc.root.children[0] {
        ContentItem::Paragraph(para) => {
            // Should have lines with text content
            assert!(!para.lines.is_empty(), "Paragraph should have lines");
        }
        _ => panic!("Expected paragraph"),
    }
}

// ============================================================================
// TRIFECTA TESTS - Document Structure
// ============================================================================

#[test]
fn test_trifecta_010_round_trip() {
    // Start with Lex, export to Markdown, import back to Lex
    let lex_src = std::fs::read_to_string(
        "../docs/specs/v1/trifecta/010-paragraphs-sessions-flat-single.lex",
    )
    .expect("trifecta 010 file should exist");

    let lex_doc = lex_parser::lex::transforms::standard::STRING_TO_AST
        .run(lex_src.to_string())
        .unwrap();

    // Export to Markdown
    let md = MarkdownFormat.serialize(&lex_doc).unwrap();

    // Import back to Lex
    let lex_doc2 = md_to_lex(&md);

    // Should have sessions
    let has_sessions = lex_doc2
        .root
        .children
        .iter()
        .any(|el| matches!(el, ContentItem::Session(_)));
    assert!(has_sessions, "Round-trip should preserve sessions");
}

#[test]
fn test_trifecta_020_round_trip() {
    let lex_src = std::fs::read_to_string(
        "../docs/specs/v1/trifecta/020-paragraphs-sessions-flat-multiple.lex",
    )
    .expect("trifecta 020 file should exist");

    let lex_doc = lex_parser::lex::transforms::standard::STRING_TO_AST
        .run(lex_src.to_string())
        .unwrap();

    // Export to Markdown
    let md = MarkdownFormat.serialize(&lex_doc).unwrap();

    // Import back to Lex
    let lex_doc2 = md_to_lex(&md);

    // Count sessions
    let session_count = lex_doc2
        .root
        .children
        .iter()
        .filter(|el| matches!(el, ContentItem::Session(_)))
        .count();

    assert!(
        session_count >= 2,
        "Should have multiple sessions, found {}",
        session_count
    );
}

#[test]
fn test_trifecta_060_nesting_round_trip() {
    let lex_src = std::fs::read_to_string("../docs/specs/v1/trifecta/060-trifecta-nesting.lex")
        .expect("trifecta 060 file should exist");

    let lex_doc = lex_parser::lex::transforms::standard::STRING_TO_AST
        .run(lex_src.to_string())
        .unwrap();

    // Export to Markdown
    let md = MarkdownFormat.serialize(&lex_doc).unwrap();

    // Parse markdown to verify structure
    let arena = Arena::new();
    let options = ComrakOptions::default();
    let root = parse_document(&arena, &md, &options);

    // Should have multiple heading levels
    let mut heading_levels = vec![];
    for child in root.children() {
        if let comrak::nodes::NodeValue::Heading(h) = &child.data.borrow().value {
            heading_levels.push(h.level);
        }
    }

    assert!(
        !heading_levels.is_empty(),
        "Should have headings from sessions"
    );
    assert!(
        heading_levels.iter().max().unwrap_or(&1) >= &2,
        "Should have nested heading levels"
    );

    // Import back to Lex
    let lex_doc2 = md_to_lex(&md);

    // Should have both paragraphs and sessions and lists
    let has_paragraphs = lex_doc2
        .root
        .children
        .iter()
        .any(|el| matches!(el, ContentItem::Paragraph(_)));
    let has_sessions = lex_doc2
        .root
        .children
        .iter()
        .any(|el| matches!(el, ContentItem::Session(_)));
    let has_lists = lex_doc2
        .root
        .children
        .iter()
        .any(|el| matches!(el, ContentItem::List(_)));

    assert!(
        has_paragraphs || has_sessions || has_lists,
        "Should have various element types"
    );
}

// ============================================================================
// BENCHMARK TESTS
// ============================================================================

#[test]
fn test_kitchensink_round_trip() {
    let lex_src = std::fs::read_to_string("../docs/specs/v1/benchmark/010-kitchensink.lex")
        .expect("kitchensink file should exist");

    let lex_doc = lex_parser::lex::transforms::standard::STRING_TO_AST
        .run(lex_src.to_string())
        .unwrap();

    // Export to Markdown
    let md = MarkdownFormat.serialize(&lex_doc).unwrap();

    // Import back to Lex
    let lex_doc2 = md_to_lex(&md);

    // Verify we have multiple element types
    fn check_elements(elements: &[ContentItem], flags: &mut (bool, bool, bool, bool)) {
        for el in elements {
            match el {
                ContentItem::Paragraph(_) => flags.0 = true,
                ContentItem::Session(s) => {
                    flags.1 = true;
                    check_elements(&s.children, flags);
                }
                ContentItem::List(_) => flags.2 = true,
                ContentItem::VerbatimBlock(_) => flags.3 = true,
                _ => {}
            }
        }
    }

    let mut flags = (false, false, false, false);
    check_elements(&lex_doc2.root.children, &mut flags);
    let (has_paragraph, has_session, has_list, has_verbatim) = flags;

    assert!(has_paragraph, "Kitchensink should have paragraphs");
    assert!(has_session, "Kitchensink should have sessions");
    assert!(has_list, "Kitchensink should have lists");
    assert!(has_verbatim, "Kitchensink should have verbatim blocks");
}
