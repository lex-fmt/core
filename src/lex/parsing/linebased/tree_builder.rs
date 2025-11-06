//! Tree Builder - Converts flat token stream to LineContainer tree
//!
//! This module contains the logic previously in the pipeline (ToLineTokensMapper and
//! IndentationToTreeMapper) but now localized to the line-based parser since it's the
//! only consumer that needs this hierarchical structure.
//!
//! # Responsibilities
//!
//! 1. Group flat tokens into lines (delimited by Newline)
//! 2. Classify each line by type (SubjectLine, ListLine, etc.)
//! 3. Build hierarchical tree based on Indent/Dedent markers
//! 4. Convert to LineContainer structure expected by parser

use crate::lex::lexing::linebased::tokens_linebased::{LineContainer, LineToken, LineType};
use crate::lex::lexing::tokens_core::Token;
use std::ops::Range as ByteRange;

/// Build a LineContainer tree from a flat stream of tokens.
///
/// This is the main entry point that orchestrates the conversion:
/// 1. Groups tokens into classified lines
/// 2. Builds hierarchical structure based on indentation
/// 3. Wraps in root Container as expected by parser
///
/// # Arguments
///
/// * `tokens` - Flat vector of (Token, Range) pairs from pipeline
///
/// # Returns
///
/// A LineContainer tree ready for the line-based parser
pub fn build_line_container(tokens: Vec<(Token, ByteRange<usize>)>) -> LineContainer {
    // Step 1: Group into classified lines
    let line_tokens = group_into_lines(tokens);

    // Step 2: Build hierarchical structure
    let children = build_hierarchy(line_tokens);

    // Step 3: Wrap in root container
    LineContainer::Container { children }
}

/// Build a flat vector of LineTokens from a flat stream of tokens.
///
/// This is useful for debugging/serialization purposes when you need
/// to see the classified lines without the hierarchical structure.
///
/// # Arguments
///
/// * `tokens` - Flat vector of (Token, Range) pairs from pipeline
///
/// # Returns
///
/// A vector of LineTokens (flat, no hierarchy)
pub fn build_line_tokens(tokens: Vec<(Token, ByteRange<usize>)>) -> Vec<LineToken> {
    group_into_lines(tokens)
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

/// Build hierarchical structure from flat list of LineTokens based on Indent/Dedent.
///
/// This implements the logic from IndentationToTreeMapper:
/// - Maintains a stack of nesting levels
/// - Indent starts a new nested level
/// - Dedent closes current level and attaches to parent
/// - Regular lines accumulate at current level
fn build_hierarchy(line_tokens: Vec<LineToken>) -> Vec<LineContainer> {
    // Stack of pending children at each indentation level
    let mut stack: Vec<Vec<LineContainer>> = vec![Vec::new()];
    let mut pending_containers: Vec<LineContainer> = Vec::new();

    for line_token in line_tokens {
        match line_token.line_type {
            LineType::Indent => {
                // Flush pending containers before entering nested level
                if !pending_containers.is_empty() {
                    let current_level = stack.last_mut().expect("Stack never empty");
                    current_level.append(&mut pending_containers);
                }
                // Start a new nesting level
                stack.push(Vec::new());
            }
            LineType::Dedent => {
                // Flush pending containers before closing level
                if !pending_containers.is_empty() {
                    let current_level = stack.last_mut().expect("Stack never empty");
                    current_level.append(&mut pending_containers);
                }
                // Close current level and attach as children to last container in parent
                if let Some(children_containers) = stack.pop() {
                    let parent_level = stack.last_mut().expect("Stack never empty");

                    if let Some(last_parent) = parent_level.last_mut() {
                        // Attach the nested children to the last parent container
                        // If the parent is already a Container, extend its children
                        // Otherwise, we need to convert Token to a structure that can hold children
                        match last_parent {
                            LineContainer::Container { ref mut children } => {
                                children.extend(children_containers);
                            }
                            LineContainer::Token(_) => {
                                // Need to replace Token with [Token, Container]
                                // Extract the token first
                                let token = std::mem::replace(
                                    last_parent,
                                    LineContainer::Container {
                                        children: Vec::new(),
                                    },
                                );
                                // Now last_parent is a placeholder Container
                                // Replace it with the proper sequence
                                *last_parent = token; // Put token back temporarily

                                // We need to insert a container after this token
                                // So we'll actually just append it to parent_level
                                parent_level.push(LineContainer::Container {
                                    children: children_containers,
                                });
                            }
                        }
                    } else {
                        // If no parent exists, create a container node to hold children
                        parent_level.push(LineContainer::Container {
                            children: children_containers,
                        });
                    }
                }
            }
            _ => {
                // Accumulate regular line tokens at current level
                pending_containers.push(LineContainer::Token(line_token));
            }
        }
    }

    // Flush any remaining pending containers at root level
    if !pending_containers.is_empty() {
        let root_level = stack.last_mut().expect("Stack never empty");
        root_level.append(&mut pending_containers);
    }

    // Return the root level containers
    stack.pop().expect("Stack should contain root level")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_simple_paragraph() {
        let tokens = vec![
            (Token::Text("Hello".to_string()), 0..5),
            (Token::Whitespace, 5..6),
            (Token::Text("world".to_string()), 6..11),
            (Token::Newline, 11..12),
        ];

        let container = build_line_container(tokens);

        match container {
            LineContainer::Container { children } => {
                assert_eq!(children.len(), 1);
                match &children[0] {
                    LineContainer::Token(line_token) => {
                        assert_eq!(line_token.line_type, LineType::ParagraphLine);
                        assert_eq!(line_token.source_tokens.len(), 4);
                    }
                    _ => panic!("Expected Token"),
                }
            }
            _ => panic!("Expected Container at root"),
        }
    }

    #[test]
    fn test_classify_subject_line() {
        let tokens = vec![
            Token::Text("Title".to_string()),
            Token::Colon,
            Token::Newline,
        ];
        assert_eq!(classify_line_tokens(&tokens), LineType::SubjectLine);
    }

    #[test]
    fn test_classify_list_line() {
        let tokens = vec![
            Token::Dash,
            Token::Whitespace,
            Token::Text("Item".to_string()),
            Token::Newline,
        ];
        assert_eq!(classify_line_tokens(&tokens), LineType::ListLine);
    }

    #[test]
    fn test_build_with_indentation() {
        let tokens = vec![
            (Token::Text("Title".to_string()), 0..5),
            (Token::Colon, 5..6),
            (Token::Newline, 6..7),
            (Token::Indent(vec![(Token::Indentation, 7..11)]), 0..0),
            (Token::Text("Content".to_string()), 11..18),
            (Token::Newline, 18..19),
            (Token::Dedent(vec![]), 0..0),
        ];

        let container = build_line_container(tokens);

        match container {
            LineContainer::Container { children } => {
                // Should have title token and then a container with the indented content
                assert!(children.len() >= 1);

                // First child should be the title
                match &children[0] {
                    LineContainer::Token(line_token) => {
                        assert_eq!(line_token.line_type, LineType::SubjectLine);
                    }
                    _ => panic!("Expected Token for title"),
                }
            }
            _ => panic!("Expected Container at root"),
        }
    }
}
