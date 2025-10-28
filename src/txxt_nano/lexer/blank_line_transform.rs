//! Blank line transformation for txxt lexer
//!
//! This module transforms sequences of consecutive Newline tokens into semantic BlankLine tokens.
//! A blank line is defined as two or more consecutive Newline tokens.
//!
//! This transformation should be applied AFTER indentation transformation, so that the
//! indentation levels are properly established before we identify blank lines.

use crate::txxt_nano::lexer::tokens::Token;

/// Transform consecutive Newline tokens into BlankLine tokens
///
/// This function processes a token stream and converts sequences of 2 or more
/// consecutive Newline tokens into a Newline token followed by a BlankLine token.
/// A single Newline token is kept as-is.
///
/// # Algorithm
///
/// 1. Iterate through the token stream
/// 2. When we encounter a Newline token, count consecutive Newlines
/// 3. If there's 1 Newline: emit it as-is
/// 4. If there are 2+ Newlines: emit 1 Newline (to end the current line), then 1 BlankLine
/// 5. Preserve all other tokens unchanged
///
/// # Example
///
/// Input tokens: `[Text, Newline, Newline, Text, Newline]`
/// Output tokens: `[Text, Newline, BlankLine, Text, Newline]`
pub fn transform_blank_lines(tokens: Vec<Token>) -> Vec<Token> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        if matches!(tokens[i], Token::Newline) {
            // Count consecutive Newline tokens
            let mut newline_count = 0;
            let mut j = i;
            while j < tokens.len() && matches!(tokens[j], Token::Newline) {
                newline_count += 1;
                j += 1;
            }

            // Emit the first Newline (ends the current line)
            result.push(Token::Newline);

            // If we have 2+ consecutive newlines, also emit a BlankLine token
            // This represents the blank line(s) between block elements
            if newline_count >= 2 {
                result.push(Token::BlankLine);
            }

            // Move past all the newlines we just processed
            i = j;
        } else {
            // Non-newline token, just copy it
            result.push(tokens[i].clone());
            i += 1;
        }
    }

    result
}

/// Transform blank lines while preserving source spans
/// Blank lines (sequences of 2+ newlines) become Newline followed by BlankLine token
/// The BlankLine token gets an empty span (0..0) since it's synthetic
pub fn transform_blank_lines_with_spans(
    tokens_with_spans: Vec<(Token, std::ops::Range<usize>)>,
) -> Vec<(Token, std::ops::Range<usize>)> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < tokens_with_spans.len() {
        if matches!(tokens_with_spans[i].0, Token::Newline) {
            // Count consecutive Newline tokens
            let mut newline_count = 0;
            let mut j = i;
            while j < tokens_with_spans.len() && matches!(tokens_with_spans[j].0, Token::Newline) {
                newline_count += 1;
                j += 1;
            }

            // Emit the first Newline with its original span (ends the current line)
            result.push((Token::Newline, tokens_with_spans[i].1.clone()));

            // If we have 2+ consecutive newlines, also emit a BlankLine token with empty span
            if newline_count >= 2 {
                result.push((Token::BlankLine, 0..0));
            }

            // Move past all the newlines we just processed
            i = j;
        } else {
            // Non-newline token, just copy it with its span
            result.push(tokens_with_spans[i].clone());
            i += 1;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_newline_unchanged() {
        let input = vec![
            Token::Text("a".to_string()),
            Token::Newline,
            Token::Text("b".to_string()),
        ];
        let result = transform_blank_lines(input);
        assert_eq!(
            result,
            vec![
                Token::Text("a".to_string()),
                Token::Newline,
                Token::Text("b".to_string()),
            ]
        );
    }

    #[test]
    fn test_double_newline_becomes_newline_then_blank_line() {
        let input = vec![
            Token::Text("t".to_string()),
            Token::Newline,
            Token::Newline,
            Token::Text("t".to_string()),
        ];
        let result = transform_blank_lines(input);
        assert_eq!(
            result,
            vec![
                Token::Text("t".to_string()),
                Token::Newline,
                Token::BlankLine,
                Token::Text("t".to_string())
            ]
        );
    }

    #[test]
    fn test_triple_newline_becomes_newline_then_blank_line() {
        let input = vec![
            Token::Text("t".to_string()),
            Token::Newline,
            Token::Newline,
            Token::Newline,
            Token::Text("t".to_string()),
        ];
        let result = transform_blank_lines(input);
        assert_eq!(
            result,
            vec![
                Token::Text("t".to_string()),
                Token::Newline,
                Token::BlankLine,
                Token::Text("t".to_string())
            ]
        );
    }

    #[test]
    fn test_multiple_blank_lines() {
        let input = vec![
            Token::Text("t".to_string()),
            Token::Newline,
            Token::Newline,
            Token::Text("t".to_string()),
            Token::Newline,
            Token::Newline,
            Token::Newline,
            Token::Text("t".to_string()),
        ];
        let result = transform_blank_lines(input);
        assert_eq!(
            result,
            vec![
                Token::Text("t".to_string()),
                Token::Newline,
                Token::BlankLine,
                Token::Text("t".to_string()),
                Token::Newline,
                Token::BlankLine,
                Token::Text("t".to_string())
            ]
        );
    }

    #[test]
    fn test_blank_line_at_end() {
        let input = vec![Token::Text("t".to_string()), Token::Newline, Token::Newline];
        let result = transform_blank_lines(input);
        assert_eq!(
            result,
            vec![
                Token::Text("t".to_string()),
                Token::Newline,
                Token::BlankLine
            ]
        );
    }

    #[test]
    fn test_blank_line_at_start() {
        let input = vec![Token::Newline, Token::Newline, Token::Text("t".to_string())];
        let result = transform_blank_lines(input);
        assert_eq!(
            result,
            vec![
                Token::Newline,
                Token::BlankLine,
                Token::Text("t".to_string())
            ]
        );
    }

    #[test]
    fn test_consecutive_blank_lines() {
        // Multiple consecutive newlines (4) become Newline + BlankLine
        let input = vec![
            Token::Text("t".to_string()),
            Token::Newline,
            Token::Newline,
            Token::Newline,
            Token::Newline,
            Token::Text("t".to_string()),
        ];
        let result = transform_blank_lines(input);
        // 4 consecutive newlines become Newline (ends line) + BlankLine (blank lines)
        assert_eq!(
            result,
            vec![
                Token::Text("t".to_string()),
                Token::Newline,
                Token::BlankLine,
                Token::Text("t".to_string())
            ]
        );
    }

    #[test]
    fn test_empty_input() {
        let input = vec![];
        let result = transform_blank_lines(input);
        assert_eq!(result, vec![]);
    }

    #[test]
    fn test_only_newlines() {
        let input = vec![Token::Newline, Token::Newline, Token::Newline];
        let result = transform_blank_lines(input);
        assert_eq!(result, vec![Token::Newline, Token::BlankLine]);
    }

    #[test]
    fn test_preserves_other_tokens() {
        let input = vec![
            Token::Text("t".to_string()),
            Token::Whitespace,
            Token::Newline,
            Token::Dash,
            Token::Whitespace,
            Token::Newline,
            Token::Newline,
            Token::Text("t".to_string()),
        ];
        let result = transform_blank_lines(input);
        assert_eq!(
            result,
            vec![
                Token::Text("t".to_string()),
                Token::Whitespace,
                Token::Newline,
                Token::Dash,
                Token::Whitespace,
                Token::Newline,
                Token::BlankLine,
                Token::Text("t".to_string())
            ]
        );
    }

    #[test]
    fn test_blank_line_with_spans() {
        let input = vec![
            (Token::Text("t".to_string()), 0..4),
            (Token::Newline, 4..5),
            (Token::Newline, 5..6),
            (Token::Text("t".to_string()), 6..10),
        ];
        let result = transform_blank_lines_with_spans(input);
        assert_eq!(
            result,
            vec![
                (Token::Text("t".to_string()), 0..4),
                (Token::Newline, 4..5),
                (Token::BlankLine, 0..0),
                (Token::Text("t".to_string()), 6..10),
            ]
        );
    }
}
