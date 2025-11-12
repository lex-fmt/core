//! Unit tests for isolated definition elements
//!
//! Tests definition parsing in isolation following the on-lexplore.lex guidelines:
//! - Use Lexplore to load centralized test files
//! - Use assert_ast for deep structure verification
//! - Test isolated elements (one element per test)
//! - Verify content and structure, not just counts

use lex::lex::testing::assert_ast;
use lex::lex::testing::lexplore::Lexplore;

#[test]
fn test_definition_01_flat_simple() {
    // definition-01-flat-simple.lex: Definition with single paragraph content
    let doc = Lexplore::definition(1).parse();

    // Document: Definition + trailing paragraph "Something to finish the element"
    assert_ast(&doc)
        .item_count(2)
        .item(0, |item| {
            item.assert_definition()
                .subject("Cache")
                .child_count(1)
                .child(0, |child| {
                    child
                        .assert_paragraph()
                        .text_contains("Temporary storage for frequently accessed data");
                });
        })
        .item(1, |item| {
            item.assert_paragraph()
                .text_contains("Something to finish the element");
        });
}

#[test]
fn test_definition_02_flat_multi_paragraph() {
    // definition-02-flat-multi-paragraph.lex: Definition with multiple paragraphs
    let doc = Lexplore::definition(2).parse();

    assert_ast(&doc)
        .item_count(2)
        .item(0, |item| {
            item.assert_definition()
                .subject("Microservice")
                .child_count(2) // Two paragraphs in definition
                .child(0, |child| {
                    child
                        .assert_paragraph()
                        .text_contains("architectural style");
                })
                .child(1, |child| {
                    child
                        .assert_paragraph()
                        .text_contains("independently deployable");
                });
        })
        .item(1, |item| {
            item.assert_paragraph()
                .text_contains("Something to finish the element");
        });
}

#[test]
fn test_definition_03_flat_with_list() {
    // definition-03-flat-with-list.lex: Due to blank line after "HTTP Methods:",
    // parser treats this as an unnumbered Session, not a Definition.
    // Definitions require content immediately after the colon (no blank line).
    let doc = Lexplore::definition(3).parse();

    assert_ast(&doc)
        .item_count(2)
        .item(0, |item| {
            // Parsed as Session because of blank line after colon
            item.assert_session()
                .label("HTTP Methods:")
                .child_count(1)
                .child(0, |child| {
                    child
                        .assert_list()
                        .item_count(4)
                        .item(0, |list_item| {
                            list_item.text_contains("GET: Retrieve resources");
                        })
                        .item(1, |list_item| {
                            list_item.text_contains("POST: Create resources");
                        })
                        .item(2, |list_item| {
                            list_item.text_contains("PUT: Update resources");
                        })
                        .item(3, |list_item| {
                            list_item.text_contains("DELETE: Remove resources");
                        });
                });
        })
        .item(1, |item| {
            item.assert_paragraph()
                .text_contains("Something to finish the element");
        });
}

#[test]
fn test_definition_05_nested_with_list() {
    // definition-05-nested-with-list.lex: Definition with paragraphs + list content
    let doc = Lexplore::definition(5).parse();

    assert_ast(&doc).item_count(1).item(0, |item| {
        item.assert_definition()
            .subject("Programming Concepts")
            .child_count(3)
            .child(0, |child| {
                child
                    .assert_paragraph()
                    .text_contains("fundamental ideas in programming");
            })
            .child(1, |child| {
                child
                    .assert_list()
                    .item_count(3)
                    .item(0, |li| {
                        li.text_contains("Variables");
                    })
                    .item(1, |li| {
                        li.text_contains("Functions");
                    })
                    .item(2, |li| {
                        li.text_contains("Loops");
                    });
            })
            .child(2, |child| {
                child
                    .assert_paragraph()
                    .text_contains("core building blocks");
            });
    });
}

#[test]
#[ignore = "Line parser still drops nested definitions; verified via lex-to-treeviz"]
fn test_definition_06_nested_definitions() {
    // definition-06-nested-definitions.lex: Nested definition hierarchy (Authentication -> OAuth -> OAuth 2.0)
    let doc = Lexplore::definition(6).parse();

    assert_ast(&doc).item_count(1).item(0, |item| {
        item.assert_definition()
            .subject("Authentication")
            .child(0, |child| {
                child.assert_paragraph().text_contains("verifying identity");
            })
            .child(1, |child| {
                child
                    .assert_definition()
                    .subject("OAuth")
                    .child(0, |grandchild| {
                        grandchild
                            .assert_paragraph()
                            .text_contains("access delegation");
                    })
                    .child(1, |grandchild| {
                        grandchild
                            .assert_definition()
                            .subject("OAuth 2.0")
                            .child(0, |leaf| {
                                leaf.assert_paragraph().text_contains("current version");
                            });
                    });
            })
            .child(2, |child| {
                child.assert_definition().subject("JWT").child(0, |leaf| {
                    leaf.assert_paragraph().text_contains("JSON Web Tokens");
                });
            });
    });
}

#[test]
fn test_definition_07_nested_deep_only() {
    // definition-07-nested-deep-only.lex: Deeply nested definitions chain
    let doc = Lexplore::definition(7).parse();

    assert_ast(&doc).item_count(1).item(0, |item| {
        item.assert_definition()
            .subject("Computer Science")
            .child(0, |child| {
                child
                    .assert_paragraph()
                    .text_contains("computation and information");
            })
            .child(1, |child| {
                child
                    .assert_definition()
                    .subject("Algorithms")
                    .child(0, |grandchild| {
                        grandchild
                            .assert_paragraph()
                            .text_contains("Step-by-step procedures");
                    })
                    .child(1, |grandchild| {
                        grandchild
                            .assert_definition()
                            .subject("Sorting")
                            .child(0, |leaf| {
                                leaf.assert_paragraph().text_contains("Organizing data");
                            })
                            .child(1, |leaf| {
                                leaf.assert_definition()
                                    .subject("QuickSort")
                                    .child(0, |detail| {
                                        detail
                                            .assert_paragraph()
                                            .text_contains("divide-and-conquer");
                                    });
                            });
                    });
            });
    });
}

#[test]
fn test_definition_90_document_simple() {
    let doc = Lexplore::definition(90).parse();

    assert!(!doc.root.children.is_empty(), "definition-90 should parse");
}

#[test]
#[ignore]
fn test_definitions_overview_document() {
    // definitions.lex: Specification overview covering syntax/disambiguation
    let doc = Lexplore::from_path("docs/specs/v1/elements/definition/definitions.lex").parse();

    assert_ast(&doc)
        .item_count(4)
        .item(0, |item| {
            item.assert_paragraph().text("Definitions");
        })
        .item(1, |item| {
            item.assert_session()
                .label("Introduction")
                .child(0, |child| {
                    child
                        .assert_paragraph()
                        .text_contains("core element for explaining terms");
                });
        })
        .item(2, |item| {
            item.assert_session()
                .label("Syntax")
                .child(0, |child| {
                    child
                        .assert_definition()
                        .subject("Subject")
                        .child(0, |leaf| {
                            leaf.assert_paragraph().text_contains("Content here");
                        });
                })
                .child(1, |child| {
                    child.assert_paragraph().text_contains("Key rule");
                });
        })
        .item(3, |item| {
            item.assert_paragraph().text("Disambiguation from Sessions");
        });
}
