//! Line Classification
//!
//!     Core classification logic for determining line types based on token patterns. This module
//!     contains the classifiers used by the lexer to categorize lines.
//!
//!     Since the grammar operates mostly over lines, and each line must be tokenized into one
//!     category during the lexing stage, classification is crucial. In the real world, a line might
//!     be more than one possible category. For example a line might have a sequence marker and a
//!     subject marker (for example "1. Recap:").
//!
//!     For this reason, line tokens can be OR tokens at times (like SubjectOrListItemLine), and at
//!     other times the order of line categorization is crucial to getting the right result. While
//!     there are only a few consequential marks in lines (blank, data, subject, list) having them
//!     denormalized is required to have parsing simpler.
//!
//!     The definitive set is the LineType enum. See the [line](crate::lex::token::line) module for
//!     the complete list of line types.
//!
//! Classification Order
//!
//!     Classification follows this specific order (important for correctness):
//!         1. Blank lines
//!         2. Annotation end lines (only :: marker, no other content)
//!         3. Annotation start lines (follows annotation grammar)
//!         4. Data lines (:: label params? without closing ::)
//!         5. List lines starting with list marker AND ending with colon -> SubjectOrListItemLine
//!         6. List lines (starting with list marker)
//!         7. Subject lines (ending with colon)
//!         8. Default to paragraph
//!
//!     This ordering ensures that more specific patterns (like annotation lines) are matched before
//!     more general ones (like subject lines).

use crate::lex::annotation::analyze_annotation_header_tokens;
use crate::lex::token::{LineType, Token};

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
pub fn classify_line_tokens(tokens: &[Token]) -> LineType {
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

    // DATA_LINE: :: label params? without closing ::
    if is_data_line(tokens) {
        return LineType::DataLine;
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
///
/// Blank lines are semantically significant in Lex (they separate paragraphs and are required
/// before/after session titles), but only their existence matters, not the exact whitespace content.
fn is_blank_line(tokens: &[Token]) -> bool {
    tokens.iter().all(|t| {
        matches!(
            t,
            Token::Whitespace(_) | Token::Indentation | Token::BlankLine(_)
        )
    })
}

/// Check if line is an annotation end line: only :: marker (and optional whitespace/newline)
///
/// This must be checked before annotation start lines to avoid misclassifying end markers
/// as start markers. Annotation end lines have only a single :: marker with no other content.
fn is_annotation_end_line(tokens: &[Token]) -> bool {
    // Find all non-whitespace/non-newline tokens
    let content_tokens: Vec<_> = tokens
        .iter()
        .filter(|t| {
            !matches!(
                t,
                Token::Whitespace(_) | Token::BlankLine(_) | Token::Indentation
            )
        })
        .collect();

    // Must have exactly one token and it must be LexMarker
    content_tokens.len() == 1 && matches!(content_tokens[0], Token::LexMarker)
}

/// Check if line is an annotation start line: follows annotation grammar
/// Grammar: <lex-marker><space><label>(<space><parameters>)? <lex-marker> <content>?
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
            Token::Indentation | Token::Whitespace(_) => continue,
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
        && !matches!(tokens[first_marker_idx + 1], Token::Whitespace(_))
    {
        return false;
    }

    // Must have a second LexMarker somewhere after the first
    let mut second_marker_idx = None;
    for (i, token) in tokens.iter().enumerate().skip(first_marker_idx + 1) {
        if matches!(token, Token::LexMarker) {
            second_marker_idx = Some(i);
            break;
        }
    }

    let Some(second_marker_idx) = second_marker_idx else {
        return false;
    };

    // Require a label between the markers
    let header_tokens = &tokens[first_marker_idx + 1..second_marker_idx];
    analyze_annotation_header_tokens(header_tokens).has_label
}

/// Check if a line is a data line (:: label params? without closing ::)
fn is_data_line(tokens: &[Token]) -> bool {
    if tokens.is_empty() {
        return false;
    }

    // Find first LexMarker after optional indentation/whitespace
    let mut first_marker_idx = None;
    for (i, token) in tokens.iter().enumerate() {
        match token {
            Token::Indentation | Token::Whitespace(_) => continue,
            Token::LexMarker => {
                first_marker_idx = Some(i);
                break;
            }
            _ => return false,
        }
    }

    let Some(first_marker_idx) = first_marker_idx else {
        return false;
    };

    // After first marker we expect whitespace
    if first_marker_idx + 1 >= tokens.len()
        || !matches!(tokens[first_marker_idx + 1], Token::Whitespace(_))
    {
        return false;
    }

    // Data lines must not contain a second LexMarker before newline
    if tokens[first_marker_idx + 1..]
        .iter()
        .any(|t| matches!(t, Token::LexMarker))
    {
        return false;
    }

    // Collect header tokens (until newline) and ensure we have a label
    let mut header_tokens = Vec::new();
    for token in tokens[first_marker_idx + 1..].iter() {
        if matches!(token, Token::BlankLine(_)) {
            continue;
        }
        header_tokens.push(token.clone());
    }

    if header_tokens.is_empty() {
        return false;
    }

    analyze_annotation_header_tokens(&header_tokens).has_label
}

/// Check if line starts with a list marker (after optional indentation)
///
/// List markers can be:
/// - Plain: "-" followed by whitespace
/// - Numbered: "1." or "1)" followed by whitespace
/// - Alphabetic: "a." or "a)" followed by whitespace
/// - Roman numerals: "I.", "II.", etc. followed by whitespace
/// - Parenthetical: "(1)", "(a)", "(I)" followed by whitespace
///
/// The marker must be at the start of the line (after optional indentation) and must be
/// followed by whitespace to distinguish from other uses (e.g., arithmetic expressions like "7 * 8").
pub fn has_list_marker(tokens: &[Token]) -> bool {
    let mut i = 0;

    // Skip leading indentation and whitespace
    while i < tokens.len() && matches!(tokens[i], Token::Indentation | Token::Whitespace(_)) {
        i += 1;
    }

    // Check for plain list marker: Dash Whitespace
    if i + 1 < tokens.len()
        && matches!(tokens[i], Token::Dash)
        && matches!(tokens[i + 1], Token::Whitespace(_))
    {
        return true;
    }

    // Check for parenthetical list marker: OpenParen (Number | Letter | RomanNumeral) CloseParen Whitespace
    if i + 3 < tokens.len()
        && matches!(tokens[i], Token::OpenParen)
        && matches!(tokens[i + 3], Token::Whitespace(_))
        && matches!(tokens[i + 2], Token::CloseParen)
    {
        let has_number = matches!(tokens[i + 1], Token::Number(_));
        let has_letter = matches!(tokens[i + 1], Token::Text(ref s) if is_single_letter(s));
        let has_roman = matches!(tokens[i + 1], Token::Text(ref s) if is_roman_numeral(s));

        if has_number || has_letter || has_roman {
            return true;
        }
    }

    // Check for ordered list marker: (Number | Letter | RomanNumeral) (Period | CloseParen) Whitespace
    if i + 2 < tokens.len() {
        let has_number = matches!(tokens[i], Token::Number(_));
        let has_letter = matches!(tokens[i], Token::Text(ref s) if is_single_letter(s));
        let has_roman = matches!(tokens[i], Token::Text(ref s) if is_roman_numeral(s));
        let has_separator = matches!(tokens[i + 1], Token::Period | Token::CloseParen);
        let has_space = matches!(tokens[i + 2], Token::Whitespace(_));

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
///
/// Subject lines (for definitions, verbatim blocks, and sessions) end with a colon.
/// Trailing whitespace and newlines are ignored when checking for the colon.
pub fn ends_with_colon(tokens: &[Token]) -> bool {
    // Find last non-whitespace token before newline
    let mut i = tokens.len() as i32 - 1;

    while i >= 0 {
        let token = &tokens[i as usize];
        match token {
            Token::BlankLine(_) | Token::Whitespace(_) => {
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
    fn test_classify_paragraph_line() {
        let tokens = vec![
            Token::Text("Hello".to_string()),
            Token::Whitespace(1),
            Token::Text("world".to_string()),
            Token::BlankLine(Some("\n".to_string())),
        ];
        assert_eq!(classify_line_tokens(&tokens), LineType::ParagraphLine);
    }

    #[test]
    fn test_classify_subject_line() {
        let tokens = vec![
            Token::Text("Title".to_string()),
            Token::Colon,
            Token::BlankLine(Some("\n".to_string())),
        ];
        assert_eq!(classify_line_tokens(&tokens), LineType::SubjectLine);
    }

    #[test]
    fn test_classify_list_line() {
        let tokens = vec![
            Token::Dash,
            Token::Whitespace(1),
            Token::Text("Item".to_string()),
            Token::BlankLine(Some("\n".to_string())),
        ];
        assert_eq!(classify_line_tokens(&tokens), LineType::ListLine);
    }

    #[test]
    fn test_classify_blank_line() {
        let tokens = vec![
            Token::Whitespace(1),
            Token::BlankLine(Some("\n".to_string())),
        ];
        assert_eq!(classify_line_tokens(&tokens), LineType::BlankLine);
    }

    #[test]
    fn test_classify_annotation_start_line() {
        let tokens = vec![
            Token::LexMarker,
            Token::Whitespace(1),
            Token::Text("label".to_string()),
            Token::Whitespace(1),
            Token::LexMarker,
            Token::BlankLine(Some("\n".to_string())),
        ];
        assert_eq!(classify_line_tokens(&tokens), LineType::AnnotationStartLine);
    }

    #[test]
    fn test_classify_data_line() {
        let tokens = vec![
            Token::LexMarker,
            Token::Whitespace(1),
            Token::Text("label".to_string()),
            Token::BlankLine(Some("\n".to_string())),
        ];
        assert_eq!(classify_line_tokens(&tokens), LineType::DataLine);
    }

    #[test]
    fn test_annotation_line_without_label_falls_back_to_paragraph() {
        let tokens = vec![
            Token::LexMarker,
            Token::Whitespace(1),
            Token::Text("version".to_string()),
            Token::Equals,
            Token::Number("3.11".to_string()),
            Token::Whitespace(1),
            Token::LexMarker,
            Token::BlankLine(Some("\n".to_string())),
        ];

        assert_eq!(classify_line_tokens(&tokens), LineType::ParagraphLine);
    }

    #[test]
    fn test_classify_annotation_end_line() {
        let tokens = vec![Token::LexMarker, Token::BlankLine(Some("\n".to_string()))];
        assert_eq!(classify_line_tokens(&tokens), LineType::AnnotationEndLine);
    }

    #[test]
    fn test_classify_subject_or_list_item_line() {
        let tokens = vec![
            Token::Dash,
            Token::Whitespace(1),
            Token::Text("Item".to_string()),
            Token::Colon,
            Token::BlankLine(Some("\n".to_string())),
        ];
        assert_eq!(
            classify_line_tokens(&tokens),
            LineType::SubjectOrListItemLine
        );
    }

    #[test]
    fn test_ordered_list_markers() {
        // Number-based
        let tokens = vec![
            Token::Number("1".to_string()),
            Token::Period,
            Token::Whitespace(1),
            Token::Text("Item".to_string()),
        ];
        assert!(has_list_marker(&tokens));

        // Letter-based
        let tokens = vec![
            Token::Text("a".to_string()),
            Token::Period,
            Token::Whitespace(1),
            Token::Text("Item".to_string()),
        ];
        assert!(has_list_marker(&tokens));

        // Roman numeral
        let tokens = vec![
            Token::Text("I".to_string()),
            Token::Period,
            Token::Whitespace(1),
            Token::Text("Item".to_string()),
        ];
        assert!(has_list_marker(&tokens));

        // With close paren
        let tokens = vec![
            Token::Number("1".to_string()),
            Token::CloseParen,
            Token::Whitespace(1),
            Token::Text("Item".to_string()),
        ];
        assert!(has_list_marker(&tokens));
    }
}
