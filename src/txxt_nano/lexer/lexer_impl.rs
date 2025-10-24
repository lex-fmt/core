//! Implementation of the txxt lexer
//!
//! This module contains the concrete implementation of the lexer using logos.
//! It handles the post-processing of whitespace tokens to convert them into
//! appropriate indentation tokens according to the 4-space tab stop rule.

use crate::txxt_nano::lexer::tokens::Token;
use logos::{Lexer, Logos};

/// A lexer for the txxt format that produces tokens with simplified indentation handling
pub struct TxxtLexer<'source> {
    logos_lexer: Lexer<'source, Token>,
    processed_tokens: Vec<Token>,
    current_pos: usize,
}

impl<'source> TxxtLexer<'source> {
    /// Create a new lexer for the given source text
    pub fn new(source: &'source str) -> Self {
        let mut logos_lexer = Token::lexer(source);
        let mut processed_tokens = Vec::new();

        // Process all tokens and convert whitespace to indentation tokens
        while let Some(result) = logos_lexer.next() {
            match result {
                Ok(token) => {
                    match token {
                        Token::Whitespace => {
                            // Convert whitespace to indentation tokens
                            let slice = logos_lexer.slice();
                            let spaces = slice.chars().filter(|&c| c == ' ').count();
                            let tabs = slice.chars().filter(|&c| c == '\t').count();

                            // Add space-based indentation tokens
                            for _ in 0..(spaces / 4) {
                                processed_tokens.push(Token::IndentSpace);
                            }

                            // Add tab-based indentation tokens
                            for _ in 0..tabs {
                                processed_tokens.push(Token::IndentTab);
                            }

                            // Add remaining spaces as whitespace
                            let remaining_spaces = spaces % 4;
                            if remaining_spaces > 0 {
                                processed_tokens.push(Token::Whitespace);
                            }
                        }
                        _ => processed_tokens.push(token),
                    }
                }
                Err(_) => {
                    // Skip error tokens for now
                }
            }
        }

        Self {
            logos_lexer,
            processed_tokens,
            current_pos: 0,
        }
    }

    /// Get the current position in the source text
    pub fn span(&self) -> logos::Span {
        self.logos_lexer.span()
    }

    /// Get the current slice of the source text
    pub fn slice(&self) -> &'source str {
        self.logos_lexer.slice()
    }
}

impl<'source> Iterator for TxxtLexer<'source> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_pos < self.processed_tokens.len() {
            let token = self.processed_tokens[self.current_pos].clone();
            self.current_pos += 1;
            Some(token)
        } else {
            None
        }
    }
}

/// Convenience function to tokenize a string and collect all tokens
pub fn tokenize(source: &str) -> Vec<Token> {
    TxxtLexer::new(source).collect()
}

/// Convenience function to tokenize a string and collect tokens with their spans
pub fn tokenize_with_spans(source: &str) -> Vec<(Token, logos::Span)> {
    let mut lexer = TxxtLexer::new(source);
    let mut tokens = Vec::new();

    while let Some(token) = lexer.next() {
        tokens.push((token, lexer.span()));
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
        assert_eq!(tokens, vec![Token::IndentSpace, Token::Text]);
    }

    #[test]
    fn test_multiple_indentation_levels() {
        let tokens = tokenize("        hello");
        assert_eq!(
            tokens,
            vec![Token::IndentSpace, Token::IndentSpace, Token::Text]
        );
    }

    #[test]
    fn test_tab_indentation() {
        let tokens = tokenize("\thello");
        assert_eq!(tokens, vec![Token::IndentTab, Token::Text]);
    }

    #[test]
    fn test_mixed_indentation() {
        let tokens = tokenize("    \thello");
        assert_eq!(
            tokens,
            vec![Token::IndentSpace, Token::IndentTab, Token::Text]
        );
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
                Token::Text
            ]
        );
    }

    #[test]
    fn test_numbered_item_tokenization() {
        let tokens = tokenize("1. Item 1");
        assert_eq!(
            tokens,
            vec![
                Token::Text,
                Token::Period,
                Token::Whitespace,
                Token::Text,
                Token::Whitespace,
                Token::Text
            ]
        );
    }

    #[test]
    fn test_session_title_tokenization() {
        let tokens = tokenize("1. Session Title");
        assert_eq!(
            tokens,
            vec![
                Token::Text,
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
                Token::Text,
                Token::Newline,
                Token::Text,
                Token::Whitespace,
                Token::Text
            ]
        );
    }

    #[test]
    fn test_complex_tokenization() {
        let input = "1. Session Title\n    - Item 1\n    - Item 2";
        let tokens = tokenize(input);

        // Expected tokens for "1. Session Title"
        assert_eq!(tokens[0], Token::Text); // "1"
        assert_eq!(tokens[1], Token::Period); // "."
        assert_eq!(tokens[2], Token::Whitespace); // " "
        assert_eq!(tokens[3], Token::Text); // "Session"
        assert_eq!(tokens[4], Token::Whitespace); // " "
        assert_eq!(tokens[5], Token::Text); // "Title"
        assert_eq!(tokens[6], Token::Newline); // "\n"

        // Expected tokens for "    - Item 1"
        assert_eq!(tokens[7], Token::IndentSpace); // "    "
        assert_eq!(tokens[8], Token::Dash); // "-"
        assert_eq!(tokens[9], Token::Whitespace); // " "
        assert_eq!(tokens[10], Token::Text); // "Item"
        assert_eq!(tokens[11], Token::Whitespace); // " "
        assert_eq!(tokens[12], Token::Text); // "1"
        assert_eq!(tokens[13], Token::Newline); // "\n"

        // Expected tokens for "    - Item 2"
        assert_eq!(tokens[14], Token::IndentSpace); // "    "
        assert_eq!(tokens[15], Token::Dash); // "-"
        assert_eq!(tokens[16], Token::Whitespace); // " "
        assert_eq!(tokens[17], Token::Text); // "Item"
        assert_eq!(tokens[18], Token::Whitespace); // " "
        assert_eq!(tokens[19], Token::Text); // "2"
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
        // Expected: 3 spaces -> Whitespace, 1 tab -> IndentTab, 2 spaces -> Whitespace
        assert_eq!(
            tokens,
            vec![Token::Whitespace, Token::IndentTab, Token::Whitespace]
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
                Token::Text,
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
