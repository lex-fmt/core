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
