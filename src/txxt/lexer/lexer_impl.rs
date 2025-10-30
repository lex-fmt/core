//! Core tokenization implementation for the txxt lexer
//!
//! This module provides the raw tokenization using the logos lexer library.
//! The actual tokenization is handled entirely by logos. Additional transformations
//! (whitespace processing, indentation transformation, blank line transformation)
//! are applied by the transformation pipeline in the transformations module.

use crate::txxt::lexer::tokens::Token;
use logos::Logos;

/// Tokenize source code with location information
///
/// This function performs raw tokenization using the logos lexer, returning tokens
/// paired with their source locations. Additional transformations (whitespace processing,
/// indentation handling, blank line handling) should be applied by the caller.
pub fn tokenize_with_locations(source: &str) -> Vec<(Token, logos::Span)> {
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
    fn test_tokenize_with_locations() {
        let tokens_with_locations = tokenize_with_locations("hello world");
        assert_eq!(tokens_with_locations.len(), 3);

        // Check that tokens are correct
        assert_eq!(tokens_with_locations[0].0, Token::Text("hello".to_string()));
        assert_eq!(tokens_with_locations[1].0, Token::Whitespace);
        assert_eq!(tokens_with_locations[2].0, Token::Text("world".to_string()));
    }

    #[test]
    fn test_empty_input() {
        let tokens_with_locations = tokenize_with_locations("");
        assert_eq!(tokens_with_locations, vec![]);
    }

    #[test]
    fn test_complex_tokenization() {
        let input = "1. Session Title\n    - Item 1\n    - Item 2";
        let tokens_with_locations = tokenize_with_locations(input);

        // Expected tokens for "1. Session Title"
        assert_eq!(tokens_with_locations[0].0, Token::Number("1".to_string())); // "1"
        assert_eq!(tokens_with_locations[1].0, Token::Period); // "."
        assert_eq!(tokens_with_locations[2].0, Token::Whitespace); // " "
        assert_eq!(
            tokens_with_locations[3].0,
            Token::Text("Session".to_string())
        ); // "Session"
        assert_eq!(tokens_with_locations[4].0, Token::Whitespace); // " "
        assert_eq!(tokens_with_locations[5].0, Token::Text("Title".to_string())); // "Title"
        assert_eq!(tokens_with_locations[6].0, Token::Newline); // "\n"

        // Expected tokens for "    - Item 1"
        assert_eq!(tokens_with_locations[7].0, Token::Indent); // "    "
        assert_eq!(tokens_with_locations[8].0, Token::Dash); // "-"
        assert_eq!(tokens_with_locations[9].0, Token::Whitespace); // " "
        assert_eq!(tokens_with_locations[10].0, Token::Text("Item".to_string())); // "Item"
        assert_eq!(tokens_with_locations[11].0, Token::Whitespace); // " "
        assert_eq!(tokens_with_locations[12].0, Token::Number("1".to_string())); // "1"
        assert_eq!(tokens_with_locations[13].0, Token::Newline); // "\n"

        // Expected tokens for "    - Item 2"
        assert_eq!(tokens_with_locations[14].0, Token::Indent); // "    "
        assert_eq!(tokens_with_locations[15].0, Token::Dash); // "-"
        assert_eq!(tokens_with_locations[16].0, Token::Whitespace); // " "
        assert_eq!(tokens_with_locations[17].0, Token::Text("Item".to_string())); // "Item"
        assert_eq!(tokens_with_locations[18].0, Token::Whitespace); // " "
        assert_eq!(tokens_with_locations[19].0, Token::Number("2".to_string()));
        // "2"
    }

    #[test]
    fn test_whitespace_only() {
        let tokens_with_locations = tokenize_with_locations("   \t  ");
        // Expected: 3 spaces -> Whitespace, 1 tab -> Indent, 2 spaces -> Whitespace
        assert_eq!(tokens_with_locations.len(), 3);
        assert_eq!(tokens_with_locations[0].0, Token::Whitespace);
        assert_eq!(tokens_with_locations[1].0, Token::Indent);
        assert_eq!(tokens_with_locations[2].0, Token::Whitespace);
    }
}
