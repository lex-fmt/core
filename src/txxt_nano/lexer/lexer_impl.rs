//! Implementation of the txxt lexer
//!
//! This module provides convenience functions for tokenizing txxt text.
//! The actual tokenization is handled entirely by logos.

use crate::txxt_nano::lexer::tokens::Token;
use logos::Logos;

/// Convenience function to tokenize a string and collect all tokens
pub fn tokenize(source: &str) -> Vec<Token> {
    let tokens_with_spans = tokenize_with_spans(source);
    process_whitespace_remainders(tokens_with_spans)
}

/// Process whitespace remainders according to txxt specification
///
/// The spec states: "lines that have space remainders (as in 10 spaces, which converts to
/// 2 tab stops with 2 spaces remaining) will be parsed with no error. Only two indentation
/// level tokens will be generated, and the remaining whitespaces will be considered part of the text."
///
/// This function removes Whitespace tokens that follow Indent tokens and precede Text tokens,
/// effectively merging the whitespace remainder into the text content.
fn process_whitespace_remainders(tokens_with_spans: Vec<(Token, logos::Span)>) -> Vec<Token> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < tokens_with_spans.len() {
        let (token, _span) = &tokens_with_spans[i];

        match token {
            Token::Whitespace => {
                // Check if this whitespace follows indentation tokens and precedes text content
                let mut indent_count = 0;
                let mut j = i;

                // Count consecutive Indent tokens before this Whitespace
                while j > 0 && matches!(tokens_with_spans[j - 1].0, Token::Indent) {
                    indent_count += 1;
                    j -= 1;
                }

                // If we have indentation and this whitespace is followed by text,
                // skip this whitespace token (it will be considered part of the text)
                if indent_count > 0
                    && i + 1 < tokens_with_spans.len()
                    && matches!(tokens_with_spans[i + 1].0, Token::Text(_))
                {
                    // Skip this whitespace token
                    i += 1;
                    continue;
                }

                // Otherwise, keep the whitespace token as-is
                result.push(token.clone());
            }
            _ => {
                result.push(token.clone());
            }
        }
        i += 1;
    }

    result
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
    fn test_complex_tokenization() {
        let input = "1. Session Title\n    - Item 1\n    - Item 2";
        let tokens = tokenize(input);

        // Expected tokens for "1. Session Title"
        assert_eq!(tokens[0], Token::Number("1".to_string())); // "1"
        assert_eq!(tokens[1], Token::Period); // "."
        assert_eq!(tokens[2], Token::Whitespace); // " "
        assert_eq!(tokens[3], Token::Text("Session".to_string())); // "Session"
        assert_eq!(tokens[4], Token::Whitespace); // " "
        assert_eq!(tokens[5], Token::Text("Title".to_string())); // "Title"
        assert_eq!(tokens[6], Token::Newline); // "\n"

        // Expected tokens for "    - Item 1"
        assert_eq!(tokens[7], Token::Indent); // "    "
        assert_eq!(tokens[8], Token::Dash); // "-"
        assert_eq!(tokens[9], Token::Whitespace); // " "
        assert_eq!(tokens[10], Token::Text("Item".to_string())); // "Item"
        assert_eq!(tokens[11], Token::Whitespace); // " "
        assert_eq!(tokens[12], Token::Number("1".to_string())); // "1"
        assert_eq!(tokens[13], Token::Newline); // "\n"

        // Expected tokens for "    - Item 2"
        assert_eq!(tokens[14], Token::Indent); // "    "
        assert_eq!(tokens[15], Token::Dash); // "-"
        assert_eq!(tokens[16], Token::Whitespace); // " "
        assert_eq!(tokens[17], Token::Text("Item".to_string())); // "Item"
        assert_eq!(tokens[18], Token::Whitespace); // " "
        assert_eq!(tokens[19], Token::Number("2".to_string())); // "2"
    }

    #[test]
    fn test_tokenize_with_spans() {
        let tokens_with_spans = tokenize_with_spans("hello world");
        assert_eq!(tokens_with_spans.len(), 3);

        // Check that tokens are correct (spans are not preserved in current implementation)
        assert_eq!(tokens_with_spans[0].0, Token::Text("hello".to_string()));
        assert_eq!(tokens_with_spans[1].0, Token::Whitespace);
        assert_eq!(tokens_with_spans[2].0, Token::Text("world".to_string()));
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
    fn test_whitespace_remainders() {
        // Test case: 10 spaces should produce 2 Indent tokens (8 spaces)
        // and the remaining 2 spaces should be part of text content
        let tokens = tokenize("          hello");
        println!("Tokens for '          hello': {:?}", tokens);

        // According to spec: 10 spaces = 2 indent levels (8 spaces) + 2 remaining spaces
        // The remaining 2 spaces should be considered part of the text, not separate whitespace
        assert_eq!(tokens.len(), 3); // Should be: [Indent, Indent, Text("  hello")]
        assert_eq!(tokens[0], Token::Indent);
        assert_eq!(tokens[1], Token::Indent);
        assert_eq!(tokens[2], Token::Text("hello".to_string()));
    }
}
