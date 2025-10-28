//! Helper functions for parser conversions
//!
//! Common patterns extracted from conversion logic to reduce duplication.

use crate::txxt_nano::lexer::Token;

/// Check if a token is a text-like token (content that can appear in lines)
///
/// This includes: Text, Whitespace, Numbers, Punctuation, and common symbols
pub(crate) fn is_text_token(token: &Token) -> bool {
    matches!(
        token,
        Token::Text(_)
            | Token::Whitespace
            | Token::Number(_)
            | Token::Dash
            | Token::Period
            | Token::OpenParen
            | Token::CloseParen
            | Token::Colon
            | Token::Comma
            | Token::Quote
            | Token::Equals
    )
}
