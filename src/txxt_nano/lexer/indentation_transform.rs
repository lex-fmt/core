//! Indentation transformation for txxt lexer
//!
//! This module transforms raw Indent tokens into semantic IndentLevel and DedentLevel tokens
//! based on the indentation levels in the document.

use crate::txxt_nano::lexer::tokens::Token;

/// Transform raw Indent tokens into semantic IndentLevel and DedentLevel tokens
///
/// This function processes a token stream and converts consecutive Indent tokens
/// into appropriate IndentLevel and DedentLevel tokens based on the indentation level changes.
///
/// # Algorithm
///
/// 1. Track the current indentation level (number of Indent tokens)
/// 2. For each line, count the Indent tokens at the beginning
/// 3. Compare with the previous line's indentation level:
///    - If greater: emit IndentLevel tokens for each additional level
///    - If less: emit DedentLevel tokens for each reduced level
///    - If equal: no indentation tokens needed
/// 4. Replace Indent tokens with the appropriate semantic tokens
/// 5. Always add a final DedentLevel to close the document structure
///
/// # Example
///
/// Input tokens: `[Text, Newline, Indent, Indent, Dash, Newline, Indent, Text]`
/// Output tokens: `[Text, Newline, IndentLevel, IndentLevel, Dash, Newline, DedentLevel, Text, DedentLevel]`
pub fn transform_indentation(tokens: Vec<Token>) -> Vec<Token> {
    let mut result = Vec::new();
    let mut current_level = 0;
    let mut i = 0;

    while i < tokens.len() {
        // Find the start of the current line
        let line_start = find_line_start(&tokens, i);

        // Count Indent tokens at the beginning of this line
        let line_indent_level = count_line_indent_steps(&tokens, line_start);

        // Check if this line is blank (only contains indentation and newline)
        let is_blank_line = is_line_blank(&tokens, line_start);

        // Skip blank lines - they don't affect indentation level
        if is_blank_line {
            // Just add the newline token and continue
            // Blank lines preserve the current indentation level
            let mut j = line_start;
            while j < tokens.len() && !matches!(tokens[j], Token::Newline) {
                j += 1;
            }
            if j < tokens.len() && matches!(tokens[j], Token::Newline) {
                result.push(Token::Newline);
                j += 1;
            }
            i = j;
            continue;
        }

        // Calculate the target indentation level for this line
        let target_level = line_indent_level;

        // Generate appropriate IndentLevel/DedentLevel tokens
        match target_level.cmp(&current_level) {
            std::cmp::Ordering::Greater => {
                // Need to indent: add IndentLevel tokens for each additional level
                for _ in 0..(target_level - current_level) {
                    result.push(Token::IndentLevel);
                }
            }
            std::cmp::Ordering::Less => {
                // Need to dedent: add DedentLevel tokens for each reduced level
                for _ in 0..(current_level - target_level) {
                    result.push(Token::DedentLevel);
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
            if j < tokens.len() && matches!(tokens[j], Token::Indent) {
                j += 1;
            }
        }

        // Process the rest of the line, keeping all remaining tokens
        while j < tokens.len() && !matches!(tokens[j], Token::Newline) {
            result.push(tokens[j].clone());
            j += 1;
        }

        // Add the newline token if we haven't reached the end
        if j < tokens.len() && matches!(tokens[j], Token::Newline) {
            result.push(Token::Newline);
            j += 1;
        }

        i = j;
    }

    // Add dedents to close all remaining indentation levels
    for _ in 0..current_level {
        result.push(Token::DedentLevel);
    }

    result
}

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
    while i < tokens.len() && matches!(tokens[i], Token::Indent) {
        i += 1;
    }

    // Check if the next token is a newline (or end of file)
    i >= tokens.len() || matches!(tokens[i], Token::Newline)
}

/// Count consecutive Indent tokens at the beginning of a line
fn count_line_indent_steps(tokens: &[Token], start: usize) -> usize {
    let mut count = 0;
    let mut i = start;

    while i < tokens.len() && matches!(tokens[i], Token::Indent) {
        count += 1;
        i += 1;
    }

    count
}

/// Transform indentation while preserving source locations
/// Synthetic tokens (IndentLevel, DedentLevel) are given meaningful locations:
/// - IndentLevel: location covers the Indent tokens it represents
/// - DedentLevel: location at the start of the line where dedentation occurs
pub fn transform_indentation_with_locations(
    tokens_with_locations: Vec<(Token, std::ops::Range<usize>)>,
) -> Vec<(Token, std::ops::Range<usize>)> {
    // Extract just the tokens for processing
    let tokens: Vec<Token> = tokens_with_locations
        .iter()
        .map(|(t, _)| t.clone())
        .collect();

    let mut result = Vec::new();
    let mut current_level = 0;
    let mut i = 0;

    while i < tokens.len() {
        // Find the start of the current line
        let line_start = find_line_start(&tokens, i);

        // Count Indent tokens at the beginning of this line
        let line_indent_level = count_line_indent_steps(&tokens, line_start);

        // Check if this line is blank (only contains indentation and newline)
        let is_blank_line = is_line_blank(&tokens, line_start);

        // Skip blank lines - they don't affect indentation level
        if is_blank_line {
            let mut j = line_start;
            while j < tokens.len() && !matches!(tokens[j], Token::Newline) {
                j += 1;
            }
            if j < tokens.len() && matches!(tokens[j], Token::Newline) {
                // Preserve the newline location
                result.push((Token::Newline, tokens_with_locations[j].1.clone()));
                j += 1;
            }
            i = j;
            continue;
        }

        // Calculate the target indentation level for this line
        let target_level = line_indent_level;

        // Generate appropriate IndentLevel/DedentLevel tokens with meaningful locations
        match target_level.cmp(&current_level) {
            std::cmp::Ordering::Greater => {
                // IndentLevel tokens: each gets the location of its corresponding Indent token
                let indent_start_idx = line_start;
                for level_idx in 0..(target_level - current_level) {
                    let indent_token_idx = indent_start_idx + current_level + level_idx;
                    if indent_token_idx < tokens_with_locations.len()
                        && matches!(tokens_with_locations[indent_token_idx].0, Token::Indent)
                    {
                        // Use the location of the Indent token
                        let location = tokens_with_locations[indent_token_idx].1.clone();
                        result.push((Token::IndentLevel, location));
                    } else {
                        // Fallback: use the location of the first content token on this line
                        let fallback_location = if line_start < tokens_with_locations.len() {
                            tokens_with_locations[line_start].1.start
                                ..tokens_with_locations[line_start].1.start
                        } else {
                            0..0
                        };
                        result.push((Token::IndentLevel, fallback_location));
                    }
                }
            }
            std::cmp::Ordering::Less => {
                // DedentLevel tokens: use the location at the start of the new line (where dedent occurs)
                // This represents the position where we "return" to a previous indentation level
                let dedent_location = if line_start < tokens_with_locations.len() {
                    // Point to the start of the first token on the new line
                    let start = tokens_with_locations[line_start].1.start;
                    start..start
                } else {
                    // End of file: use empty location at the end
                    let end = if !tokens_with_locations.is_empty() {
                        tokens_with_locations.last().unwrap().1.end
                    } else {
                        0
                    };
                    end..end
                };

                for _ in 0..(current_level - target_level) {
                    result.push((Token::DedentLevel, dedent_location.clone()));
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
            if j < tokens.len() && matches!(tokens[j], Token::Indent) {
                j += 1;
            }
        }

        // Process the rest of the line, keeping all remaining tokens with locations
        while j < tokens.len() && !matches!(tokens[j], Token::Newline) {
            result.push((tokens[j].clone(), tokens_with_locations[j].1.clone()));
            j += 1;
        }

        // Add the newline token if we haven't reached the end
        if j < tokens.len() && matches!(tokens[j], Token::Newline) {
            result.push((Token::Newline, tokens_with_locations[j].1.clone()));
            j += 1;
        }

        i = j;
    }

    // Add dedents to close all remaining indentation levels
    // These occur at the end of file, so use the end position
    let eof_location = if !tokens_with_locations.is_empty() {
        let end = tokens_with_locations.last().unwrap().1.end;
        end..end
    } else {
        0..0
    };

    for _ in 0..current_level {
        result.push((Token::DedentLevel, eof_location.clone()));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_indentation() {
        let input = vec![
            Token::Text("a".to_string()),
            Token::Newline,
            Token::Indent,
            Token::Dash,
            Token::Newline,
        ];

        let result = transform_indentation(input);

        assert_eq!(
            result,
            vec![
                Token::Text("a".to_string()),
                Token::Newline,
                Token::IndentLevel,
                Token::Dash,
                Token::Newline,
                Token::DedentLevel, // Dedent from level 1 to level 0
            ]
        );
    }

    #[test]
    fn test_multiple_indent_levels() {
        let input = vec![
            Token::Text("a".to_string()),
            Token::Newline,
            Token::Indent,
            Token::Indent,
            Token::Dash,
            Token::Newline,
            Token::Indent,
            Token::Text("b".to_string()),
            Token::Newline,
        ];

        let result = transform_indentation(input);

        assert_eq!(
            result,
            vec![
                Token::Text("a".to_string()),
                Token::Newline,
                Token::IndentLevel,
                Token::IndentLevel,
                Token::Dash,
                Token::Newline,
                Token::DedentLevel,
                Token::Text("b".to_string()),
                Token::Newline,
                Token::DedentLevel, // Dedent from level 1 to level 0
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
            Token::Indent,
            Token::Dash,
            Token::Whitespace,
            Token::Text("Item".to_string()),
            Token::Whitespace,
            Token::Number("1".to_string()),
            Token::Newline,
            // Line 3: "    - Item 2"
            Token::Indent,
            Token::Dash,
            Token::Whitespace,
            Token::Text("Item".to_string()),
            Token::Whitespace,
            Token::Number("2".to_string()),
            Token::Newline,
            // Line 4: "        - Nested"
            Token::Indent,
            Token::Indent,
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

        let result = transform_indentation(input);

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
                Token::IndentLevel,
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
                Token::IndentLevel,
                Token::Dash,
                Token::Whitespace,
                Token::Text("Nested".to_string()),
                Token::Newline,
                // Line 5
                Token::DedentLevel,
                Token::DedentLevel,
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

        let result = transform_indentation(input.clone());

        // No changes expected - no indentation, no DedentLevel at EOF
        assert_eq!(result, input);
    }

    #[test]
    fn test_empty_input() {
        let input = vec![];
        let result = transform_indentation(input);
        assert_eq!(result, vec![]);
    }

    #[test]
    fn test_single_line() {
        let input = vec![Token::Text("a".to_string())];
        let result = transform_indentation(input);
        assert_eq!(result, vec![Token::Text("a".to_string())]);
    }

    #[test]
    fn test_blank_lines() {
        // Test case: blank lines should not affect indentation level
        let input = vec![
            Token::Text("a".to_string()),
            Token::Newline,
            Token::Indent,
            Token::Dash,
            Token::Newline,
            Token::Newline, // blank line
            Token::Dash,
            Token::Newline,
        ];

        let result = transform_indentation(input);

        assert_eq!(
            result,
            vec![
                Token::Text("a".to_string()),
                Token::Newline,
                Token::IndentLevel,
                Token::Dash,
                Token::Newline,
                Token::Newline,     // blank line preserved
                Token::DedentLevel, // dedent from level 1 to level 0
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
            Token::Indent,
            Token::Dash,
            Token::Newline,
            Token::Indent,
            Token::Newline, // blank line with indentation
            Token::Dash,
            Token::Newline,
        ];

        let result = transform_indentation(input);

        assert_eq!(
            result,
            vec![
                Token::Text("a".to_string()),
                Token::Newline,
                Token::IndentLevel,
                Token::Dash,
                Token::Newline,
                Token::Newline,     // blank line preserved
                Token::DedentLevel, // dedent from level 1 to level 0
                Token::Dash,
                Token::Newline,
            ]
        );
    }

    #[test]
    fn test_whitespace_remainders() {
        // Test case with whitespace remainders (10 spaces = 2 indent levels + 2 remaining)
        let input = vec![
            Token::Indent,
            Token::Indent,
            Token::Text("  hello".to_string()), // This represents "  hello" (2 spaces + text)
            Token::Newline,
        ];

        let result = transform_indentation(input);

        assert_eq!(
            result,
            vec![
                Token::IndentLevel,
                Token::IndentLevel,
                Token::Text("  hello".to_string()),
                Token::Newline,
                Token::DedentLevel, // Dedent from level 2 to level 1
                Token::DedentLevel, // Dedent from level 1 to level 0
            ]
        );
    }

    #[test]
    fn test_file_ending_while_indented() {
        // Test case: file ending while indented should emit proper dedents
        let input = vec![
            Token::Text("a".to_string()),
            Token::Newline,
            Token::Indent,
            Token::Dash,
            Token::Newline,
            Token::Indent,
            Token::Indent,
            Token::Text("b".to_string()),
            // File ends here without explicit dedents
        ];

        let result = transform_indentation(input);

        assert_eq!(
            result,
            vec![
                Token::Text("a".to_string()),
                Token::Newline,
                Token::IndentLevel,
                Token::Dash,
                Token::Newline,
                Token::IndentLevel,
                Token::Text("b".to_string()),
                Token::DedentLevel, // Should dedent from level 2 to level 1
                Token::DedentLevel, // Should dedent from level 1 to level 0
            ]
        );
    }

    #[test]
    fn test_sharp_drop_in_indentation() {
        // Test case: sharp drop from level 3 to level 0
        let input = vec![
            Token::Text("a".to_string()),
            Token::Newline,
            Token::Indent,
            Token::Indent,
            Token::Indent,
            Token::Dash,
            Token::Newline,
            Token::Text("b".to_string()), // Back to level 0
            Token::Newline,
        ];

        let result = transform_indentation(input);

        assert_eq!(
            result,
            vec![
                Token::Text("a".to_string()),
                Token::Newline,
                Token::IndentLevel,
                Token::IndentLevel,
                Token::IndentLevel,
                Token::Dash,
                Token::Newline,
                Token::DedentLevel, // Dedent from level 3 to level 2
                Token::DedentLevel, // Dedent from level 2 to level 1
                Token::DedentLevel, // Dedent from level 1 to level 0
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
            Token::Indent,
            Token::Dash,
            Token::Newline,
            Token::Newline, // blank line 1
            Token::Newline, // blank line 2
            Token::Newline, // blank line 3
            Token::Dash,    // Should be at same level as first dash
            Token::Newline,
        ];

        let result = transform_indentation(input);

        assert_eq!(
            result,
            vec![
                Token::Text("a".to_string()),
                Token::Newline,
                Token::IndentLevel,
                Token::Dash,
                Token::Newline,
                Token::Newline,     // blank line 1
                Token::Newline,     // blank line 2
                Token::Newline,     // blank line 3
                Token::DedentLevel, // Dedent from level 1 to level 0
                Token::Dash,        // Now at level 0
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

        let result = transform_indentation(input);

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
            Token::Indent,
            Token::Indent,
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
            Token::Indent,
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
            Token::Indent,
            Token::Indent,
            Token::Text("Foo".to_string()),
            Token::Newline,
            // Line 2: "        Foo2" (2 indent levels)
            Token::Indent,
            Token::Indent,
            Token::Text("Foo2".to_string()),
            Token::Newline,
            // Line 3: "    " (1 indent level BUT NO CONTENT - should be ignored)
            Token::Indent,
            Token::Newline,
            // Line 4: "        Bar" (2 indent levels)
            Token::Indent,
            Token::Indent,
            Token::Text("Bar".to_string()),
            Token::Newline,
        ];

        let result = transform_indentation(input);

        // Expected: Level stays at 2, no dedent/re-indent around the blank line
        assert_eq!(
            result,
            vec![
                // Line 1
                Token::IndentLevel, // From 0 to 1
                Token::IndentLevel, // From 1 to 2
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
                Token::DedentLevel, // From 2 to 1
                Token::DedentLevel, // From 1 to 0
            ],
            "Blank lines with only spaces should NOT produce dedent/indent tokens"
        );
    }

    // ========== SPAN TESTS ==========
    // Tests to verify that synthetic tokens (IndentLevel, DedentLevel) have correct locations

    #[test]
    fn test_indent_level_tokens_have_correct_locations() {
        // Test: IndentLevel tokens should have locations that correspond to the Indent tokens they represent
        // Input: "a\n    b" (a, newline, 4 spaces, b)
        let input = vec![
            (Token::Text("a".to_string()), 0..1), // "a" at position 0-1
            (Token::Newline, 1..2),               // "\n" at position 1-2
            (Token::Indent, 2..6),                // "    " (4 spaces) at position 2-6
            (Token::Text("b".to_string()), 6..7), // "b" at position 6-7
        ];

        let result = transform_indentation_with_locations(input);

        // Expected:
        // - Text("a") with location 0..1
        // - Newline with location 1..2
        // - IndentLevel with location 2..6 (covers the Indent token)
        // - Text("b") with location 6..7
        // - DedentLevel with location 7..7 (at EOF)

        assert_eq!(result.len(), 5);
        assert_eq!(result[0], (Token::Text("a".to_string()), 0..1));
        assert_eq!(result[1], (Token::Newline, 1..2));
        assert_eq!(result[2].0, Token::IndentLevel);
        assert_eq!(
            result[2].1,
            2..6,
            "IndentLevel should have location of its Indent token"
        );
        assert_eq!(result[3], (Token::Text("b".to_string()), 6..7));
        assert_eq!(result[4].0, Token::DedentLevel);
        assert_eq!(
            result[4].1,
            7..7,
            "EOF DedentLevel should point to end of file"
        );
    }

    #[test]
    fn test_multiple_indent_levels_have_correct_locations() {
        // Test: Multiple IndentLevel tokens should each have locations of their respective Indent tokens
        // Input: "a\n        b" (a, newline, 8 spaces = 2 indent levels, b)
        let input = vec![
            (Token::Text("a".to_string()), 0..1),   // "a"
            (Token::Newline, 1..2),                 // "\n"
            (Token::Indent, 2..6),                  // first 4 spaces (indent level 1)
            (Token::Indent, 6..10),                 // second 4 spaces (indent level 2)
            (Token::Text("b".to_string()), 10..11), // "b"
        ];

        let result = transform_indentation_with_locations(input);

        // Should have: Text, Newline, IndentLevel, IndentLevel, Text, DedentLevel, DedentLevel
        assert_eq!(result.len(), 7);
        assert_eq!(result[2].0, Token::IndentLevel);
        assert_eq!(
            result[2].1,
            2..6,
            "First IndentLevel should have location 2..6"
        );
        assert_eq!(result[3].0, Token::IndentLevel);
        assert_eq!(
            result[3].1,
            6..10,
            "Second IndentLevel should have location 6..10"
        );
    }

    #[test]
    fn test_dedent_level_tokens_have_correct_locations() {
        // Test: DedentLevel tokens should have locations at the position where dedentation occurs
        // Input: "a\n    b\nc" (a, newline, 4 spaces, b, newline, c)
        let input = vec![
            (Token::Text("a".to_string()), 0..1), // "a"
            (Token::Newline, 1..2),               // "\n"
            (Token::Indent, 2..6),                // "    "
            (Token::Text("b".to_string()), 6..7), // "b"
            (Token::Newline, 7..8),               // "\n"
            (Token::Text("c".to_string()), 8..9), // "c" (dedented back to level 0)
        ];

        let result = transform_indentation_with_locations(input);

        // Expected:
        // - Text("a"), Newline, IndentLevel, Text("b"), Newline, DedentLevel, Text("c")
        assert_eq!(result.len(), 7);
        assert_eq!(result[5].0, Token::DedentLevel);
        assert_eq!(
            result[5].1,
            8..8,
            "DedentLevel should point to start of dedented line"
        );
    }

    #[test]
    fn test_multiple_dedent_levels_have_correct_locations() {
        // Test: Multiple DedentLevel tokens should all have the same location (position of dedentation)
        // Input: "a\n        b\nc" (2 levels in, then 2 levels out)
        let input = vec![
            (Token::Text("a".to_string()), 0..1),
            (Token::Newline, 1..2),
            (Token::Indent, 2..6),
            (Token::Indent, 6..10),
            (Token::Text("b".to_string()), 10..11),
            (Token::Newline, 11..12),
            (Token::Text("c".to_string()), 12..13), // Back to level 0
        ];

        let result = transform_indentation_with_locations(input);

        // Expected: Text("a"), Newline, IndentLevel, IndentLevel, Text("b"), Newline, DedentLevel, DedentLevel, Text("c")
        // Should have 2 DedentLevel tokens before Text("c")
        assert_eq!(result.len(), 9);
        assert_eq!(result[6].0, Token::DedentLevel);
        assert_eq!(
            result[6].1,
            12..12,
            "First DedentLevel should point to position 12"
        );
        assert_eq!(result[7].0, Token::DedentLevel);
        assert_eq!(
            result[7].1,
            12..12,
            "Second DedentLevel should point to position 12"
        );
        assert_eq!(result[8], (Token::Text("c".to_string()), 12..13));
    }

    #[test]
    fn test_eof_dedent_uses_correct_location() {
        // Test: DedentLevel tokens at end of file should use the EOF position
        // Input: "a\n    b" (ends while indented)
        let input = vec![
            (Token::Text("a".to_string()), 0..1),
            (Token::Newline, 1..2),
            (Token::Indent, 2..6),
            (Token::Text("b".to_string()), 6..7),
        ];

        let result = transform_indentation_with_locations(input);

        // Last token should be DedentLevel with location at EOF (7..7)
        let last = result.last().unwrap();
        assert_eq!(last.0, Token::DedentLevel);
        assert_eq!(last.1, 7..7, "EOF DedentLevel should use end position");
    }

    #[test]
    fn test_locations_with_real_txxt_content() {
        // Test with actual txxt content: a simple list
        let source = "Item 1\n    - Subitem A\n    - Subitem B";
        // Positions: 0..6 "Item 1", 6..7 "\n", 7..11 "    ", 11..12 "-", 12..13 " ",
        //            13..20 "Subitem", 20..21 " ", 21..22 "A", 22..23 "\n",
        //            23..27 "    ", 27..28 "-", 28..29 " ", 29..36 "Subitem", 36..37 " ", 37..38 "B"

        let tokens_with_locations = crate::txxt_nano::lexer::tokenize_with_locations(source);
        let result = transform_indentation_with_locations(tokens_with_locations);

        // Find the IndentLevel token
        let indent_level_pos = result
            .iter()
            .position(|(t, _)| matches!(t, Token::IndentLevel))
            .unwrap();
        let (indent_token, indent_location) = &result[indent_level_pos];

        assert_eq!(*indent_token, Token::IndentLevel);
        assert_ne!(
            *indent_location,
            0..0,
            "IndentLevel should not have empty location"
        );
        assert_eq!(
            indent_location.start, 7,
            "IndentLevel should start at position 7"
        );
        assert_eq!(
            indent_location.end, 11,
            "IndentLevel should end at position 11"
        );

        // Find the DedentLevel token (should be at end)
        let dedent_pos = result
            .iter()
            .position(|(t, _)| matches!(t, Token::DedentLevel))
            .unwrap();
        let (dedent_token, dedent_location) = &result[dedent_pos];

        assert_eq!(*dedent_token, Token::DedentLevel);
        assert_ne!(
            *dedent_location,
            0..0,
            "DedentLevel should not have empty location"
        );
    }

    #[test]
    fn test_blank_lines_preserve_location_tracking() {
        // Test that blank lines don't break location tracking for indentation
        let input = vec![
            (Token::Text("a".to_string()), 0..1),
            (Token::Newline, 1..2),
            (Token::Newline, 2..3), // Blank line (will be handled by blank_line_transform)
            (Token::Indent, 3..7),
            (Token::Text("b".to_string()), 7..8),
        ];

        let result = transform_indentation_with_locations(input);

        // The IndentLevel should still have correct location
        let indent_pos = result
            .iter()
            .position(|(t, _)| matches!(t, Token::IndentLevel))
            .unwrap();
        assert_eq!(
            result[indent_pos].1,
            3..7,
            "IndentLevel location should be preserved"
        );
    }
}
