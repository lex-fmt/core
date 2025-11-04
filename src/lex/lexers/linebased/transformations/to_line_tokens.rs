//! Linebased transformation: raw tokens → line tokens
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
//! - ANNOTATION_END_LINE: Line containing only :: marker
//! - ANNOTATION_START_LINE: Line following annotation grammar: :: <label>? <params>? :: <content>?
//! - SUBJECT_LINE: Line ending with colon
//! - LIST_LINE: Line starting with list marker (-, 1., a., I., etc.)
//! - SUBJECT_OR_LIST_ITEM_LINE: Line starting with list marker and ending with colon
//! - PARAGRAPH_LINE: Any other line
//! - INDENT_LEVEL / DEDENT_LEVEL: Structural tokens (pass through unchanged)

use crate::lex::lexers::linebased::tokens::{LineToken, LineType};
use crate::lex::lexers::tokens::Token;

/// Transform flat token stream into line tokens.
///
/// Groups consecutive tokens into semantic line units. Each line token preserves
/// the original raw tokens with their spans and classifies the line type.
///
/// Input: Flat token stream from lexer transformations (whitespace, indentation, blank-line processed)
///        as (Token, Range<usize>) tuples
/// Output: Vector of LineTokens where each token represents one logical line with both
///         source_tokens and token_spans properly populated
///
/// Example:
/// ```text
/// Input tokens:
///   [(Text("Title"), 0..5), (Colon, 5..6), (Newline, 6..7), (Indent, 7..11), (Text("Content"), 11..18), (Newline, 18..19)]
///
/// Output line tokens:
///   [
///     LineToken {
///       source_tokens: [Text("Title"), Colon, Newline],
///       token_spans: [0..5, 5..6, 6..7],
///       line_type: SubjectLine
///     },
///     LineToken {
///       source_tokens: [Indent],
///       token_spans: [7..11],
///       line_type: Indent
///     },
///     LineToken {
///       source_tokens: [Text("Content"), Newline],
///       token_spans: [11..18, 18..19],
///       line_type: ParagraphLine
///     },
///   ]
/// ```
pub fn _to_line_tokens(tokens: Vec<(Token, std::ops::Range<usize>)>) -> Vec<LineToken> {
    let mut line_tokens = Vec::new();
    let mut current_line = Vec::new();

    for (token, span) in tokens {
        let is_newline = matches!(token, Token::Newline);
        let is_blank_line_token = matches!(token, Token::BlankLine(_));

        // Structural tokens (Indent, Dedent, BlankLine) are pass-through
        // They appear alone, not as part of lines
        if matches!(token, Token::Indent(_)) {
            if !current_line.is_empty() {
                line_tokens.push(classify_and_create_line_token(current_line));
                current_line = Vec::new();
            }
            // Indent tokens ALWAYS contain source tokens in production:
            // vec![(Token::Indent, range)] from sem_indentation transformation
            let (source_tokens, token_spans) = if let Token::Indent(ref sources) = token {
                // Extract the stored source tokens
                let (toks, spans): (Vec<_>, Vec<_>) = sources.iter().cloned().unzip();
                (toks, spans)
            } else {
                unreachable!("Token matches Indent but pattern failed")
            };
            line_tokens.push(LineToken {
                source_tokens,
                token_spans,
                line_type: LineType::Indent,
            });
            continue;
        }

        if matches!(token, Token::Dedent(_)) {
            if !current_line.is_empty() {
                line_tokens.push(classify_and_create_line_token(current_line));
                current_line = Vec::new();
            }
            // Dedent tokens are purely structural - ALWAYS have empty source_tokens vec![]
            // in production (sem_indentation.rs:133). Store the Dedent token itself.
            line_tokens.push(LineToken {
                source_tokens: vec![token],
                token_spans: vec![span],
                line_type: LineType::Dedent,
            });
            continue;
        }

        // BlankLine tokens are also structural - they represent a blank line by themselves
        if is_blank_line_token {
            if !current_line.is_empty() {
                line_tokens.push(classify_and_create_line_token(current_line));
                current_line = Vec::new();
            }
            // BlankLine tokens ALWAYS contain source tokens in production:
            // vec![(Token::Newline, range), ...] from transform_blank_lines
            let (source_tokens, token_spans) = if let Token::BlankLine(ref sources) = token {
                // Extract the stored source tokens (Newline tokens from 2nd onwards)
                let (toks, spans): (Vec<_>, Vec<_>) = sources.iter().cloned().unzip();
                (toks, spans)
            } else {
                unreachable!("Token matches BlankLine but pattern failed")
            };
            line_tokens.push(LineToken {
                source_tokens,
                token_spans,
                line_type: LineType::BlankLine,
            });
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

    line_tokens
}

/// Classify tokens and create a line token with the appropriate type.
///
/// Takes a vector of (Token, Range) tuples and unzips them into separate
/// source_tokens and token_spans vectors for the LineToken.
fn classify_and_create_line_token(token_tuples: Vec<(Token, std::ops::Range<usize>)>) -> LineToken {
    // Unzip the tuples into separate vectors
    let (source_tokens, token_spans): (Vec<Token>, Vec<std::ops::Range<usize>>) =
        token_tuples.into_iter().unzip();

    let line_type = classify_line_tokens(&source_tokens);
    LineToken {
        source_tokens,
        token_spans,
        line_type,
    }
}

/// Determine the type of a line based on its tokens.
///
/// Classification follows this specific order (important for correctness):
/// 1. Blank lines
/// 2. Annotation end lines (only :: marker, no other content)
/// 3. Annotation start lines (follows annotation grammar)
/// 4. List lines starting with list marker AND ending with colon -> SubjectOrListItemLine
/// 5. List lines (starting with list marker)
/// 6. Subject lines (ending with colon)
/// 7. Default to paragraph
fn classify_line_tokens(tokens: &[Token]) -> LineType {
    if tokens.is_empty() {
        return LineType::ParagraphLine;
    }

    // BLANK_LINE: Only whitespace and newline tokens
    if is_blank_line(tokens) {
        return LineType::BlankLine;
    }

    // ANNOTATION_END_LINE: Only :: marker (and optional whitespace/newline)
    if is_annotation_end_line(tokens) {
        return LineType::AnnotationEndLine;
    }

    // ANNOTATION_START_LINE: Follows annotation grammar with :: markers
    if is_annotation_start_line(tokens) {
        return LineType::AnnotationStartLine;
    }

    // Check if line both starts with list marker AND ends with colon
    let has_list_marker = has_list_marker(tokens);
    let has_colon = ends_with_colon(tokens);

    if has_list_marker && has_colon {
        return LineType::SubjectOrListItemLine;
    }

    // LIST_LINE: Starts with list marker
    if has_list_marker {
        return LineType::ListLine;
    }

    // SUBJECT_LINE: Ends with colon
    if has_colon {
        return LineType::SubjectLine;
    }

    // Default: PARAGRAPH_LINE
    LineType::ParagraphLine
}

/// Check if line is blank (only whitespace and newline)
fn is_blank_line(tokens: &[Token]) -> bool {
    tokens.iter().all(|t| {
        matches!(
            t,
            Token::Whitespace | Token::Indentation | Token::Newline | Token::BlankLine(_)
        )
    })
}

/// Check if line is an annotation end line: only :: marker (and optional whitespace/newline)
fn is_annotation_end_line(tokens: &[Token]) -> bool {
    // Find all non-whitespace/non-newline tokens
    let content_tokens: Vec<_> = tokens
        .iter()
        .filter(|t| !matches!(t, Token::Whitespace | Token::Newline | Token::Indentation))
        .collect();

    // Must have exactly one token and it must be LexMarker
    content_tokens.len() == 1 && matches!(content_tokens[0], Token::LexMarker)
}

/// Check if line is an annotation start line: follows annotation grammar
/// Grammar: <lex-marker><space>(<label><space>)?<parameters>? <lex-marker> <content>?
fn is_annotation_start_line(tokens: &[Token]) -> bool {
    if tokens.is_empty() {
        return false;
    }

    // Must contain at least one LexMarker
    let marker_count = tokens
        .iter()
        .filter(|t| matches!(t, Token::LexMarker))
        .count();
    if marker_count < 1 {
        return false;
    }

    // Find first LexMarker position (after optional leading whitespace)
    let mut first_marker_idx = None;
    for (i, token) in tokens.iter().enumerate() {
        match token {
            Token::Indentation | Token::Whitespace => continue,
            Token::LexMarker => {
                first_marker_idx = Some(i);
                break;
            }
            _ => break, // Non-whitespace, non-marker: not an annotation line
        }
    }

    let first_marker_idx = match first_marker_idx {
        Some(idx) => idx,
        None => return false,
    };

    // After first marker, must have whitespace (or be end of line)
    if first_marker_idx + 1 < tokens.len()
        && !matches!(tokens[first_marker_idx + 1], Token::Whitespace)
    {
        return false;
    }

    // Must have a second LexMarker somewhere after the first
    let has_second_marker = tokens[first_marker_idx + 1..]
        .iter()
        .any(|t| matches!(t, Token::LexMarker));

    has_second_marker
}

/// Check if line starts with a list marker (after optional indentation)
fn has_list_marker(tokens: &[Token]) -> bool {
    let mut i = 0;

    // Skip leading indentation and whitespace
    while i < tokens.len() && matches!(tokens[i], Token::Indentation | Token::Whitespace) {
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
    s.len() == 1 && s.chars().next().is_some_and(|c| c.is_alphabetic())
}

/// Check if a string is a Roman numeral (I, II, III, IV, V, etc.)
fn is_roman_numeral(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    // Check if all characters are valid Roman numeral characters
    s.chars()
        .all(|c| matches!(c, 'I' | 'V' | 'X' | 'L' | 'C' | 'D' | 'M'))
        && s.chars().next().is_some_and(|c| c.is_uppercase())
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
        assert_eq!(line, LineType::BlankLine);
    }

    #[test]
    fn test_annotation_start_line_classification() {
        let tokens = vec![
            Token::LexMarker,
            Token::Whitespace,
            Token::Text("note".to_string()),
            Token::Whitespace,
            Token::LexMarker,
            Token::Newline,
        ];
        let line = classify_line_tokens(&tokens);
        assert_eq!(line, LineType::AnnotationStartLine);
    }

    #[test]
    fn test_annotation_end_line_classification() {
        let tokens = vec![Token::LexMarker, Token::Newline];
        let line = classify_line_tokens(&tokens);
        assert_eq!(line, LineType::AnnotationEndLine);
    }

    #[test]
    fn test_annotation_end_line_with_whitespace() {
        let tokens = vec![
            Token::Whitespace,
            Token::LexMarker,
            Token::Whitespace,
            Token::Newline,
        ];
        let line = classify_line_tokens(&tokens);
        assert_eq!(line, LineType::AnnotationEndLine);
    }

    #[test]
    fn test_subject_line_classification() {
        let tokens = vec![
            Token::Text("Title".to_string()),
            Token::Colon,
            Token::Newline,
        ];
        let line = classify_line_tokens(&tokens);
        assert_eq!(line, LineType::SubjectLine);
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
        assert_eq!(line, LineType::SubjectLine);
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
        assert_eq!(line, LineType::ListLine);
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
        assert_eq!(line, LineType::ListLine);
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
        assert_eq!(line, LineType::ListLine);
    }

    #[test]
    fn test_list_line_with_indentation() {
        let tokens = vec![
            Token::Indentation,
            Token::Dash,
            Token::Whitespace,
            Token::Text("Item".to_string()),
            Token::Newline,
        ];
        let line = classify_line_tokens(&tokens);
        assert_eq!(line, LineType::ListLine);
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
        assert_eq!(line, LineType::ListLine);
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
        assert_eq!(line, LineType::ListLine);
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
        assert_eq!(line, LineType::ListLine);
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
        assert_eq!(line, LineType::ListLine);
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
        assert_eq!(line, LineType::ParagraphLine);
    }

    // Test helper: Convert Vec<Token> to Vec<(Token, Range)> with dummy spans
    fn with_dummy_spans(tokens: Vec<Token>) -> Vec<(Token, std::ops::Range<usize>)> {
        tokens
            .into_iter()
            .enumerate()
            .map(|(i, t)| (t, i..i + 1))
            .collect()
    }

    #[test]
    fn test_transform_preserves_source_tokens() {
        // Create realistic tokens with actual source tokens like production code does
        let tokens = vec![
            (Token::Text("Title".to_string()), 0..5),
            (Token::Colon, 5..6),
            (Token::Newline, 6..7),
            // Indent with real source token (like sem_indentation creates)
            (Token::Indent(vec![(Token::Indentation, 7..11)]), 0..0),
            (Token::Text("Content".to_string()), 11..18),
            (Token::Newline, 18..19),
        ];

        let line_tokens = _to_line_tokens(tokens);

        assert_eq!(line_tokens.len(), 3);

        // First line: subject line with source tokens preserved
        assert_eq!(line_tokens[0].line_type, LineType::SubjectLine);
        assert_eq!(
            line_tokens[0].source_tokens,
            vec![
                Token::Text("Title".to_string()),
                Token::Colon,
                Token::Newline,
            ]
        );

        // Second: Indent extracts its source token (Token::Indent)
        assert_eq!(line_tokens[1].line_type, LineType::Indent);
        assert_eq!(line_tokens[1].source_tokens, vec![Token::Indentation]);
        assert_eq!(line_tokens[1].token_spans, vec![7..11]);

        // Third: paragraph line
        assert_eq!(line_tokens[2].line_type, LineType::ParagraphLine);
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

        let line_tokens = _to_line_tokens(with_dummy_spans(tokens));

        // Should produce: paragraph, blank line, list line
        assert_eq!(line_tokens.len(), 3);
        assert_eq!(line_tokens[0].line_type, LineType::ParagraphLine);
        assert_eq!(line_tokens[1].line_type, LineType::BlankLine);
        assert_eq!(line_tokens[2].line_type, LineType::ListLine);
    }

    #[test]
    fn test_subject_or_list_item_line() {
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
        // Should be classified as SubjectOrListItemLine
        assert_eq!(line, LineType::SubjectOrListItemLine);
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
        assert_eq!(line, LineType::ParagraphLine);
    }

    #[test]
    fn test_annotation_start_and_subject_line_precedence() {
        // A line that looks like both annotation (has ::) and subject (ends with :)
        // Annotation check comes BEFORE subject check, so AnnotationStartLine should win
        let tokens = vec![
            Token::LexMarker,
            Token::Whitespace,
            Token::Text("note".to_string()),
            Token::Whitespace,
            Token::LexMarker,
            Token::Whitespace,
            Token::Text("description".to_string()),
            Token::Colon,
            Token::Newline,
        ];

        let line = classify_line_tokens(&tokens);
        // ANNOTATION_START_LINE takes precedence (checked before SUBJECT_LINE)
        assert_eq!(line, LineType::AnnotationStartLine);
    }
}
