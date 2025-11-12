//! Tests for spec/overview documents that don't map to numbered element loaders

use lex_parser::lex::pipeline::Parser;
use lex_parser::lex::testing::assert_ast;
use lex_parser::lex::testing::lexplore::Lexplore;
use rstest::rstest;

#[rstest(parser => [Parser::Linebased])]
fn test_labels_spec_document(parser: Parser) {
    let doc = Lexplore::from_path("docs/specs/v1/elements/label/labels.lex").parse_with(parser);

    assert_ast(&doc)
        .item(0, |item| {
            item.assert_paragraph().text("Labels");
        })
        .item(1, |item| {
            item.assert_session()
                .label_contains("Introduction")
                .child(0, |child| {
                    child
                        .assert_paragraph()
                        .text_contains("identifiers for annotations");
                });
        });
}

#[rstest(parser => [Parser::Linebased])]
fn test_parameters_spec_document(parser: Parser) {
    let doc =
        Lexplore::from_path("docs/specs/v1/elements/parameter/parameters.lex").parse_with(parser);

    assert_ast(&doc)
        .item(0, |item| {
            item.assert_paragraph().text("Parameters");
        })
        .item(1, |item| {
            item.assert_session().label("Introduction");
        });
}

#[rstest(parser => [Parser::Linebased])]
fn test_verbatim_spec_document(parser: Parser) {
    let doc =
        Lexplore::from_path("docs/specs/v1/elements/verbatim/verbatim.lex").parse_with(parser);

    assert_ast(&doc)
        .item(0, |item| {
            item.assert_paragraph().text("Verbatim Blocks");
        })
        .item(1, |item| {
            item.assert_session().label("Introduction");
        })
        .item(2, |item| {
            item.assert_session().label("Syntax");
        });
}

#[rstest(parser => [Parser::Linebased])]
fn test_template_document_simple(parser: Parser) {
    let doc =
        Lexplore::from_path("docs/specs/v1/elements/XXX-document-simple.lex").parse_with(parser);

    assert_ast(&doc)
        .item(0, |item| {
            item.assert_paragraph()
                .text_contains("Trifecta Flat Structure Test");
        })
        .item(2, |item| {
            item.assert_session()
                .label("1. Session with Paragraph Content {{session-title}}")
                .child(2, |child| {
                    child.assert_paragraph().text("<insert element here>");
                });
        });
}

#[rstest(parser => [Parser::Linebased])]
fn test_template_document_tricky(parser: Parser) {
    let doc =
        Lexplore::from_path("docs/specs/v1/elements/XXX-document-tricky.lex").parse_with(parser);

    assert_ast(&doc)
        .item(0, |item| {
            item.assert_paragraph()
                .text_contains("Trifecta Nesting Test");
        })
        .item(2, |item| {
            item.assert_session()
                .label("1. Root Session {{session-title}}")
                .child(1, |child| {
                    child
                        .assert_session()
                        .label("1.1. Sub-session with Paragraph {{session-title}}")
                        .child(1, |list_child| {
                            list_child.assert_list().item_count(2);
                        });
                });
        });
}
