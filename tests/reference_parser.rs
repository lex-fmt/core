//! Integration tests for the reference parser.

use lex::lex::parsing::parse_document;
use lex::lex::testing::assert_ast;
use lex::lex::testing::lexplore::Lexplore;

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

#[test]
fn test_dialog_parsing() {
    // Tests that dash-prefixed lines without proper list formatting are parsed as a paragraph
    let source = Lexplore::paragraph(9).source();
    let doc = parse_document(&source).unwrap();

    assert_ast(&doc).item_count(1).item(0, |item| {
        item.assert_paragraph()
            .text("- Hi mom!!.\n- Hi kiddo.")
            .line_count(2);
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
    // Renamed from 050 to 070 to avoid duplicate numbers
    let source =
        Lexplore::from_path("docs/specs/v1/trifecta/070-trifecta-flat-simple.lex").source();
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
    let source = Lexplore::trifecta(60).source();
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
    // Using definition-90-document-simple.lex which tests definitions in context
    let source = Lexplore::definition(90).source();
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

// Annotation and verbatim block tests have been moved to their respective element modules:
// - elements/annotations.rs for annotation tests
// - elements/verbatim.rs for verbatim block tests

#[test]
fn test_regression_definition_with_list_followed_by_definition() {
    // Issue: https://github.com/arthur-debert/lex/issues/41
    // See: docs/specs/v1/regression-bugs/parser-definition-list-transition.lex

    let source = std::fs::read_to_string(
        "docs/specs/v1/regression-bugs/parser-definition-list-transition.lex",
    )
    .expect("Failed to load regression test file");

    let doc = parse_document(&source)
        .expect("Parser should handle definition with list followed by definition");

    // Verify structure using assert_ast
    assert_ast(&doc)
        .item_count(2)
        .item(0, |item| {
            item.assert_definition();
        })
        .item(1, |item| {
            item.assert_definition();
        });
}
