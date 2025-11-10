//! Unit tests for isolated list elements
//!
//! Tests list parsing in isolation following the on-lexplore.lex guidelines:
//! - Use Lexplore to load centralized test files
//! - Use assert_ast for deep structure verification
//! - Test isolated elements (one element per test)
//! - Verify content and structure, not just counts

use lex::lex::testing::assert_ast;
use lex::lex::testing::lexplore::Lexplore;

#[test]
fn test_list_01_flat_simple_dash() {
    // list-01-flat-simple-dash.lex: Paragraph "Test:" followed by dash list
    let doc = Lexplore::list(1).parse();

    // Document has: Paragraph + List
    assert_ast(&doc)
        .item_count(2)
        .item(0, |item| {
            item.assert_paragraph().text("Test:");
        })
        .item(1, |item| {
            item.assert_list()
                .item_count(3)
                .item(0, |list_item| {
                    list_item.text_contains("First item");
                })
                .item(1, |list_item| {
                    list_item.text_contains("Second item");
                })
                .item(2, |list_item| {
                    list_item.text_contains("Third item");
                });
        });
}

#[test]
fn test_list_02_flat_numbered() {
    // list-02-flat-numbered.lex: Paragraph + numbered list
    let doc = Lexplore::list(2).parse();

    assert_ast(&doc)
        .item_count(2)
        .item(0, |item| {
            item.assert_paragraph().text("Test:");
        })
        .item(1, |item| {
            item.assert_list()
                .item_count(3)
                .item(0, |list_item| {
                    list_item.text_contains("First numbered item");
                })
                .item(1, |list_item| {
                    list_item.text_contains("Second numbered item");
                })
                .item(2, |list_item| {
                    list_item.text_contains("Third numbered item");
                });
        });
}

#[test]
fn test_list_07_nested_simple() {
    // list-07-nested-simple.lex: Paragraph + two-level nested list
    // Tests nesting structure: outer list items with nested lists
    let doc = Lexplore::list(7).parse();

    assert_ast(&doc)
        .item_count(2)
        .item(0, |item| {
            item.assert_paragraph().text("Test:");
        })
        .item(1, |item| {
            item.assert_list()
                .item_count(2) // Two outer items
                .item(0, |list_item| {
                    // First outer item: has nested list with 2 items
                    list_item
                        .text_contains("First outer item")
                        .child_count(1)
                        .child(0, |nested| {
                            nested
                                .assert_list()
                                .item_count(2)
                                .item(0, |inner| {
                                    inner.text_contains("First nested item");
                                })
                                .item(1, |inner| {
                                    inner.text_contains("Second nested item");
                                });
                        });
                })
                .item(1, |list_item| {
                    // Second outer item: blank line causes the nested list
                    // to be parsed as a paragraph containing the list marker text
                    list_item
                        .text_contains("Second outer item")
                        .child_count(1)
                        .child(0, |para| {
                            // The nested list is parsed as a paragraph due to blank line
                            para.assert_paragraph()
                                .text_contains("- Another nested item");
                        });
                });
        });
}
