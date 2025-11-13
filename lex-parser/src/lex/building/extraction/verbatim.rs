//! Verbatim Block Data Extraction
//!
//! Extracts primitive data (text, byte ranges) from normalized token vectors
//! for building Verbatim AST nodes. Handles indentation wall stripping.

use crate::lex::token::normalization::utilities::{compute_bounding_box, extract_text};
use crate::lex::token::Token;
use std::ops::Range as ByteRange;

/// Token buckets for a single verbatim subject/content pair prior to extraction.
#[derive(Debug, Clone)]
pub struct VerbatimGroupTokenLines {
    pub subject_tokens: Vec<(Token, ByteRange<usize>)>,
    pub content_token_lines: Vec<Vec<(Token, ByteRange<usize>)>>,
}

/// Extracted data for an individual verbatim group item.
#[derive(Debug, Clone)]
pub(in crate::lex::building) struct VerbatimGroupData {
    pub subject_text: String,
    pub subject_byte_range: ByteRange<usize>,
    pub content_lines: Vec<(String, ByteRange<usize>)>,
}

/// Extracted data for building a VerbatimBlock AST node.
///
/// Contains the ordered verbatim groups with indentation wall already stripped.
#[derive(Debug, Clone)]
pub(in crate::lex::building) struct VerbatimBlockkData {
    pub groups: Vec<VerbatimGroupData>,
}

/// Calculate the indentation wall from content token lines.
///
/// The "wall" is the minimum indentation level across all content lines.
/// This is determined by counting leading Indent tokens.
///
/// # Arguments
///
/// * `content_token_lines` - Token vectors for each content line
///
/// # Returns
///
/// The number of Indent tokens to strip from each line (the wall depth)
fn calculate_indentation_wall(content_token_lines: &[Vec<(Token, ByteRange<usize>)>]) -> usize {
    if content_token_lines.is_empty() {
        return 0;
    }

    content_token_lines
        .iter()
        .map(|tokens| {
            // Count leading Indent tokens
            tokens
                .iter()
                .take_while(|(token, _)| matches!(token, Token::Indent(_) | Token::Indentation))
                .count()
        })
        .min()
        .unwrap_or(0)
}

/// Strip indentation wall from a line of tokens.
///
/// Removes the first `wall_depth` Indent/Indentation tokens from the line.
///
/// # Arguments
///
/// * `tokens` - Token vector for one line
/// * `wall_depth` - Number of leading Indent tokens to remove
///
/// # Returns
///
/// Token vector with wall stripped
fn strip_indentation_wall(
    tokens: Vec<(Token, ByteRange<usize>)>,
    wall_depth: usize,
) -> Vec<(Token, ByteRange<usize>)> {
    let mut skipped = 0;
    tokens
        .into_iter()
        .skip_while(|(token, _)| {
            if skipped < wall_depth && matches!(token, Token::Indent(_) | Token::Indentation) {
                skipped += 1;
                true
            } else {
                false
            }
        })
        .collect()
}

/// Extract verbatim block data from grouped subject/content tokens.
///
/// Applies indentation wall stripping per group so nested verbatim blocks share
/// identical extracted content regardless of indentation depth.
pub(in crate::lex::building) fn extract_verbatim_block_data(
    groups: Vec<VerbatimGroupTokenLines>,
    source: &str,
) -> VerbatimBlockkData {
    let groups = groups
        .into_iter()
        .map(|group| extract_verbatim_group(group, source))
        .collect();

    VerbatimBlockkData { groups }
}

fn extract_verbatim_group(
    VerbatimGroupTokenLines {
        subject_tokens,
        mut content_token_lines,
    }: VerbatimGroupTokenLines,
    source: &str,
) -> VerbatimGroupData {
    let subject_byte_range = compute_bounding_box(&subject_tokens);
    let subject_text = extract_text(subject_byte_range.clone(), source)
        .trim()
        .to_string();

    let wall_depth = calculate_indentation_wall(&content_token_lines);
    content_token_lines = content_token_lines
        .into_iter()
        .map(|tokens| strip_indentation_wall(tokens, wall_depth))
        .collect();

    let content_lines: Vec<(String, ByteRange<usize>)> = content_token_lines
        .into_iter()
        .map(|tokens| {
            if tokens.is_empty() {
                (String::new(), 0..0)
            } else {
                let byte_range = compute_bounding_box(&tokens);
                let line_text = extract_text(byte_range.clone(), source);
                (line_text, byte_range)
            }
        })
        .collect();

    VerbatimGroupData {
        subject_text,
        subject_byte_range,
        content_lines,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_indentation_wall() {
        // Two lines, one with 1 indent, one with 2 indents -> wall is 1
        let lines = vec![
            vec![(Token::Indentation, 0..4), (Token::Text("a".into()), 4..5)],
            vec![
                (Token::Indentation, 0..4),
                (Token::Indentation, 4..8),
                (Token::Text("b".into()), 8..9),
            ],
        ];

        let wall = calculate_indentation_wall(&lines);
        assert_eq!(wall, 1);
    }

    #[test]
    fn test_extract_verbatim_block_data_strips_wall() {
        let source = "Code:\n    line1\n        line2";

        let subject_tokens = vec![(Token::Text("Code".to_string()), 0..4)];

        let content_lines = vec![
            vec![
                (Token::Indentation, 6..10),
                (Token::Text("line1".to_string()), 10..15),
            ],
            vec![
                (Token::Indentation, 16..20),
                (Token::Indentation, 20..24),
                (Token::Text("line2".to_string()), 24..29),
            ],
        ];

        let group = VerbatimGroupTokenLines {
            subject_tokens,
            content_token_lines: content_lines,
        };

        let data = extract_verbatim_block_data(vec![group], source);

        assert_eq!(data.groups.len(), 1);
        assert_eq!(data.groups[0].subject_text, "Code");
        // Wall of 1 indent should be stripped from both lines
        // So line1 has no indent, line2 has 1 indent left
        assert_eq!(data.groups[0].content_lines.len(), 2);
        assert_eq!(data.groups[0].content_lines[0].0, "line1");
        assert_eq!(data.groups[0].content_lines[1].0, "    line2");
    }
}
