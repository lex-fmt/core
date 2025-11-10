//! Unit tests for isolated paragraph elements
//!
//! Tests paragraph parsing in isolation following the on-lexplore.lex guidelines:
//! - Use Lexplore to load centralized test files
//! - Use assert_ast for deep structure verification
//! - Test isolated elements (one element per test)
//! - Verify content and structure, not just counts

use lex::lex::testing::assert_ast;
use lex::lex::testing::lexplore::Lexplore;

#[test]
fn test_paragraph_01_flat_oneline() {
    // paragraph-01-flat-oneline.lex: "This is a simple paragraph with just one line."
    let doc = Lexplore::paragraph(1).parse();

    // Verify the document contains exactly one paragraph with expected content
    assert_ast(&doc).item_count(1).item(0, |item| {
        item.assert_paragraph()
            .text("This is a simple paragraph with just one line.")
            .line_count(1);
    });
}

#[test]
fn test_paragraph_02_flat_multiline() {
    // paragraph-02-flat-multiline.lex: Three lines of text
    let doc = Lexplore::paragraph(2).parse();

    assert_ast(&doc).item_count(1).item(0, |item| {
        item.assert_paragraph()
            .text_contains("This is a multi-line paragraph")
            .text_contains("It continues on the second line")
            .text_contains("And even has a third line")
            .line_count(3);
    });
}

#[test]
fn test_paragraph_03_flat_special_chars() {
    // paragraph-03-flat-special-chars.lex: Tests that special characters are preserved
    let doc = Lexplore::paragraph(3).parse();

    assert_ast(&doc).item_count(1).item(0, |item| {
        item.assert_paragraph()
            .text_contains("!@#$%^&*()_+-=[]{}|;':\",./<>?")
            .text_contains("special characters")
            .line_count(1);
    });
}
