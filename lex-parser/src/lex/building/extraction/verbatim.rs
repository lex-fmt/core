//! Verbatim Block Data Extraction
use crate::lex::ast::elements::verbatim::VerbatimBlockMode;
use crate::lex::token::line::{LineToken, LineType};
use crate::lex::token::normalization::utilities::{compute_bounding_box, extract_text};
use crate::lex::token::Token;
use std::ops::Range as ByteRange;

const FULLWIDTH_INDENT_COLUMN: usize = 1; // Zero-based column index
const INFLOW_INDENT_STEP_COLUMNS: usize = 4;

/// Extracted data for an individual verbatim group item.
#[derive(Debug, Clone)]
pub(in crate::lex::building) struct VerbatimGroupData {
    pub subject_text: String,
    pub subject_byte_range: ByteRange<usize>,
    pub content_lines: Vec<(String, ByteRange<usize>)>,
}

/// Extracted data for building a VerbatimBlock AST node.
#[derive(Debug, Clone)]
pub(in crate::lex::building) struct VerbatimBlockData {
    pub groups: Vec<VerbatimGroupData>,
    pub mode: VerbatimBlockMode,
}

pub(in crate::lex::building) fn extract_verbatim_block_data(
    subject_line: &LineToken,
    content_lines: &[LineToken],
    source: &str,
) -> VerbatimBlockData {
    let mode = detect_mode(content_lines, source);
    let subject_column = first_visual_column(subject_line, source).unwrap_or(0);
    let wall_column = match mode {
        VerbatimBlockMode::Fullwidth => FULLWIDTH_INDENT_COLUMN,
        VerbatimBlockMode::Inflow => subject_column + INFLOW_INDENT_STEP_COLUMNS,
    };

    let groups = split_groups(subject_line, content_lines, subject_column, source)
        .into_iter()
        .map(|(subject, lines)| extract_group(subject, lines, wall_column, source))
        .collect();

    VerbatimBlockData { groups, mode }
}

fn detect_mode(content_lines: &[LineToken], source: &str) -> VerbatimBlockMode {
    for line in content_lines {
        if is_effectively_blank(line) {
            continue;
        }
        if let Some(column) = first_visual_column(line, source) {
            if column == FULLWIDTH_INDENT_COLUMN {
                return VerbatimBlockMode::Fullwidth;
            } else {
                break;
            }
        }
    }
    VerbatimBlockMode::Inflow
}

fn split_groups(
    first_subject: &LineToken,
    content_lines: &[LineToken],
    base_subject_column: usize,
    source: &str,
) -> Vec<(LineToken, Vec<LineToken>)> {
    let mut groups = Vec::new();
    let mut current_subject = first_subject.clone();
    let mut current_content: Vec<LineToken> = Vec::new();

    for line in content_lines {
        if line.line_type == LineType::BlankLine
            && is_effectively_blank(line)
            && current_content.is_empty()
        {
            continue;
        }
        if is_new_group_subject(line, base_subject_column, source) {
            groups.push((current_subject, current_content));
            current_subject = line.clone();
            current_content = Vec::new();
        } else {
            current_content.push(line.clone());
        }
    }

    groups.push((current_subject, current_content));
    groups
}

fn extract_group(
    subject_line: LineToken,
    content_lines: Vec<LineToken>,
    wall_column: usize,
    source: &str,
) -> VerbatimGroupData {
    let subject_pairs: Vec<_> = subject_line
        .source_token_pairs()
        .into_iter()
        .filter(|(token, _)| !matches!(token, Token::Colon | Token::BlankLine(_)))
        .collect();
    let subject_byte_range = if subject_pairs.is_empty() {
        0..0
    } else {
        compute_bounding_box(&subject_pairs)
    };
    let subject_text = extract_text(subject_byte_range.clone(), source)
        .trim()
        .to_string();

    let content_lines: Vec<(String, ByteRange<usize>)> = content_lines
        .into_iter()
        .map(|line| extract_content_line(line, wall_column, source))
        .collect();

    VerbatimGroupData {
        subject_text,
        subject_byte_range,
        content_lines,
    }
}

fn extract_content_line(
    line: LineToken,
    wall_column: usize,
    source: &str,
) -> (String, ByteRange<usize>) {
    let bounds = line_bounds(&line);
    if bounds.is_none() {
        return (String::new(), 0..0);
    }
    let (line_start, line_end) = bounds.unwrap();
    let trimmed_end = trim_trailing_newline(source, line_start, line_end);
    if trimmed_end <= line_start {
        return (String::new(), line_start..line_start);
    }

    let start_offset = advance_to_wall(source, line_start, trimmed_end, wall_column);
    if start_offset >= trimmed_end {
        return (String::new(), trimmed_end..trimmed_end);
    }

    let text = source[start_offset..trimmed_end].to_string();
    (text, start_offset..trimmed_end)
}

fn is_effectively_blank(line: &LineToken) -> bool {
    line.source_tokens.iter().all(|token| token.is_whitespace())
}

fn is_new_group_subject(line: &LineToken, base_column: usize, source: &str) -> bool {
    if !matches!(
        line.line_type,
        LineType::SubjectLine | LineType::SubjectOrListItemLine
    ) {
        return false;
    }
    first_visual_column(line, source) == Some(base_column)
}

fn first_visual_column(line: &LineToken, source: &str) -> Option<usize> {
    line.source_token_pairs()
        .into_iter()
        .find(|(token, _)| !token.is_whitespace())
        .map(|(_, range)| visual_column_at(range.start, source))
}

fn visual_column_at(offset: usize, source: &str) -> usize {
    let line_start = source[..offset].rfind('\n').map(|idx| idx + 1).unwrap_or(0);
    let mut column = 0;
    let mut idx = line_start;
    while idx < offset {
        let ch = source[idx..].chars().next().unwrap();
        if ch == '\r' {
            idx += 1;
            continue;
        }
        if ch.is_whitespace() {
            column += whitespace_width(ch);
        } else {
            column += 1;
        }
        idx += ch.len_utf8();
    }
    column
}

fn line_bounds(line: &LineToken) -> Option<(usize, usize)> {
    let pairs = line.source_token_pairs();
    if pairs.is_empty() {
        None
    } else {
        let range = compute_bounding_box(&pairs);
        Some((range.start, range.end))
    }
}

fn trim_trailing_newline(source: &str, start: usize, mut end: usize) -> usize {
    while end > start {
        let byte = source.as_bytes()[end - 1];
        if byte == b'\n' || byte == b'\r' {
            end -= 1;
        } else {
            break;
        }
    }
    end
}

fn advance_to_wall(source: &str, start: usize, end: usize, wall_column: usize) -> usize {
    let mut column = 0;
    let mut offset = start;
    while offset < end && column < wall_column {
        let ch = source[offset..].chars().next().unwrap();
        if !ch.is_whitespace() {
            break;
        }
        column += whitespace_width(ch);
        offset += ch.len_utf8();
    }
    offset.min(end)
}

fn whitespace_width(ch: char) -> usize {
    match ch {
        '\t' => INFLOW_INDENT_STEP_COLUMNS,
        _ => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::token::Token;

    struct SourceBuilder {
        text: String,
    }

    impl SourceBuilder {
        fn new() -> Self {
            Self {
                text: String::new(),
            }
        }

        fn push(&mut self, fragment: &str) -> ByteRange<usize> {
            let start = self.text.len();
            self.text.push_str(fragment);
            start..self.text.len()
        }
    }

    fn line_token(line_type: LineType, parts: Vec<(Token, ByteRange<usize>)>) -> LineToken {
        let (tokens, spans): (Vec<_>, Vec<_>) = parts.into_iter().unzip();
        LineToken {
            source_tokens: tokens,
            token_spans: spans,
            line_type,
        }
    }

    fn subject_line(builder: &mut SourceBuilder, indent_levels: usize, label: &str) -> LineToken {
        let mut parts = Vec::new();
        for _ in 0..indent_levels {
            let range = builder.push("    ");
            parts.push((Token::Indentation, range));
        }
        let range = builder.push(label);
        parts.push((Token::Text(label.to_string()), range));
        let range = builder.push(":");
        parts.push((Token::Colon, range));
        let range = builder.push("\n");
        parts.push((Token::BlankLine(Some("\n".to_string())), range));
        line_token(LineType::SubjectLine, parts)
    }

    fn content_line(builder: &mut SourceBuilder, indent_spaces: usize, text: &str) -> LineToken {
        let mut parts = Vec::new();
        for _ in 0..indent_spaces {
            let range = builder.push(" ");
            parts.push((Token::Whitespace, range));
        }
        if !text.is_empty() {
            let range = builder.push(text);
            parts.push((Token::Text(text.to_string()), range));
        }
        let range = builder.push("\n");
        parts.push((Token::BlankLine(Some("\n".to_string())), range));
        line_token(LineType::ParagraphLine, parts)
    }

    #[test]
    fn detects_fullwidth_mode_and_trims_wall() {
        let mut builder = SourceBuilder::new();
        let subject = subject_line(&mut builder, 0, "Fullwidth Example");
        let content = content_line(&mut builder, 1, "Header | Value | Notes");

        let data = extract_verbatim_block_data(&subject, &[content], &builder.text);

        assert_eq!(data.mode, VerbatimBlockMode::Fullwidth);
        assert_eq!(data.groups.len(), 1);
        assert_eq!(data.groups[0].content_lines.len(), 1);
        assert_eq!(data.groups[0].content_lines[0].0, "Header | Value | Notes");
        assert!(data.groups[0].content_lines[0].1.start < data.groups[0].content_lines[0].1.end);
    }

    #[test]
    fn splits_groups_and_strips_inflow_wall() {
        let mut builder = SourceBuilder::new();
        let subject = subject_line(&mut builder, 1, "Snippet");
        let line1 = content_line(&mut builder, 8, "line one");
        let line2 = content_line(&mut builder, 8, "line two");
        let second_subject = subject_line(&mut builder, 1, "Another block");
        let line3 = content_line(&mut builder, 8, "inner body");

        let content = vec![line1, line2, second_subject.clone(), line3];
        let data = extract_verbatim_block_data(&subject, &content, &builder.text);

        assert_eq!(data.mode, VerbatimBlockMode::Inflow);
        assert_eq!(data.groups.len(), 2);
        assert_eq!(data.groups[0].subject_text, "Snippet");
        assert_eq!(data.groups[0].content_lines[0].0, "line one");
        assert_eq!(data.groups[1].subject_text, "Another block");
        assert_eq!(data.groups[1].content_lines[0].0, "inner body");
    }
}
