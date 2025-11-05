//! Data Extraction from Tokens
//!
//! This module extracts primitive data (text, byte ranges, metadata) from normalized
//! token vectors. It returns data structures containing only primitives - no AST types.
//!
//! # Architecture
//!
//! ```text
//! Vec<(Token, Range<usize>)> → Data Extraction → Data Structs (primitives only)
//!                               ↓
//!                               - Extract text from source
//!                               - Compute byte range bounding boxes
//!                               - Process tokens intelligently
//!                               ↓
//!                               { text: String, byte_range: Range<usize> }
//! ```
//!
//! # Responsibilities
//!
//! - Extract text from source using token byte ranges
//! - Compute bounding boxes from token ranges
//! - Implement smart token processing (e.g., indentation wall stripping)
//! - Return primitive data structures (String, Range<usize>, etc.)
//! - **NO** AST types (ast::Range is converted later in ast_creation)
//!
//! # Key Design Principle
//!
//! This layer works with **primitives only**. Byte ranges stay as `Range<usize>`.
//! The conversion to `ast::Range` happens later in the ast_creation layer.

use crate::lex::lexers::tokens_core::Token;
use crate::lex::parsers::common::token_processing::{compute_bounding_box, extract_text};
use std::ops::Range as ByteRange;

// ============================================================================
// DATA STRUCTURES
// ============================================================================

/// Extracted data for building a Paragraph AST node.
///
/// Contains the text and byte ranges for each line, plus the overall byte range.
/// All ranges are byte offsets (Range<usize>), not ast::Range.
#[derive(Debug, Clone)]
pub struct ParagraphData {
    /// Text and byte range for each line in the paragraph
    pub text_lines: Vec<(String, ByteRange<usize>)>,
    /// Overall byte range spanning all lines
    pub overall_byte_range: ByteRange<usize>,
}

/// Extracted data for building a Session AST node.
///
/// Contains the title text and its byte range.
#[derive(Debug, Clone)]
pub struct SessionData {
    /// The session title text
    pub title_text: String,
    /// Byte range of the title
    pub title_byte_range: ByteRange<usize>,
}

/// Extracted data for building a Definition AST node.
///
/// Contains the subject text and its byte range.
#[derive(Debug, Clone)]
pub struct DefinitionData {
    /// The definition subject text
    pub subject_text: String,
    /// Byte range of the subject
    pub subject_byte_range: ByteRange<usize>,
}

/// Extracted data for building a ListItem AST node.
///
/// Contains the marker text and its byte range.
#[derive(Debug, Clone)]
pub struct ListItemData {
    /// The list item marker text (e.g., "-", "1.", "a)")
    pub marker_text: String,
    /// Byte range of the marker
    pub marker_byte_range: ByteRange<usize>,
}

/// Extracted data for building an Annotation AST node.
///
/// Contains the label text and its byte range.
#[derive(Debug, Clone)]
pub struct AnnotationData {
    /// The annotation label text
    pub label_text: String,
    /// Byte range of the label
    pub label_byte_range: ByteRange<usize>,
}

/// Extracted data for building a ForeignBlock AST node.
///
/// Contains subject, content, and their byte ranges.
/// The content text has the indentation wall already stripped.
#[derive(Debug, Clone)]
pub struct ForeignBlockData {
    /// The foreign block subject text
    pub subject_text: String,
    /// Byte range of the subject
    pub subject_byte_range: ByteRange<usize>,
    /// The content text (with indentation wall stripped)
    pub content_text: String,
    /// Byte range of the content
    pub content_byte_range: ByteRange<usize>,
}

// ============================================================================
// PARAGRAPH EXTRACTION
// ============================================================================

/// Extract paragraph data from token lines.
///
/// Each inner vector represents one line of the paragraph.
/// Extracts text and computes byte ranges for each line and the overall paragraph.
///
/// # Arguments
///
/// * `token_lines` - Normalized token vectors, one per line
/// * `source` - The original source string
///
/// # Returns
///
/// ParagraphData containing text and byte ranges for the paragraph
///
/// # Example
///
/// ```rust,ignore
/// let token_lines = vec![
///     vec![(Token::Text("line1".into()), 0..5)],
///     vec![(Token::Text("line2".into()), 6..11)],
/// ];
/// let data = extract_paragraph_data(token_lines, source);
/// assert_eq!(data.text_lines.len(), 2);
/// ```
pub fn extract_paragraph_data(
    token_lines: Vec<Vec<(Token, ByteRange<usize>)>>,
    source: &str,
) -> ParagraphData {
    let text_lines: Vec<(String, ByteRange<usize>)> = token_lines
        .iter()
        .map(|tokens| {
            let byte_range = compute_bounding_box(tokens);
            let text = extract_text(byte_range.clone(), source);
            (text, byte_range)
        })
        .collect();

    // Compute overall byte range from all tokens
    let all_tokens: Vec<(Token, ByteRange<usize>)> = token_lines.into_iter().flatten().collect();
    let overall_byte_range = if all_tokens.is_empty() {
        0..0
    } else {
        compute_bounding_box(&all_tokens)
    };

    ParagraphData {
        text_lines,
        overall_byte_range,
    }
}

// ============================================================================
// SESSION EXTRACTION
// ============================================================================

/// Extract session data from title tokens.
///
/// # Arguments
///
/// * `tokens` - Normalized token vector for the session title
/// * `source` - The original source string
///
/// # Returns
///
/// SessionData containing the title text and byte range
pub fn extract_session_data(tokens: Vec<(Token, ByteRange<usize>)>, source: &str) -> SessionData {
    let title_byte_range = compute_bounding_box(&tokens);
    let title_text = extract_text(title_byte_range.clone(), source);

    SessionData {
        title_text,
        title_byte_range,
    }
}

// ============================================================================
// DEFINITION EXTRACTION
// ============================================================================

/// Extract definition data from subject tokens.
///
/// # Arguments
///
/// * `tokens` - Normalized token vector for the definition subject
/// * `source` - The original source string
///
/// # Returns
///
/// DefinitionData containing the subject text and byte range
pub fn extract_definition_data(
    tokens: Vec<(Token, ByteRange<usize>)>,
    source: &str,
) -> DefinitionData {
    let subject_byte_range = compute_bounding_box(&tokens);
    let subject_text = extract_text(subject_byte_range.clone(), source);

    DefinitionData {
        subject_text,
        subject_byte_range,
    }
}

// ============================================================================
// LIST ITEM EXTRACTION
// ============================================================================

/// Extract list item data from marker tokens.
///
/// # Arguments
///
/// * `tokens` - Normalized token vector for the list item marker
/// * `source` - The original source string
///
/// # Returns
///
/// ListItemData containing the marker text and byte range
pub fn extract_list_item_data(
    tokens: Vec<(Token, ByteRange<usize>)>,
    source: &str,
) -> ListItemData {
    let marker_byte_range = compute_bounding_box(&tokens);
    let marker_text = extract_text(marker_byte_range.clone(), source);

    ListItemData {
        marker_text,
        marker_byte_range,
    }
}

// ============================================================================
// ANNOTATION EXTRACTION
// ============================================================================

/// Extract annotation data from label tokens.
///
/// # Arguments
///
/// * `tokens` - Normalized token vector for the annotation label
/// * `source` - The original source string
///
/// # Returns
///
/// AnnotationData containing the label text and byte range
pub fn extract_annotation_data(
    tokens: Vec<(Token, ByteRange<usize>)>,
    source: &str,
) -> AnnotationData {
    let label_byte_range = compute_bounding_box(&tokens);
    let label_text = extract_text(label_byte_range.clone(), source);

    AnnotationData {
        label_text,
        label_byte_range,
    }
}

// ============================================================================
// FOREIGN BLOCK EXTRACTION
// ============================================================================

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

/// Extract foreign block data from subject, content, and closing tokens.
///
/// This function implements indentation wall stripping:
/// 1. Calculate the wall (minimum indentation across all content lines)
/// 2. Strip that many Indent tokens from the start of each line
/// 3. Extract text from the remaining tokens
///
/// This ensures that foreign blocks at different nesting levels have identical content.
///
/// # Arguments
///
/// * `subject_tokens` - Normalized tokens for the subject line
/// * `content_token_lines` - Normalized token vectors for each content line
/// * `source` - The original source string
///
/// # Returns
///
/// ForeignBlockData with the indentation wall stripped from content
///
/// # Example
///
/// ```rust,ignore
/// // Top-level foreign block: "Code:\n    line1\n    line2"
/// // Content tokens have 1 Indent each
///
/// // Nested foreign block: "Session:\n    Code:\n        line1\n        line2"
/// // Content tokens have 2 Indents each
///
/// // After extraction, both have identical content: "line1\nline2"
/// ```
pub fn extract_foreign_block_data(
    subject_tokens: Vec<(Token, ByteRange<usize>)>,
    mut content_token_lines: Vec<Vec<(Token, ByteRange<usize>)>>,
    source: &str,
) -> ForeignBlockData {
    // Extract subject
    let subject_byte_range = compute_bounding_box(&subject_tokens);
    let subject_text = extract_text(subject_byte_range.clone(), source);

    // Calculate and strip indentation wall
    let wall_depth = calculate_indentation_wall(&content_token_lines);

    // Strip wall from each line
    content_token_lines = content_token_lines
        .into_iter()
        .map(|tokens| strip_indentation_wall(tokens, wall_depth))
        .collect();

    // Extract content text line by line
    let mut content_text = String::new();
    let mut first_line = true;

    for tokens in &content_token_lines {
        // Add newline between lines (including before empty lines to preserve blank lines)
        if !first_line {
            content_text.push('\n');
        }

        // Extract text from tokens (empty lines will contribute empty string)
        if !tokens.is_empty() {
            let byte_range = compute_bounding_box(tokens);
            let line_text = extract_text(byte_range, source);
            content_text.push_str(&line_text);
        }

        first_line = false;
    }

    // Compute overall content byte range (before wall stripping)
    let all_content_tokens: Vec<(Token, ByteRange<usize>)> =
        content_token_lines.into_iter().flatten().collect();
    let content_byte_range = if all_content_tokens.is_empty() {
        0..0
    } else {
        compute_bounding_box(&all_content_tokens)
    };

    ForeignBlockData {
        subject_text,
        subject_byte_range,
        content_text,
        content_byte_range,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_paragraph_data() {
        let source = "hello world";
        let token_lines = vec![vec![(Token::Text("hello".to_string()), 0..5)]];

        let data = extract_paragraph_data(token_lines, source);

        assert_eq!(data.text_lines.len(), 1);
        assert_eq!(data.text_lines[0].0, "hello");
        assert_eq!(data.text_lines[0].1, 0..5);
        assert_eq!(data.overall_byte_range, 0..5);
    }

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
    fn test_extract_foreign_block_data_strips_wall() {
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

        let data = extract_foreign_block_data(subject_tokens, content_lines, source);

        assert_eq!(data.subject_text, "Code");
        // Wall of 1 indent should be stripped from both lines
        // So line1 has no indent, line2 has 1 indent left
        assert_eq!(data.content_text, "line1\n    line2");
    }
}
