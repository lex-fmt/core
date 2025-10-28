//! Detokenizer for the txxt format
//!
//! This module provides functionality to convert a stream of tokens back into a string.
use crate::txxt_nano::lexer::tokens::Token;

impl ToTxxtString for Token {
    fn to_txxt_string(&self) -> String {
        match self {
            Token::TxxtMarker => "::".to_string(),
            Token::Indent => "    ".to_string(),
            Token::Whitespace => " ".to_string(),
            Token::Newline => "\n".to_string(),
            Token::Dash => "-".to_string(),
            Token::Period => ".".to_string(),
            Token::OpenParen => "(".to_string(),
            Token::CloseParen => ")".to_string(),
            Token::Colon => ":".to_string(),
            Token::Comma => ",".to_string(),
            Token::Quote => "\"".to_string(),
            Token::Equals => "=".to_string(),
            Token::Number(s) => s.clone(),
            Token::Text(s) => s.clone(),
            // The following tokens are synthetic and should not be part of the detokenized output
            Token::IndentLevel | Token::DedentLevel | Token::DocStart | Token::DocEnd => {
                String::new()
            }
        }
    }
}

/// Trait for converting a token to its string representation
pub trait ToTxxtString {
    fn to_txxt_string(&self) -> String;
}

/// Detokenize a stream of tokens into a string
pub fn detokenize(tokens: &[Token]) -> String {
    let mut result = String::new();
    let mut indent_level = 0;

    for token in tokens {
        match token {
            Token::IndentLevel => indent_level += 1,
            Token::DedentLevel => indent_level -= 1,
            Token::Newline => {
                result.push('\n');
            }
            _ => {
                if result.ends_with('\n') || result.is_empty() {
                    for _ in 0..indent_level {
                        result.push_str("    ");
                    }
                }
                result.push_str(&token.to_txxt_string());
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::txxt_nano::lexer::tokenize;

    #[test]
    fn test_detokenize_simple_paragraph() {
        let source = "Simple Paragraphs Test {{paragraph}}";
        let tokens = tokenize(source);
        let detokenized = detokenize(&tokens);
        assert_eq!(detokenized, source);
    }

    #[test]
    fn test_detokenize_multiline_paragraph() {
        let source = "This is a multi-line paragraph.\nIt continues on the second line.\nAnd even has a third line. {{paragraph}}";
        let tokens = tokenize(source);
        let detokenized = detokenize(&tokens);
        assert_eq!(detokenized, source);
    }

    #[test]
    fn test_detokenize_simple_list() {
        let source = "- First item {{list-item}}\n- Second item {{list-item}}\n- Third item {{list-item}}";
        let tokens = tokenize(source);
        let detokenized = detokenize(&tokens);
        assert_eq!(detokenized, source);
    }

    #[test]
    fn test_detokenize_session() {
        let source = "1. Introduction {{session-title}}\n\n    This is the content of the session. It contains a paragraph that is indented relative to the session title. {{paragraph}}";
        let tokens = tokenize(source);
        let detokenized = detokenize(&tokens);
        assert_eq!(detokenized, source);
    }

    #[test]
    fn test_detokenize_with_indentation() {
        let source = "1. Session\n    - Item 1\n        - Nested Item\n    - Item 2";
        let raw_tokens = tokenize(source);
        let tokens = crate::txxt_nano::lexer::indentation_transform::transform_indentation(raw_tokens);
        let detokenized = detokenize(&tokens);
        assert_eq!(detokenized, source);
    }
}
