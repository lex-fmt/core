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
        if target_level > current_level {
            // Need to indent: add IndentLevel tokens for each additional level
            for _ in 0..(target_level - current_level) {
                result.push(Token::IndentLevel);
            }
        } else if target_level < current_level {
            // Need to dedent: add DedentLevel tokens for each reduced level
            for _ in 0..(current_level - target_level) {
                result.push(Token::DedentLevel);
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

/// Transform indentation while preserving source spans
/// Synthetic tokens (IndentLevel, DedentLevel) are given empty spans (0..0)
pub fn transform_indentation_with_spans(
    tokens_with_spans: Vec<(Token, std::ops::Range<usize>)>,
) -> Vec<(Token, std::ops::Range<usize>)> {
    // Extract just the tokens for processing
    let tokens: Vec<Token> = tokens_with_spans.iter().map(|(t, _)| t.clone()).collect();

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
                // Preserve the newline span
                result.push((Token::Newline, tokens_with_spans[j].1.clone()));
                j += 1;
            }
            i = j;
            continue;
        }

        // Calculate the target indentation level for this line
        let target_level = line_indent_level;

        // Generate appropriate IndentLevel/DedentLevel tokens with empty spans
        if target_level > current_level {
            for _ in 0..(target_level - current_level) {
                result.push((Token::IndentLevel, 0..0));
            }
        } else if target_level < current_level {
            for _ in 0..(current_level - target_level) {
                result.push((Token::DedentLevel, 0..0));
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

        // Process the rest of the line, keeping all remaining tokens with spans
        while j < tokens.len() && !matches!(tokens[j], Token::Newline) {
            result.push((tokens[j].clone(), tokens_with_spans[j].1.clone()));
            j += 1;
        }

        // Add the newline token if we haven't reached the end
        if j < tokens.len() && matches!(tokens[j], Token::Newline) {
            result.push((Token::Newline, tokens_with_spans[j].1.clone()));
            j += 1;
        }

        i = j;
    }

    // Add dedents to close all remaining indentation levels (with empty spans)
    for _ in 0..current_level {
        result.push((Token::DedentLevel, 0..0));
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
}
