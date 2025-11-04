//! Blank line mapper for TokenStream pipeline
//!
//! This mapper transforms sequences of consecutive Newline tokens into semantic
//! BlankLine tokens. A blank line is defined as two or more consecutive Newline tokens.
//!
//! # Logic
//!
//! 1. Iterate through tokens
//! 2. When encountering a Newline, count consecutive Newlines
//! 3. If 1 Newline: emit it as-is
//! 4. If 2+ Newlines: emit 1 Newline (to end current line) + 1 BlankLine
//! 5. Preserve all other tokens unchanged
//!
//! This is a pure adaptation of the existing transform_blank_lines transformation
//! to the TokenStream architecture.

use crate::lex::lexers::tokens::Token;
use crate::lex::pipeline::mapper::{StreamMapper, TransformationError};
use crate::lex::pipeline::stream::TokenStream;
use std::ops::Range as ByteRange;

/// A mapper that transforms consecutive Newline tokens into BlankLine tokens.
///
/// This transformation only operates on flat token streams and preserves all
/// token ranges exactly as they appear in the source.
pub struct BlankLinesMapper;

impl BlankLinesMapper {
    /// Create a new BlankLinesMapper.
    pub fn new() -> Self {
        BlankLinesMapper
    }
}

impl Default for BlankLinesMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamMapper for BlankLinesMapper {
    fn map_flat(
        &mut self,
        tokens: Vec<(Token, ByteRange<usize>)>,
    ) -> Result<TokenStream, TransformationError> {
        let mut result = Vec::new();
        let mut i = 0;

        while i < tokens.len() {
            if matches!(tokens[i].0, Token::Newline) {
                // Count consecutive Newline tokens
                let start = i;
                let mut end = i;
                while end < tokens.len() && matches!(tokens[end].0, Token::Newline) {
                    end += 1;
                }
                let newline_count = end - start;

                // Always emit the first newline
                result.push(tokens[start].clone());

                // If we have 2+ consecutive newlines, emit a BlankLine token
                // Store all the extra newline tokens (from 2nd onwards) as source_tokens
                if newline_count >= 2 {
                    let source_tokens: Vec<(Token, ByteRange<usize>)> =
                        tokens[start + 1..end].to_vec();
                    // Placeholder span 0..0 - will never be used, AST construction unrolls source_tokens
                    result.push((Token::BlankLine(source_tokens), 0..0));
                }

                // Move past all processed newlines
                i = end;
            } else {
                // Non-newline token, just copy it with its location
                result.push(tokens[i].clone());
                i += 1;
            }
        }

        Ok(TokenStream::Flat(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::lexers::tokens::Token;
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
        let mut mapper = BlankLinesMapper::new();
        let result = mapper.map_flat(with_loc(input)).unwrap();
        match result {
            TokenStream::Flat(tokens) => {
                let stripped = strip_loc(tokens);
                assert_eq!(
                    stripped,
                    vec![
                        Token::Text("a".to_string()),
                        Token::Newline,
                        Token::Text("b".to_string()),
                    ]
                );
            }
            _ => panic!("Expected Flat stream"),
        }
    }

    #[test]
    fn test_double_newline_becomes_newline_then_blank_line() {
        let input = vec![
            Token::Text("t".to_string()),
            Token::Newline,
            Token::Newline,
            Token::Text("t".to_string()),
        ];
        let mut mapper = BlankLinesMapper::new();
        let result = mapper.map_flat(with_loc(input)).unwrap();
        match result {
            TokenStream::Flat(tokens) => {
                let stripped = strip_loc(tokens);
                assert_eq!(
                    stripped,
                    vec![
                        Token::Text("t".to_string()),
                        Token::Newline,
                        Token::BlankLine(vec![]),
                        Token::Text("t".to_string())
                    ]
                );
            }
            _ => panic!("Expected Flat stream"),
        }
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
        let mut mapper = BlankLinesMapper::new();
        let result = mapper.map_flat(with_loc(input)).unwrap();
        match result {
            TokenStream::Flat(tokens) => {
                let stripped = strip_loc(tokens);
                assert_eq!(
                    stripped,
                    vec![
                        Token::Text("t".to_string()),
                        Token::Newline,
                        Token::BlankLine(vec![]),
                        Token::Text("t".to_string())
                    ]
                );
            }
            _ => panic!("Expected Flat stream"),
        }
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
        let mut mapper = BlankLinesMapper::new();
        let result = mapper.map_flat(with_loc(input)).unwrap();
        match result {
            TokenStream::Flat(tokens) => {
                let stripped = strip_loc(tokens);
                assert_eq!(
                    stripped,
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
            _ => panic!("Expected Flat stream"),
        }
    }

    #[test]
    fn test_blank_line_at_end() {
        let input = vec![Token::Text("t".to_string()), Token::Newline, Token::Newline];
        let mut mapper = BlankLinesMapper::new();
        let result = mapper.map_flat(with_loc(input)).unwrap();
        match result {
            TokenStream::Flat(tokens) => {
                let stripped = strip_loc(tokens);
                assert_eq!(
                    stripped,
                    vec![
                        Token::Text("t".to_string()),
                        Token::Newline,
                        Token::BlankLine(vec![])
                    ]
                );
            }
            _ => panic!("Expected Flat stream"),
        }
    }

    #[test]
    fn test_blank_line_at_start() {
        let input = vec![Token::Newline, Token::Newline, Token::Text("t".to_string())];
        let mut mapper = BlankLinesMapper::new();
        let result = mapper.map_flat(with_loc(input)).unwrap();
        match result {
            TokenStream::Flat(tokens) => {
                let stripped = strip_loc(tokens);
                assert_eq!(
                    stripped,
                    vec![
                        Token::Newline,
                        Token::BlankLine(vec![]),
                        Token::Text("t".to_string())
                    ]
                );
            }
            _ => panic!("Expected Flat stream"),
        }
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
        let mut mapper = BlankLinesMapper::new();
        let result = mapper.map_flat(with_loc(input)).unwrap();
        match result {
            TokenStream::Flat(tokens) => {
                let stripped = strip_loc(tokens);
                assert_eq!(
                    stripped,
                    vec![
                        Token::Text("t".to_string()),
                        Token::Newline,
                        Token::BlankLine(vec![]),
                        Token::Text("t".to_string())
                    ]
                );
            }
            _ => panic!("Expected Flat stream"),
        }
    }

    #[test]
    fn test_empty_input() {
        let input = vec![];
        let mut mapper = BlankLinesMapper::new();
        let result = mapper.map_flat(with_loc(input)).unwrap();
        match result {
            TokenStream::Flat(tokens) => {
                let stripped = strip_loc(tokens);
                assert_eq!(stripped, vec![]);
            }
            _ => panic!("Expected Flat stream"),
        }
    }

    #[test]
    fn test_only_newlines() {
        let input = vec![Token::Newline, Token::Newline, Token::Newline];
        let mut mapper = BlankLinesMapper::new();
        let result = mapper.map_flat(with_loc(input)).unwrap();
        match result {
            TokenStream::Flat(tokens) => {
                let stripped = strip_loc(tokens);
                assert_eq!(stripped, vec![Token::Newline, Token::BlankLine(vec![])]);
            }
            _ => panic!("Expected Flat stream"),
        }
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
        let mut mapper = BlankLinesMapper::new();
        let result = mapper.map_flat(with_loc(input)).unwrap();
        match result {
            TokenStream::Flat(tokens) => {
                let stripped = strip_loc(tokens);
                assert_eq!(
                    stripped,
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
            _ => panic!("Expected Flat stream"),
        }
    }

    #[test]
    fn test_blank_lines_with_locations() {
        let input: Tokens = vec![
            mk_token(Token::Text("t".to_string()), 0, 4),
            mk_token(Token::Newline, 4, 5),
            mk_token(Token::Newline, 5, 6),
            mk_token(Token::Text("t".to_string()), 6, 10),
        ];

        let mut mapper = BlankLinesMapper::new();
        let result = mapper.map_flat(input).unwrap();
        match result {
            TokenStream::Flat(tokens) => {
                // Verify source_tokens are captured correctly (Immutable Log principle)
                assert_eq!(tokens.len(), 4);
                if let Token::BlankLine(source_tokens) = &tokens[2].0 {
                    assert_eq!(
                        source_tokens.len(),
                        1,
                        "BlankLine should capture 1 source token"
                    );
                    assert_eq!(
                        source_tokens[0].0,
                        Token::Newline,
                        "Source token should be Newline"
                    );
                    assert_eq!(
                        source_tokens[0].1,
                        5..6,
                        "Source token should have correct range"
                    );
                } else {
                    panic!("Expected BlankLine token at position 2");
                }

                let expected: Tokens = vec![
                    mk_token(Token::Text("t".to_string()), 0, 4),
                    mk_token(Token::Newline, 4, 5),
                    mk_token(Token::BlankLine(vec![(Token::Newline, 5..6)]), 0, 0),
                    mk_token(Token::Text("t".to_string()), 6, 10),
                ];
                assert_eq!(tokens, expected);
            }
            _ => panic!("Expected Flat stream"),
        }
    }

    // ========== location TESTS ==========

    #[test]
    fn test_blank_line_token_has_correct_location_for_double_newline() {
        let input: Tokens = vec![
            mk_token(Token::Text("a".to_string()), 0, 1),
            mk_token(Token::Newline, 1, 2),
            mk_token(Token::Newline, 2, 3),
            mk_token(Token::Text("b".to_string()), 3, 4),
        ];

        let mut mapper = BlankLinesMapper::new();
        let result = mapper.map_flat(input).unwrap();
        match result {
            TokenStream::Flat(tokens) => {
                assert_eq!(tokens.len(), 4);
                assert_eq!(tokens[0], mk_token(Token::Text("a".to_string()), 0, 1));
                assert_eq!(tokens[1], mk_token(Token::Newline, 1, 2));
                assert_eq!(tokens[2].0, Token::BlankLine(vec![(Token::Newline, 2..3)]));
                assert_eq!(tokens[2].1, 0..0, "BlankLine uses placeholder span");
                assert_eq!(tokens[3], mk_token(Token::Text("b".to_string()), 3, 4));
            }
            _ => panic!("Expected Flat stream"),
        }
    }

    #[test]
    fn test_blank_line_token_has_correct_location_for_triple_newline() {
        let input: Tokens = vec![
            mk_token(Token::Text("a".to_string()), 0, 1),
            mk_token(Token::Newline, 1, 2),
            mk_token(Token::Newline, 2, 3),
            mk_token(Token::Newline, 3, 4),
            mk_token(Token::Text("b".to_string()), 4, 5),
        ];

        let mut mapper = BlankLinesMapper::new();
        let result = mapper.map_flat(input).unwrap();
        match result {
            TokenStream::Flat(tokens) => {
                assert_eq!(tokens.len(), 4);

                // Verify source_tokens are captured correctly (Immutable Log principle)
                if let Token::BlankLine(source_tokens) = &tokens[2].0 {
                    assert_eq!(
                        source_tokens.len(),
                        2,
                        "BlankLine should capture 2 source tokens"
                    );
                    assert_eq!(source_tokens[0].0, Token::Newline);
                    assert_eq!(source_tokens[0].1, 2..3, "First source token range");
                    assert_eq!(source_tokens[1].0, Token::Newline);
                    assert_eq!(source_tokens[1].1, 3..4, "Second source token range");
                } else {
                    panic!("Expected BlankLine token at position 2");
                }

                assert_eq!(
                    tokens[2].0,
                    Token::BlankLine(vec![(Token::Newline, 2..3), (Token::Newline, 3..4)])
                );
                assert_eq!(tokens[2].1, 0..0, "BlankLine uses placeholder span");
            }
            _ => panic!("Expected Flat stream"),
        }
    }

    #[test]
    fn test_blank_line_token_has_correct_location_for_many_newlines() {
        let input: Tokens = vec![
            mk_token(Token::Text("a".to_string()), 0, 1),
            mk_token(Token::Newline, 1, 2),
            mk_token(Token::Newline, 2, 3),
            mk_token(Token::Newline, 3, 4),
            mk_token(Token::Newline, 4, 5),
            mk_token(Token::Newline, 5, 6),
            mk_token(Token::Text("b".to_string()), 6, 7),
        ];

        let mut mapper = BlankLinesMapper::new();
        let result = mapper.map_flat(input).unwrap();
        match result {
            TokenStream::Flat(tokens) => {
                assert_eq!(tokens.len(), 4);
                assert_eq!(
                    tokens[2].0,
                    Token::BlankLine(vec![
                        (Token::Newline, 2..3),
                        (Token::Newline, 3..4),
                        (Token::Newline, 4..5),
                        (Token::Newline, 5..6)
                    ])
                );
                assert_eq!(tokens[2].1, 0..0, "BlankLine uses placeholder span");
            }
            _ => panic!("Expected Flat stream"),
        }
    }

    #[test]
    fn test_multiple_blank_lines_each_have_correct_locations() {
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

        let mut mapper = BlankLinesMapper::new();
        let result = mapper.map_flat(input).unwrap();
        match result {
            TokenStream::Flat(tokens) => {
                assert_eq!(tokens.len(), 7);

                // First BlankLine (contains 2nd newline)
                assert_eq!(tokens[2].0, Token::BlankLine(vec![(Token::Newline, 2..3)]));
                assert_eq!(tokens[2].1, 0..0, "BlankLine uses placeholder span");

                // Second BlankLine (contains 2nd and 3rd newlines)
                assert_eq!(
                    tokens[5].0,
                    Token::BlankLine(vec![(Token::Newline, 5..6), (Token::Newline, 6..7)])
                );
                assert_eq!(tokens[5].1, 0..0, "BlankLine uses placeholder span");
            }
            _ => panic!("Expected Flat stream"),
        }
    }

    #[test]
    fn test_blank_line_at_start_has_correct_location() {
        let input: Tokens = vec![
            mk_token(Token::Newline, 0, 1),
            mk_token(Token::Newline, 1, 2),
            mk_token(Token::Text("a".to_string()), 2, 3),
        ];

        let mut mapper = BlankLinesMapper::new();
        let result = mapper.map_flat(input).unwrap();
        match result {
            TokenStream::Flat(tokens) => {
                assert_eq!(tokens.len(), 3);
                assert_eq!(tokens[0], mk_token(Token::Newline, 0, 1));
                assert_eq!(tokens[1].0, Token::BlankLine(vec![(Token::Newline, 1..2)]));
                assert_eq!(tokens[1].1, 0..0, "BlankLine uses placeholder span");
            }
            _ => panic!("Expected Flat stream"),
        }
    }

    #[test]
    fn test_blank_line_at_end_has_correct_location() {
        let input: Tokens = vec![
            mk_token(Token::Text("a".to_string()), 0, 1),
            mk_token(Token::Newline, 1, 2),
            mk_token(Token::Newline, 2, 3),
        ];

        let mut mapper = BlankLinesMapper::new();
        let result = mapper.map_flat(input).unwrap();
        match result {
            TokenStream::Flat(tokens) => {
                assert_eq!(tokens.len(), 3);
                assert_eq!(tokens[2].0, Token::BlankLine(vec![(Token::Newline, 2..3)]));
                assert_eq!(tokens[2].1, 0..0, "BlankLine uses placeholder span");
            }
            _ => panic!("Expected Flat stream"),
        }
    }

    #[test]
    fn test_locations_with_real_lex_content() {
        let source = "First paragraph\n\nSecond paragraph";
        let tokens = crate::lex::lexers::tokenize(source);

        let mut mapper = BlankLinesMapper::new();
        let result = mapper.map_flat(tokens).unwrap();
        match result {
            TokenStream::Flat(tokens) => {
                let blank_line_pos = tokens
                    .iter()
                    .position(|(t, _)| matches!(t, Token::BlankLine(_)));
                assert!(blank_line_pos.is_some(), "Should have a BlankLine token");

                let (blank_token, blank_location) = &tokens[blank_line_pos.unwrap()];
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
                assert_eq!(*blank_location, 0..0, "BlankLine uses placeholder span");
            }
            _ => panic!("Expected Flat stream"),
        }
    }

    #[test]
    fn test_no_blank_line_preserves_all_locations() {
        let input: Tokens = vec![
            mk_token(Token::Text("a".to_string()), 0, 1),
            mk_token(Token::Newline, 1, 2),
            mk_token(Token::Text("b".to_string()), 2, 3),
        ];

        let mut mapper = BlankLinesMapper::new();
        let result = mapper.map_flat(input.clone()).unwrap();
        match result {
            TokenStream::Flat(tokens) => {
                assert_eq!(tokens, input);
            }
            _ => panic!("Expected Flat stream"),
        }
    }
}
