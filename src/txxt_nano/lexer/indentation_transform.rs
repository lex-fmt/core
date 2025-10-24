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

        // Process the rest of the line, skipping Indent tokens
        let mut j = line_start;
        while j < tokens.len() && !matches!(tokens[j], Token::Newline) {
            if !matches!(tokens[j], Token::Indent) {
                result.push(tokens[j].clone());
            }
            j += 1;
        }

        // Add the newline token if we haven't reached the end
        if j < tokens.len() && matches!(tokens[j], Token::Newline) {
            result.push(Token::Newline);
            j += 1;
        }

        i = j;
    }

    // Always add a final DedentLevel to close the document structure
    result.push(Token::DedentLevel);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_indentation() {
        let input = vec![
            Token::Text,
            Token::Newline,
            Token::Indent,
            Token::Dash,
            Token::Newline,
        ];

        let result = transform_indentation(input);

        assert_eq!(
            result,
            vec![
                Token::Text,
                Token::Newline,
                Token::IndentLevel,
                Token::Dash,
                Token::Newline,
                Token::DedentLevel,
            ]
        );
    }

    #[test]
    fn test_multiple_indent_levels() {
        let input = vec![
            Token::Text,
            Token::Newline,
            Token::Indent,
            Token::Indent,
            Token::Dash,
            Token::Newline,
            Token::Indent,
            Token::Text,
            Token::Newline,
        ];

        let result = transform_indentation(input);

        assert_eq!(
            result,
            vec![
                Token::Text,
                Token::Newline,
                Token::IndentLevel,
                Token::IndentLevel,
                Token::Dash,
                Token::Newline,
                Token::DedentLevel,
                Token::Text,
                Token::Newline,
                Token::DedentLevel,
            ]
        );
    }

    #[test]
    fn test_nested_structure() {
        let input = vec![
            // Line 1: "1. Session"
            Token::Number,
            Token::Period,
            Token::Whitespace,
            Token::Text,
            Token::Newline,
            // Line 2: "    - Item 1"
            Token::Indent,
            Token::Dash,
            Token::Whitespace,
            Token::Text,
            Token::Whitespace,
            Token::Number,
            Token::Newline,
            // Line 3: "    - Item 2"
            Token::Indent,
            Token::Dash,
            Token::Whitespace,
            Token::Text,
            Token::Whitespace,
            Token::Number,
            Token::Newline,
            // Line 4: "        - Nested"
            Token::Indent,
            Token::Indent,
            Token::Dash,
            Token::Whitespace,
            Token::Text,
            Token::Newline,
            // Line 5: "2. Another"
            Token::Number,
            Token::Period,
            Token::Whitespace,
            Token::Text,
            Token::Newline,
        ];

        let result = transform_indentation(input);

        // Expected: Indent for line 2, Indent for line 4, Dedent for line 5
        assert_eq!(
            result,
            vec![
                // Line 1
                Token::Number,
                Token::Period,
                Token::Whitespace,
                Token::Text,
                Token::Newline,
                // Line 2
                Token::IndentLevel,
                Token::Dash,
                Token::Whitespace,
                Token::Text,
                Token::Whitespace,
                Token::Number,
                Token::Newline,
                // Line 3
                Token::Dash,
                Token::Whitespace,
                Token::Text,
                Token::Whitespace,
                Token::Number,
                Token::Newline,
                // Line 4
                Token::IndentLevel,
                Token::Dash,
                Token::Whitespace,
                Token::Text,
                Token::Newline,
                // Line 5
                Token::DedentLevel,
                Token::DedentLevel,
                Token::Number,
                Token::Period,
                Token::Whitespace,
                Token::Text,
                Token::Newline,
                Token::DedentLevel,
            ]
        );
    }

    #[test]
    fn test_no_indentation() {
        let input = vec![Token::Text, Token::Newline, Token::Text, Token::Newline];

        let result = transform_indentation(input.clone());

        let mut expected = input.clone();
        expected.push(Token::DedentLevel);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_empty_input() {
        let input = vec![];
        let result = transform_indentation(input);
        assert_eq!(result, vec![Token::DedentLevel]);
    }

    #[test]
    fn test_single_line() {
        let input = vec![Token::Text];
        let result = transform_indentation(input);
        assert_eq!(result, vec![Token::Text, Token::DedentLevel]);
    }

    #[test]
    fn test_whitespace_remainders() {
        // Test case with whitespace remainders (10 spaces = 2 indent levels + 2 remaining)
        let input = vec![
            Token::Indent,
            Token::Indent,
            Token::Text, // This represents "  hello" (2 spaces + text)
            Token::Newline,
        ];

        let result = transform_indentation(input);

        assert_eq!(
            result,
            vec![
                Token::IndentLevel,
                Token::IndentLevel,
                Token::Text,
                Token::Newline,
                Token::DedentLevel,
            ]
        );
    }

    #[test]
    fn test_count_line_indent_steps() {
        let tokens = vec![Token::Indent, Token::Indent, Token::Dash, Token::Text];

        assert_eq!(count_line_indent_steps(&tokens, 0), 2);
        assert_eq!(count_line_indent_steps(&tokens, 2), 0);
    }

    #[test]
    fn test_find_line_start() {
        let tokens = vec![Token::Text, Token::Newline, Token::Indent, Token::Dash];

        assert_eq!(find_line_start(&tokens, 0), 0);
        assert_eq!(find_line_start(&tokens, 2), 2);
        assert_eq!(find_line_start(&tokens, 3), 2);
    }
}
