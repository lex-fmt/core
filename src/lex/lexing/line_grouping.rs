//! Line Grouping
//!
//! Groups flat tokens into classified LineTokens.
//! This module contains the core grouping logic that calls classifiers
//! and creates LineToken structures.

use crate::lex::lexing::line_classification::classify_line_tokens;
use crate::lex::lexing::tokens_core::Token;
use crate::lex::lexing::tokens_linebased::{LineToken, LineType};
use std::ops::Range as ByteRange;

/// Group flat tokens into classified LineTokens.
///
/// This implements the logic from ToLineTokensMapper:
/// - Groups consecutive tokens into lines (delimited by Newline)
/// - Classifies each line by type
/// - Handles structural tokens (Indent, Dedent, BlankLine) specially
pub fn group_into_lines(tokens: Vec<(Token, ByteRange<usize>)>) -> Vec<LineToken> {
    let mut line_tokens = Vec::new();
    let mut current_line = Vec::new();

    for (token, span) in tokens {
        let is_newline = matches!(token, Token::Newline);
        let is_blank_line_token = matches!(token, Token::BlankLine(_));

        // Structural tokens (Indent, Dedent, BlankLine) are pass-through
        if let Token::Indent(ref sources) = token {
            // Flush any accumulated line first
            if !current_line.is_empty() {
                line_tokens.push(classify_and_create_line_token(current_line));
                current_line = Vec::new();
            }
            // Extract the stored source tokens from Indent
            let (source_tokens, token_spans): (Vec<_>, Vec<_>) = sources.iter().cloned().unzip();
            line_tokens.push(LineToken {
                source_tokens,
                token_spans,
                line_type: LineType::Indent,
            });
            continue;
        }

        if let Token::Dedent(_) = token {
            // Flush any accumulated line first
            if !current_line.is_empty() {
                line_tokens.push(classify_and_create_line_token(current_line));
                current_line = Vec::new();
            }
            // Dedent tokens are purely structural
            line_tokens.push(LineToken {
                source_tokens: vec![token],
                token_spans: vec![span],
                line_type: LineType::Dedent,
            });
            continue;
        }

        // BlankLine tokens are also structural
        if is_blank_line_token {
            // Flush any accumulated line first
            if !current_line.is_empty() {
                line_tokens.push(classify_and_create_line_token(current_line));
                current_line = Vec::new();
            }
            // Extract the stored source tokens from BlankLine
            if let Token::BlankLine(ref sources) = token {
                let (source_tokens, token_spans): (Vec<_>, Vec<_>) =
                    sources.iter().cloned().unzip();
                line_tokens.push(LineToken {
                    source_tokens,
                    token_spans,
                    line_type: LineType::BlankLine,
                });
            }
            continue;
        }

        // Accumulate token-span tuples for current line
        current_line.push((token, span));

        // Newline marks end of line
        if is_newline {
            line_tokens.push(classify_and_create_line_token(current_line));
            current_line = Vec::new();
        }
    }

    // Handle any remaining tokens (if input doesn't end with newline)
    if !current_line.is_empty() {
        line_tokens.push(classify_and_create_line_token(current_line));
    }

    // Apply dialog line detection
    apply_dialog_detection(line_tokens)
}

/// Classify tokens and create a LineToken with the appropriate LineType.
fn classify_and_create_line_token(token_tuples: Vec<(Token, ByteRange<usize>)>) -> LineToken {
    let (source_tokens, token_spans): (Vec<_>, Vec<_>) = token_tuples.into_iter().unzip();
    let line_type = classify_line_tokens(&source_tokens);

    LineToken {
        source_tokens,
        token_spans,
        line_type,
    }
}

/// Apply dialog line detection logic.
///
/// In the linebased parser, once a dialog line is detected, all subsequent lines
/// are also treated as dialog lines until the end of the block.
fn apply_dialog_detection(mut line_tokens: Vec<LineToken>) -> Vec<LineToken> {
    let mut in_dialog = false;

    for line_token in &mut line_tokens {
        if line_token.line_type != LineType::ListLine {
            in_dialog = false;
        }

        if in_dialog {
            line_token.line_type = LineType::DialogLine;
        } else if line_token.line_type == LineType::ListLine {
            let non_whitespace_tokens: Vec<_> = line_token
                .source_tokens
                .iter()
                .filter(|t| !t.is_whitespace())
                .collect();

            if non_whitespace_tokens.len() >= 2 {
                let last_token = non_whitespace_tokens.last().unwrap();
                let second_to_last_token = non_whitespace_tokens[non_whitespace_tokens.len() - 2];

                if last_token.is_end_punctuation() && second_to_last_token.is_end_punctuation() {
                    line_token.line_type = LineType::DialogLine;
                    in_dialog = true;
                }
            }
        }
    }

    line_tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_single_line() {
        let tokens = vec![
            (Token::Text("Hello".to_string()), 0..5),
            (Token::Newline, 5..6),
        ];

        let line_tokens = group_into_lines(tokens);

        assert_eq!(line_tokens.len(), 1);
        assert_eq!(line_tokens[0].line_type, LineType::ParagraphLine);
        assert_eq!(line_tokens[0].source_tokens.len(), 2);
        assert_eq!(line_tokens[0].token_spans.len(), 2);
    }

    #[test]
    fn test_group_multiple_lines() {
        let tokens = vec![
            (Token::Text("Line1".to_string()), 0..5),
            (Token::Newline, 5..6),
            (Token::Text("Line2".to_string()), 6..11),
            (Token::Newline, 11..12),
        ];

        let line_tokens = group_into_lines(tokens);

        assert_eq!(line_tokens.len(), 2);
        assert_eq!(line_tokens[0].line_type, LineType::ParagraphLine);
        assert_eq!(line_tokens[1].line_type, LineType::ParagraphLine);
    }

    #[test]
    fn test_group_with_indent_dedent() {
        let tokens = vec![
            (Token::Text("Title".to_string()), 0..5),
            (Token::Colon, 5..6),
            (Token::Newline, 6..7),
            (Token::Indent(vec![(Token::Indentation, 7..11)]), 0..0),
            (Token::Text("Content".to_string()), 11..18),
            (Token::Newline, 18..19),
            (Token::Dedent(vec![]), 0..0),
        ];

        let line_tokens = group_into_lines(tokens);

        assert_eq!(line_tokens.len(), 4); // Title, Indent, Content, Dedent
        assert_eq!(line_tokens[0].line_type, LineType::SubjectLine);
        assert_eq!(line_tokens[1].line_type, LineType::Indent);
        assert_eq!(line_tokens[2].line_type, LineType::ParagraphLine);
        assert_eq!(line_tokens[3].line_type, LineType::Dedent);
    }

    #[test]
    fn test_group_with_blank_line_token() {
        let tokens = vec![
            (Token::Text("Line1".to_string()), 0..5),
            (Token::Newline, 5..6),
            (
                Token::BlankLine(vec![(Token::Whitespace, 6..7), (Token::Newline, 7..8)]),
                0..0,
            ),
            (Token::Text("Line2".to_string()), 8..13),
            (Token::Newline, 13..14),
        ];

        let line_tokens = group_into_lines(tokens);

        assert_eq!(line_tokens.len(), 3);
        assert_eq!(line_tokens[0].line_type, LineType::ParagraphLine);
        assert_eq!(line_tokens[1].line_type, LineType::BlankLine);
        assert_eq!(line_tokens[2].line_type, LineType::ParagraphLine);
    }

    #[test]
    fn test_dialog_detection() {
        let tokens = vec![
            (Token::Dash, 0..1),
            (Token::Whitespace, 1..2),
            (Token::Text("Hello".to_string()), 2..7),
            (Token::Period, 7..8),
            (Token::Period, 8..9),
            (Token::Newline, 9..10),
            (Token::Dash, 10..11),
            (Token::Whitespace, 11..12),
            (Token::Text("World".to_string()), 12..17),
            (Token::Newline, 17..18),
        ];

        let line_tokens = group_into_lines(tokens);

        assert_eq!(line_tokens.len(), 2);
        assert_eq!(line_tokens[0].line_type, LineType::DialogLine); // First list with double punctuation
        assert_eq!(line_tokens[1].line_type, LineType::DialogLine); // Subsequent list item in dialog
    }

    #[test]
    fn test_preserves_ranges() {
        let tokens = vec![
            (Token::Text("Hello".to_string()), 0..5),
            (Token::Whitespace, 5..6),
            (Token::Text("world".to_string()), 6..11),
            (Token::Newline, 11..12),
        ];

        let line_tokens = group_into_lines(tokens);

        assert_eq!(line_tokens[0].token_spans[0], 0..5);
        assert_eq!(line_tokens[0].token_spans[1], 5..6);
        assert_eq!(line_tokens[0].token_spans[2], 6..11);
        assert_eq!(line_tokens[0].token_spans[3], 11..12);
    }
}
