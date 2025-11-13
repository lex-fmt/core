//! Line Classification
//!
//! Core classification logic for determining line types based on token patterns.
//! This module contains the stateful classifier for verbatim block detection.
use crate::lex::token::{LineToken, LineType, Token};
/// A stateful classifier that identifies verbatim blocks and re-tags their content lines.
pub struct VerbatimClassifier {
    buffer: Vec<LineToken>,
    in_verbatim_block: bool,
    verbatim_start_indent: usize,
}
impl VerbatimClassifier {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            in_verbatim_block: false,
            verbatim_start_indent: 0,
        }
    }
    /// Processes a line, buffering it if it's part of a potential verbatim block.
    ///
    /// Returns a vector of processed lines. This will be empty if the line is buffered,
    /// or will contain the re-tagged verbatim block lines if a block is completed.
    pub fn process_line(&mut self, line: LineToken) -> Vec<LineToken> {
        if !self.in_verbatim_block {
            if line.line_type == LineType::SubjectLine {
                // Potential start of a verbatim block. Start buffering.
                self.in_verbatim_block = true;
                self.verbatim_start_indent = get_indent_level(&line.source_tokens);
                self.buffer.push(line);
                return Vec::new();
            } else {
                // Not in a verbatim block, so just pass the line through.
                return vec![line];
            }
        }
        // We are in a potential verbatim block.
        let current_indent = get_indent_level(&line.source_tokens);
        if self.buffer.len() == 1 && current_indent <= self.verbatim_start_indent {
            // This is not a verbatim block, as the first line after the subject is not indented.
            // Flush the buffer and process the current line.
            self.in_verbatim_block = false;
            let mut buffered_lines = self.buffer.drain(..).collect::<Vec<_>>();
            buffered_lines.push(line);
            return buffered_lines;
        }
        if current_indent == self.verbatim_start_indent
            && line.line_type == LineType::AnnotationStartLine
        {
            // This is the closing annotation line.
            // Re-tag the buffered lines and flush them.
            self.buffer.push(line);
            let mut processed_lines = Vec::new();
            let buffer_len = self.buffer.len();
            for (i, mut buffered_line) in self.buffer.drain(..).enumerate() {
                if i > 0 && i < buffer_len - 1 {
                    buffered_line.line_type = LineType::VerbatimContentLine;
                }
                processed_lines.push(buffered_line);
            }
            self.in_verbatim_block = false;
            return processed_lines;
        } else {
            // This is a content line. Buffer it.
            self.buffer.push(line);
            return Vec::new();
        }
    }
    /// Flushes any remaining lines from the buffer.
    ///
    /// This is necessary to handle cases where the file ends while still inside
    /// a potential verbatim block.
    pub fn flush(&mut self) -> Vec<LineToken> {
        self.in_verbatim_block = false;
        self.buffer.drain(..).collect()
    }
}
/// Gets the indentation level of a line from its tokens.
fn get_indent_level(tokens: &[Token]) -> usize {
    tokens
        .iter()
        .take_while(|token| matches!(token, Token::Indentation))
        .count()
}
/// Determine the type of a line based on its tokens.
pub fn classify_line_tokens(tokens: &[Token]) -> LineType {
    if tokens.is_empty() {
        return LineType::ParagraphLine;
    }
    if is_blank_line(tokens) {
        return LineType::BlankLine;
    }
    if is_annotation_end_line(tokens) {
        return LineType::AnnotationEndLine;
    }
    if is_annotation_start_line(tokens) {
        return LineType::AnnotationStartLine;
    }
    let has_list_marker = has_list_marker(tokens);
    let has_colon = ends_with_colon(tokens);
    if has_list_marker && has_colon {
        return LineType::SubjectOrListItemLine;
    }
    if has_list_marker {
        return LineType::ListLine;
    }
    if has_colon {
        return LineType::SubjectLine;
    }
    LineType::ParagraphLine
}
/// Check if line is blank (only whitespace and newline)
fn is_blank_line(tokens: &[Token]) -> bool {
    tokens.iter().all(|t| {
        matches!(
            t,
            Token::Whitespace | Token::Indentation | Token::BlankLine(_)
        )
    })
}
/// Check if line is an annotation end line: only :: marker (and optional whitespace/newline)
fn is_annotation_end_line(tokens: &[Token]) -> bool {
    let content_tokens: Vec<_> = tokens
        .iter()
        .filter(|t| {
            !matches!(
                t,
                Token::Whitespace | Token::BlankLine(_) | Token::Indentation
            )
        })
        .collect();
    content_tokens.len() == 1 && matches!(content_tokens[0], Token::LexMarker)
}
/// Check if line is an annotation start line: follows annotation grammar
fn is_annotation_start_line(tokens: &[Token]) -> bool {
    if tokens.is_empty() {
        return false;
    }
    let marker_count = tokens
        .iter()
        .filter(|t| matches!(t, Token::LexMarker))
        .count();
    if marker_count < 1 {
        return false;
    }
    let mut first_marker_idx = None;
    for (i, token) in tokens.iter().enumerate() {
        match token {
            Token::Indentation | Token::Whitespace => continue,
            Token::LexMarker => {
                first_marker_idx = Some(i);
                break;
            }
            _ => break,
        }
    }
    let first_marker_idx = match first_marker_idx {
        Some(idx) => idx,
        None => return false,
    };
    if first_marker_idx + 1 < tokens.len()
        && !matches!(tokens[first_marker_idx + 1], Token::Whitespace)
    {
        return false;
    }
    tokens[first_marker_idx + 1..]
        .iter()
        .any(|t| matches!(t, Token::LexMarker))
}
/// Check if line starts with a list marker (after optional indentation)
pub fn has_list_marker(tokens: &[Token]) -> bool {
    let mut i = 0;
    while i < tokens.len() && matches!(tokens[i], Token::Indentation | Token::Whitespace) {
        i += 1;
    }
    if i + 1 < tokens.len()
        && matches!(tokens[i], Token::Dash)
        && matches!(tokens[i + 1], Token::Whitespace)
    {
        return true;
    }
    if i + 3 < tokens.len()
        && matches!(tokens[i], Token::OpenParen)
        && matches!(tokens[i + 3], Token::Whitespace)
        && matches!(tokens[i + 2], Token::CloseParen)
    {
        let has_number = matches!(tokens[i + 1], Token::Number(_));
        let has_letter = matches!(tokens[i + 1], Token::Text(ref s) if is_single_letter(s));
        let has_roman = matches!(tokens[i + 1], Token::Text(ref s) if is_roman_numeral(s));
        if has_number || has_letter || has_roman {
            return true;
        }
    }
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
    s.chars()
        .all(|c| matches!(c, 'I' | 'V' | 'X' | 'L' | 'C' | 'D' | 'M'))
        && s.chars().next().is_some_and(|c| c.is_uppercase())
}
/// Check if line ends with colon (ignoring trailing whitespace and newline)
pub fn ends_with_colon(tokens: &[Token]) -> bool {
    let mut i = tokens.len() as i32 - 1;
    while i >= 0 {
        let token = &tokens[i as usize];
        match token {
            Token::BlankLine(_) | Token::Whitespace => {
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
            Token::Whitespace,
            Token::Text("world".to_string()),
            Token::BlankLine(Some("\n".to_string())),
        ];
        assert_eq!(classify_line_tokens(&tokens), LineType::ParagraphLine);
    }
}
