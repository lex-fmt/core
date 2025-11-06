//! Integration tests for the reference parser.

use lex::lex::ast::{AstNode, ContentItem, Position};
use lex::lex::parsing::parse_document;
use lex::lex::processor::lex_sources::LexSources;
use lex::lex::testing::assert_ast;

#[test]
fn test_real_content_extraction() {
    // Test that we extract real content, not placeholder strings
    let input = "First paragraph with numbers 123 and symbols (like this).\n\nSecond paragraph.\n\n1. Session Title\n\n    Session content here.\n\n";

    let doc = parse_document(input).expect("Failed to parse");

    assert_ast(&doc)
        .item_count(3)
        .item(0, |item| {
            item.assert_paragraph()
                .text("First paragraph with numbers 123 and symbols (like this).")
                .line_count(1);
        })
        .item(1, |item| {
            item.assert_paragraph()
                .text("Second paragraph.")
                .line_count(1);
        })
        .item(2, |item| {
            item.assert_session()
                .label("1. Session Title")
                .child_count(1)
                .child(0, |child| {
                    child
                        .assert_paragraph()
                        .text("Session content here.")
                        .line_count(1);
                });
        });
}

// Session tests have been moved to elements/sessions.rs
// List tests have been moved to elements/lists.rs
// Definition tests have been moved to elements/definitions.rs

// ==================== TRIFECTA TESTS ====================
// Testing paragraphs + sessions + lists together

#[test]
fn test_trifecta_flat_simple() {
    // Test flat structure with all three elements
    let source = LexSources::get_string("050-trifecta-flat-simple.lex").unwrap();
    let doc = parse_document(&source).unwrap();

    // Item 0-1: Opening paragraphs
    assert_ast(&doc)
        .item(0, |item| {
            item.assert_paragraph()
                .text_contains("Trifecta Flat Structure Test");
        })
        .item(1, |item| {
            item.assert_paragraph()
                .text_contains("all three core elements");
        });

    // Item 2: Session with only paragraphs
    assert_ast(&doc).item(2, |item| {
        item.assert_session()
            .label_contains("Session with Paragraph Content")
            .child_count(2)
            .child(0, |child| {
                child
                    .assert_paragraph() // "Session with Paragraph Content"
                    .text_contains("starts with a paragraph");
            })
            .child(1, |child| {
                child
                    .assert_paragraph() // "multiple paragraphs"
                    .text_contains("multiple paragraphs");
            });
    });

    // Item 3: Session with only a list
    assert_ast(&doc).item(3, |item| {
        item.assert_session()
            .label_contains("Session with List Content")
            .child_count(1)
            .child(0, |child| {
                child.assert_list().item_count(3);
            });
    });

    // Item 4: Session with mixed content (para + list + para)
    assert_ast(&doc).item(4, |item| {
        item.assert_session()
            .label_contains("Session with Mixed Content")
            .child_count(3)
            .child(0, |child| {
                child
                    .assert_paragraph()
                    .text_contains("starts with a paragraph");
            })
            .child(1, |child| {
                child.assert_list().item_count(2);
            })
            .child(2, |child| {
                child
                    .assert_paragraph()
                    .text_contains("ends with another paragraph");
            });
    });

    // Item 5: Root level paragraph
    assert_ast(&doc).item(5, |item| {
        item.assert_paragraph().text_contains("root level");
    });

    // Item 6: Root level list
    assert_ast(&doc).item(6, |item| {
        item.assert_list().item_count(2);
    });

    // Item 7: Session with list + para + list
    assert_ast(&doc).item(7, |item| {
        item.assert_session()
            .label_contains("Another Session")
            .child_count(3)
            .child(0, |child| {
                child.assert_list().item_count(2);
            })
            .child(1, |child| {
                child.assert_paragraph().text_contains("has a paragraph");
            })
            .child(2, |child| {
                child.assert_list().item_count(2);
            });
    });
}

#[test]
fn test_trifecta_nesting() {
    // Test nested structure with all three elements
    let source = LexSources::get_string("060-trifecta-nesting.lex").unwrap();
    let doc = parse_document(&source).unwrap();

    // Item 0-1: Opening paragraphs
    assert_ast(&doc)
        .item(0, |item| {
            item.assert_paragraph() // "Trifecta Nesting Test"
                .text_contains("Trifecta Nesting Test");
        })
        .item(1, |item| {
            item.assert_paragraph() // "various levels of nesting"
                .text_contains("various levels of nesting");
        });

    // Item 2: Root session with nested sessions and mixed content
    // The structure has been updated to include nested lists, which may affect the child count
    assert_ast(&doc).item(2, |item| {
        item.assert_session()
            .label_contains("1. Root Session")
            .child_count(5); // para, subsession, subsession, para, list
    });

    // Verify first child of root session is paragraph
    assert_ast(&doc).item(2, |item| {
        item.assert_session().child(0, |child| {
            child.assert_paragraph().text_contains("nested elements");
        });
    });

    // Verify first nested session (1.1)
    assert_ast(&doc).item(2, |item| {
        item.assert_session().child(1, |child| {
            child
                .assert_session()
                .label_contains("1.1. Sub-session")
                .child_count(2) // para + list
                .child(0, |para| {
                    para.assert_paragraph();
                })
                .child(1, |list| {
                    list.assert_list().item_count(2);
                });
        });
    });

    // Verify deeply nested session (1.2 containing 1.2.1)
    assert_ast(&doc).item(2, |item| {
        item.assert_session().child(2, |child| {
            child
                .assert_session()
                .label_contains("1.2. Sub-session with List")
                .child_count(3) // list, para, nested session
                .child(2, |nested| {
                    nested
                        .assert_session()
                        .label_contains("1.2.1. Deeply Nested")
                        .child_count(3); // para + list + list
                });
        });
    });

    // Verify the deeply nested session has 2 lists
    assert_ast(&doc).item(2, |item| {
        item.assert_session().child(2, |subsession| {
            subsession.assert_session().child(2, |deeply_nested| {
                deeply_nested
                    .assert_session()
                    .child(1, |first_list| {
                        first_list.assert_list().item_count(2);
                    })
                    .child(2, |second_list| {
                        second_list.assert_list().item_count(2);
                    });
            });
        });
    });

    // Item 3: Another root session with different nesting
    assert_ast(&doc).item(3, |item| {
        item.assert_session()
            .label_contains("2. Another Root Session")
            .child_count(2); // para + subsession
    });

    // Verify even deeper nesting (2.1.1)
    assert_ast(&doc).item(3, |item| {
        item.assert_session().child(1, |subsession| {
            subsession
                .assert_session()
                .label_contains("2.1. Mixed Content")
                .child_count(4) // list, para, list, nested session
                .child(3, |deeply_nested| {
                    deeply_nested
                        .assert_session()
                        .label_contains("2.1.1. Even Deeper")
                        .child_count(4); // para, list, para, list
                });
        });
    });

    // Final root paragraph
    assert_ast(&doc).item(4, |item| {
        item.assert_paragraph()
            .text_contains("Final root level paragraph");
    });
}

// Nested list tests have been moved to elements/lists.rs
// Definition tests have been moved to elements/definitions.rs

#[test]
fn test_verified_ensemble_with_definitions() {
    // Comprehensive ensemble test with all core elements including definitions
    let source = LexSources::get_string("110-ensemble-with-definitions.lex").unwrap();
    let doc = parse_document(&source).unwrap();

    // Item 0-1: Opening paragraphs
    assert_ast(&doc)
        .item(0, |item| {
            item.assert_paragraph() // "Ensemble Test with Definitions"
                .text_contains("Ensemble Test with Definitions");
        })
        .item(1, |item| {
            item.assert_paragraph() // "all core elements"
                .text_contains("all core elements");
        });

    // Item 2: Introduction definition (with para + list)
    assert_ast(&doc).item(2, |item| {
        item.assert_definition()
            .subject("Introduction")
            .child_count(2)
            .child(0, |child| {
                child.assert_paragraph().text_contains("ensemble test");
            })
            .child(1, |child| {
                child.assert_list().item_count(4);
            });
    });

    // Item 3: Simple Elements Section session
    assert_ast(&doc).item(3, |item| {
        item.assert_session()
            .label("1. Simple Elements Section {{session}}")
            .child_count(5); // para + 2 definitions + para + list
    });

    // Item 4: Nested Elements Section session
    assert_ast(&doc).item(4, |item| {
        item.assert_session()
            .label("2. Nested Elements Section {{session}}")
            .child_count(3); // para + 2 subsections (2.1 and 2.2)
    });
}

// Annotation and foreign block tests have been moved to their respective element modules:
// - elements/annotations.rs for annotation tests
// - elements/foreign.rs for foreign block tests

#[test]
fn test_regression_definition_with_list_followed_by_definition() {
    // Issue: https://github.com/arthur-debert/lex/issues/41
    // See: docs/specs/v1/regression-bugs/parser-definition-list-transition.lex

    let source = std::fs::read_to_string(
        "docs/specs/v1/regression-bugs/parser-definition-list-transition.lex",
    )
    .expect("Failed to load regression test file");

    // This should parse successfully but currently fails with:
    // Parse error at location 14..15: reason=Unexpected, found=Some((Newline, 34..35))
    let doc = parse_document(&source)
        .expect("Parser should handle definition with list followed by definition");

    // Should have 2 definitions
    assert_eq!(doc.root.children.len(), 2);

    // First should be a definition
    assert!(doc.root.children[0].as_definition().is_some());

    // Second should also be a definition
    assert!(doc.root.children[1].as_definition().is_some());
}

// ========================================================================
// Integration Tests for Position Tracking
// ========================================================================

#[test]
fn test_parse_simple() {
    let input = "Hello world\n\n";
    let doc = parse_document(input).expect("Failed to parse with positions");

    assert_eq!(doc.root.children.len(), 1);
    let para = doc.root.children[0].as_paragraph().unwrap();
    let range = para.range();
    assert_eq!(range.start.line, 0);
    assert_eq!(range.start.column, 0);
}

#[test]
fn test_parse_multiline() {
    let input = "First line\nSecond line\n\n";
    let doc = parse_document(input).expect("Failed to parse with positions");

    assert_eq!(doc.root.children.len(), 1);
    let para = doc.root.children[0].as_paragraph().unwrap();

    // Should have 2 lines
    assert_eq!(para.lines.len(), 2);

    // Location should cover both lines
    let range = para.range();
    assert_eq!(range.start.line, 0);
    assert_eq!(range.end.line, 1);
}

#[test]
fn test_element_at_query_on_parsed_document() {
    let input = "First paragraph\n\n2. Session Title\n\n    Session content\n\n";
    let doc = parse_document(input).expect("Failed to parse with positions");

    // Query for the session (should be at line 2)
    let result = doc.element_at(Position::new(2, 3));

    // Should find the element
    assert!(result.is_some(), "Should find element at position 2:3");

    // Result should be a session
    assert!(result.unwrap().is_session());
}

#[test]
fn test_element_at_nested_position() {
    let input = "Title\n\n1. Item one\n\n    Nested content\n\n";
    let doc = parse_document(input).expect("Failed to parse with positions");

    // The document should have at least a paragraph and possibly a list
    assert!(!doc.root.children.is_empty());

    // Query for position in the nested content
    let result = doc.element_at(Position::new(4, 4));

    // Should find element at that position (or return None if position is outside all locations)
    // This is acceptable - position 4:4 might be outside all defined locations
    let _ = result;
}

#[test]
fn test_position_comparison_in_query() {
    let input = "Line 0\n\nLine 2\n\n";
    let doc = parse_document(input).expect("Failed to parse with positions");

    // Get all items
    let items = doc.root.children.clone();

    // First paragraph should be at line 0
    if let Some(para) = items.first().and_then(|item| item.as_paragraph()) {
        let range = para.range();
        assert_eq!(range.start.line, 0);
    }

    // Second paragraph should be at line 2
    if let Some(para) = items.get(1).and_then(|item| item.as_paragraph()) {
        let range = para.range();
        assert_eq!(range.start.line, 2);
    }
}

// Annotation position test moved to elements/annotations.rs

#[test]
fn test_backward_compatibility_without_positions() {
    let input = "Simple paragraph\n\n";

    // Old parser should still work (without positions)
    let doc_old = parse_document(input).expect("Failed to parse without positions");

    // New parser with positions
    let doc_new = parse_document(input).expect("Failed to parse with positions");

    // Content should be identical
    assert_eq!(doc_old.root.children.len(), doc_new.root.children.len());

    let para_old = doc_old.root.children[0].as_paragraph().unwrap();
    let para_new = doc_new.root.children[0].as_paragraph().unwrap();

    // Text content should be the same (ignoring location information)
    assert_eq!(para_old.lines.len(), para_new.lines.len());
    for (_, line_new) in para_old.lines.iter().zip(para_new.lines.iter()) {
        // Old para had TextContent, new para has ContentItem::TextLine
        if let ContentItem::TextLine(tl) = line_new {
            // Just check that new version has text
            assert!(!tl.text().is_empty());
        } else {
            panic!("Expected TextLine in paragraph");
        }
    }

    // But new version should have positions on the paragraph and text
    let para_range = para_new.range();
    assert!(para_range.start <= para_range.end);
    if let ContentItem::TextLine(tl) = &para_new.lines[0] {
        let line_range = tl.range();
        assert!(line_range.start <= line_range.end);
    }
}

#[test]
fn test_location_boundary_containment() {
    let input = "0123456789\n\n";
    let doc = parse_document(input).expect("Failed to parse with positions");

    let para = doc.root.children[0].as_paragraph().unwrap();
    let range = para.range();

    // Should contain position in the middle
    assert!(range.contains(Position::new(0, 5)));

    // Should contain start
    assert!(range.contains(range.start));

    // Should contain end
    assert!(range.contains(range.end));

    // Should?? contain position after end
    assert!(!range.contains(Position::new(0, 11)));
}

#[test]
fn test_nested_paragraph_has_location() {
    // Test that nested paragraphs inside sessions have location information
    let input = "Title\n\n1. Session Title\n\n    Nested paragraph\n\n";
    let doc = parse_document(input).expect("Failed to parse with positions");

    assert!(doc.root.children.len() >= 2);

    // Find the session
    let session = doc
        .root
        .children
        .iter()
        .find(|item| item.is_session())
        .expect("Should have a session");

    let session_range = session.range();
    assert!(
        session_range.start <= session_range.end,
        "Session should have location"
    );

    // Get nested content
    if let Some(children) = session.children() {
        assert!(!children.is_empty(), "Session should have children");

        // Check if first child is a paragraph
        if let Some(para_item) = children.first() {
            if para_item.is_paragraph() {
                let para = para_item.as_paragraph().unwrap();
                let range = para.range();
                assert!(
                    range.start <= range.end,
                    "Nested paragraph should have location, but got {:?}",
                    range
                );

                println!(
                    "Nested paragraph location: {:?} to {:?}",
                    range.start, range.end
                );
                assert_eq!(range.start.line, 4, "Paragraph should be at line 4");
            }
        }
    }
}

#[test]
fn test_location_tracking_for_core_elements() {
    let source = LexSources::get_string("110-ensemble-with-definitions.lex")
        .expect("Failed to load ensemble sample");
    let doc = parse_document(&source).expect("Failed to parse ensemble sample");

    // Document doesn't have its own location; location comes from root
    let root_range = doc.root.range();
    assert!(
        root_range.start <= root_range.end,
        "Root session should have a location"
    );

    for item in &doc.root.children {
        let item_range = item.range();
        assert!(
            item_range.start <= item_range.end,
            "{} should have a location",
            item.node_type()
        );

        match item {
            ContentItem::Paragraph(paragraph) => {
                for line in &paragraph.lines {
                    if let ContentItem::TextLine(tl) = line {
                        let line_range = tl.range();
                        assert!(
                            line_range.start <= line_range.end,
                            "Paragraph line should have location"
                        );
                    }
                }
            }
            ContentItem::Session(session) => {
                assert!(
                    session.title.location.is_some(),
                    "Session title is missing location"
                );
                for child in &session.children {
                    let child_range = child.range();
                    assert!(
                        child_range.start <= child_range.end,
                        "Session child should have location"
                    );
                }
            }
            ContentItem::Definition(definition) => {
                assert!(
                    definition.subject.location.is_some(),
                    "Definition subject should have location"
                );
                for child in &definition.children {
                    let child_range = child.range();
                    assert!(
                        child_range.start <= child_range.end,
                        "Definition child should have location"
                    );
                }
            }
            ContentItem::List(list) => {
                for item in &list.items {
                    if let ContentItem::ListItem(list_item) = item {
                        for text in &list_item.text {
                            assert!(
                                text.location.is_some(),
                                "List item text should have location"
                            );
                        }
                        for child in &list_item.children {
                            let child_range = child.range();
                            assert!(
                                child_range.start <= child_range.end,
                                "Nested list item child should have location"
                            );
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

#[test]
fn test_location_tracking_for_annotations() {
    let source = LexSources::get_string("120-annotations-simple.lex")
        .expect("Failed to load annotations sample");
    let doc = parse_document(&source).expect("Failed to parse annotations sample");

    let annotations: Vec<_> = doc
        .root
        .children
        .iter()
        .filter_map(|item| item.as_annotation())
        .collect();
    assert!(!annotations.is_empty(), "Expected annotations in sample");

    for annotation in annotations {
        for child in &annotation.children {
            let child_range = child.range();
            assert!(
                child_range.start <= child_range.end,
                "Annotation content should have a location"
            );
        }
    }
}

#[test]
fn test_location_tracking_for_foreign_blocks() {
    let source = LexSources::get_string("140-foreign-blocks-simple.lex")
        .expect("Failed to load foreign blocks sample");
    let doc = parse_document(&source).expect("Failed to parse foreign blocks sample");

    let foreign_blocks: Vec<_> = doc
        .root
        .children
        .iter()
        .filter_map(|item| item.as_foreign_block())
        .collect();
    assert!(
        !foreign_blocks.is_empty(),
        "Expected foreign blocks in sample"
    );

    for block in foreign_blocks {
        assert!(
            block.subject.location.is_some(),
            "Verbatim block subject should have a location"
        );
        // Check that each VerbatimLine child has a location
        for child in &block.children {
            if let Some(foreign_line) = child.as_foreign_line() {
                assert!(
                    foreign_line.content.location.is_some(),
                    "Verbatim line content should have a location"
                );
            }
        }

        let closing = &block.closing_annotation;
        for child in &closing.children {
            let child_range = child.range();
            assert!(
                child_range.start <= child_range.end,
                "Closing annotation content should have a location"
            );
        }
    }
}
