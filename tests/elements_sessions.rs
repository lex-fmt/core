//! Unit tests for isolated session elements
//!
//! Tests session parsing in isolation following the on-lexplore.lex guidelines:
//! - Use Lexplore to load centralized test files
//! - Use assert_ast for deep structure verification
//! - Test isolated elements (one element per test)
//! - Verify content and structure, not just counts

use lex::lex::testing::assert_ast;
use lex::lex::testing::lexplore::Lexplore;

#[test]
fn test_session_01_flat_simple() {
    // session-01-flat-simple.lex: Session with title "Introduction" and one paragraph
    let doc = Lexplore::session(1).parse();

    assert_ast(&doc).item_count(1).item(0, |item| {
        item.assert_session()
            .label("Introduction")
            .child_count(1)
            .child(0, |child| {
                child
                    .assert_paragraph()
                    .text_contains("simple session with a title");
            });
    });
}

#[test]
fn test_session_02_flat_numbered_title() {
    // session-02-flat-numbered-title.lex: Session with numbered title "1. Introduction:"
    let doc = Lexplore::session(2).parse();

    assert_ast(&doc).item_count(1).item(0, |item| {
        item.assert_session()
            .label("1. Introduction:")
            .child_count(1)
            .child(0, |child| {
                child
                    .assert_paragraph()
                    .text_contains("numbered title marker");
            });
    });
}

#[test]
fn test_session_05_nested_simple() {
    // session-05-nested-simple.lex: Document with paragraphs and nested sessions
    let doc = Lexplore::session(5).parse();

    // Document structure: Para, Para, Session, Para, Session, Para
    assert_ast(&doc)
        .item_count(6)
        .item(0, |item| {
            item.assert_paragraph()
                .text_contains("Paragraphs and Single Session Test");
        })
        .item(1, |item| {
            item.assert_paragraph()
                .text_contains("combination of paragraphs");
        })
        .item(2, |item| {
            item.assert_session()
                .label("1. Introduction {{session-title}}")
                .child_count(2)
                .child(0, |child| {
                    child
                        .assert_paragraph()
                        .text_contains("content of the session");
                })
                .child(1, |child| {
                    child
                        .assert_paragraph()
                        .text_contains("multiple paragraphs");
                });
        })
        .item(3, |item| {
            item.assert_paragraph()
                .text_contains("paragraph comes after the session");
        })
        .item(4, |item| {
            item.assert_session()
                .label("Another Session {{session-title}}")
                .child_count(1)
                .child(0, |child| {
                    child
                        .assert_paragraph()
                        .text_contains("multiple sessions at the same level");
                });
        })
        .item(5, |item| {
            item.assert_paragraph()
                .text_contains("Final paragraph at the root level");
        });
}
