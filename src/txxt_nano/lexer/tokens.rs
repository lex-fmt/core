//! Token definitions for the txxt format
//!
//! This module defines all the tokens that can be produced by the txxt lexer.
//! The tokens are defined using the logos derive macro for efficient tokenization.

use logos::Logos;

/// All possible tokens in the txxt format
#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    // Special markers
    #[token("::")]
    TxxtMarker,

    // Indentation (simplified - one token per 4 spaces or tab)
    #[regex(r"    ", priority = 2)] // Exactly 4 spaces
    IndentSpace,
    #[regex(r"\t", priority = 2)] // Single tab
    IndentTab,

    // Line breaks
    #[token("\n")]
    Newline,

    // Sequence markers
    #[token("-")]
    Dash,
    #[token(".")]
    Period,
    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,
    #[token(":")]
    Colon,

    // Whitespace (excluding newlines and indentation)
    #[regex(r"[ ]+", priority = 1)] // Only spaces, lower priority than indentation
    Whitespace,

    // Text content (catch-all for non-special characters)
    #[regex(r"[^\s\n\t\-\.\(\):]+")]
    Text,
}

impl Token {
    /// Check if this token represents indentation
    pub fn is_indent(&self) -> bool {
        matches!(self, Token::IndentSpace | Token::IndentTab)
    }

    /// Check if this token is whitespace (including indentation)
    pub fn is_whitespace(&self) -> bool {
        matches!(
            self,
            Token::IndentSpace | Token::IndentTab | Token::Whitespace | Token::Newline
        )
    }

    /// Check if this token is a sequence marker
    pub fn is_sequence_marker(&self) -> bool {
        matches!(
            self,
            Token::Dash | Token::Period | Token::OpenParen | Token::CloseParen
        )
    }

    /// Check if this token is text content
    pub fn is_text(&self) -> bool {
        matches!(self, Token::Text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::txxt_nano::lexer::TxxtLexer;

    #[test]
    fn test_txxt_marker() {
        let mut lexer = TxxtLexer::new("::");
        assert_eq!(lexer.next(), Some(Token::TxxtMarker));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn test_indentation_tokens() {
        // Test 4 spaces
        let mut lexer = TxxtLexer::new("    ");
        let token = lexer.next();
        println!("Token for '    ': {:?}", token);
        assert_eq!(token, Some(Token::IndentSpace));
        assert_eq!(lexer.next(), None);

        // Test tab
        let mut lexer = TxxtLexer::new("\t");
        let token = lexer.next();
        println!("Token for '\\t': {:?}", token);
        assert_eq!(token, Some(Token::IndentTab));
        assert_eq!(lexer.next(), None);

        // Test multiple indent levels
        let mut lexer = TxxtLexer::new("        "); // 8 spaces = 2 indent levels
        let token1 = lexer.next();
        let token2 = lexer.next();
        println!("Tokens for '        ': {:?}, {:?}", token1, token2);
        assert_eq!(token1, Some(Token::IndentSpace));
        assert_eq!(token2, Some(Token::IndentSpace));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn test_sequence_markers() {
        let mut lexer = TxxtLexer::new("- . ( ) :");
        assert_eq!(lexer.next(), Some(Token::Dash));
        assert_eq!(lexer.next(), Some(Token::Whitespace));
        assert_eq!(lexer.next(), Some(Token::Period));
        assert_eq!(lexer.next(), Some(Token::Whitespace));
        assert_eq!(lexer.next(), Some(Token::OpenParen));
        assert_eq!(lexer.next(), Some(Token::Whitespace));
        assert_eq!(lexer.next(), Some(Token::CloseParen));
        assert_eq!(lexer.next(), Some(Token::Whitespace));
        assert_eq!(lexer.next(), Some(Token::Colon));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn test_text_tokens() {
        let mut lexer = TxxtLexer::new("hello world");
        assert_eq!(lexer.next(), Some(Token::Text));
        assert_eq!(lexer.next(), Some(Token::Whitespace));
        assert_eq!(lexer.next(), Some(Token::Text));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn test_newline_token() {
        let mut lexer = TxxtLexer::new("\n");
        assert_eq!(lexer.next(), Some(Token::Newline));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn test_mixed_content() {
        let mut lexer = TxxtLexer::new("1. Hello world\n    - Item 1");
        assert_eq!(lexer.next(), Some(Token::Text)); // "1"
        assert_eq!(lexer.next(), Some(Token::Period));
        assert_eq!(lexer.next(), Some(Token::Whitespace));
        assert_eq!(lexer.next(), Some(Token::Text)); // "Hello"
        assert_eq!(lexer.next(), Some(Token::Whitespace));
        assert_eq!(lexer.next(), Some(Token::Text)); // "world"
        assert_eq!(lexer.next(), Some(Token::Newline));
        assert_eq!(lexer.next(), Some(Token::IndentSpace));
        assert_eq!(lexer.next(), Some(Token::Dash));
        assert_eq!(lexer.next(), Some(Token::Whitespace));
        assert_eq!(lexer.next(), Some(Token::Text)); // "Item"
        assert_eq!(lexer.next(), Some(Token::Whitespace));
        assert_eq!(lexer.next(), Some(Token::Text)); // "1"
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn test_token_predicates() {
        assert!(Token::IndentSpace.is_indent());
        assert!(Token::IndentTab.is_indent());
        assert!(!Token::Text.is_indent());

        assert!(Token::IndentSpace.is_whitespace());
        assert!(Token::Whitespace.is_whitespace());
        assert!(Token::Newline.is_whitespace());
        assert!(!Token::Text.is_whitespace());

        assert!(Token::Dash.is_sequence_marker());
        assert!(Token::Period.is_sequence_marker());
        assert!(!Token::Text.is_sequence_marker());

        assert!(Token::Text.is_text());
        assert!(!Token::Dash.is_text());
    }
}
