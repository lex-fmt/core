//! Unit tests for isolated annotation elements
//!
//! Tests annotation parsing in isolation following the on-lexplore.lex guidelines:
//! - Use Lexplore to load centralized test files
//! - Use assert_ast for deep structure verification
//! - Test isolated elements (one element per test)
//! - Verify content and structure, not just counts

use lex_parser::lex::pipeline::Parser;
use lex_parser::lex::testing::assert_ast;
use lex_parser::lex::testing::lexplore::Lexplore;
use lex_parser::lex::testing::workspace_path;
use rstest::rstest;

#[rstest(parser => [Parser::Linebased])]
fn test_annotation_01_flat_marker_simple(parser: Parser) {
    // annotation-01-flat-marker-simple.lex: Simple marker annotation ":: note ::"
    let doc = Lexplore::annotation(1).parse_with(parser);

    assert_ast(&doc).item_count(1).item(0, |item| {
        item.assert_annotation().label("note");
    });
}

#[rstest(parser => [Parser::Linebased])]
fn test_annotation_02_flat_marker_with_params(parser: Parser) {
    // annotation-02-flat-marker-with-params.lex: Marker with parameter "severity=high"
    let doc = Lexplore::annotation(2).parse_with(parser);

    assert_ast(&doc).item_count(1).item(0, |item| {
        item.assert_annotation()
            .label("warning")
            .parameter_count(1)
            .parameter(0, "severity", "high");
    });
}

#[rstest(parser => [Parser::Linebased])]
fn test_annotation_03_flat_inline_text(parser: Parser) {
    // annotation-03-flat-inline-text.lex: Single-line annotation with inline text
    let doc = Lexplore::annotation(3).parse_with(parser);

    assert_ast(&doc).item_count(1).item(0, |item| {
        item.assert_annotation()
            .label("note")
            .child_count(1)
            .child(0, |child| {
                child
                    .assert_paragraph()
                    .text_contains("Important information");
            });
    });
}

#[rstest(parser => [Parser::Linebased])]
fn test_annotation_04_flat_inline_with_params(parser: Parser) {
    // annotation-04-flat-inline-with-params.lex: Single-line annotation with params and inline text
    let doc = Lexplore::annotation(4).parse_with(parser);

    assert_ast(&doc).item_count(1).item(0, |item| {
        item.assert_annotation()
            .label("warning")
            .parameter_count(1)
            .parameter(0, "severity", "high")
            .child_count(1)
            .child(0, |child| {
                child
                    .assert_paragraph()
                    .text_contains("Check this carefully");
            });
    });
}

#[rstest(parser => [Parser::Linebased])]
fn test_annotation_05_flat_block_paragraph(parser: Parser) {
    // annotation-05-flat-block-paragraph.lex: Block annotation with paragraph content
    let doc = Lexplore::annotation(5).parse_with(parser);

    assert_ast(&doc).item_count(1).item(0, |item| {
        item.assert_annotation()
            .label("note")
            .child_count(1)
            .child(0, |child| {
                child
                    .assert_paragraph()
                    .text_contains("important note that requires a detailed explanation");
            });
    });
}

#[rstest(parser => [Parser::Linebased])]
fn test_annotation_06_flat_block_multi_paragraph(parser: Parser) {
    // annotation-06-flat-block-multi-paragraph.lex: Block annotation spanning two paragraphs
    let doc = Lexplore::annotation(6).parse_with(parser);

    assert_ast(&doc).item_count(1).item(0, |item| {
        item.assert_annotation()
            .label("note")
            .parameter_count(1)
            .parameter(0, "author", "\"Jane Doe\"")
            .child_count(2)
            .child(0, |child| {
                child
                    .assert_paragraph()
                    .text_contains("important note that requires a detailed explanation");
            })
            .child(1, |child| {
                child
                    .assert_paragraph()
                    .text_contains("span multiple paragraphs");
            });
    });
}

#[rstest(parser => [Parser::Linebased])]
fn test_annotation_07_flat_block_with_list(parser: Parser) {
    // annotation-07-flat-block-with-list.lex: Block annotation mixing paragraph and list content
    let doc = Lexplore::annotation(7).parse_with(parser);

    assert_ast(&doc).item_count(1).item(0, |item| {
        item.assert_annotation()
            .label("warning")
            .parameter_count(1)
            .parameter(0, "severity", "critical")
            .child_count(2)
            .child(0, |child| {
                child
                    .assert_paragraph()
                    .text_contains("The following items must be addressed before deployment");
            })
            .child(1, |child| {
                child
                    .assert_list()
                    .item_count(3)
                    .item(0, |li| {
                        li.text_contains("Security vulnerabilities");
                    })
                    .item(1, |li| {
                        li.text_contains("Performance issues");
                    })
                    .item(2, |li| {
                        li.text_contains("Documentation gaps");
                    });
            });
    });
}

#[rstest(parser => [Parser::Linebased])]
fn test_annotation_08_nested_with_list_and_paragraph(parser: Parser) {
    // annotation-08-nested-with-list-and-paragraph.lex: Paragraph + list + paragraph inside annotation
    let doc = Lexplore::annotation(8).parse_with(parser);

    assert_ast(&doc).item_count(1).item(0, |item| {
        item.assert_annotation()
            .label("note")
            .parameter_count(1)
            .parameter(0, "author", "\"Jane\"")
            .child_count(3)
            .child(0, |child| {
                child
                    .assert_paragraph()
                    .text_contains("multiple types of content");
            })
            .child(1, |child| {
                child
                    .assert_list()
                    .item_count(3)
                    .item(0, |li| {
                        li.text_contains("First item");
                    })
                    .item(1, |li| {
                        li.text_contains("Second item");
                    })
                    .item(2, |li| {
                        li.text_contains("Third item");
                    });
            })
            .child(2, |child| {
                child
                    .assert_paragraph()
                    .text_contains("A paragraph after the list");
            });
    });
}

#[rstest(parser => [Parser::Linebased])]
fn test_annotation_09_nested_definition_inside(parser: Parser) {
    // annotation-09-nested-definition-inside.lex: Definition entries inside annotation block
    let doc = Lexplore::annotation(9).parse_with(parser);

    assert_ast(&doc).item_count(1).item(0, |item| {
        item.assert_annotation()
            .label("documentation")
            .child_count(4)
            .child(0, |child| {
                child
                    .assert_paragraph()
                    .text_contains("documents some terms");
            })
            .child(1, |child| {
                child
                    .assert_definition()
                    .subject("API")
                    .child(0, |def_para| {
                        def_para
                            .assert_paragraph()
                            .text_contains("Application Programming Interface");
                    });
            })
            .child(2, |child| {
                child
                    .assert_definition()
                    .subject("REST")
                    .child(0, |def_para| {
                        def_para
                            .assert_paragraph()
                            .text_contains("Representational State Transfer");
                    });
            })
            .child(3, |child| {
                child.assert_paragraph().text_contains("Final notes");
            });
    });
}

#[rstest(parser => [Parser::Linebased])]
fn test_annotation_10_nested_complex(parser: Parser) {
    // annotation-10-nested-complex.lex: Mixed paragraphs, nested lists, and parameters
    let doc = Lexplore::annotation(10).parse_with(parser);

    assert_ast(&doc).item_count(1).item(0, |item| {
        item.assert_annotation()
            .label("review")
            .parameter_count(1)
            .parameter(0, "status", "pending")
            .child_count(4)
            .child(0, |child| {
                child
                    .assert_paragraph()
                    .text_contains("Review findings and recommendations");
            })
            .child(1, |child| {
                child.assert_paragraph().text("Issues Found:");
            })
            .child(2, |child| {
                child
                    .assert_list()
                    .item_count(2)
                    .item(0, |li| {
                        li.text_contains("Performance bottleneck")
                            .child_count(2)
                            .child(0, |para| {
                                para.assert_paragraph()
                                    .text_contains("needs immediate attention");
                            })
                            .child(1, |nested| {
                                nested
                                    .assert_list()
                                    .item_count(2)
                                    .item(0, |inner| {
                                        inner.text_contains("Memory leak in handler");
                                    })
                                    .item(1, |inner| {
                                        inner.text_contains("Slow database queries");
                                    });
                            });
                    })
                    .item(1, |li| {
                        li.text_contains("Security concerns")
                            .child_count(1)
                            .child(0, |para| {
                                para.assert_paragraph().text_contains("Review required");
                            });
                    });
            })
            .child(3, |child| {
                child
                    .assert_paragraph()
                    .text_contains("Conclusion paragraph");
            });
    });
}

#[rstest(parser => [Parser::Linebased])]
#[ignore] // TODO: Complex document - line parser still has issues with session titles ending in colons
          // This is tested more simply in test_session_09_flat_colon_title
fn test_annotations_overview_document(parser: Parser) {
    // annotations.lex: Specification overview document for annotations
    let doc = Lexplore::from_path(workspace_path(
        "docs/specs/v1/elements/annotation/annotations.lex",
    ))
    .parse_with(parser);

    assert_ast(&doc)
        .item(0, |item| {
            item.assert_paragraph().text("Annotations ");
        })
        .item(1, |item| {
            item.assert_session()
                .label("Introduction")
                .child(0, |child| {
                    child
                        .assert_paragraph()
                        .text_contains("Annotations are a core element");
                })
                .child(1, |child| {
                    child.assert_paragraph().text_contains("provide labels");
                })
                .child(2, |child| {
                    child.assert_paragraph().text("Core features:");
                })
                .child(3, |child| {
                    child.assert_list().item_count(3);
                });
        })
        .item(2, |item| {
            item.assert_paragraph().text("Syntax Forms:");
        });
}
