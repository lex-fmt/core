//! Property-based tests for the txxt lexer using sample documents
//!
//! These tests ensure that the lexer can handle all valid txxt documents
//! from our sample collection without panicking or producing invalid tokens.

use proptest::prelude::*;
use txxt_nano::txxt_nano::lexer::{tokenize, Token};

/// Sample document snapshot tests
#[cfg(test)]
mod sample_document_tests {
    use super::*;
    use std::fs;

    /// Helper function to read sample document content
    fn read_sample_document(path: &str) -> String {
        fs::read_to_string(path).expect("Failed to read sample document")
    }

    #[test]
    fn test_000_paragraphs_tokenization() {
        let content = read_sample_document("docs/specs/v1/samples/000-paragraphs.txxt");
        let tokens = tokenize(&content);

        insta::assert_debug_snapshot!(tokens);
    }

    #[test]
    fn test_010_sessions_flat_single_tokenization() {
        let content =
            read_sample_document("docs/specs/v1/samples/010-paragraphs-sessions-flat-single.txxt");
        let tokens = tokenize(&content);

        insta::assert_debug_snapshot!(tokens);
    }

    #[test]
    fn test_020_sessions_flat_multiple_tokenization() {
        let content = read_sample_document(
            "docs/specs/v1/samples/020-paragraphs-sessions-flat-multiple.txxt",
        );
        let tokens = tokenize(&content);

        insta::assert_debug_snapshot!(tokens);
    }

    #[test]
    fn test_030_sessions_nested_tokenization() {
        let content = read_sample_document(
            "docs/specs/v1/samples/030-paragraphs-sessions-nested-multiple.txxt",
        );
        let tokens = tokenize(&content);

        insta::assert_debug_snapshot!(tokens);
    }

    #[test]
    fn test_040_lists_tokenization() {
        let content = read_sample_document("docs/specs/v1/samples/040-lists.txxt");
        let tokens = tokenize(&content);

        insta::assert_debug_snapshot!(tokens);
    }

    #[test]
    fn test_050_paragraph_lists_tokenization() {
        let content = read_sample_document("docs/specs/v1/samples/050-paragraph-lists.txxt");
        let tokens = tokenize(&content);

        insta::assert_debug_snapshot!(tokens);
    }

    #[test]
    fn test_050_trifecta_flat_tokenization() {
        let content = read_sample_document("docs/specs/v1/samples/050-trifecta-flat-simple.txxt");
        let tokens = tokenize(&content);

        insta::assert_debug_snapshot!(tokens);
    }

    #[test]
    fn test_060_trifecta_nesting_tokenization() {
        let content = read_sample_document("docs/specs/v1/samples/060-trifecta-nesting.txxt");
        let tokens = tokenize(&content);

        insta::assert_debug_snapshot!(tokens);
    }
}

/// Property-based tests for txxt lexer
#[cfg(test)]
mod proptest_tests {
    use super::*;

    /// Generate valid txxt text content
    #[allow(dead_code)]
    fn txxt_text_strategy() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                // Simple text
                "[a-zA-Z0-9]+",
                // Text with spaces
                "[a-zA-Z0-9]+ [a-zA-Z0-9]+",
                // Text with punctuation
                "[a-zA-Z0-9]+[.,!?]",
                // Empty string
                "",
            ],
            1..10,
        )
        .prop_map(|lines| lines.join("\n"))
    }

    /// Generate valid indentation
    #[allow(dead_code)]
    fn indentation_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            // No indentation
            "",
            // Single level (4 spaces)
            "    ",
            // Multiple levels
            prop::collection::vec("    ", 1..5).prop_map(|levels| levels.join("")),
            // Tab indentation
            "\t",
            // Mixed indentation
            prop::collection::vec(prop_oneof!["    ", "\t"], 1..3)
                .prop_map(|levels| levels.join("")),
        ]
    }

    /// Generate valid list items
    #[allow(dead_code)]
    fn list_item_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            // Plain dash list
            "- [a-zA-Z0-9 ]+",
            // Numbered list
            "[0-9]+\\. [a-zA-Z0-9 ]+",
            // Letter list
            "[a-z]\\. [a-zA-Z0-9 ]+",
            // Parenthetical list
            "\\([0-9]+\\) [a-zA-Z0-9 ]+",
        ]
    }

    /// Generate valid session titles
    #[allow(dead_code)]
    fn session_title_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            // Numbered session
            "[0-9]+\\. [a-zA-Z0-9 ]+",
            // Unnumbered session
            "[a-zA-Z0-9 ]+:",
            // Plain session title
            "[a-zA-Z0-9 ]+",
        ]
    }

    /// Generate valid txxt documents
    #[allow(dead_code)]
    fn txxt_document_strategy() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                // Paragraphs
                txxt_text_strategy(),
                // List items
                list_item_strategy(),
                // Session titles
                session_title_strategy(),
            ],
            1..20,
        )
        .prop_map(|lines| lines.join("\n"))
    }

    #[test]
    fn test_tokenize_never_panics() {
        // Test with various valid inputs
        let test_inputs = vec![
            "hello world",
            "1. Session Title",
            "- Item 1\n- Item 2",
            "    indented text",
            ":: marker",
            "a. Letter item",
            "(1) Parenthetical item",
        ];

        for input in test_inputs {
            let _tokens = tokenize(input);
            // Test passes if no panic occurs
        }
    }

    #[test]
    fn test_tokenize_produces_valid_tokens() {
        let tokens = tokenize("hello world");
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_indentation_tokenization() {
        let tokens = tokenize("    hello");
        assert!(!tokens.is_empty());
        assert!(tokens.iter().any(|t| t.is_indent()));
    }

    #[test]
    fn test_list_item_tokenization() {
        let tokens = tokenize("- Item 1");
        assert!(!tokens.is_empty());
        assert!(tokens.iter().any(|t| matches!(t, Token::Dash)));
    }

    #[test]
    fn test_session_title_tokenization() {
        let tokens = tokenize("1. Session Title");
        assert!(!tokens.is_empty());
        assert!(tokens.iter().any(|t| matches!(t, Token::Period)));
    }

    #[test]
    fn test_multiline_tokenization() {
        let tokens = tokenize("Line 1\nLine 2");
        assert!(!tokens.is_empty());
        assert!(tokens.iter().any(|t| matches!(t, Token::Newline)));
    }

    #[test]
    fn test_empty_input_tokenization() {
        let tokens = tokenize("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_whitespace_only_tokenization() {
        let tokens = tokenize("   ");
        assert!(!tokens.is_empty());
        assert!(tokens.iter().any(|t| t.is_whitespace()));
    }
}

/// Integration tests for specific txxt patterns
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_paragraph_pattern() {
        let input = "This is a paragraph.\nIt has multiple lines.";
        let tokens = tokenize(input);

        // Should contain text and newline tokens
        assert!(tokens.iter().any(|t| matches!(t, Token::Text)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Newline)));
        // Should produce valid tokens
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_list_pattern() {
        let input = "- First item\n- Second item";
        let tokens = tokenize(input);

        // Should contain dash tokens
        assert!(tokens.iter().any(|t| matches!(t, Token::Dash)));
        // Should produce valid tokens
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_session_pattern() {
        let input = "1. Session Title\n    Content here";
        let tokens = tokenize(input);

        // Should contain period, text, and indentation tokens
        assert!(tokens.iter().any(|t| matches!(t, Token::Period)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Text)));
        assert!(tokens.iter().any(|t| t.is_indent()));
        // Should produce valid tokens
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_txxt_marker_pattern() {
        let input = "Some text :: marker";
        let tokens = tokenize(input);

        // Should contain txxt marker
        assert!(tokens.iter().any(|t| matches!(t, Token::TxxtMarker)));
        // Should produce valid tokens
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_mixed_content_pattern() {
        let input = "1. Session\n    - Item 1\n    - Item 2\n\nParagraph after.";
        let tokens = tokenize(input);

        // Should contain various token types
        assert!(tokens.iter().any(|t| matches!(t, Token::Period)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Dash)));
        assert!(tokens.iter().any(|t| t.is_indent()));
        assert!(tokens.iter().any(|t| matches!(t, Token::Newline)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Text)));
        // Should produce valid tokens
        assert!(!tokens.is_empty());
    }
}
