//! Line Token Grouping Transformation
//!
//! Groups flat tokens into line-based groups with classification.
//! This transformation:
//! - Groups consecutive tokens into lines (delimited by Newline)
//! - Classifies each line by type (SubjectLine, ListLine, etc.)
//! - Handles structural tokens (Indent, Dedent, BlankLine) specially
//! - Applies dialog line detection
//!
//! Converts: TokenStream::Flat → TokenStream::Grouped

use crate::lex::lexing::tokens_core::Token;
use crate::lex::lexing::tokens_linebased::{LineToken, LineType};
use crate::lex::pipeline::mapper::{StreamMapper, TransformationError};
use crate::lex::pipeline::stream::{GroupType, GroupedTokens, TokenStream};
use std::ops::Range as ByteRange;

/// Transformation that groups flat tokens into line-based groups.
pub struct LineTokenGroupingMapper;

impl LineTokenGroupingMapper {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LineTokenGroupingMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamMapper for LineTokenGroupingMapper {
    fn map_flat(
        &mut self,
        tokens: Vec<(Token, ByteRange<usize>)>,
    ) -> Result<TokenStream, TransformationError> {
        // Group tokens into LineTokens
        let line_tokens = group_into_lines(tokens);

        // Convert LineTokens to GroupedTokens
        let grouped_tokens: Vec<GroupedTokens> = line_tokens
            .into_iter()
            .map(|line_token| GroupedTokens {
                source_tokens: line_token.source_token_pairs(),
                group_type: GroupType::Line(line_token.line_type),
            })
            .collect();

        Ok(TokenStream::Grouped(grouped_tokens))
    }
}

/// Group flat tokens into classified LineTokens.
///
/// This implements the logic from ToLineTokensMapper:
/// - Groups consecutive tokens into lines (delimited by Newline)
/// - Classifies each line by type
/// - Handles structural tokens (Indent, Dedent, BlankLine) specially
fn group_into_lines(tokens: Vec<(Token, ByteRange<usize>)>) -> Vec<LineToken> {
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
