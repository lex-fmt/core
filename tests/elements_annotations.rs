//! Unit tests for isolated annotation elements
//!
//! Tests annotation parsing in isolation following the on-lexplore.lex guidelines:
//! - Use Lexplore to load centralized test files
//! - Use assert_ast for deep structure verification
//! - Test isolated elements (one element per test)
//! - Verify content and structure, not just counts

use lex::lex::testing::assert_ast;
use lex::lex::testing::lexplore::Lexplore;

#[test]
fn test_annotation_01_flat_marker_simple() {
    // annotation-01-flat-marker-simple.lex: Simple marker annotation ":: note ::"
    let doc = Lexplore::annotation(1).parse();

    assert_ast(&doc).item_count(1).item(0, |item| {
        item.assert_annotation().label("note");
    });
}

#[test]
fn test_annotation_02_flat_marker_with_params() {
    // annotation-02-flat-marker-with-params.lex: Marker with parameter "severity=high"
    let doc = Lexplore::annotation(2).parse();

    assert_ast(&doc).item_count(1).item(0, |item| {
        item.assert_annotation()
            .label("warning")
            .parameter_count(1)
            .parameter(0, "severity", "high");
    });
}

#[test]
fn test_annotation_05_flat_block_paragraph() {
    // annotation-05-flat-block-paragraph.lex: Block annotation with paragraph content
    let doc = Lexplore::annotation(5).parse();

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
