//! Unit tests for isolated verbatim elements using linebased parser
//!
//! Tests verbatim block parsing in isolation following the on-lexplore.lex guidelines:
//! - Use Lexplore to load centralized test files
//! - Use assert_ast for deep structure verification
//! - Test isolated elements (one element per test)
//! - Verify content and structure, not just counts
//!
//! ## Known Issues
//!
//! - test_verbatim_05_flat_special_chars: Verbatim content with :: markers is not handled correctly
//!   (This is a content parsing issue, not an annotation parsing issue)

use lex::lex::pipeline::DocumentLoader;
use lex::lex::testing::assert_ast;
use lex::lex::testing::lexplore::{Lexplore, Parser};

#[test]
fn test_verbatim_01_flat_simple_code() {
    // verbatim-01-flat-simple-code.lex: Verbatim block with simple code
    let source = Lexplore::verbatim(1).source();
    let doc = DocumentLoader::new()
        .parse_with(&source, Parser::Linebased)
        .unwrap();

    assert_ast(&doc).item_count(1).item(0, |item| {
        item.assert_verbatim_block()
            .subject("Code Example")
            .closing_label("javascript")
            .content_contains("function hello()");
    });
}

#[test]
fn test_verbatim_02_flat_with_caption() {
    // verbatim-02-flat-with-caption.lex: Verbatim block with caption in closing annotation
    let source = Lexplore::verbatim(2).source();
    let doc = DocumentLoader::new()
        .parse_with(&source, Parser::Linebased)
        .unwrap();

    assert_ast(&doc).item_count(1).item(0, |item| {
        item.assert_verbatim_block()
            .subject("API Response")
            .closing_label("json")
            .content_contains("{\"status\": \"ok\"");
    });
}

#[test]
fn test_verbatim_03_flat_with_params() {
    // verbatim-03-flat-with-params.lex: Verbatim with parameters in closing annotation
    let source = Lexplore::verbatim(3).source();
    let doc = DocumentLoader::new()
        .parse_with(&source, Parser::Linebased)
        .unwrap();

    assert_ast(&doc).item_count(1).item(0, |item| {
        item.assert_verbatim_block()
            .subject("Configuration")
            .closing_label("nginx")
            .has_closing_parameter_with_value("format", "pretty")
            .content_contains("server {");
    });
}

#[test]
fn test_verbatim_04_flat_marker_form() {
    // verbatim-04-flat-marker-form.lex: Single-line marker-style verbatim
    // Note: In marker form, content after closing :: is part of the annotation, not verbatim content
    let source = Lexplore::verbatim(4).source();
    let doc = DocumentLoader::new()
        .parse_with(&source, Parser::Linebased)
        .unwrap();

    assert_ast(&doc).item_count(1).item(0, |item| {
        item.assert_verbatim_block()
            .subject("Sunset Photo")
            .closing_label("image")
            .has_closing_parameter_with_value("src", "sunset.jpg")
            .line_count(0); // Marker form has no content lines
    });
}

#[test]
#[ignore] // BUG: Linebased parser annotation labels are empty
#[should_panic(expected = "Expected VerbatimBlock, found Paragraph")]
fn test_verbatim_05_flat_special_chars() {
    // verbatim-05-flat-special-chars.lex: Verbatim with :: markers in content
    // BUG: Reference parser incorrectly parses this as Paragraph instead of VerbatimBlock
    // The :: markers in content confuse the reference parser
    let source = Lexplore::verbatim(5).source();
    let doc = DocumentLoader::new()
        .parse_with(&source, Parser::Linebased)
        .unwrap();

    assert_ast(&doc).item_count(1).item(0, |item| {
        item.assert_verbatim_block()
            .subject("Special Characters")
            .closing_label("javascript")
            .content_contains("// This content has :: markers")
            .content_contains("return \"::\"");
    });
}

#[test]
fn test_verbatim_06_nested_in_definition() {
    // verbatim-06-nested-in-definition.lex: Verbatim nested inside a definition
    let source = Lexplore::verbatim(6).source();
    let doc = DocumentLoader::new()
        .parse_with(&source, Parser::Linebased)
        .unwrap();

    assert_ast(&doc).item_count(1).item(0, |item| {
        item.assert_definition()
            .subject("JavaScript Example")
            .child_count(3); // para, verbatim, para
    });

    // Verify first paragraph
    assert_ast(&doc).item(0, |item| {
        item.assert_definition().child(0, |para| {
            para.assert_paragraph()
                .text_contains("demonstrates closure");
        });
    });

    // Verify verbatim block
    assert_ast(&doc).item(0, |item| {
        item.assert_definition().child(1, |verbatim| {
            verbatim
                .assert_verbatim_block()
                .subject("Implementation")
                .closing_label("javascript")
                .content_contains("function counter()")
                .line_count(3); // function counter() {, }, and blank line
        });
    });

    // Verify second paragraph
    assert_ast(&doc).item(0, |item| {
        item.assert_definition().child(2, |para| {
            para.assert_paragraph()
                .text_contains("simple closure pattern");
        });
    });
}

#[test]
fn test_verbatim_07_nested_in_list() {
    // verbatim-07-nested-in-list.lex: Verbatim blocks nested in list items
    let source = Lexplore::verbatim(7).source();
    let doc = DocumentLoader::new()
        .parse_with(&source, Parser::Linebased)
        .unwrap();

    assert_ast(&doc).item_count(2); // para, list

    // Verify title paragraph
    assert_ast(&doc).item(0, |item| {
        item.assert_paragraph().text_contains("Code examples");
    });

    // Verify list with 2 items
    assert_ast(&doc).item(1, |item| {
        item.assert_list().item_count(2);
    });

    // Verify first list item with Python verbatim
    assert_ast(&doc).item(1, |item| {
        item.assert_list().item(0, |list_item| {
            list_item
                .text_contains("Python example")
                .child_count(1) // verbatim
                .child(0, |verbatim| {
                    verbatim
                        .assert_verbatim_block()
                        .subject("Simple function")
                        .closing_label("python")
                        .content_contains("def hello():")
                        .line_count(1); // def hello():
                });
        });
    });

    // Verify second list item with JavaScript verbatim
    assert_ast(&doc).item(1, |item| {
        item.assert_list().item(1, |list_item| {
            list_item
                .text_contains("JavaScript example")
                .child_count(1) // verbatim
                .child(0, |verbatim| {
                    verbatim
                        .assert_verbatim_block()
                        .subject("Another function")
                        .closing_label("javascript")
                        .content_contains("const greet")
                        .line_count(2); // const greet = () => "hello"; and blank line
                });
        });
    });
}

#[test]
fn test_verbatim_08_nested_deep() {
    // verbatim-08-nested-deep.lex: Deeply nested verbatim (3 levels)
    let source = Lexplore::verbatim(8).source();
    let doc = DocumentLoader::new()
        .parse_with(&source, Parser::Linebased)
        .unwrap();

    assert_ast(&doc).item_count(1).item(0, |item| {
        item.assert_definition()
            .subject("Programming Languages")
            .child_count(2); // para, definition
    });

    // Verify paragraph at level 1
    assert_ast(&doc).item(0, |item| {
        item.assert_definition().child(0, |para| {
            para.assert_paragraph()
                .text_contains("Overview of different languages");
        });
    });

    // Verify nested definition at level 2
    assert_ast(&doc).item(0, |item| {
        item.assert_definition().child(1, |def2| {
            def2.assert_definition()
                .subject("Scripting Languages")
                .child_count(2); // para, definition
        });
    });

    // Verify paragraph at level 2
    assert_ast(&doc).item(0, |item| {
        item.assert_definition().child(1, |def2| {
            def2.assert_definition().child(0, |para| {
                para.assert_paragraph()
                    .text_contains("Languages for automation");
            });
        });
    });

    // Verify nested definition at level 3
    assert_ast(&doc).item(0, |item| {
        item.assert_definition().child(1, |def2| {
            def2.assert_definition().child(1, |def3| {
                def3.assert_definition().subject("Python").child_count(1); // verbatim
            });
        });
    });

    // Verify verbatim at level 3
    assert_ast(&doc).item(0, |item| {
        item.assert_definition().child(1, |def2| {
            def2.assert_definition().child(1, |def3| {
                def3.assert_definition().child(0, |verbatim| {
                    verbatim
                        .assert_verbatim_block()
                        .subject("Example code")
                        .closing_label("python")
                        .content_contains("#!/usr/bin/env python3")
                        .content_contains("print(\"Hello, World!\")");
                });
            });
        });
    });
}

#[test]
fn test_verbatim_09_flat_simple_beyond_wall() {
    // verbatim-09-flat-simple-beyong-wall.lex: Verbatim with content beyond indentation wall
    // Content beyond the wall gets its indentation normalized
    let source = Lexplore::verbatim(9).source();
    let doc = DocumentLoader::new()
        .parse_with(&source, Parser::Linebased)
        .unwrap();

    assert_ast(&doc).item_count(1).item(0, |item| {
        item.assert_verbatim_block()
            .subject("Code Example")
            .closing_label("javascript")
            .content_contains("function hello() {")
            .line_count(1); // Only the function declaration line at the wall level
    });
}

#[test]
fn test_verbatim_10_flat_simple_empty() {
    // verbatim-10-flat-simple-empty.lex: Empty verbatim block
    let source = Lexplore::verbatim(10).source();
    let doc = DocumentLoader::new()
        .parse_with(&source, Parser::Linebased)
        .unwrap();

    assert_ast(&doc).item_count(1).item(0, |item| {
        item.assert_verbatim_block()
            .subject("Code Example")
            .closing_label("javascript")
            .line_count(0); // No content lines
    });
}
