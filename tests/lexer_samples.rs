//! Integration tests for lexer using sample documents
//!
//! These tests verify that the lexer correctly tokenizes all sample documents
//! from the specification, using snapshot testing to catch regressions.

use lex::lex::lexing::{lex, Token};
use std::fs;

/// Helper to prepare token stream and call lex pipeline
fn lex_helper(source: &str) -> Vec<(Token, std::ops::Range<usize>)> {
    let source_with_newline = lex::lex::lexing::ensure_source_ends_with_newline(source);
    let token_stream = lex::lex::lexing::base_tokenization::tokenize(&source_with_newline);
    lex(token_stream)
}

/// Helper function to read sample document content
fn read_sample_document(path: &str) -> String {
    fs::read_to_string(path).expect("Failed to read sample document")
}

#[test]
fn test_000_paragraphs_tokenization() {
    let content = read_sample_document("docs/specs/v1/samples/000-paragraphs.lex");
    let tokens = lex_helper(&content);

    insta::assert_debug_snapshot!(tokens);
}

#[test]
fn test_010_sessions_flat_single_tokenization() {
    let content =
        read_sample_document("docs/specs/v1/samples/010-paragraphs-sessions-flat-single.lex");
    let tokens = lex_helper(&content);

    insta::assert_debug_snapshot!(tokens);
}

#[test]
fn test_020_sessions_flat_multiple_tokenization() {
    let content =
        read_sample_document("docs/specs/v1/samples/020-paragraphs-sessions-flat-multiple.lex");
    let tokens = lex_helper(&content);

    insta::assert_debug_snapshot!(tokens);
}

#[test]
fn test_030_sessions_nested_tokenization() {
    let content =
        read_sample_document("docs/specs/v1/samples/030-paragraphs-sessions-nested-multiple.lex");
    let tokens = lex_helper(&content);

    insta::assert_debug_snapshot!(tokens);
}

#[test]
fn test_040_lists_tokenization() {
    let content = read_sample_document("docs/specs/v1/samples/040-lists.lex");
    let tokens = lex_helper(&content);

    insta::assert_debug_snapshot!(tokens);
}

#[test]
fn test_050_paragraph_lists_tokenization() {
    let content = read_sample_document("docs/specs/v1/samples/050-paragraph-lists.lex");
    let tokens = lex_helper(&content);

    insta::assert_debug_snapshot!(tokens);
}

#[test]
fn test_050_trifecta_flat_tokenization() {
    let content = read_sample_document("docs/specs/v1/samples/050-trifecta-flat-simple.lex");
    let tokens = lex_helper(&content);

    insta::assert_debug_snapshot!(tokens);
}

#[test]
fn test_060_trifecta_nesting_tokenization() {
    let content = read_sample_document("docs/specs/v1/samples/060-trifecta-nesting.lex");
    let tokens = lex_helper(&content);

    insta::assert_debug_snapshot!(tokens);
}
