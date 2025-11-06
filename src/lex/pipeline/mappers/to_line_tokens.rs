//! Line token grouping and classification mapper for TokenStream pipeline
//!
//! This mapper converts a flat stream of tokens into a shallow tree of line-classified nodes.
//! Each node represents one logical line with its LineType classification.
//!
//! # Logic
//!
//! 1. Groups consecutive tokens into lines (delimited by Newline tokens)
//! 2. Classifies each line according to its content (SubjectLine, ListLine, etc.)
//! 3. Handles structural tokens (Indent, Dedent, BlankLine) as standalone nodes
//! 4. Creates TokenStreamNodes with line_type set and children: None (shallow tree)
//!
//! # Input/Output
//!
//! - **Input**: `TokenStream::Flat` - flat token stream after whitespace/indentation/blanklines processing
//! - **Output**: `TokenStream::Tree` - shallow tree with one node per line, no nesting
//!
//! This is a pure adaptation of the existing to_line_tokens transformation
//! to the TokenStream architecture.

use crate::lex::lexing::linebased::tokens_linebased::LineType;
use crate::lex::lexing::tokens_core::Token;
use crate::lex::pipeline::mapper::{StreamMapper, TransformationError};
use crate::lex::pipeline::stream::{TokenStream, TokenStreamNode};
use std::ops::Range as ByteRange;

/// A mapper that groups tokens into classified lines.
///
/// This transformation only operates on flat token streams and produces a shallow
/// tree structure where each node represents one line with its classification.
pub struct ToLineTokensMapper;

impl ToLineTokensMapper {
    /// Create a new ToLineTokensMapper.
    pub fn new() -> Self {
        ToLineTokensMapper
    }
}

impl Default for ToLineTokensMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamMapper for ToLineTokensMapper {
    fn map_flat(
        &mut self,
        tokens: Vec<(Token, ByteRange<usize>)>,
    ) -> Result<TokenStream, TransformationError> {
        let mut nodes = Vec::new();
        let mut current_line = Vec::new();

        for (token, span) in tokens {
            let is_newline = matches!(token, Token::Newline);
            let is_blank_line_token = matches!(token, Token::BlankLine(_));

            // Structural tokens (Indent, Dedent, BlankLine) are pass-through
            // They appear alone, not as part of lines
            if let Token::Indent(ref sources) = token {
                // Flush any accumulated line first
                if !current_line.is_empty() {
                    nodes.push(classify_and_create_node(current_line));
                    current_line = Vec::new();
                }
                // Extract the stored source tokens from Indent
                let (source_tokens, token_spans): (Vec<_>, Vec<_>) =
                    sources.iter().cloned().unzip();
                nodes.push(TokenStreamNode {
                    tokens: source_tokens.into_iter().zip(token_spans).collect(),
                    children: None,
                    line_type: Some(LineType::Indent),
                });
                continue;
            }

            if let Token::Dedent(_) = token {
                // Flush any accumulated line first
                if !current_line.is_empty() {
                    nodes.push(classify_and_create_node(current_line));
                    current_line = Vec::new();
                }
                // Dedent tokens are purely structural - store the Dedent token itself
                nodes.push(TokenStreamNode {
                    tokens: vec![(token, span)],
                    children: None,
                    line_type: Some(LineType::Dedent),
                });
                continue;
            }

            // BlankLine tokens are also structural - they represent a blank line by themselves
            if is_blank_line_token {
                // Flush any accumulated line first
                if !current_line.is_empty() {
                    nodes.push(classify_and_create_node(current_line));
                    current_line = Vec::new();
                }
                // Extract the stored source tokens from BlankLine
                if let Token::BlankLine(ref sources) = token {
                    let (source_tokens, token_spans): (Vec<_>, Vec<_>) =
                        sources.iter().cloned().unzip();
                    nodes.push(TokenStreamNode {
                        tokens: source_tokens.into_iter().zip(token_spans).collect(),
                        children: None,
                        line_type: Some(LineType::BlankLine),
                    });
                }
                continue;
            }

            // Accumulate token-span tuples for current line
            current_line.push((token, span));

            // Newline marks end of line
            if is_newline {
                nodes.push(classify_and_create_node(current_line));
                current_line = Vec::new();
            }
        }

        // Handle any remaining tokens (if input doesn't end with newline)
        if !current_line.is_empty() {
            nodes.push(classify_and_create_node(current_line));
        }

        Ok(TokenStream::Tree(nodes))
    }
}

/// Classify tokens and create a TokenStreamNode with the appropriate LineType.
fn classify_and_create_node(token_tuples: Vec<(Token, ByteRange<usize>)>) -> TokenStreamNode {
    // Extract just the tokens for classification
    let tokens: Vec<Token> = token_tuples.iter().map(|(t, _)| t.clone()).collect();
    let line_type = classify_line_tokens(&tokens);

    TokenStreamNode {
        tokens: token_tuples,
        children: None,
        line_type: Some(line_type),
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

    // Test helper: Convert Vec<Token> to Vec<(Token, Range)> with dummy spans
    fn with_dummy_spans(tokens: Vec<Token>) -> Vec<(Token, ByteRange<usize>)> {
        tokens
            .into_iter()
            .enumerate()
            .map(|(i, t)| (t, i..i + 1))
            .collect()
    }

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

        let mut mapper = ToLineTokensMapper::new();
        let result = mapper.map_flat(tokens).unwrap();

        match result {
            TokenStream::Tree(nodes) => {
                assert_eq!(nodes.len(), 3);

                // First line: subject line with source tokens preserved
                assert_eq!(nodes[0].line_type, Some(LineType::SubjectLine));
                assert_eq!(nodes[0].tokens.len(), 3);
                assert!(matches!(nodes[0].tokens[0].0, Token::Text(_)));
                assert!(matches!(nodes[0].tokens[1].0, Token::Colon));
                assert!(matches!(nodes[0].tokens[2].0, Token::Newline));

                // Second: Indent extracts its source token (Token::Indentation)
                assert_eq!(nodes[1].line_type, Some(LineType::Indent));
                assert_eq!(nodes[1].tokens.len(), 1);
                assert!(matches!(nodes[1].tokens[0].0, Token::Indentation));
                assert_eq!(nodes[1].tokens[0].1, 7..11);

                // Third: paragraph line
                assert_eq!(nodes[2].line_type, Some(LineType::ParagraphLine));
                assert_eq!(nodes[2].tokens.len(), 2);
                assert!(matches!(nodes[2].tokens[0].0, Token::Text(_)));
                assert!(matches!(nodes[2].tokens[1].0, Token::Newline));
            }
            _ => panic!("Expected Tree stream"),
        }
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

        let mut mapper = ToLineTokensMapper::new();
        let result = mapper.map_flat(with_dummy_spans(tokens)).unwrap();

        match result {
            TokenStream::Tree(nodes) => {
                // Should produce: paragraph, blank line, list line
                assert_eq!(nodes.len(), 3);
                assert_eq!(nodes[0].line_type, Some(LineType::ParagraphLine));
                assert_eq!(nodes[1].line_type, Some(LineType::BlankLine));
                assert_eq!(nodes[2].line_type, Some(LineType::ListLine));
            }
            _ => panic!("Expected Tree stream"),
        }
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

    #[test]
    fn test_mapper_produces_shallow_tree() {
        // Verify that all nodes have children: None (shallow tree)
        let tokens = vec![
            (Token::Text("Line1".to_string()), 0..5),
            (Token::Newline, 5..6),
            (Token::Text("Line2".to_string()), 6..11),
            (Token::Newline, 11..12),
        ];

        let mut mapper = ToLineTokensMapper::new();
        let result = mapper.map_flat(tokens).unwrap();

        match result {
            TokenStream::Tree(nodes) => {
                assert_eq!(nodes.len(), 2);
                // All nodes should have no children (shallow tree)
                for node in &nodes {
                    assert!(node.children.is_none());
                }
            }
            _ => panic!("Expected Tree stream"),
        }
    }

    #[test]
    fn test_preserves_token_ranges() {
        // Verify that byte ranges are preserved exactly
        let tokens = vec![
            (Token::Text("hello".to_string()), 0..5),
            (Token::Whitespace, 5..6),
            (Token::Text("world".to_string()), 6..11),
            (Token::Newline, 11..12),
        ];

        let mut mapper = ToLineTokensMapper::new();
        let result = mapper.map_flat(tokens).unwrap();

        match result {
            TokenStream::Tree(nodes) => {
                assert_eq!(nodes.len(), 1);
                let node = &nodes[0];
                // Verify each range is preserved exactly
                assert_eq!(node.tokens[0].1, 0..5);
                assert_eq!(node.tokens[1].1, 5..6);
                assert_eq!(node.tokens[2].1, 6..11);
                assert_eq!(node.tokens[3].1, 11..12);
            }
            _ => panic!("Expected Tree stream"),
        }
    }
}
