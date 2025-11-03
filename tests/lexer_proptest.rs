//! Property-based tests for the txxt lexer using sample documents
//!
//! These tests ensure that the lexer can handle all valid txxt documents
//! from our sample collection without panicking or producing invalid tokens.

use proptest::prelude::*;
use txxt::txxt::lexers::{lex, Token};

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
        let tokens = lex(&content);

        insta::assert_debug_snapshot!(tokens);
    }

    #[test]
    fn test_010_sessions_flat_single_tokenization() {
        let content =
            read_sample_document("docs/specs/v1/samples/010-paragraphs-sessions-flat-single.txxt");
        let tokens = lex(&content);

        insta::assert_debug_snapshot!(tokens);
    }

    #[test]
    fn test_020_sessions_flat_multiple_tokenization() {
        let content = read_sample_document(
            "docs/specs/v1/samples/020-paragraphs-sessions-flat-multiple.txxt",
        );
        let tokens = lex(&content);

        insta::assert_debug_snapshot!(tokens);
    }

    #[test]
    fn test_030_sessions_nested_tokenization() {
        let content = read_sample_document(
            "docs/specs/v1/samples/030-paragraphs-sessions-nested-multiple.txxt",
        );
        let tokens = lex(&content);

        insta::assert_debug_snapshot!(tokens);
    }

    #[test]
    fn test_040_lists_tokenization() {
        let content = read_sample_document("docs/specs/v1/samples/040-lists.txxt");
        let tokens = lex(&content);

        insta::assert_debug_snapshot!(tokens);
    }

    #[test]
    fn test_050_paragraph_lists_tokenization() {
        let content = read_sample_document("docs/specs/v1/samples/050-paragraph-lists.txxt");
        let tokens = lex(&content);

        insta::assert_debug_snapshot!(tokens);
    }

    #[test]
    fn test_050_trifecta_flat_tokenization() {
        let content = read_sample_document("docs/specs/v1/samples/050-trifecta-flat-simple.txxt");
        let tokens = lex(&content);

        insta::assert_debug_snapshot!(tokens);
    }

    #[test]
    fn test_060_trifecta_nesting_tokenization() {
        let content = read_sample_document("docs/specs/v1/samples/060-trifecta-nesting.txxt");
        let tokens = lex(&content);

        insta::assert_debug_snapshot!(tokens);
    }
}

/// Property-based tests for txxt lexer
#[cfg(test)]
mod proptest_tests {
    use super::*;

    /// Generate valid txxt text content
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

    // Property-based tests using the strategies above
    proptest! {
        #[test]
        fn test_tokenize_never_panics(input in txxt_document_strategy()) {
            // The lexer should never panic on any valid txxt input
            let _tokens = lex(&input);
        }

        #[test]
        fn test_tokenize_produces_valid_tokens(input in txxt_document_strategy()) {
            // All tokens should be valid Token variants
            let tokens = lex(&input);
            for (token, _) in tokens {
                match token {
                    Token::TxxtMarker | Token::Indent | Token::IndentLevel(_) | Token::DedentLevel(_) |
                    Token::BlankLine(_) | Token::Whitespace | Token::Newline | Token::Dash | Token::Period |
                    Token::OpenParen | Token::CloseParen | Token::Colon | Token::Comma |
                    Token::Quote | Token::Equals | Token::Number(_) | Token::Text(_) => {
                        // All valid tokens
                    }
                }
            }
        }

        #[test]
        fn test_indentation_tokenization(input in indentation_strategy()) {
            // Indentation should produce appropriate indentation-related tokens
            // Note: lex() transforms Indent tokens to IndentLevel/DedentLevel
            let tokens = lex(&input);

            // After lex(), indentation tokens are transformed:
            // - Indent tokens become IndentLevel tokens (only if line has content after indentation)
            // - Blank lines (indentation followed only by newline) don't produce IndentLevel
            // - At end of file, DedentLevel tokens close the indentation

            if input.is_empty() {
                // No indentation means no indent/dedent tokens
                let indent_related_count = tokens.iter().filter(|(t, _)| {
                    matches!(t, Token::IndentLevel(_) | Token::DedentLevel(_) | Token::Indent)
                }).count();
                assert_eq!(indent_related_count, 0);
            } else if !input.chars().any(|c| !c.is_whitespace()) {
                // Pure whitespace (with or without indentation) becomes a blank line
                // Blank lines don't produce IndentLevel tokens
                let indent_related_count = tokens.iter().filter(|(t, _)| {
                    matches!(t, Token::IndentLevel(_) | Token::DedentLevel(_) | Token::Indent)
                }).count();
                assert_eq!(indent_related_count, 0);
            } else {
                // Input has actual content after indentation
                let indent_level_count = tokens.iter().filter(|(t, _)| matches!(t, Token::IndentLevel(_))).count();

                // Count expected indent levels based on input
                let expected_indents = {
                    // Count tabs (each tab = 1 indent)
                    let tab_count = input.matches('\t').count();
                    // Count groups of 4 spaces (each group = 1 indent)
                    let space_count = input.split('\t').map(|s| s.len() / 4).sum::<usize>();
                    tab_count + space_count
                };

                assert_eq!(indent_level_count, expected_indents);
            }
        }

        #[test]
        fn test_list_item_tokenization(input in list_item_strategy()) {
            // List items should contain appropriate markers
            let tokens = lex(&input);

            if input.starts_with('-') {
                assert!(tokens.iter().any(|(t, _)| matches!(t, Token::Dash)));
            } else if input.contains('.') && input.chars().next().unwrap().is_ascii_digit() {
                assert!(tokens.iter().any(|(t, _)| matches!(t, Token::Number(_))));
                assert!(tokens.iter().any(|(t, _)| matches!(t, Token::Period)));
            } else if input.starts_with('(') {
                assert!(tokens.iter().any(|(t, _)| matches!(t, Token::OpenParen)));
                assert!(tokens.iter().any(|(t, _)| matches!(t, Token::CloseParen)));
            }
        }

        #[test]
        fn test_session_title_tokenization(input in session_title_strategy()) {
            // Session titles should contain appropriate markers
            let tokens = lex(&input);

            if input.contains(':') {
                assert!(tokens.iter().any(|(t, _)| matches!(t, Token::Colon)));
            } else if input.contains('.') && input.chars().next().unwrap().is_ascii_digit() {
                assert!(tokens.iter().any(|(t, _)| matches!(t, Token::Number(_))));
                assert!(tokens.iter().any(|(t, _)| matches!(t, Token::Period)));
            }
        }

        #[test]
        fn test_multiline_tokenization(input in txxt_text_strategy()) {
            // Multiline text should contain Newline tokens
            let tokens = lex(&input);

            if input.contains('\n') {
                assert!(tokens.iter().any(|(t, _)| matches!(t, Token::Newline)));
            }
        }

        #[test]
        fn test_empty_input_tokenization(input in "") {
            // Empty input should produce no tokens
            let tokens = lex(&input);
            assert!(tokens.is_empty());
        }

        #[test]
        fn test_whitespace_only_tokenization(input in "[ ]{0,10}") {
            // Whitespace-only input should produce appropriate tokens
            let tokens = lex(&input);

            if input.is_empty() {
                assert!(tokens.is_empty());
            } else {
                // Should contain only whitespace-related tokens
                for (token, _) in tokens {
                    assert!(token.is_whitespace());
                }
            }
        }
    }
}

/// Integration tests for specific txxt patterns
#[cfg(test)]
mod integration_tests {
    use super::*;
    use txxt::txxt::testing::factories::mk_tokens;

    #[test]
    fn test_paragraph_pattern() {
        let input = "This is a paragraph.\nIt has multiple lines.";
        let tokens = lex(input);

        // Exact token sequence validation
        // lex() adds a trailing newline and applies full transformations
        assert_eq!(
            tokens,
            mk_tokens(&[
                (Token::Text("This".to_string()), 0, 4),
                (Token::Whitespace, 4, 5),
                (Token::Text("is".to_string()), 5, 7),
                (Token::Whitespace, 7, 8),
                (Token::Text("a".to_string()), 8, 9),
                (Token::Whitespace, 9, 10),
                (Token::Text("paragraph".to_string()), 10, 19),
                (Token::Period, 19, 20),
                (Token::Newline, 20, 21),
                (Token::Text("It".to_string()), 21, 23),
                (Token::Whitespace, 23, 24),
                (Token::Text("has".to_string()), 24, 27),
                (Token::Whitespace, 27, 28),
                (Token::Text("multiple".to_string()), 28, 36),
                (Token::Whitespace, 36, 37),
                (Token::Text("lines".to_string()), 37, 42),
                (Token::Period, 42, 43),
                (Token::Newline, 43, 44),
            ])
        );
    }

    #[test]
    fn test_list_pattern() {
        let input = "- First item\n- Second item";
        let tokens = lex(input);

        // Exact token sequence validation
        // lex() adds a trailing newline and applies full transformations
        assert_eq!(
            tokens,
            mk_tokens(&[
                (Token::Dash, 0, 1),
                (Token::Whitespace, 1, 2),
                (Token::Text("First".to_string()), 2, 7),
                (Token::Whitespace, 7, 8),
                (Token::Text("item".to_string()), 8, 12),
                (Token::Newline, 12, 13),
                (Token::Dash, 13, 14),
                (Token::Whitespace, 14, 15),
                (Token::Text("Second".to_string()), 15, 21),
                (Token::Whitespace, 21, 22),
                (Token::Text("item".to_string()), 22, 26),
                (Token::Newline, 26, 27),
            ])
        );
    }

    #[test]
    fn test_session_pattern() {
        let input = "1. Session Title\n    Content here";
        let tokens = lex(input);

        // Exact token sequence validation
        // lex() transforms Indent -> IndentLevel and adds trailing newline
        assert_eq!(
            tokens,
            mk_tokens(&[
                (Token::Number("1".to_string()), 0, 1),
                (Token::Period, 1, 2),
                (Token::Whitespace, 2, 3),
                (Token::Text("Session".to_string()), 3, 10),
                (Token::Whitespace, 10, 11),
                (Token::Text("Title".to_string()), 11, 16),
                (Token::Newline, 16, 17),
                (Token::IndentLevel(vec![(Token::Indent, 17..21)]), 0, 0),
                (Token::Text("Content".to_string()), 21, 28),
                (Token::Whitespace, 28, 29),
                (Token::Text("here".to_string()), 29, 33),
                (Token::Newline, 33, 34),
                (Token::DedentLevel(vec![]), 0, 0),
            ])
        );
    }

    #[test]
    fn test_txxt_marker_pattern() {
        let input = "Some text :: marker";
        let tokens = lex(input);

        // Exact token sequence validation
        // lex() adds a trailing newline
        assert_eq!(
            tokens,
            mk_tokens(&[
                (Token::Text("Some".to_string()), 0, 4),
                (Token::Whitespace, 4, 5),
                (Token::Text("text".to_string()), 5, 9),
                (Token::Whitespace, 9, 10),
                (Token::TxxtMarker, 10, 12),
                (Token::Whitespace, 12, 13),
                (Token::Text("marker".to_string()), 13, 19),
                (Token::Newline, 19, 20),
            ])
        );
    }

    #[test]
    fn test_mixed_content_pattern() {
        let input = "1. Session\n    - Item 1\n    - Item 2\n\nParagraph after.";
        let tokens = lex(input);

        // Exact token sequence validation
        // lex() transforms Indent -> IndentLevel and consecutive Newlines -> BlankLine
        assert_eq!(
            tokens,
            mk_tokens(&[
                (Token::Number("1".to_string()), 0, 1),
                (Token::Period, 1, 2),
                (Token::Whitespace, 2, 3),
                (Token::Text("Session".to_string()), 3, 10),
                (Token::Newline, 10, 11),
                (Token::IndentLevel(vec![(Token::Indent, 11..15)]), 0, 0),
                (Token::Dash, 15, 16),
                (Token::Whitespace, 16, 17),
                (Token::Text("Item".to_string()), 17, 21),
                (Token::Whitespace, 21, 22),
                (Token::Number("1".to_string()), 22, 23),
                (Token::Newline, 23, 24),
                (Token::Dash, 28, 29),
                (Token::Whitespace, 29, 30),
                (Token::Text("Item".to_string()), 30, 34),
                (Token::Whitespace, 34, 35),
                (Token::Number("2".to_string()), 35, 36),
                (Token::Newline, 36, 37),
                (Token::BlankLine(vec![(Token::Newline, 37..38)]), 0, 0),
                (Token::DedentLevel(vec![]), 0, 0),
                (Token::Text("Paragraph".to_string()), 38, 47),
                (Token::Whitespace, 47, 48),
                (Token::Text("after".to_string()), 48, 53),
                (Token::Period, 53, 54),
                (Token::Newline, 54, 55),
            ])
        );
    }
}
