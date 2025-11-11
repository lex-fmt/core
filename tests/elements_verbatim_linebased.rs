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
                .content_contains("let count = 0;")
                .content_contains("return () => ++count;")
                .line_count(5); // function counter() {, let count = 0;, return, }, and blank line
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
                        .content_contains("return \"world\"")
                        .line_count(3); // def hello():, return "world", and blank line
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
            .content_contains("return \"world\";")
            .content_contains("}")
            .line_count(4); // function hello(), return, closing brace, and blank line
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

// Note: verbatim-11-group-shell.lex is tested as part of the document-level tests.
// The linebased parser should handle verbatim groups the same way as the reference parser.
// If verbatim groups work in the reference parser, they should work in the linebased parser too.

#[test]
fn test_verbatim_13_group_spades() {
    // verbatim-13-group-spades.lex: Verbatim group with multiple pairs and blank lines
    let source = Lexplore::verbatim(13).source();
    let doc = DocumentLoader::new()
        .parse_with(&source, Parser::Linebased)
        .unwrap();

    assert_ast(&doc).item_count(2).item(0, |item| {
        item.assert_verbatim_block()
            .subject("This is a groupped Verbatim Block, this is the first Group:")
            .closing_label("shell")
            .group_count(4)
            .group(0, |group| {
                group
                    .subject("This is a groupped Verbatim Block, this is the first Group:")
                    .content_contains("$ pwd # always te staring point");
            })
            .group(1, |group| {
                group
                    .subject("Now that you know where you are, lets find out what's around you:")
                    .content_contains("$ ls")
                    .content_contains("$ ls -r # recursive");
            })
            .group(2, |group| {
                group
                    .subject("And let's go places:")
                    .content_contains("$ cd <path to go>");
            })
            .group(3, |group| {
                group
                    .subject("Feeling lost, let's get back home:")
                    .content_contains("$ cd ~");
            });
    });

    // Verify paragraph after the verbatim group
    assert_ast(&doc).item(1, |item| {
        item.assert_paragraph()
            .text_contains(
                "Note that verbatim blocks conetents can have any number of blank lines, including None."
            );
    });
}

#[test]
#[ignore] // TODO: Parser issue - verbatim groups within sessions are not correctly parsed.
          // See test_verbatim_12_document_simple in elements_verbatim.rs for details.
fn test_verbatim_12_document_simple() {
    // verbatim-12-document-simple.lex: Document with mix of verbatim blocks, groups, and general content
    let source = Lexplore::verbatim(12).source();
    let doc = DocumentLoader::new()
        .parse_with(&source, Parser::Linebased)
        .unwrap();

    // Verify first paragraph
    assert_ast(&doc).item(0, |item| {
        item.assert_paragraph()
            .text_contains("Trifecta Flat Structure Test");
    });

    // Verify first session contains verbatim group
    assert_ast(&doc).item(2, |item| {
        item.assert_session()
            .label_contains("Session with Paragraph Content")
            .child_count(3)
            .child(2, |verbatim| {
                verbatim
                    .assert_verbatim_block()
                    .subject("This is a groupped Verbatim Block, this is the first Group;")
                    .closing_label("shell")
                    .group_count(4)
                    .group(0, |group| {
                        group
                            .subject("This is a groupped Verbatim Block, this is the first Group;")
                            .content_contains("$ pwd # always te staring point");
                    })
                    .group(1, |group| {
                        group
                            .subject(
                                "Now that you know where you are, lets find out what's around you:",
                            )
                            .content_contains("$ ls");
                    });
            });
    });

    // Verify image marker verbatim block
    assert_ast(&doc).item(7, |item| {
        item.assert_verbatim_block()
            .subject("This is an Image Verbatim Representation:")
            .closing_label("image")
            .assert_marker_form()
            .has_closing_parameter_with_value("src", "image.jpg");
    });

    // Verify final verbatim block
    assert_ast(&doc).item(10, |item| {
        item.assert_verbatim_block()
            .subject("Say goodbye mom:")
            .closing_label("javascript")
            .content_contains("alert(\"Goodbye mom!\")");
    });
}
