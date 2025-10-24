//! Implementation of the txxt lexer
//!
//! This module provides convenience functions for tokenizing txxt text.
//! The actual tokenization is handled entirely by logos.

use crate::txxt_nano::lexer::tokens::Token;
use logos::Logos;

/// Convenience function to tokenize a string and collect all tokens
pub fn tokenize(source: &str) -> Vec<Token> {
    Token::lexer(source)
        .filter_map(|result| result.ok())
        .collect()
}

/// Convenience function to tokenize a string and collect tokens with their spans
pub fn tokenize_with_spans(source: &str) -> Vec<(Token, logos::Span)> {
    let mut lexer = Token::lexer(source);
    let mut tokens = Vec::new();

    while let Some(result) = lexer.next() {
        if let Ok(token) = result {
            tokens.push((token, lexer.span()));
        }
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tokenization() {
        let tokens = tokenize("hello world");
        assert_eq!(tokens, vec![Token::Text, Token::Whitespace, Token::Text]);
    }

    #[test]
    fn test_indentation_tokenization() {
        let tokens = tokenize("    hello");
        assert_eq!(tokens, vec![Token::Indent, Token::Text]);
    }

    #[test]
    fn test_multiple_indentation_levels() {
        let tokens = tokenize("        hello");
        assert_eq!(tokens, vec![Token::Indent, Token::Indent, Token::Text]);
    }

    #[test]
    fn test_tab_indentation() {
        let tokens = tokenize("\thello");
        assert_eq!(tokens, vec![Token::Indent, Token::Text]);
    }

    #[test]
    fn test_mixed_indentation() {
        let tokens = tokenize("    \thello");
        assert_eq!(tokens, vec![Token::Indent, Token::Indent, Token::Text]);
    }

    #[test]
    fn test_list_item_tokenization() {
        let tokens = tokenize("- Item 1");
        assert_eq!(
            tokens,
            vec![
                Token::Dash,
                Token::Whitespace,
                Token::Text,
                Token::Whitespace,
                Token::Number
            ]
        );
    }

    #[test]
    fn test_numbered_item_tokenization() {
        let tokens = tokenize("1. Item 1");
        assert_eq!(
            tokens,
            vec![
                Token::Number,
                Token::Period,
                Token::Whitespace,
                Token::Text,
                Token::Whitespace,
                Token::Number
            ]
        );
    }

    #[test]
    fn test_session_title_tokenization() {
        let tokens = tokenize("1. Session Title");
        assert_eq!(
            tokens,
            vec![
                Token::Number,
                Token::Period,
                Token::Whitespace,
                Token::Text,
                Token::Whitespace,
                Token::Text
            ]
        );
    }

    #[test]
    fn test_txxt_marker_tokenization() {
        let tokens = tokenize("::");
        assert_eq!(tokens, vec![Token::TxxtMarker]);
    }

    #[test]
    fn test_multiline_tokenization() {
        let tokens = tokenize("Line 1\nLine 2");
        assert_eq!(
            tokens,
            vec![
                Token::Text,
                Token::Whitespace,
                Token::Number,
                Token::Newline,
                Token::Text,
                Token::Whitespace,
                Token::Number
            ]
        );
    }

    #[test]
    fn test_complex_tokenization() {
        let input = "1. Session Title\n    - Item 1\n    - Item 2";
        let tokens = tokenize(input);

        // Expected tokens for "1. Session Title"
        assert_eq!(tokens[0], Token::Number); // "1"
        assert_eq!(tokens[1], Token::Period); // "."
        assert_eq!(tokens[2], Token::Whitespace); // " "
        assert_eq!(tokens[3], Token::Text); // "Session"
        assert_eq!(tokens[4], Token::Whitespace); // " "
        assert_eq!(tokens[5], Token::Text); // "Title"
        assert_eq!(tokens[6], Token::Newline); // "\n"

        // Expected tokens for "    - Item 1"
        assert_eq!(tokens[7], Token::Indent); // "    "
        assert_eq!(tokens[8], Token::Dash); // "-"
        assert_eq!(tokens[9], Token::Whitespace); // " "
        assert_eq!(tokens[10], Token::Text); // "Item"
        assert_eq!(tokens[11], Token::Whitespace); // " "
        assert_eq!(tokens[12], Token::Number); // "1"
        assert_eq!(tokens[13], Token::Newline); // "\n"

        // Expected tokens for "    - Item 2"
        assert_eq!(tokens[14], Token::Indent); // "    "
        assert_eq!(tokens[15], Token::Dash); // "-"
        assert_eq!(tokens[16], Token::Whitespace); // " "
        assert_eq!(tokens[17], Token::Text); // "Item"
        assert_eq!(tokens[18], Token::Whitespace); // " "
        assert_eq!(tokens[19], Token::Number); // "2"
    }

    #[test]
    fn test_tokenize_with_spans() {
        let tokens_with_spans = tokenize_with_spans("hello world");
        assert_eq!(tokens_with_spans.len(), 3);

        // Check that tokens are correct (spans are not preserved in current implementation)
        assert_eq!(tokens_with_spans[0].0, Token::Text);
        assert_eq!(tokens_with_spans[1].0, Token::Whitespace);
        assert_eq!(tokens_with_spans[2].0, Token::Text);
    }

    #[test]
    fn test_empty_input() {
        let tokens = tokenize("");
        assert_eq!(tokens, vec![]);
    }

    #[test]
    fn test_whitespace_only() {
        let tokens = tokenize("   \t  ");
        // Expected: 3 spaces -> Whitespace, 1 tab -> Indent, 2 spaces -> Whitespace
        assert_eq!(
            tokens,
            vec![Token::Whitespace, Token::Indent, Token::Whitespace]
        );
    }

    #[test]
    fn test_newline_only() {
        let tokens = tokenize("\n");
        assert_eq!(tokens, vec![Token::Newline]);
    }

    #[test]
    fn test_sequence_markers() {
        let tokens = tokenize("(1) a. -");
        assert_eq!(
            tokens,
            vec![
                Token::OpenParen,
                Token::Number,
                Token::CloseParen,
                Token::Whitespace,
                Token::Text,
                Token::Period,
                Token::Whitespace,
                Token::Dash
            ]
        );
    }
}
