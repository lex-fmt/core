//! Experimental transformation: raw tokens → line tokens
//!
//! This transformation converts a flat stream of raw tokens (output from the current
//! lexer's 3 existing steps) into line tokens, where each token represents one logical line.
//!
//! Each line token stores the original raw tokens that created it, allowing later
//! stages to pass these tokens directly to existing AST constructors, which automatically
//! handles location tracking and AST creation.
//!
//! Line token types:
//! - BLANK_LINE: Empty line
//! - ANNOTATION_LINE: Line with :: markers
//! - SUBJECT_LINE: Line ending with colon
//! - LIST_LINE: Line starting with list marker (-, 1., a., I., etc.)
//! - PARAGRAPH_LINE: Any other line
//! - INDENT_LEVEL / DEDENT_LEVEL: Structural tokens (pass through unchanged)

use crate::txxt::lexer::tokens::{LineToken, LineTokenType, Token};

/// Transform flat token stream into line tokens.
///
/// Groups consecutive tokens into semantic line units. Each line token preserves
/// the original raw tokens and classifies the line type.
///
/// Input: Flat token stream from lexer transformations (whitespace, indentation, blank-line processed)
/// Output: Vector of LineTokens where each token represents one logical line
///
/// Example:
/// ```text
/// Input tokens:
///   [Text("Title"), Colon, Newline, Indent, Text("Content"), Newline]
///
/// Output line tokens:
///   [
///     LineToken { source_tokens: [Text("Title"), Colon, Newline], line_type: SubjectLine },
///     LineToken { source_tokens: [Indent], line_type: IndentLevel },
///     LineToken { source_tokens: [Text("Content"), Newline], line_type: ParagraphLine },
///   ]
/// ```
pub fn experimental_transform_to_line_tokens(tokens: Vec<Token>) -> Vec<LineToken> {
    let mut line_tokens = Vec::new();
    let mut current_line = Vec::new();

    for token in tokens {
        // Structural tokens (IndentLevel, DedentLevel) are pass-through
        // They appear alone, not as part of lines
        if matches!(token, Token::IndentLevel) {
            if !current_line.is_empty() {
                line_tokens.push(classify_and_create_line_token(current_line));
                current_line = Vec::new();
            }
            line_tokens.push(LineToken {
                source_tokens: vec![token],
                line_type: LineTokenType::IndentLevel,
            });
            continue;
        }

        if matches!(token, Token::DedentLevel) {
            if !current_line.is_empty() {
                line_tokens.push(classify_and_create_line_token(current_line));
                current_line = Vec::new();
            }
            line_tokens.push(LineToken {
                source_tokens: vec![token],
                line_type: LineTokenType::DedentLevel,
            });
            continue;
        }

        // Accumulate tokens for current line
        current_line.push(token.clone());

        // Newline marks end of line
        if matches!(token, Token::Newline) {
            line_tokens.push(classify_and_create_line_token(current_line));
            current_line = Vec::new();
        }
    }

    // Handle any remaining tokens (if input doesn't end with newline)
    if !current_line.is_empty() {
        line_tokens.push(classify_and_create_line_token(current_line));
    }

    line_tokens
}

/// Classify tokens and create a line token with the appropriate type.
fn classify_and_create_line_token(tokens: Vec<Token>) -> LineToken {
    let line_type = classify_line_tokens(&tokens);
    LineToken {
        source_tokens: tokens,
        line_type,
    }
}

/// Determine the type of a line based on its tokens.
fn classify_line_tokens(tokens: &[Token]) -> LineTokenType {
    if tokens.is_empty() {
        return LineTokenType::ParagraphLine;
    }

    // BLANK_LINE: Only whitespace and newline tokens
    if is_blank_line(tokens) {
        return LineTokenType::BlankLine;
    }

    // ANNOTATION_LINE: Contains TxxtMarker (::)
    if contains_txxt_marker(tokens) {
        return LineTokenType::AnnotationLine;
    }

    // LIST_LINE: Starts with list marker (after optional indentation/whitespace)
    if is_list_line(tokens) {
        return LineTokenType::ListLine;
    }

    // SUBJECT_LINE: Ends with colon
    if ends_with_colon(tokens) {
        return LineTokenType::SubjectLine;
    }

    // Default: PARAGRAPH_LINE
    LineTokenType::ParagraphLine
}

/// Check if line is blank (only whitespace and newline)
fn is_blank_line(tokens: &[Token]) -> bool {
    tokens.iter().all(|t| {
        matches!(
            t,
            Token::Whitespace | Token::Indent | Token::Newline | Token::BlankLine
        )
    })
}

/// Check if line contains TxxtMarker (::)
fn contains_txxt_marker(tokens: &[Token]) -> bool {
    tokens.iter().any(|t| matches!(t, Token::TxxtMarker))
}

/// Check if line is a list item (starts with list marker after optional indentation)
fn is_list_line(tokens: &[Token]) -> bool {
    let mut i = 0;

    // Skip leading indentation and whitespace
    while i < tokens.len() && matches!(tokens[i], Token::Indent | Token::Whitespace) {
        i += 1;
    }

    // Check for plain list marker: Dash Whitespace
    if i + 1 < tokens.len()
        && matches!(tokens[i], Token::Dash)
        && matches!(tokens[i + 1], Token::Whitespace)
    {
        return true;
    }

    // Check for ordered list marker: (Number | Letter | RomanNumeral) (Period | CloseParen) Whitespace
    if i + 2 < tokens.len() {
        let has_number = matches!(tokens[i], Token::Number(_));
        let has_letter = matches!(tokens[i], Token::Text(ref s) if is_single_letter(s));
        let has_roman = matches!(tokens[i], Token::Text(ref s) if is_roman_numeral(s));
        let has_separator = matches!(tokens[i + 1], Token::Period | Token::CloseParen);
        let has_space = matches!(tokens[i + 2], Token::Whitespace);

        if (has_number || has_letter || has_roman) && has_separator && has_space {
            return true;
        }
    }

    false
}

/// Check if a string is a single letter (a-z, A-Z)
fn is_single_letter(s: &str) -> bool {
    s.len() == 1 && s.chars().next().map_or(false, |c| c.is_alphabetic())
}

/// Check if a string is a Roman numeral (I, II, III, IV, V, etc.)
fn is_roman_numeral(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    // Check if all characters are valid Roman numeral characters
    s.chars()
        .all(|c| matches!(c, 'I' | 'V' | 'X' | 'L' | 'C' | 'D' | 'M'))
        && s.chars().next().map_or(false, |c| c.is_uppercase())
}

/// Check if line ends with colon (ignoring trailing whitespace and newline)
fn ends_with_colon(tokens: &[Token]) -> bool {
    // Find last non-whitespace token before newline
    let mut i = tokens.len() as i32 - 1;

    while i >= 0 {
        let token = &tokens[i as usize];
        match token {
            Token::Newline | Token::Whitespace => {
                i -= 1;
            }
            Token::Colon => return true,
            _ => return false,
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blank_line_classification() {
        let tokens = vec![Token::Whitespace, Token::Newline];
        let line = classify_line_tokens(&tokens);
        assert_eq!(line, LineTokenType::BlankLine);
    }

    #[test]
    fn test_annotation_line_classification() {
        let tokens = vec![
            Token::TxxtMarker,
            Token::Whitespace,
            Token::Text("note".to_string()),
            Token::Whitespace,
            Token::TxxtMarker,
            Token::Newline,
        ];
        let line = classify_line_tokens(&tokens);
        assert_eq!(line, LineTokenType::AnnotationLine);
    }

    #[test]
    fn test_subject_line_classification() {
        let tokens = vec![
            Token::Text("Title".to_string()),
            Token::Colon,
            Token::Newline,
        ];
        let line = classify_line_tokens(&tokens);
        assert_eq!(line, LineTokenType::SubjectLine);
    }

    #[test]
    fn test_subject_line_with_spaces() {
        let tokens = vec![
            Token::Text("Title".to_string()),
            Token::Whitespace,
            Token::Text("with".to_string()),
            Token::Whitespace,
            Token::Text("spaces".to_string()),
            Token::Colon,
            Token::Whitespace,
            Token::Newline,
        ];
        let line = classify_line_tokens(&tokens);
        assert_eq!(line, LineTokenType::SubjectLine);
    }

    #[test]
    fn test_list_line_dash_marker() {
        let tokens = vec![
            Token::Dash,
            Token::Whitespace,
            Token::Text("Item".to_string()),
            Token::Newline,
        ];
        let line = classify_line_tokens(&tokens);
        assert_eq!(line, LineTokenType::ListLine);
    }

    #[test]
    fn test_list_line_number_marker() {
        let tokens = vec![
            Token::Number("1".to_string()),
            Token::Period,
            Token::Whitespace,
            Token::Text("First".to_string()),
            Token::Newline,
        ];
        let line = classify_line_tokens(&tokens);
        assert_eq!(line, LineTokenType::ListLine);
    }

    #[test]
    fn test_list_line_paren_marker() {
        let tokens = vec![
            Token::Number("1".to_string()),
            Token::CloseParen,
            Token::Whitespace,
            Token::Text("First".to_string()),
            Token::Newline,
        ];
        let line = classify_line_tokens(&tokens);
        assert_eq!(line, LineTokenType::ListLine);
    }

    #[test]
    fn test_list_line_with_indentation() {
        let tokens = vec![
            Token::Indent,
            Token::Dash,
            Token::Whitespace,
            Token::Text("Item".to_string()),
            Token::Newline,
        ];
        let line = classify_line_tokens(&tokens);
        assert_eq!(line, LineTokenType::ListLine);
    }

    #[test]
    fn test_list_line_letter_marker() {
        let tokens = vec![
            Token::Text("a".to_string()),
            Token::Period,
            Token::Whitespace,
            Token::Text("First".to_string()),
            Token::Newline,
        ];
        let line = classify_line_tokens(&tokens);
        assert_eq!(line, LineTokenType::ListLine);
    }

    #[test]
    fn test_list_line_letter_marker_uppercase() {
        let tokens = vec![
            Token::Text("A".to_string()),
            Token::CloseParen,
            Token::Whitespace,
            Token::Text("Item".to_string()),
            Token::Newline,
        ];
        let line = classify_line_tokens(&tokens);
        assert_eq!(line, LineTokenType::ListLine);
    }

    #[test]
    fn test_list_line_roman_numeral_marker() {
        let tokens = vec![
            Token::Text("I".to_string()),
            Token::Period,
            Token::Whitespace,
            Token::Text("First".to_string()),
            Token::Newline,
        ];
        let line = classify_line_tokens(&tokens);
        assert_eq!(line, LineTokenType::ListLine);
    }

    #[test]
    fn test_list_line_roman_numeral_multi_char() {
        let tokens = vec![
            Token::Text("III".to_string()),
            Token::CloseParen,
            Token::Whitespace,
            Token::Text("Third".to_string()),
            Token::Newline,
        ];
        let line = classify_line_tokens(&tokens);
        assert_eq!(line, LineTokenType::ListLine);
    }

    #[test]
    fn test_paragraph_line() {
        let tokens = vec![
            Token::Text("Just".to_string()),
            Token::Whitespace,
            Token::Text("some".to_string()),
            Token::Whitespace,
            Token::Text("text".to_string()),
            Token::Newline,
        ];
        let line = classify_line_tokens(&tokens);
        assert_eq!(line, LineTokenType::ParagraphLine);
    }

    #[test]
    fn test_transform_preserves_source_tokens() {
        let tokens = vec![
            Token::Text("Title".to_string()),
            Token::Colon,
            Token::Newline,
            Token::IndentLevel,
            Token::Text("Content".to_string()),
            Token::Newline,
        ];

        let line_tokens = experimental_transform_to_line_tokens(tokens.clone());

        assert_eq!(line_tokens.len(), 3);

        // First line: subject line with source tokens preserved
        assert_eq!(line_tokens[0].line_type, LineTokenType::SubjectLine);
        assert_eq!(
            line_tokens[0].source_tokens,
            vec![
                Token::Text("Title".to_string()),
                Token::Colon,
                Token::Newline,
            ]
        );

        // Second: IndentLevel pass-through
        assert_eq!(line_tokens[1].line_type, LineTokenType::IndentLevel);
        assert_eq!(line_tokens[1].source_tokens, vec![Token::IndentLevel]);

        // Third: paragraph line
        assert_eq!(line_tokens[2].line_type, LineTokenType::ParagraphLine);
        assert_eq!(
            line_tokens[2].source_tokens,
            vec![Token::Text("Content".to_string()), Token::Newline,]
        );
    }

    #[test]
    fn test_transform_multiple_lines() {
        let tokens = vec![
            Token::Text("Para".to_string()),
            Token::Newline,
            Token::Whitespace,
            Token::Newline,
            Token::Dash,
            Token::Whitespace,
            Token::Text("Item".to_string()),
            Token::Newline,
        ];

        let line_tokens = experimental_transform_to_line_tokens(tokens);

        // Should produce: paragraph, blank line, list line
        assert_eq!(line_tokens.len(), 3);
        assert_eq!(line_tokens[0].line_type, LineTokenType::ParagraphLine);
        assert_eq!(line_tokens[1].line_type, LineTokenType::BlankLine);
        assert_eq!(line_tokens[2].line_type, LineTokenType::ListLine);
    }

    #[test]
    fn test_list_item_or_subject_line() {
        // A line that is both list marker AND ends with colon
        // e.g., "1. This is great, see:"
        let tokens = vec![
            Token::Number("1".to_string()),
            Token::Period,
            Token::Whitespace,
            Token::Text("This".to_string()),
            Token::Whitespace,
            Token::Text("is".to_string()),
            Token::Whitespace,
            Token::Text("great".to_string()),
            Token::Colon,
            Token::Newline,
        ];

        let line = classify_line_tokens(&tokens);
        // LIST_LINE takes precedence in classification
        assert_eq!(line, LineTokenType::ListLine);
    }

    #[test]
    fn test_colon_in_middle_is_not_subject_line() {
        let tokens = vec![
            Token::Text("Some".to_string()),
            Token::Colon,
            Token::Text("text".to_string()),
            Token::Newline,
        ];
        let line = classify_line_tokens(&tokens);
        // Colon is not at end, so not a subject line
        assert_eq!(line, LineTokenType::ParagraphLine);
    }
}
