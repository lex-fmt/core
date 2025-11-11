//! Unit tests for isolated list elements
//!
//! Tests list parsing in isolation following the on-lexplore.lex guidelines:
//! - Use Lexplore to load centralized test files
//! - Use assert_ast for deep structure verification
//! - Test isolated elements (one element per test)
//! - Verify content and structure, not just counts

use lex::lex::pipeline::Parser;
use lex::lex::testing::assert_ast;
use lex::lex::testing::lexplore::Lexplore;
use rstest::rstest;

#[rstest(parser => [Parser::Reference, Parser::Linebased])]
fn test_list_01_flat_simple_dash(parser: Parser) {
    // list-01-flat-simple-dash.lex: Paragraph "Test:" followed by dash list
    let doc = Lexplore::list(1).parse_with(parser);

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

#[rstest(parser => [Parser::Reference, Parser::Linebased])]
fn test_list_02_flat_numbered(parser: Parser) {
    // list-02-flat-numbered.lex: Paragraph + numbered list
    let doc = Lexplore::list(2).parse_with(parser);

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

#[rstest(parser => [Parser::Reference, Parser::Linebased])]
fn test_list_07_nested_simple(parser: Parser) {
    // list-07-nested-simple.lex: Paragraph + two-level nested list
    // Tests nesting structure: outer list items with nested lists
    let doc = Lexplore::list(7).parse_with(parser);

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

#[rstest(parser => [Parser::Reference, Parser::Linebased])]
fn test_list_03_flat_alphabetical(parser: Parser) {
    let doc = Lexplore::list(3).parse_with(parser);

    assert_ast(&doc)
        .item_count(2)
        .item(0, |item| {
            item.assert_paragraph().text("Test:");
        })
        .item(1, |item| {
            item.assert_list()
                .item_count(3)
                .item(0, |list_item| {
                    list_item.text_contains("First letter item");
                })
                .item(1, |list_item| {
                    list_item.text_contains("Second letter item");
                })
                .item(2, |list_item| {
                    list_item.text_contains("Third letter item");
                });
        });
}

#[rstest(parser => [Parser::Reference, Parser::Linebased])]
fn test_list_04_flat_mixed_markers(parser: Parser) {
    let doc = Lexplore::list(4).parse_with(parser);

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

#[rstest(parser => [Parser::Reference, Parser::Linebased])]
fn test_list_05_flat_parenthetical(parser: Parser) {
    let doc = Lexplore::list(5).parse_with(parser);

    assert_ast(&doc)
        .item_count(2)
        .item(0, |item| {
            item.assert_paragraph().text("Test:");
        })
        .item(1, |item| {
            item.assert_list()
                .item_count(3)
                .item(0, |list_item| {
                    list_item.text_contains("First parenthetical item");
                })
                .item(1, |list_item| {
                    list_item.text_contains("Second parenthetical item");
                })
                .item(2, |list_item| {
                    list_item.text_contains("Third parenthetical item");
                });
        });
}

#[rstest(parser => [Parser::Reference, Parser::Linebased])]
fn test_list_06_flat_roman_numerals(parser: Parser) {
    let doc = Lexplore::list(6).parse_with(parser);

    assert_ast(&doc)
        .item_count(2)
        .item(0, |item| {
            item.assert_paragraph().text("Test:");
        })
        .item(1, |item| {
            item.assert_list()
                .item_count(3)
                .item(0, |list_item| {
                    list_item.text_contains("First roman item");
                })
                .item(1, |list_item| {
                    list_item.text_contains("Second roman item");
                })
                .item(2, |list_item| {
                    list_item.text_contains("Third roman item");
                });
        });
}

#[rstest(parser => [Parser::Reference, Parser::Linebased])]
fn test_list_08_nested_with_paragraph(parser: Parser) {
    let doc = Lexplore::list(8).parse_with(parser);

    assert_ast(&doc)
        .item_count(2)
        .item(0, |item| {
            item.assert_paragraph().text("Test:");
        })
        .item(1, |item| {
            item.assert_list()
                .item_count(2)
                .item(0, |list_item| {
                    list_item
                        .text_contains("First item with nested content")
                        .child_count(3)
                        .child(0, |child| {
                            child
                                .assert_paragraph()
                                .text_contains("paragraph nested inside the list item");
                        })
                        .child(1, |child| {
                            child
                                .assert_list()
                                .item_count(2)
                                .item(0, |nested| {
                                    nested.text_contains("Nested list item one");
                                })
                                .item(1, |nested| {
                                    nested.text_contains("Nested list item two");
                                });
                        })
                        .child(2, |child| {
                            child
                                .assert_paragraph()
                                .text_contains("Another paragraph after the nested list");
                        });
                })
                .item(1, |list_item| {
                    list_item
                        .text_contains("Second item")
                        .child_count(1)
                        .child(0, |child| {
                            child.assert_paragraph().text_contains("Final paragraph.");
                        });
                });
        });
}

#[rstest(parser => [Parser::Reference, Parser::Linebased])]
fn test_list_09_nested_three_levels(parser: Parser) {
    let doc = Lexplore::list(9).parse_with(parser);

    assert_ast(&doc)
        .item_count(2)
        .item(0, |item| {
            item.assert_paragraph().text("Test:");
        })
        .item(1, |item| {
            item.assert_list()
                .item_count(2)
                .item(0, |outer| {
                    outer
                        .text_contains("Outer level one")
                        .child_count(1)
                        .child(0, |child| {
                            child
                                .assert_list()
                                .item_count(2)
                                .item(0, |middle| {
                                    middle
                                        .text_contains("Middle level one")
                                        .child_count(1)
                                        .child(0, |inner_list| {
                                            inner_list.assert_list().item_count(2);
                                        });
                                })
                                .item(1, |middle| {
                                    middle.text_contains("Middle level two");
                                });
                        });
                })
                .item(1, |outer| {
                    outer.text_contains("Outer level two");
                });
        });
}

#[rstest(parser => [Parser::Reference, Parser::Linebased])]
#[ignore]
fn test_list_10_nested_deep_only(parser: Parser) {
    let doc = Lexplore::list(10).parse_with(parser);

    assert_ast(&doc)
        .item_count(2)
        .item(0, |item| {
            item.assert_paragraph().text("Test:");
        })
        .item(1, |item| {
            item.assert_list().item_count(1).item(0, |outer| {
                outer
                    .text_contains("Level one")
                    .child_count(1)
                    .child(0, |child| {
                        child.assert_list().item_count(1).item(0, |level_two| {
                            level_two.text_contains("Level two").child_count(1).child(
                                0,
                                |level_three| {
                                    level_three
                                        .assert_list()
                                        .item_count(1)
                                        .item(0, |level_four| {
                                            level_four
                                                .text_contains("Level three")
                                                .child_count(1)
                                                .child(0, |deep| {
                                                    deep.assert_list().item_count(1).item(
                                                        0,
                                                        |leaf| {
                                                            leaf.text_contains("Level four");
                                                        },
                                                    );
                                                });
                                        });
                                },
                            );
                        });
                    });
            });
        });
}

#[rstest(parser => [Parser::Reference, Parser::Linebased])]
#[ignore]
fn test_list_11_nested_balanced(parser: Parser) {
    let doc = Lexplore::list(11).parse_with(parser);

    assert_ast(&doc).item_count(1).item(0, |item| {
        item.assert_list()
            .item_count(2)
            .item(0, |outer| {
                outer
                    .text_contains("item 1")
                    .child_count(1)
                    .child(0, |child| {
                        child.assert_list().item_count(2);
                    });
            })
            .item(1, |outer| {
                outer
                    .text_contains("item 2")
                    .child_count(1)
                    .child(0, |child| {
                        child.assert_list().item_count(1);
                    });
            });
    });
}

#[rstest(parser => [Parser::Reference, Parser::Linebased])]
#[ignore = "Reference parser rejects nested sessions inside list items"]
fn test_list_12_nested_three_full_form(parser: Parser) {
    let doc = Lexplore::list(12).parse_with(parser);

    assert_ast(&doc)
        .item_count(2)
        .item(0, |item| {
            item.assert_paragraph().text("Test:");
        })
        .item(1, |item| {
            item.assert_list().item_count(2);
        });
}

#[rstest(parser => [Parser::Reference, Parser::Linebased])]
fn test_lists_overview_document(parser: Parser) {
    let doc = Lexplore::from_path("docs/specs/v1/elements/list/lists.lex").parse_with(parser);

    assert_ast(&doc)
        .item(0, |item| {
            item.assert_paragraph().text("Lists");
        })
        .item(1, |item| {
            item.assert_session()
                .label("Introduction")
                .child(0, |child| {
                    child
                        .assert_paragraph()
                        .text_contains("organize related items in sequence");
                });
        })
        .item(2, |item| {
            item.assert_session().label("Syntax");
        });
}
