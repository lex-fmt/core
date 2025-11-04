//! Blank line transformation for lex lexer
//!
//! This module transforms sequences of consecutive Newline tokens into semantic BlankLine tokens.
//! A blank line is defined as two or more consecutive Newline tokens.
//!
//! This transformation should be applied AFTER indentation transformation, so that the
//! indentation levels are properly established before we identify blank lines.

use crate::lex::lexers::tokens::Token;
use crate::lex::lexers::transformations::Transformation;

/// Blank line transformation
///
/// Converts sequences of consecutive Newline tokens (2 or more) into BlankLine tokens.
/// This helps parsers identify paragraph boundaries and other blank-line-delimited structures.
pub struct TransformBlankLines;

impl Transformation for TransformBlankLines {
    fn name(&self) -> &str {
        "transform_blank_lines"
    }

    fn description(&self) -> &str {
        "Convert consecutive Newline tokens (2+) into BlankLine tokens to mark paragraph boundaries"
    }

    fn transform(
        &self,
        tokens: Vec<(Token, std::ops::Range<usize>)>,
    ) -> Vec<(Token, std::ops::Range<usize>)> {
        transform_blank_lines(tokens)
    }
}

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
/// removed non-location-only transform; use location-preserving API below
/// Transform blank lines while preserving source locations
/// Blank lines (sequences of 2+ newlines) become Newline followed by BlankLine token
/// The BlankLine token gets the location covering the extra newlines (from 2nd newline onwards)
pub fn transform_blank_lines(
    tokens: Vec<(Token, std::ops::Range<usize>)>,
) -> Vec<(Token, std::ops::Range<usize>)> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        if matches!(tokens[i].0, Token::Newline) {
            // Count consecutive Newline tokens and collect their locations
            let mut newline_count = 0;
            let mut j = i;
            while j < tokens.len() && matches!(tokens[j].0, Token::Newline) {
                newline_count += 1;
                j += 1;
            }

            // Emit the first Newline with its original location (ends the current line)
            result.push((Token::Newline, tokens[i].1.clone()));

            // If we have 2+ consecutive newlines, emit a BlankLine token
            // Store all the extra newline tokens (from 2nd onwards) as source_tokens
            if newline_count >= 2 {
                // Collect the extra newline tokens (from 2nd to last)
                let source_tokens: Vec<(Token, std::ops::Range<usize>)> = tokens[i + 1..j].to_vec();

                // Placeholder span 0..0 - will never be used, AST construction unrolls source_tokens
                result.push((Token::BlankLine(source_tokens), 0..0));
            }

            // Move past all the newlines we just processed
            i = j;
        } else {
            // Non-newline token, just copy it with its location
            result.push(tokens[i].clone());
            i += 1;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::testing::factories::{mk_token, Tokens};

    fn with_loc(tokens: Vec<Token>) -> Tokens {
        tokens
            .into_iter()
            .enumerate()
            .map(|(idx, token)| mk_token(token, idx, idx + 1))
            .collect()
    }

    fn strip_loc(pairs: Tokens) -> Vec<Token> {
        pairs
            .into_iter()
            .map(|(t, _)| {
                // Normalize source_tokens to empty for test comparison
                match t {
                    Token::Indent(_) => Token::Indent(vec![]),
                    Token::Dedent(_) => Token::Dedent(vec![]),
                    Token::BlankLine(_) => Token::BlankLine(vec![]),
                    other => other,
                }
            })
            .collect()
    }

    #[test]
    fn test_single_newline_unchanged() {
        let input = vec![
            Token::Text("a".to_string()),
            Token::Newline,
            Token::Text("b".to_string()),
        ];
        let result = strip_loc(transform_blank_lines(with_loc(input)));
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
        let result = strip_loc(transform_blank_lines(with_loc(input)));
        assert_eq!(
            result,
            vec![
                Token::Text("t".to_string()),
                Token::Newline,
                Token::BlankLine(vec![]),
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
        let result = strip_loc(transform_blank_lines(with_loc(input)));
        assert_eq!(
            result,
            vec![
                Token::Text("t".to_string()),
                Token::Newline,
                Token::BlankLine(vec![]),
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
        let result = strip_loc(transform_blank_lines(with_loc(input)));
        assert_eq!(
            result,
            vec![
                Token::Text("t".to_string()),
                Token::Newline,
                Token::BlankLine(vec![]),
                Token::Text("t".to_string()),
                Token::Newline,
                Token::BlankLine(vec![]),
                Token::Text("t".to_string())
            ]
        );
    }

    #[test]
    fn test_blank_line_at_end() {
        let input = vec![Token::Text("t".to_string()), Token::Newline, Token::Newline];
        let result = strip_loc(transform_blank_lines(with_loc(input)));
        assert_eq!(
            result,
            vec![
                Token::Text("t".to_string()),
                Token::Newline,
                Token::BlankLine(vec![])
            ]
        );
    }

    #[test]
    fn test_blank_line_at_start() {
        let input = vec![Token::Newline, Token::Newline, Token::Text("t".to_string())];
        let result = strip_loc(transform_blank_lines(with_loc(input)));
        assert_eq!(
            result,
            vec![
                Token::Newline,
                Token::BlankLine(vec![]),
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
        let result = strip_loc(transform_blank_lines(with_loc(input)));
        // 4 consecutive newlines become Newline (ends line) + BlankLine (blank lines)
        assert_eq!(
            result,
            vec![
                Token::Text("t".to_string()),
                Token::Newline,
                Token::BlankLine(vec![]),
                Token::Text("t".to_string())
            ]
        );
    }

    #[test]
    fn test_empty_input() {
        let input = vec![];
        let result = strip_loc(transform_blank_lines(with_loc(input)));
        assert_eq!(result, vec![]);
    }

    #[test]
    fn test_only_newlines() {
        let input = vec![Token::Newline, Token::Newline, Token::Newline];
        let result = strip_loc(transform_blank_lines(with_loc(input)));
        assert_eq!(result, vec![Token::Newline, Token::BlankLine(vec![])]);
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
        let result = strip_loc(transform_blank_lines(with_loc(input)));
        assert_eq!(
            result,
            vec![
                Token::Text("t".to_string()),
                Token::Whitespace,
                Token::Newline,
                Token::Dash,
                Token::Whitespace,
                Token::Newline,
                Token::BlankLine(vec![]),
                Token::Text("t".to_string())
            ]
        );
    }

    #[test]
    fn test_blank_lines() {
        let input: Tokens = vec![
            mk_token(Token::Text("t".to_string()), 0, 4),
            mk_token(Token::Newline, 4, 5),
            mk_token(Token::Newline, 5, 6),
            mk_token(Token::Text("t".to_string()), 6, 10),
        ];

        let result = transform_blank_lines(input);
        let expected: Tokens = vec![
            mk_token(Token::Text("t".to_string()), 0, 4),
            mk_token(Token::Newline, 4, 5),
            mk_token(Token::BlankLine(vec![(Token::Newline, 5..6)]), 0, 0),
            mk_token(Token::Text("t".to_string()), 6, 10),
        ];
        assert_eq!(result, expected);
    }

    // ========== location TESTS ==========
    // Tests to verify that BlankLine tokens have correct locations

    #[test]
    fn test_blank_line_token_has_correct_location_for_double_newline() {
        // Test: BlankLine should cover the location of extra newlines (from 2nd onwards)
        // Input: "a\n\nb" where positions are: "a" 0..1, "\n" 1..2, "\n" 2..3, "b" 3..4
        use crate::lex::testing::factories::{mk_token, Tokens};
        let input: Tokens = vec![
            mk_token(Token::Text("a".to_string()), 0, 1),
            mk_token(Token::Newline, 1, 2),
            mk_token(Token::Newline, 2, 3),
            mk_token(Token::Text("b".to_string()), 3, 4),
        ];

        let result = transform_blank_lines(input);

        // Expected: Text("a"), Newline, BlankLine, Text("b")
        assert_eq!(result.len(), 4);
        assert_eq!(result[0], mk_token(Token::Text("a".to_string()), 0, 1));
        assert_eq!(result[1], mk_token(Token::Newline, 1, 2));
        assert_eq!(result[2].0, Token::BlankLine(vec![(Token::Newline, 2..3)]));
        assert_eq!(result[2].1, 0..0, "BlankLine uses placeholder span");
        assert_eq!(result[3], mk_token(Token::Text("b".to_string()), 3, 4));
    }

    #[test]
    fn test_blank_line_token_has_correct_location_for_triple_newline() {
        // Test: BlankLine should cover the location from 2nd to last newline
        // Input: "a\n\n\nb" where positions are: "a" 0..1, "\n" 1..2, "\n" 2..3, "\n" 3..4, "b" 4..5
        use crate::lex::testing::factories::{mk_token, Tokens};
        let input: Tokens = vec![
            mk_token(Token::Text("a".to_string()), 0, 1),
            mk_token(Token::Newline, 1, 2),
            mk_token(Token::Newline, 2, 3),
            mk_token(Token::Newline, 3, 4),
            mk_token(Token::Text("b".to_string()), 4, 5),
        ];

        let result = transform_blank_lines(input);

        // Expected: Text("a"), Newline, BlankLine, Text("b")
        assert_eq!(result.len(), 4);
        assert_eq!(
            result[2].0,
            Token::BlankLine(vec![(Token::Newline, 2..3), (Token::Newline, 3..4)])
        );
        assert_eq!(result[2].1, 0..0, "BlankLine uses placeholder span");
    }

    #[test]
    fn test_blank_line_token_has_correct_location_for_many_newlines() {
        // Test: BlankLine should cover all extra newlines
        // Input: "a\n\n\n\n\nb" (5 newlines total)
        use crate::lex::testing::factories::{mk_token, Tokens};
        let input: Tokens = vec![
            mk_token(Token::Text("a".to_string()), 0, 1),
            mk_token(Token::Newline, 1, 2),
            mk_token(Token::Newline, 2, 3),
            mk_token(Token::Newline, 3, 4),
            mk_token(Token::Newline, 4, 5),
            mk_token(Token::Newline, 5, 6),
            mk_token(Token::Text("b".to_string()), 6, 7),
        ];

        let result = transform_blank_lines(input);

        // Expected: Text("a"), Newline, BlankLine, Text("b")
        assert_eq!(result.len(), 4);
        assert_eq!(
            result[2].0,
            Token::BlankLine(vec![
                (Token::Newline, 2..3),
                (Token::Newline, 3..4),
                (Token::Newline, 4..5),
                (Token::Newline, 5..6)
            ])
        );
        assert_eq!(result[2].1, 0..0, "BlankLine uses placeholder span");
    }

    #[test]
    fn test_multiple_blank_lines_each_have_correct_locations() {
        // Test: Multiple BlankLine tokens should each have their own correct locations
        // Input: "a\n\nb\n\n\nc"
        use crate::lex::testing::factories::{mk_token, Tokens};
        let input: Tokens = vec![
            mk_token(Token::Text("a".to_string()), 0, 1),
            mk_token(Token::Newline, 1, 2),
            mk_token(Token::Newline, 2, 3),
            mk_token(Token::Text("b".to_string()), 3, 4),
            mk_token(Token::Newline, 4, 5),
            mk_token(Token::Newline, 5, 6),
            mk_token(Token::Newline, 6, 7),
            mk_token(Token::Text("c".to_string()), 7, 8),
        ];

        let result = transform_blank_lines(input);

        // Expected: Text("a"), Newline, BlankLine, Text("b"), Newline, BlankLine, Text("c")
        assert_eq!(result.len(), 7);

        // First BlankLine (contains 2nd newline)
        assert_eq!(result[2].0, Token::BlankLine(vec![(Token::Newline, 2..3)]));
        assert_eq!(result[2].1, 0..0, "BlankLine uses placeholder span");

        // Second BlankLine (contains 2nd and 3rd newlines)
        assert_eq!(
            result[5].0,
            Token::BlankLine(vec![(Token::Newline, 5..6), (Token::Newline, 6..7)])
        );
        assert_eq!(result[5].1, 0..0, "BlankLine uses placeholder span");
    }

    #[test]
    fn test_blank_line_at_start_has_correct_location() {
        // Test: BlankLine at document start
        // Input: "\n\na" (starts with blank line)
        let input: Tokens = vec![
            mk_token(Token::Newline, 0, 1),
            mk_token(Token::Newline, 1, 2),
            mk_token(Token::Text("a".to_string()), 2, 3),
        ];

        let result = transform_blank_lines(input);

        // Expected: Newline, BlankLine, Text("a")
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], mk_token(Token::Newline, 0, 1));
        assert_eq!(result[1].0, Token::BlankLine(vec![(Token::Newline, 1..2)]));
        assert_eq!(result[1].1, 0..0, "BlankLine uses placeholder span");
    }

    #[test]
    fn test_blank_line_at_end_has_correct_location() {
        // Test: BlankLine at document end
        // Input: "a\n\n" (ends with blank line)
        let input: Tokens = vec![
            mk_token(Token::Text("a".to_string()), 0, 1),
            mk_token(Token::Newline, 1, 2),
            mk_token(Token::Newline, 2, 3),
        ];

        let result = transform_blank_lines(input);

        // Expected: Text("a"), Newline, BlankLine
        assert_eq!(result.len(), 3);
        assert_eq!(result[2].0, Token::BlankLine(vec![(Token::Newline, 2..3)]));
        assert_eq!(result[2].1, 0..0, "BlankLine uses placeholder span");
    }

    #[test]
    fn test_locations_with_real_lex_content() {
        // Test with actual lex content
        let source = "First paragraph\n\nSecond paragraph";
        // Positions: "First paragraph" 0..15, "\n" 15..16, "\n" 16..17, "Second paragraph" 17..33

        let tokens = crate::lex::lexers::tokenize(source);
        let result = transform_blank_lines(tokens);

        // Find the BlankLine token
        let blank_line_pos = result
            .iter()
            .position(|(t, _)| matches!(t, Token::BlankLine(_)));
        assert!(blank_line_pos.is_some(), "Should have a BlankLine token");

        let (blank_token, blank_location) = &result[blank_line_pos.unwrap()];
        // BlankLine should contain the 2nd newline token with its span
        assert!(matches!(blank_token, Token::BlankLine(_)));
        if let Token::BlankLine(source_tokens) = blank_token {
            assert_eq!(
                source_tokens.len(),
                1,
                "BlankLine should have one source token"
            );
            assert_eq!(source_tokens[0].0, Token::Newline);
            assert_eq!(
                source_tokens[0].1,
                16..17,
                "Source newline should be at 16..17"
            );
        }
        // BlankLine uses placeholder span
        assert_eq!(*blank_location, 0..0, "BlankLine uses placeholder span");
    }

    #[test]
    fn test_no_blank_line_preserves_all_locations() {
        // Test: When there are no blank lines, all locations should be preserved
        let input: Tokens = vec![
            mk_token(Token::Text("a".to_string()), 0, 1),
            mk_token(Token::Newline, 1, 2),
            mk_token(Token::Text("b".to_string()), 2, 3),
        ];

        let result = transform_blank_lines(input.clone());

        // Should be identical to input since no blank lines
        assert_eq!(result, input);
    }
}
