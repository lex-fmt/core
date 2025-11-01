//! Property-based tests for the txxt lexer using sample documents
//!
//! These tests ensure that the lexer can handle all valid txxt documents
//! from our sample collection without panicking or producing invalid tokens.

use proptest::prelude::*;
use txxt::txxt::lexer::{lex, Token};

/// Helper: strip locations from lexer output
fn strip_loc(pairs: Vec<(Token, std::ops::Range<usize>)>) -> Vec<Token> {
    pairs.into_iter().map(|(t, _)| t).collect()
}

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
        let tokens = strip_loc(lex(&content));

        insta::assert_debug_snapshot!(tokens);
    }

    #[test]
    fn test_010_sessions_flat_single_tokenization() {
        let content =
            read_sample_document("docs/specs/v1/samples/010-paragraphs-sessions-flat-single.txxt");
        let tokens = strip_loc(lex(&content));

        insta::assert_debug_snapshot!(tokens);
    }

    #[test]
    fn test_020_sessions_flat_multiple_tokenization() {
        let content = read_sample_document(
            "docs/specs/v1/samples/020-paragraphs-sessions-flat-multiple.txxt",
        );
        let tokens = strip_loc(lex(&content));

        insta::assert_debug_snapshot!(tokens);
    }

    #[test]
    fn test_030_sessions_nested_tokenization() {
        let content = read_sample_document(
            "docs/specs/v1/samples/030-paragraphs-sessions-nested-multiple.txxt",
        );
        let tokens = strip_loc(lex(&content));

        insta::assert_debug_snapshot!(tokens);
    }

    #[test]
    fn test_040_lists_tokenization() {
        let content = read_sample_document("docs/specs/v1/samples/040-lists.txxt");
        let tokens = strip_loc(lex(&content));

        insta::assert_debug_snapshot!(tokens);
    }

    #[test]
    fn test_050_paragraph_lists_tokenization() {
        let content = read_sample_document("docs/specs/v1/samples/050-paragraph-lists.txxt");
        let tokens = strip_loc(lex(&content));

        insta::assert_debug_snapshot!(tokens);
    }

    #[test]
    fn test_050_trifecta_flat_tokenization() {
        let content = read_sample_document("docs/specs/v1/samples/050-trifecta-flat-simple.txxt");
        let tokens = strip_loc(lex(&content));

        insta::assert_debug_snapshot!(tokens);
    }

    #[test]
    fn test_060_trifecta_nesting_tokenization() {
        let content = read_sample_document("docs/specs/v1/samples/060-trifecta-nesting.txxt");
        let tokens = strip_loc(lex(&content));

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
            let tokens = strip_loc(lex(&input));
            for token in tokens {
                match token {
                    Token::TxxtMarker | Token::Indent | Token::IndentLevel | Token::DedentLevel |
                    Token::BlankLine | Token::Whitespace | Token::Newline | Token::Dash | Token::Period |
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
            let tokens = strip_loc(lex(&input));

            // After lex(), indentation tokens are transformed:
            // - Indent tokens become IndentLevel tokens (only if line has content after indentation)
            // - Blank lines (indentation followed only by newline) don't produce IndentLevel
            // - At end of file, DedentLevel tokens close the indentation

            if input.is_empty() {
                // No indentation means no indent/dedent tokens
                let indent_related_count = tokens.iter().filter(|t| {
                    matches!(t, Token::IndentLevel | Token::DedentLevel | Token::Indent)
                }).count();
                assert_eq!(indent_related_count, 0);
            } else if !input.chars().any(|c| !c.is_whitespace()) {
                // Pure whitespace (with or without indentation) becomes a blank line
                // Blank lines don't produce IndentLevel tokens
                let indent_related_count = tokens.iter().filter(|t| {
                    matches!(t, Token::IndentLevel | Token::DedentLevel | Token::Indent)
                }).count();
                assert_eq!(indent_related_count, 0);
            } else {
                // Input has actual content after indentation
                let indent_level_count = tokens.iter().filter(|t| matches!(t, Token::IndentLevel)).count();

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
            let tokens = strip_loc(lex(&input));

            if input.starts_with('-') {
                assert!(tokens.iter().any(|t| matches!(t, Token::Dash)));
            } else if input.contains('.') && input.chars().next().unwrap().is_ascii_digit() {
                assert!(tokens.iter().any(|t| matches!(t, Token::Number(_))));
                assert!(tokens.iter().any(|t| matches!(t, Token::Period)));
            } else if input.starts_with('(') {
                assert!(tokens.iter().any(|t| matches!(t, Token::OpenParen)));
                assert!(tokens.iter().any(|t| matches!(t, Token::CloseParen)));
            }
        }

        #[test]
        fn test_session_title_tokenization(input in session_title_strategy()) {
            // Session titles should contain appropriate markers
            let tokens = strip_loc(lex(&input));

            if input.contains(':') {
                assert!(tokens.iter().any(|t| matches!(t, Token::Colon)));
            } else if input.contains('.') && input.chars().next().unwrap().is_ascii_digit() {
                assert!(tokens.iter().any(|t| matches!(t, Token::Number(_))));
                assert!(tokens.iter().any(|t| matches!(t, Token::Period)));
            }
        }

        #[test]
        fn test_multiline_tokenization(input in txxt_text_strategy()) {
            // Multiline text should contain Newline tokens
            let tokens = strip_loc(lex(&input));

            if input.contains('\n') {
                assert!(tokens.iter().any(|t| matches!(t, Token::Newline)));
            }
        }

        #[test]
        fn test_empty_input_tokenization(input in "") {
            // Empty input should produce no tokens
            let tokens = strip_loc(lex(&input));
            assert!(tokens.is_empty());
        }

        #[test]
        fn test_whitespace_only_tokenization(input in "[ ]{0,10}") {
            // Whitespace-only input should produce appropriate tokens
            let tokens = strip_loc(lex(&input));

            if input.is_empty() {
                assert!(tokens.is_empty());
            } else {
                // Should contain only whitespace-related tokens
                for token in tokens {
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

    #[test]
    fn test_paragraph_pattern() {
        let input = "This is a paragraph.\nIt has multiple lines.";
        let tokens = strip_loc(lex(input));

        // Exact token sequence validation
        // lex() adds a trailing newline and applies full transformations
        assert_eq!(
            tokens,
            vec![
                Token::Text("This".to_string()),      // "This"
                Token::Whitespace,                    // " "
                Token::Text("is".to_string()),        // "is"
                Token::Whitespace,                    // " "
                Token::Text("a".to_string()),         // "a"
                Token::Whitespace,                    // " "
                Token::Text("paragraph".to_string()), // "paragraph"
                Token::Period,                        // "."
                Token::Newline,                       // "\n"
                Token::Text("It".to_string()),        // "It"
                Token::Whitespace,                    // " "
                Token::Text("has".to_string()),       // "has"
                Token::Whitespace,                    // " "
                Token::Text("multiple".to_string()),  // "multiple"
                Token::Whitespace,                    // " "
                Token::Text("lines".to_string()),     // "lines"
                Token::Period,                        // "."
                Token::Newline,                       // "\n" (added by lex)
            ]
        );
    }

    #[test]
    fn test_list_pattern() {
        let input = "- First item\n- Second item";
        let tokens = strip_loc(lex(input));

        // Exact token sequence validation
        // lex() adds a trailing newline and applies full transformations
        assert_eq!(
            tokens,
            vec![
                Token::Dash,                       // "-"
                Token::Whitespace,                 // " "
                Token::Text("First".to_string()),  // "First"
                Token::Whitespace,                 // " "
                Token::Text("item".to_string()),   // "item"
                Token::Newline,                    // "\n"
                Token::Dash,                       // "-"
                Token::Whitespace,                 // " "
                Token::Text("Second".to_string()), // "Second"
                Token::Whitespace,                 // " "
                Token::Text("item".to_string()),   // "item"
                Token::Newline,                    // "\n" (added by lex)
            ]
        );
    }

    #[test]
    fn test_session_pattern() {
        let input = "1. Session Title\n    Content here";
        let tokens = strip_loc(lex(input));

        // Exact token sequence validation
        // lex() transforms Indent -> IndentLevel and adds trailing newline
        assert_eq!(
            tokens,
            vec![
                Token::Number("1".to_string()),     // "1"
                Token::Period,                      // "."
                Token::Whitespace,                  // " "
                Token::Text("Session".to_string()), // "Session"
                Token::Whitespace,                  // " "
                Token::Text("Title".to_string()),   // "Title"
                Token::Newline,                     // "\n"
                Token::IndentLevel,                 // IndentLevel (was Indent)
                Token::Text("Content".to_string()), // "Content"
                Token::Whitespace,                  // " "
                Token::Text("here".to_string()),    // "here"
                Token::Newline,                     // "\n" (added by lex)
                Token::DedentLevel,                 // DedentLevel (closes indentation)
            ]
        );
    }

    #[test]
    fn test_txxt_marker_pattern() {
        let input = "Some text :: marker";
        let tokens = strip_loc(lex(input));

        // Exact token sequence validation
        // lex() adds a trailing newline
        assert_eq!(
            tokens,
            vec![
                Token::Text("Some".to_string()),   // "Some"
                Token::Whitespace,                 // " "
                Token::Text("text".to_string()),   // "text"
                Token::Whitespace,                 // " "
                Token::TxxtMarker,                 // "::"
                Token::Whitespace,                 // " "
                Token::Text("marker".to_string()), // "marker"
                Token::Newline,                    // "\n" (added by lex)
            ]
        );
    }

    #[test]
    fn test_mixed_content_pattern() {
        let input = "1. Session\n    - Item 1\n    - Item 2\n\nParagraph after.";
        let tokens = strip_loc(lex(input));

        // Exact token sequence validation
        // lex() transforms Indent -> IndentLevel and consecutive Newlines -> BlankLine
        assert_eq!(
            tokens,
            vec![
                Token::Number("1".to_string()),       // "1"
                Token::Period,                        // "."
                Token::Whitespace,                    // " "
                Token::Text("Session".to_string()),   // "Session"
                Token::Newline,                       // "\n"
                Token::IndentLevel,                   // IndentLevel (was Indent)
                Token::Dash,                          // "-"
                Token::Whitespace,                    // " "
                Token::Text("Item".to_string()),      // "Item"
                Token::Whitespace,                    // " "
                Token::Number("1".to_string()),       // "1"
                Token::Newline,                       // "\n"
                Token::Dash,                          // "-"
                Token::Whitespace,                    // " "
                Token::Text("Item".to_string()),      // "Item"
                Token::Whitespace,                    // " "
                Token::Number("2".to_string()),       // "2"
                Token::Newline,                       // "\n"
                Token::BlankLine,                     // BlankLine (was 2nd Newline)
                Token::DedentLevel,                   // DedentLevel (from indent back to 0)
                Token::Text("Paragraph".to_string()), // "Paragraph"
                Token::Whitespace,                    // " "
                Token::Text("after".to_string()),     // "after"
                Token::Period,                        // "."
                Token::Newline,                       // "\n" (added by lex)
            ]
        );
    }
}
