//! Indentation transformation for lex lexer
//!
//! This module transforms raw Indent tokens into semantic Indent and Dedent tokens
//! based on the indentation levels in the document.

use crate::lex::lexers::tokens::Token;

/// Transform raw Indent tokens into semantic Indent and Dedent tokens
///
/// This function processes a token stream and converts consecutive Indent tokens
/// into appropriate Indent and Dedent tokens based on the indentation level changes.
///
/// # Algorithm
///
/// 1. Track the current indentation level (number of Indent tokens)
/// 2. For each line, count the Indent tokens at the beginning
/// 3. Compare with the previous line's indentation level:
///    - If greater: emit Indent tokens for each additional level
///    - If less: emit Dedent tokens for each reduced level
///    - If equal: no indentation tokens needed
/// 4. Replace Indent tokens with the appropriate semantic tokens
/// 5. Always add a final Dedent to close the document structure
///
/// # Example
///
/// Input tokens: `[Text, Newline, Indent, Indent, Dash, Newline, Indent, Text]`
/// Output tokens: `[Text, Newline, Indent, Indent, Dash, Newline, Dedent, Text, Dedent]`
/// Find the start of the current line, going backwards from the given position
fn find_line_start(tokens: &[Token], mut pos: usize) -> usize {
    // Go backwards to find the previous newline or start of document
    while pos > 0 {
        pos -= 1;
        if matches!(tokens[pos], Token::Newline) {
            return pos + 1;
        }
    }
    0
}

/// Check if a line is blank (only contains indentation and newline)
fn is_line_blank(tokens: &[Token], line_start: usize) -> bool {
    let mut i = line_start;

    // Skip any indentation tokens at the beginning
    while i < tokens.len() && matches!(tokens[i], Token::Indentation) {
        i += 1;
    }

    // Check if the next token is a newline (or end of file)
    i >= tokens.len() || matches!(tokens[i], Token::Newline)
}

/// Count consecutive Indent tokens at the beginning of a line
fn count_line_indent_steps(tokens: &[Token], start: usize) -> usize {
    let mut count = 0;
    let mut i = start;

    while i < tokens.len() && matches!(tokens[i], Token::Indentation) {
        count += 1;
        i += 1;
    }

    count
}

/// Transform indentation while preserving source locations
/// Synthetic tokens (Indent, Dedent) are given meaningful locations:
/// - Indent: location covers the Indent tokens it represents
/// - Dedent: location at the start of the line where dedentation occurs
pub fn sem_indentation(
    tokens: Vec<(Token, std::ops::Range<usize>)>,
) -> Vec<(Token, std::ops::Range<usize>)> {
    // Extract just the tokens for processing
    let token_kinds: Vec<Token> = tokens.iter().map(|(t, _)| t.clone()).collect();

    let mut result = Vec::new();
    let mut current_level = 0;
    let mut i = 0;

    while i < tokens.len() {
        // Find the start of the current line
        let line_start = find_line_start(&token_kinds, i);

        // Count Indent tokens at the beginning of this line
        let line_indent_level = count_line_indent_steps(&token_kinds, line_start);

        // Check if this line is blank (only contains indentation and newline)
        let is_blank_line = is_line_blank(&token_kinds, line_start);

        // Skip blank lines - they don't affect indentation level
        if is_blank_line {
            let mut j = line_start;
            while j < token_kinds.len() && !matches!(token_kinds[j], Token::Newline) {
                j += 1;
            }
            if j < token_kinds.len() && matches!(token_kinds[j], Token::Newline) {
                // Preserve the newline location
                result.push((Token::Newline, tokens[j].1.clone()));
                j += 1;
            }
            i = j;
            continue;
        }

        // Calculate the target indentation level for this line
        let target_level = line_indent_level;

        // Generate appropriate Indent/Dedent tokens storing source tokens
        match target_level.cmp(&current_level) {
            std::cmp::Ordering::Greater => {
                // Indent tokens: each stores the original Indent token it replaces
                let indent_start_idx = line_start;
                for level_idx in 0..(target_level - current_level) {
                    let indent_token_idx = indent_start_idx + current_level + level_idx;
                    let source_tokens = if indent_token_idx < token_kinds.len()
                        && matches!(token_kinds[indent_token_idx], Token::Indentation)
                    {
                        // Store the original (Token::Indent, Range<usize>) pair
                        vec![tokens[indent_token_idx].clone()]
                    } else {
                        // No corresponding Indent token (shouldn't happen in well-formed input)
                        vec![]
                    };
                    // Placeholder span 0..0 - will never be used, AST construction unrolls source_tokens
                    result.push((Token::Indent(source_tokens), 0..0));
                }
            }
            std::cmp::Ordering::Less => {
                // Dedent tokens: purely structural, don't replace any tokens
                // Store empty source_tokens since dedents are synthetic markers
                for _ in 0..(current_level - target_level) {
                    // Placeholder span 0..0 - will never be used
                    result.push((Token::Dedent(vec![]), 0..0));
                }
            }
            std::cmp::Ordering::Equal => {
                // No indentation change needed
            }
        }

        // Update current level
        current_level = target_level;

        // Skip the initial Indent tokens that were processed as indentation
        let mut j = line_start;
        for _ in 0..line_indent_level {
            if j < token_kinds.len() && matches!(token_kinds[j], Token::Indentation) {
                j += 1;
            }
        }

        // Process the rest of the line, keeping all remaining tokens with locations
        while j < token_kinds.len() && !matches!(token_kinds[j], Token::Newline) {
            result.push((token_kinds[j].clone(), tokens[j].1.clone()));
            j += 1;
        }

        // Add the newline token if we haven't reached the end
        if j < token_kinds.len() && matches!(token_kinds[j], Token::Newline) {
            result.push((Token::Newline, tokens[j].1.clone()));
            j += 1;
        }

        i = j;
    }

    // Add dedents to close all remaining indentation levels
    // These occur at the end of file - they don't replace any tokens
    for _ in 0..current_level {
        result.push((Token::Dedent(vec![]), 0..0));
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
    fn test_simple_indentation() {
        let input = vec![
            Token::Text("a".to_string()),
            Token::Newline,
            Token::Indentation,
            Token::Dash,
            Token::Newline,
        ];

        let result = strip_loc(sem_indentation(with_loc(input)));

        assert_eq!(
            result,
            vec![
                Token::Text("a".to_string()),
                Token::Newline,
                Token::Indent(vec![]),
                Token::Dash,
                Token::Newline,
                Token::Dedent(vec![]), // Dedent from level 1 to level 0
            ]
        );
    }

    #[test]
    fn test_multiple_indent_levels() {
        let input = vec![
            Token::Text("a".to_string()),
            Token::Newline,
            Token::Indentation,
            Token::Indentation,
            Token::Dash,
            Token::Newline,
            Token::Indentation,
            Token::Text("b".to_string()),
            Token::Newline,
        ];

        let result = strip_loc(sem_indentation(with_loc(input)));

        assert_eq!(
            result,
            vec![
                Token::Text("a".to_string()),
                Token::Newline,
                Token::Indent(vec![]),
                Token::Indent(vec![]),
                Token::Dash,
                Token::Newline,
                Token::Dedent(vec![]),
                Token::Text("b".to_string()),
                Token::Newline,
                Token::Dedent(vec![]), // Dedent from level 1 to level 0
            ]
        );
    }

    #[test]
    fn test_nested_structure() {
        let input = vec![
            // Line 1: "1. Session"
            Token::Number("1".to_string()),
            Token::Period,
            Token::Whitespace,
            Token::Text("Session".to_string()),
            Token::Newline,
            // Line 2: "    - Item 1"
            Token::Indentation,
            Token::Dash,
            Token::Whitespace,
            Token::Text("Item".to_string()),
            Token::Whitespace,
            Token::Number("1".to_string()),
            Token::Newline,
            // Line 3: "    - Item 2"
            Token::Indentation,
            Token::Dash,
            Token::Whitespace,
            Token::Text("Item".to_string()),
            Token::Whitespace,
            Token::Number("2".to_string()),
            Token::Newline,
            // Line 4: "        - Nested"
            Token::Indentation,
            Token::Indentation,
            Token::Dash,
            Token::Whitespace,
            Token::Text("Nested".to_string()),
            Token::Newline,
            // Line 5: "2. Another"
            Token::Number("2".to_string()),
            Token::Period,
            Token::Whitespace,
            Token::Text("Another".to_string()),
            Token::Newline,
        ];

        let result = strip_loc(sem_indentation(with_loc(input)));

        // Expected: Indent for line 2, Indent for line 4, Dedent for line 5
        assert_eq!(
            result,
            vec![
                // Line 1
                Token::Number("1".to_string()),
                Token::Period,
                Token::Whitespace,
                Token::Text("Session".to_string()),
                Token::Newline,
                // Line 2
                Token::Indent(vec![]),
                Token::Dash,
                Token::Whitespace,
                Token::Text("Item".to_string()),
                Token::Whitespace,
                Token::Number("1".to_string()),
                Token::Newline,
                // Line 3
                Token::Dash,
                Token::Whitespace,
                Token::Text("Item".to_string()),
                Token::Whitespace,
                Token::Number("2".to_string()),
                Token::Newline,
                // Line 4
                Token::Indent(vec![]),
                Token::Dash,
                Token::Whitespace,
                Token::Text("Nested".to_string()),
                Token::Newline,
                // Line 5
                Token::Dedent(vec![]),
                Token::Dedent(vec![]),
                Token::Number("2".to_string()),
                Token::Period,
                Token::Whitespace,
                Token::Text("Another".to_string()),
                Token::Newline,
            ]
        );
    }

    #[test]
    fn test_no_indentation() {
        let input = vec![
            Token::Text("a".to_string()),
            Token::Newline,
            Token::Text("b".to_string()),
            Token::Newline,
        ];

        let result = strip_loc(sem_indentation(with_loc(input.clone())));

        // No changes expected - no indentation, no Dedent at EOF
        assert_eq!(result, input);
    }

    #[test]
    fn test_empty_input() {
        let input = vec![];
        let result = strip_loc(sem_indentation(with_loc(input)));
        assert_eq!(result, vec![]);
    }

    #[test]
    fn test_single_line() {
        let input = vec![Token::Text("a".to_string())];
        let result = strip_loc(sem_indentation(with_loc(input)));
        assert_eq!(result, vec![Token::Text("a".to_string())]);
    }

    #[test]
    fn test_blank_lines() {
        // Test case: blank lines should not affect indentation level
        let input = vec![
            Token::Text("a".to_string()),
            Token::Newline,
            Token::Indentation,
            Token::Dash,
            Token::Newline,
            Token::Newline, // blank line
            Token::Dash,
            Token::Newline,
        ];

        let result = strip_loc(sem_indentation(with_loc(input)));

        assert_eq!(
            result,
            vec![
                Token::Text("a".to_string()),
                Token::Newline,
                Token::Indent(vec![]),
                Token::Dash,
                Token::Newline,
                Token::Newline,        // blank line preserved
                Token::Dedent(vec![]), // dedent from level 1 to level 0
                Token::Dash,
                Token::Newline,
            ]
        );
    }

    #[test]
    fn test_blank_lines_with_indentation() {
        // Test case: blank lines with indentation should be ignored
        let input = vec![
            Token::Text("a".to_string()),
            Token::Newline,
            Token::Indentation,
            Token::Dash,
            Token::Newline,
            Token::Indentation,
            Token::Newline, // blank line with indentation
            Token::Dash,
            Token::Newline,
        ];

        let result = strip_loc(sem_indentation(with_loc(input)));

        assert_eq!(
            result,
            vec![
                Token::Text("a".to_string()),
                Token::Newline,
                Token::Indent(vec![]),
                Token::Dash,
                Token::Newline,
                Token::Newline,        // blank line preserved
                Token::Dedent(vec![]), // dedent from level 1 to level 0
                Token::Dash,
                Token::Newline,
            ]
        );
    }

    #[test]
    fn test_whitespace_remainders() {
        // Test case with whitespace remainders (10 spaces = 2 indent levels + 2 remaining)
        let input = vec![
            Token::Indentation,
            Token::Indentation,
            Token::Text("  hello".to_string()), // This represents "  hello" (2 spaces + text)
            Token::Newline,
        ];

        let result = strip_loc(sem_indentation(with_loc(input)));

        assert_eq!(
            result,
            vec![
                Token::Indent(vec![]),
                Token::Indent(vec![]),
                Token::Text("  hello".to_string()),
                Token::Newline,
                Token::Dedent(vec![]), // Dedent from level 2 to level 1
                Token::Dedent(vec![]), // Dedent from level 1 to level 0
            ]
        );
    }

    #[test]
    fn test_file_ending_while_indented() {
        // Test case: file ending while indented should emit proper dedents
        let input = vec![
            Token::Text("a".to_string()),
            Token::Newline,
            Token::Indentation,
            Token::Dash,
            Token::Newline,
            Token::Indentation,
            Token::Indentation,
            Token::Text("b".to_string()),
            // File ends here without explicit dedents
        ];

        let result = strip_loc(sem_indentation(with_loc(input.clone())));

        assert_eq!(
            result,
            vec![
                Token::Text("a".to_string()),
                Token::Newline,
                Token::Indent(vec![]),
                Token::Dash,
                Token::Newline,
                Token::Indent(vec![]),
                Token::Text("b".to_string()),
                Token::Dedent(vec![]), // Should dedent from level 2 to level 1
                Token::Dedent(vec![]), // Should dedent from level 1 to level 0
            ]
        );
    }

    #[test]
    fn test_sharp_drop_in_indentation() {
        // Test case: sharp drop from level 3 to level 0
        let input = vec![
            Token::Text("a".to_string()),
            Token::Newline,
            Token::Indentation,
            Token::Indentation,
            Token::Indentation,
            Token::Dash,
            Token::Newline,
            Token::Text("b".to_string()), // Back to level 0
            Token::Newline,
        ];

        let result = strip_loc(sem_indentation(with_loc(input.clone())));

        assert_eq!(
            result,
            vec![
                Token::Text("a".to_string()),
                Token::Newline,
                Token::Indent(vec![]),
                Token::Indent(vec![]),
                Token::Indent(vec![]),
                Token::Dash,
                Token::Newline,
                Token::Dedent(vec![]), // Dedent from level 3 to level 2
                Token::Dedent(vec![]), // Dedent from level 2 to level 1
                Token::Dedent(vec![]), // Dedent from level 1 to level 0
                Token::Text("b".to_string()),
                Token::Newline,
            ]
        );
    }

    #[test]
    fn test_multiple_blank_lines_between_sections() {
        // Test case: multiple blank lines between indented sections
        let input = vec![
            Token::Text("a".to_string()),
            Token::Newline,
            Token::Indentation,
            Token::Dash,
            Token::Newline,
            Token::Newline, // blank line 1
            Token::Newline, // blank line 2
            Token::Newline, // blank line 3
            Token::Dash,    // Should be at same level as first dash
            Token::Newline,
        ];

        let result = strip_loc(sem_indentation(with_loc(input.clone())));

        assert_eq!(
            result,
            vec![
                Token::Text("a".to_string()),
                Token::Newline,
                Token::Indent(vec![]),
                Token::Dash,
                Token::Newline,
                Token::Newline,        // blank line 1
                Token::Newline,        // blank line 2
                Token::Newline,        // blank line 3
                Token::Dedent(vec![]), // Dedent from level 1 to level 0
                Token::Dash,           // Now at level 0
                Token::Newline,
            ]
        );
    }

    #[test]
    fn test_file_with_no_indentation() {
        // Test case: file with no indentation at all
        let input = vec![
            Token::Text("a".to_string()),
            Token::Newline,
            Token::Text("b".to_string()),
            Token::Newline,
            Token::Text("c".to_string()),
        ];

        let result = strip_loc(sem_indentation(with_loc(input.clone())));

        assert_eq!(
            result,
            vec![
                Token::Text("a".to_string()),
                Token::Newline,
                Token::Text("b".to_string()),
                Token::Newline,
                Token::Text("c".to_string()),
            ]
        );
    }

    #[test]
    fn test_count_line_indent_steps() {
        let tokens = vec![
            Token::Indentation,
            Token::Indentation,
            Token::Dash,
            Token::Text("a".to_string()),
        ];

        assert_eq!(count_line_indent_steps(&tokens, 0), 2);
        assert_eq!(count_line_indent_steps(&tokens, 2), 0);
    }

    #[test]
    fn test_find_line_start() {
        let tokens = vec![
            Token::Text("a".to_string()),
            Token::Newline,
            Token::Indentation,
            Token::Dash,
        ];

        assert_eq!(find_line_start(&tokens, 0), 0);
        assert_eq!(find_line_start(&tokens, 2), 2);
        assert_eq!(find_line_start(&tokens, 3), 2);
    }

    #[test]
    fn test_blank_line_with_spaces_does_not_dedent() {
        // Critical test: A line with only spaces (no content) should NOT produce dedent
        // Example:
        // ........Foo       (level 2)
        // ........Foo2      (level 2)
        // ....              (blank line with spaces - IGNORED)
        // ........Bar       (level 2 - NO DEDENT from Foo2 to Bar!)

        let input = vec![
            // Line 1: "        Foo" (2 indent levels)
            Token::Indentation,
            Token::Indentation,
            Token::Text("Foo".to_string()),
            Token::Newline,
            // Line 2: "        Foo2" (2 indent levels)
            Token::Indentation,
            Token::Indentation,
            Token::Text("Foo2".to_string()),
            Token::Newline,
            // Line 3: "    " (1 indent level BUT NO CONTENT - should be ignored)
            Token::Indentation,
            Token::Newline,
            // Line 4: "        Bar" (2 indent levels)
            Token::Indentation,
            Token::Indentation,
            Token::Text("Bar".to_string()),
            Token::Newline,
        ];

        let result = strip_loc(sem_indentation(with_loc(input.clone())));

        // Expected: Level stays at 2, no dedent/re-indent around the blank line
        assert_eq!(
            result,
            vec![
                // Line 1
                Token::Indent(vec![]), // From 0 to 1
                Token::Indent(vec![]), // From 1 to 2
                Token::Text("Foo".to_string()),
                Token::Newline,
                // Line 2
                Token::Text("Foo2".to_string()), // Still at level 2, no change
                Token::Newline,
                // Line 3 (blank with spaces)
                Token::Newline, // Just newline, no dedent!
                // Line 4
                Token::Text("Bar".to_string()), // Still at level 2, no dedent/re-indent!
                Token::Newline,
                // EOF
                Token::Dedent(vec![]), // From 2 to 1
                Token::Dedent(vec![]), // From 1 to 0
            ],
            "Blank lines with only spaces should NOT produce dedent/indent tokens"
        );
    }

    // ========== location TESTS ==========
    // Tests to verify that synthetic tokens (Indent, Dedent) have correct locations

    #[test]
    fn test_indent_level_tokens_have_correct_locations() {
        // Test: Indent tokens should have locations that correspond to the Indent tokens they represent
        // Input: "a\n    b" (a, newline, 4 spaces, b)
        use crate::lex::testing::factories::{mk_token, Tokens};
        let input: Tokens = vec![
            mk_token(Token::Text("a".to_string()), 0, 1), // "a" at position 0-1
            mk_token(Token::Newline, 1, 2),               // "\n" at position 1-2
            mk_token(Token::Indentation, 2, 6),           // "    " (4 spaces) at position 2-6
            mk_token(Token::Text("b".to_string()), 6, 7), // "b" at position 6-7
        ];

        let result: Vec<(Token, std::ops::Range<usize>)> = sem_indentation(input);

        // Expected:
        // - Text("a") with location 0..1
        // - Newline with location 1..2
        // - Indent with location 2..6 (covers the Indent token)
        // - Text("b") with location 6..7
        // - Dedent with location 7..7 (at EOF)

        assert_eq!(result.len(), 5);
        assert_eq!(result[0], mk_token(Token::Text("a".to_string()), 0, 1));
        assert_eq!(result[1], mk_token(Token::Newline, 1, 2));
        // Indent stores the original Indent token with its span
        assert_eq!(result[2].0, Token::Indent(vec![(Token::Indentation, 2..6)]));
        assert_eq!(result[2].1, 0..0, "Indent uses placeholder span");
        assert_eq!(result[3], mk_token(Token::Text("b".to_string()), 6, 7));
        // Dedent has no source tokens (purely structural)
        assert_eq!(result[4].0, Token::Dedent(vec![]));
        assert_eq!(result[4].1, 0..0, "Dedent uses placeholder span");
    }

    #[test]
    fn test_multiple_indent_levels_have_correct_locations() {
        // Test: Multiple Indent tokens should each have locations of their respective Indent tokens
        // Input: "a\n        b" (a, newline, 8 spaces = 2 indent levels, b)
        use crate::lex::testing::factories::{mk_token, Tokens};
        let input: Tokens = vec![
            mk_token(Token::Text("a".to_string()), 0, 1),   // "a"
            mk_token(Token::Newline, 1, 2),                 // "\n"
            mk_token(Token::Indentation, 2, 6),             // first 4 spaces (indent level 1)
            mk_token(Token::Indentation, 6, 10),            // second 4 spaces (indent level 2)
            mk_token(Token::Text("b".to_string()), 10, 11), // "b"
        ];

        let result: Vec<(Token, std::ops::Range<usize>)> = sem_indentation(input);

        // Should have: Text, Newline, Indent, Indent, Text, Dedent, Dedent
        assert_eq!(result.len(), 7);
        // First Indent stores first Indent token
        assert_eq!(result[2].0, Token::Indent(vec![(Token::Indentation, 2..6)]));
        assert_eq!(result[2].1, 0..0, "Indent uses placeholder span");
        // Second Indent stores second Indent token
        assert_eq!(
            result[3].0,
            Token::Indent(vec![(Token::Indentation, 6..10)])
        );
        assert_eq!(result[3].1, 0..0, "Indent uses placeholder span");
    }

    #[test]
    fn test_dedent_level_tokens_have_correct_locations() {
        // Test: Dedent tokens should have locations at the position where dedentation occurs
        // Input: "a\n    b\nc" (a, newline, 4 spaces, b, newline, c)
        use crate::lex::testing::factories::{mk_token, Tokens};
        let input: Tokens = vec![
            mk_token(Token::Text("a".to_string()), 0, 1), // "a"
            mk_token(Token::Newline, 1, 2),               // "\n"
            mk_token(Token::Indentation, 2, 6),           // "    "
            mk_token(Token::Text("b".to_string()), 6, 7), // "b"
            mk_token(Token::Newline, 7, 8),               // "\n"
            mk_token(Token::Text("c".to_string()), 8, 9), // "c" (dedented back to level 0)
        ];

        let result: Vec<(Token, std::ops::Range<usize>)> = sem_indentation(input);

        // Expected:
        // - Text("a"), Newline, Indent, Text("b"), Newline, Dedent, Text("c")
        assert_eq!(result.len(), 7);
        // Dedent has no source tokens (purely structural)
        assert_eq!(result[5].0, Token::Dedent(vec![]));
        assert_eq!(result[5].1, 0..0, "Dedent uses placeholder span");
    }

    #[test]
    fn test_multiple_dedent_levels_have_correct_locations() {
        // Test: Multiple Dedent tokens should all have the same location (position of dedentation)
        // Input: "a\n        b\nc" (2 levels in, then 2 levels out)
        use crate::lex::testing::factories::{mk_token, Tokens};
        let input: Tokens = vec![
            mk_token(Token::Text("a".to_string()), 0, 1),
            mk_token(Token::Newline, 1, 2),
            mk_token(Token::Indentation, 2, 6),
            mk_token(Token::Indentation, 6, 10),
            mk_token(Token::Text("b".to_string()), 10, 11),
            mk_token(Token::Newline, 11, 12),
            mk_token(Token::Text("c".to_string()), 12, 13), // Back to level 0
        ];

        let result: Vec<(Token, std::ops::Range<usize>)> = sem_indentation(input);

        // Expected: Text("a"), Newline, Indent, Indent, Text("b"), Newline, Dedent, Dedent, Text("c")
        // Should have 2 Dedent tokens before Text("c")
        assert_eq!(result.len(), 9);
        // Dedent tokens have no source tokens (purely structural)
        assert_eq!(result[6].0, Token::Dedent(vec![]));
        assert_eq!(result[6].1, 0..0, "Dedent uses placeholder span");
        assert_eq!(result[7].0, Token::Dedent(vec![]));
        assert_eq!(result[7].1, 0..0, "Dedent uses placeholder span");
        assert_eq!(result[8], mk_token(Token::Text("c".to_string()), 12, 13));
    }

    #[test]
    fn test_eof_dedent_uses_correct_location() {
        // Test: Dedent tokens at end of file should use the EOF position
        // Input: "a\n    b" (ends while indented)
        use crate::lex::testing::factories::{mk_token, Tokens};
        let input: Tokens = vec![
            mk_token(Token::Text("a".to_string()), 0, 1),
            mk_token(Token::Newline, 1, 2),
            mk_token(Token::Indentation, 2, 6),
            mk_token(Token::Text("b".to_string()), 6, 7),
        ];

        let result: Vec<(Token, std::ops::Range<usize>)> = sem_indentation(input);

        // Last token should be Dedent
        let last = result.last().unwrap();
        assert_eq!(last.0, Token::Dedent(vec![]));
        assert_eq!(last.1, 0..0, "Dedent uses placeholder span");
    }

    #[test]
    fn test_locations_with_real_lex_content() {
        // Test with actual lex content: a simple list
        let source = "Item 1\n    - Subitem A\n    - Subitem B";
        // Positions: 0..6 "Item 1", 6..7 "\n", 7..11 "    ", 11..12 "-", 12..13 " ",
        //            13..20 "Subitem", 20..21 " ", 21..22 "A", 22..23 "\n",
        //            23..27 "    ", 27..28 "-", 28..29 " ", 29..36 "Subitem", 36..37 " ", 37..38 "B"

        let tokens = crate::lex::lexers::tokenize(source);
        let result = sem_indentation(tokens);

        // Find the Indent token
        let indent_level_pos = result
            .iter()
            .position(|(t, _)| matches!(t, Token::Indent(_)))
            .unwrap();
        let (indent_token, indent_location) = &result[indent_level_pos];

        // Indent should contain the original Indent token with its span
        assert!(matches!(indent_token, Token::Indent(_)));
        if let Token::Indent(source_tokens) = indent_token {
            assert_eq!(
                source_tokens.len(),
                1,
                "Indent should have one source token"
            );
            assert_eq!(source_tokens[0].0, Token::Indentation);
            assert_eq!(
                source_tokens[0].1,
                7..11,
                "Source Indent should be at 7..11"
            );
        }
        // Indent uses placeholder span
        assert_eq!(*indent_location, 0..0, "Indent uses placeholder span");

        // Find the Dedent token (should be at end)
        let dedent_pos = result
            .iter()
            .position(|(t, _)| matches!(t, Token::Dedent(_)))
            .unwrap();
        let (dedent_token, dedent_location) = &result[dedent_pos];

        // Dedent has no source tokens (purely structural)
        assert_eq!(*dedent_token, Token::Dedent(vec![]));
        assert_eq!(*dedent_location, 0..0, "Dedent uses placeholder span");
    }

    #[test]
    fn test_blank_lines_preserve_location_tracking() {
        // Test that blank lines don't break location tracking for indentation
        use crate::lex::testing::factories::{mk_token, Tokens};
        let input: Tokens = vec![
            mk_token(Token::Text("a".to_string()), 0, 1),
            mk_token(Token::Newline, 1, 2),
            mk_token(Token::Newline, 2, 3), // Blank line (will be handled by blank_line_transform)
            mk_token(Token::Indentation, 3, 7),
            mk_token(Token::Text("b".to_string()), 7, 8),
        ];

        let result: Vec<(Token, std::ops::Range<usize>)> = sem_indentation(input);

        // The Indent should use placeholder span
        let indent_pos = result
            .iter()
            .position(|(t, _)| matches!(t, Token::Indent(_)))
            .unwrap();
        assert_eq!(result[indent_pos].1, 0..0, "Indent uses placeholder span");
    }

    #[test]
    fn test_unbalanced_indent_at_eof() {
        // Test case: file ends while indented - should emit dedents to balance
        use crate::lex::testing::factories::{mk_token, Tokens};
        let input: Tokens = vec![
            mk_token(Token::Text("a".to_string()), 0, 1),
            mk_token(Token::Newline, 1, 2),
            mk_token(Token::Indentation, 2, 6),
            mk_token(Token::Indentation, 6, 10),
            mk_token(Token::Text("b".to_string()), 10, 11),
            // EOF at indent level 2 - should emit 2 dedents
        ];

        let result: Vec<(Token, std::ops::Range<usize>)> = sem_indentation(input);

        let dedent_count = result
            .iter()
            .filter(|(t, _)| matches!(t, Token::Dedent(_)))
            .count();

        assert_eq!(
            dedent_count, 2,
            "Should have 2 dedents to close 2 open indentation levels"
        );
    }

    #[test]
    fn test_rapid_indent_dedent_cycles() {
        // Test case: multiple indent/dedent cycles
        use crate::lex::testing::factories::{mk_token, Tokens};
        let input: Tokens = vec![
            mk_token(Token::Text("a".to_string()), 0, 1),
            mk_token(Token::Newline, 1, 2),
            mk_token(Token::Indentation, 2, 6),
            mk_token(Token::Text("b".to_string()), 6, 7),
            mk_token(Token::Newline, 7, 8),
            mk_token(Token::Text("c".to_string()), 8, 9), // Back at level 0
            mk_token(Token::Newline, 9, 10),
            mk_token(Token::Indentation, 10, 14),
            mk_token(Token::Text("d".to_string()), 14, 15),
            mk_token(Token::Newline, 15, 16),
            mk_token(Token::Text("e".to_string()), 16, 17), // Back at level 0
        ];

        let result: Vec<(Token, std::ops::Range<usize>)> = sem_indentation(input);

        let dedent_count = result
            .iter()
            .filter(|(t, _)| matches!(t, Token::Dedent(_)))
            .count();
        let indent_count = result
            .iter()
            .filter(|(t, _)| matches!(t, Token::Indent(_)))
            .count();

        // Should have 2 indents and 2 dedents for the two cycles
        assert_eq!(indent_count, 2, "Should have 2 Indent tokens");
        assert_eq!(dedent_count, 2, "Should have 2 Dedent tokens");
    }

    #[test]
    fn test_deep_nesting_unbalanced() {
        // Test case: nested indentation (one level deeper each line)
        // Line 1: "a" (level 0)
        // Line 2: 1 indent + "b" (level 1) → emits 1 Indent
        // Line 3: 2 indents + "c" (level 2) → emits 1 Indent
        // EOF at level 2 - should emit 2 dedents to close
        use crate::lex::testing::factories::{mk_token, Tokens};
        let input: Tokens = vec![
            mk_token(Token::Text("a".to_string()), 0, 1),
            mk_token(Token::Newline, 1, 2),
            mk_token(Token::Indentation, 2, 6),
            mk_token(Token::Text("b".to_string()), 6, 7),
            mk_token(Token::Newline, 7, 8),
            mk_token(Token::Indentation, 8, 12),
            mk_token(Token::Indentation, 12, 16),
            mk_token(Token::Text("c".to_string()), 16, 17),
            // EOF at level 2 - should emit 2 dedents
        ];

        let result: Vec<(Token, std::ops::Range<usize>)> = sem_indentation(input);

        let dedent_count = result
            .iter()
            .filter(|(t, _)| matches!(t, Token::Dedent(_)))
            .count();
        let indent_count = result
            .iter()
            .filter(|(t, _)| matches!(t, Token::Indent(_)))
            .count();

        assert_eq!(
            indent_count, 2,
            "Should have 2 Indent tokens (1 for 0→1, 1 for 1→2)"
        );
        assert_eq!(
            dedent_count, 2,
            "Should have 2 Dedent tokens to close 2 open indentation levels at EOF"
        );
    }

    #[test]
    fn test_partial_dedent_unbalanced() {
        // Test case: partial dedent (from level 2 to level 1)
        use crate::lex::testing::factories::{mk_token, Tokens};
        let input: Tokens = vec![
            mk_token(Token::Text("a".to_string()), 0, 1),
            mk_token(Token::Newline, 1, 2),
            mk_token(Token::Indentation, 2, 6),
            mk_token(Token::Indentation, 6, 10),
            mk_token(Token::Text("b".to_string()), 10, 11),
            mk_token(Token::Newline, 11, 12),
            mk_token(Token::Text("c".to_string()), 12, 13), // Back to level 1
            mk_token(Token::Newline, 13, 14),
            // EOF at level 1 - should emit 1 dedent
        ];

        let result: Vec<(Token, std::ops::Range<usize>)> = sem_indentation(input);

        let dedent_count = result
            .iter()
            .filter(|(t, _)| matches!(t, Token::Dedent(_)))
            .count();

        assert_eq!(
            dedent_count, 2,
            "Should have 2 dedents total (1 from level 2 to 1, 1 from level 1 to 0)"
        );
    }
}
