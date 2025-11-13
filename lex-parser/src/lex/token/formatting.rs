//! Detokenizer for the lex format
//!
//! This module provides functionality to convert a stream of tokens back into a string.
//!
//! Unlike other formatters in this module which work on AST `Document` objects,
//! the detokenizer works at the token level, converting token streams back to
//! source text. This is useful for:
//!
//! - Round-trip testing (source -> tokens -> source)
//! - Token-level transformations that preserve the original format
//! - Debugging and visualization of token streams
//!
//! The detokenizer handles:
//! - Raw tokens (basic token -> string conversion)
//! - Semantic indentation tokens (Indent/Dedent) for proper formatting

use super::core::Token;

/// Trait for converting a token to its string representation
pub trait ToLexString {
    fn to_lex_string(&self) -> String;
}

impl ToLexString for Token {
    fn to_lex_string(&self) -> String {
        match self {
            Token::LexMarker => "::".to_string(),
            Token::Indentation => "    ".to_string(),
            Token::Whitespace => " ".to_string(),
            // BlankLine should always contain the newline character(s) for round-trip fidelity.
            // The logos regex always produces Some(...), but we default to "\n" for safety.
            Token::BlankLine(s) => s.as_deref().unwrap_or("\n").to_string(),
            Token::Dash => "-".to_string(),
            Token::Period => ".".to_string(),
            Token::OpenParen => "(".to_string(),
            Token::CloseParen => ")".to_string(),
            Token::Colon => ":".to_string(),
            Token::ExclamationMark => "!".to_string(),
            Token::QuestionMark => "?".to_string(),
            Token::Semicolon => ";".to_string(),
            Token::InvertedExclamationMark => "¡".to_string(),
            Token::InvertedQuestionMark => "¿".to_string(),
            Token::Ellipsis => "…".to_string(),
            Token::IdeographicFullStop => "。".to_string(),
            Token::FullwidthExclamationMark => "！".to_string(),
            Token::FullwidthQuestionMark => "？".to_string(),
            Token::ExclamationQuestionMark => "⁉".to_string(),
            Token::QuestionExclamationMark => "⁈".to_string(),
            Token::ArabicQuestionMark => "؟".to_string(),
            Token::ArabicFullStop => "۔".to_string(),
            Token::ArabicTripleDot => "؍".to_string(),
            Token::ArabicComma => "،".to_string(),
            Token::Danda => "।".to_string(),
            Token::DoubleDanda => "॥".to_string(),
            Token::BengaliCurrencyNumeratorFour => "৷".to_string(),
            Token::EthiopianFullStop => "።".to_string(),
            Token::ArmenianFullStop => "։".to_string(),
            Token::TibetanShad => "།".to_string(),
            Token::ThaiFongman => "๏".to_string(),
            Token::MyanmarComma => "၊".to_string(),
            Token::MyanmarFullStop => "။".to_string(),
            Token::Comma => ",".to_string(),
            Token::Quote => "\"".to_string(),
            Token::Equals => "=".to_string(),
            Token::Number(s) => s.clone(),
            Token::Text(s) => s.clone(),
            // The following tokens are synthetic and should not be part of the detokenized output
            Token::Indent(_) | Token::Dedent(_) => String::new(),
        }
    }
}

/// Detokenize a stream of tokens into a string
///
/// This function converts a sequence of tokens back to source text,
/// handling semantic indentation (Indent/Dedent tokens) to reconstruct
/// the proper indentation structure.
///
/// # Arguments
///
/// * `tokens` - Slice of tokens to detokenize
///
/// # Returns
///
/// A string representation of the tokens with proper indentation
///
/// # Examples
///
/// ```ignore
/// use lex::lex::formats::detokenizer::detokenize;
/// use lex::lex::lexing::tokenize;
///
/// let source = "Hello world";
/// let tokens: Vec<_> = tokenize(source).into_iter().map(|(t, _)| t).collect();
/// let result = detokenize(&tokens);
/// assert_eq!(result, source);
/// ```
pub fn detokenize(tokens: &[Token]) -> String {
    let mut result = String::new();
    let mut indent_level = 0;

    for token in tokens {
        match token {
            Token::Indent(_) => indent_level += 1,
            Token::Dedent(_) => indent_level -= 1,
            Token::BlankLine(_) => {
                result.push_str(&token.to_lex_string());
            }
            _ => {
                if result.ends_with('\n') || result.is_empty() {
                    for _ in 0..indent_level {
                        result.push_str("    ");
                    }
                }
                result.push_str(&token.to_lex_string());
            }
        }
    }

    result
}
