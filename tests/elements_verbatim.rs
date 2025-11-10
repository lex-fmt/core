//! Unit tests for isolated verbatim elements
//!
//! Tests verbatim block parsing in isolation following the on-lexplore.lex guidelines:
//! - Use Lexplore to load centralized test files
//! - Use assert_ast for deep structure verification
//! - Test isolated elements (one element per test)
//! - Verify content and structure, not just counts

use lex::lex::testing::assert_ast;
use lex::lex::testing::lexplore::Lexplore;

#[test]
fn test_verbatim_01_flat_simple_code() {
    // verbatim-01-flat-simple-code.lex: Verbatim block with simple code
    let doc = Lexplore::verbatim(1).parse();

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
    let doc = Lexplore::verbatim(2).parse();

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
    let doc = Lexplore::verbatim(3).parse();

    assert_ast(&doc).item_count(1).item(0, |item| {
        item.assert_verbatim_block()
            .subject("Configuration")
            .closing_label("nginx")
            .has_closing_parameter_with_value("format", "pretty")
            .content_contains("server {");
    });
}
