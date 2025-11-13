//! Label Data Extraction
//!
//! Extracts label tokens from annotation headers.
//! Labels appear before parameters and are not followed by '=' signs.

use crate::lex::token::Token;
use std::ops::Range as ByteRange;

/// Parse label from tokens.
///
/// Identifies which tokens belong to the label by finding tokens that are:
/// - Text, Dash, Number, or Period tokens
/// - NOT followed by an Equals sign (which would make them part of a parameter)
///
/// Returns the label tokens and the index where label ends.
pub(super) fn parse_label_tokens(
    tokens: &[(Token, ByteRange<usize>)],
) -> (Vec<(Token, ByteRange<usize>)>, usize) {
    let mut label_tokens = Vec::new();
    let mut i = 0;

    // Skip leading whitespace
    while i < tokens.len() && matches!(tokens[i].0, Token::Whitespace) {
        i += 1;
    }

    // Collect label tokens until we hit '=' or end
    while i < tokens.len() {
        match &tokens[i].0 {
            Token::Text(_) | Token::Dash | Token::Number(_) | Token::Period => {
                // Check if this sequence of key-like tokens is followed by '='
                // Need to scan ahead past all Text/Dash/Number/Period tokens
                let mut check_idx = i + 1;

                // Skip any remaining key-like tokens (for multi-token keys like "key1" = "key" + "1")
                while check_idx < tokens.len() {
                    if matches!(
                        tokens[check_idx].0,
                        Token::Text(_) | Token::Dash | Token::Number(_) | Token::Period
                    ) {
                        check_idx += 1;
                    } else {
                        break;
                    }
                }

                // Now skip whitespace
                while check_idx < tokens.len() && matches!(tokens[check_idx].0, Token::Whitespace) {
                    check_idx += 1;
                }

                // Check if we found '='
                if check_idx < tokens.len() && matches!(tokens[check_idx].0, Token::Equals) {
                    // This is the start of parameters, stop label collection
                    break;
                }

                label_tokens.push(tokens[i].clone());
                i += 1;
            }
            Token::Whitespace => {
                // Include whitespace in label
                label_tokens.push(tokens[i].clone());
                i += 1;
            }
            _ => {
                // Hit a non-label token, stop
                break;
            }
        }
    }

    (label_tokens, i)
}
