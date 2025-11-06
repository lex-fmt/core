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

use super::token::processing::{compute_bounding_box, extract_text};
use crate::lex::lexing::tokens_core::Token;
use std::ops::Range as ByteRange;

// ============================================================================
// DATA STRUCTURES
// ============================================================================

/// Extracted data for building a Paragraph AST node.
///
/// Contains the text and byte ranges for each line, plus the overall byte range.
/// All ranges are byte offsets (Range<usize>), not ast::Range.
#[derive(Debug, Clone)]
pub(super) struct ParagraphData {
    /// Text and byte range for each line in the paragraph
    pub text_lines: Vec<(String, ByteRange<usize>)>,
    /// Overall byte range spanning all lines
    pub overall_byte_range: ByteRange<usize>,
}

/// Extracted data for building a Session AST node.
///
/// Contains the title text and its byte range.
#[derive(Debug, Clone)]
pub(super) struct SessionData {
    /// The session title text
    pub title_text: String,
    /// Byte range of the title
    pub title_byte_range: ByteRange<usize>,
}

/// Extracted data for building a Definition AST node.
///
/// Contains the subject text and its byte range.
#[derive(Debug, Clone)]
pub(super) struct DefinitionData {
    /// The definition subject text
    pub subject_text: String,
    /// Byte range of the subject
    pub subject_byte_range: ByteRange<usize>,
}

/// Extracted data for building a ListItem AST node.
///
/// Contains the marker text and its byte range.
#[derive(Debug, Clone)]
pub(super) struct ListItemData {
    /// The list item marker text (e.g., "-", "1.", "a)")
    pub marker_text: String,
    /// Byte range of the marker
    pub marker_byte_range: ByteRange<usize>,
}

/// Extracted data for a parameter (key=value pair).
///
/// Contains primitive data (text and byte ranges) for constructing a Parameter AST node.
#[derive(Debug, Clone)]
pub(super) struct ParameterData {
    /// The parameter key text
    pub key_text: String,
    /// The parameter value text (optional)
    pub value_text: Option<String>,
    /// Byte range of the key
    #[allow(dead_code)]
    pub key_byte_range: ByteRange<usize>,
    /// Byte range of the value (if present)
    #[allow(dead_code)]
    pub value_byte_range: Option<ByteRange<usize>>,
    /// Overall byte range spanning the entire parameter
    pub overall_byte_range: ByteRange<usize>,
}

/// Extracted data for building an Annotation AST node.
///
/// Contains the label text, parameters, and their byte ranges.
#[derive(Debug, Clone)]
pub(super) struct AnnotationData {
    /// The annotation label text
    pub label_text: String,
    /// Byte range of the label
    pub label_byte_range: ByteRange<usize>,
    /// Extracted parameter data
    pub parameters: Vec<ParameterData>,
}

/// Extracted data for building a ForeignBlock AST node.
///
/// Contains subject, content lines, and their byte ranges.
/// The content lines have the indentation wall already stripped.
#[derive(Debug, Clone)]
pub(super) struct ForeignBlockData {
    /// The foreign block subject text
    pub subject_text: String,
    /// Byte range of the subject
    pub subject_byte_range: ByteRange<usize>,
    /// The content lines (with indentation wall stripped) - each is (text, byte_range)
    pub content_lines: Vec<(String, ByteRange<usize>)>,
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
pub(super) fn extract_paragraph_data(
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
pub(super) fn extract_session_data(
    tokens: Vec<(Token, ByteRange<usize>)>,
    source: &str,
) -> SessionData {
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
pub(super) fn extract_definition_data(
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
pub(super) fn extract_list_item_data(
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

/// Parse label from tokens.
///
/// Identifies which tokens belong to the label by finding tokens that are:
/// - Text, Dash, Number, or Period tokens
/// - NOT followed by an Equals sign (which would make them part of a parameter)
///
/// Returns the label tokens and the index where label ends.
fn parse_label_tokens(
    tokens: &[(Token, ByteRange<usize>)],
) -> (Vec<(Token, ByteRange<usize>)>, usize) {
    let mut label_tokens = Vec::new();
    let mut i = 0;

    // Skip leading whitespace
    while i < tokens.len() && matches!(tokens[i].0, Token::Whitespace) {
        i += 1;
    }

    // Collect label tokens until we hit '=' or end
    while i < tokens.len() {
        match &tokens[i].0 {
            Token::Text(_) | Token::Dash | Token::Number(_) | Token::Period => {
                // Check if this sequence of key-like tokens is followed by '='
                // Need to scan ahead past all Text/Dash/Number/Period tokens
                let mut check_idx = i + 1;

                // Skip any remaining key-like tokens (for multi-token keys like "key1" = "key" + "1")
                while check_idx < tokens.len() {
                    if matches!(
                        tokens[check_idx].0,
                        Token::Text(_) | Token::Dash | Token::Number(_) | Token::Period
                    ) {
                        check_idx += 1;
                    } else {
                        break;
                    }
                }

                // Now skip whitespace
                while check_idx < tokens.len() && matches!(tokens[check_idx].0, Token::Whitespace) {
                    check_idx += 1;
                }

                // Check if we found '='
                if check_idx < tokens.len() && matches!(tokens[check_idx].0, Token::Equals) {
                    // This is the start of parameters, stop label collection
                    break;
                }

                label_tokens.push(tokens[i].clone());
                i += 1;
            }
            Token::Whitespace => {
                // Include whitespace in label
                label_tokens.push(tokens[i].clone());
                i += 1;
            }
            _ => {
                // Hit a non-label token, stop
                break;
            }
        }
    }

    (label_tokens, i)
}

/// Parse a single parameter (key=value or just key).
///
/// Returns the parameter data and the index after this parameter.
fn parse_parameter(
    tokens: &[(Token, ByteRange<usize>)],
    start_idx: usize,
    source: &str,
) -> Option<(ParameterData, usize)> {
    let mut i = start_idx;

    // Skip leading whitespace and commas
    while i < tokens.len() && matches!(tokens[i].0, Token::Whitespace | Token::Comma) {
        i += 1;
    }

    if i >= tokens.len() {
        return None;
    }

    // Collect key tokens
    let mut key_tokens = Vec::new();
    while i < tokens.len() {
        match &tokens[i].0 {
            Token::Text(_) | Token::Dash | Token::Number(_) | Token::Period => {
                key_tokens.push(tokens[i].clone());
                i += 1;
            }
            _ => break,
        }
    }

    if key_tokens.is_empty() {
        return None;
    }

    // Skip whitespace after key
    while i < tokens.len() && matches!(tokens[i].0, Token::Whitespace) {
        i += 1;
    }

    // Check for '='
    let (value_tokens, value_range) = if i < tokens.len() && matches!(tokens[i].0, Token::Equals) {
        i += 1; // Skip '='

        // Skip whitespace after '='
        while i < tokens.len() && matches!(tokens[i].0, Token::Whitespace) {
            i += 1;
        }

        // Collect value tokens
        let mut val_tokens = Vec::new();
        let is_quoted;

        // Check if value is quoted
        if i < tokens.len() && matches!(tokens[i].0, Token::Quote) {
            is_quoted = true;
            i += 1; // Skip opening quote
            while i < tokens.len() && !matches!(tokens[i].0, Token::Quote) {
                val_tokens.push(tokens[i].clone());
                i += 1;
            }
            if i < tokens.len() && matches!(tokens[i].0, Token::Quote) {
                i += 1; // Skip closing quote
            }
        } else {
            is_quoted = false;
            // Unquoted value - collect until comma or end
            while i < tokens.len() {
                match &tokens[i].0 {
                    Token::Comma => break,
                    Token::Whitespace => {
                        // Check if there's a comma after whitespace
                        let mut peek = i + 1;
                        while peek < tokens.len() && matches!(tokens[peek].0, Token::Whitespace) {
                            peek += 1;
                        }
                        if peek < tokens.len() && matches!(tokens[peek].0, Token::Comma) {
                            break;
                        }
                        val_tokens.push(tokens[i].clone());
                        i += 1;
                    }
                    _ => {
                        val_tokens.push(tokens[i].clone());
                        i += 1;
                    }
                }
            }
        }

        if !val_tokens.is_empty() {
            let val_range = compute_bounding_box(&val_tokens);
            let val_text = extract_text(val_range.clone(), source);
            // Only trim unquoted values - quoted values should preserve spaces
            let val_text = if is_quoted {
                val_text
            } else {
                val_text.trim().to_string()
            };
            (Some(val_text), Some(val_range))
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    let key_byte_range = compute_bounding_box(&key_tokens);
    let key_text = extract_text(key_byte_range.clone(), source)
        .trim()
        .to_string();

    // Compute overall range
    let overall_start = key_tokens.first().unwrap().1.start;
    let overall_end = if let Some(ref vr) = value_range {
        vr.end
    } else {
        key_tokens.last().unwrap().1.end
    };
    let overall_byte_range = overall_start..overall_end;

    Some((
        ParameterData {
            key_text,
            value_text: value_tokens,
            key_byte_range,
            value_byte_range: value_range,
            overall_byte_range,
        },
        i,
    ))
}

/// Extract annotation data from tokens (between :: markers).
///
/// This function implements the full annotation header parsing logic:
/// 1. Identify label tokens (before any '=' sign)
/// 2. Parse parameters (key=value pairs)
/// 3. Extract text for all components
///
/// # Arguments
///
/// * `tokens` - The tokens between :: markers
/// * `source` - The original source string
///
/// # Returns
///
/// AnnotationData containing label text, parameters, and byte ranges
///
/// # Example
///
/// ```ignore
/// Input tokens: "warning severity=high, category=security"
/// Output: AnnotationData {
///   label_text: "warning",
///   parameters: [
///     { key: "severity", value: Some("high") },
///     { key: "category", value: Some("security") }
///   ]
/// }
/// ```
pub(super) fn extract_annotation_data(
    tokens: Vec<(Token, ByteRange<usize>)>,
    source: &str,
) -> AnnotationData {
    if tokens.is_empty() {
        return AnnotationData {
            label_text: String::new(),
            label_byte_range: 0..0,
            parameters: Vec::new(),
        };
    }

    // 1. Parse label
    let (label_tokens, mut i) = parse_label_tokens(&tokens);

    let (label_text, label_byte_range) = if !label_tokens.is_empty() {
        let range = compute_bounding_box(&label_tokens);
        let text = extract_text(range.clone(), source).trim().to_string();
        (text, range)
    } else {
        (String::new(), 0..0)
    };

    // 2. Parse parameters
    let mut parameters = Vec::new();
    while i < tokens.len() {
        if let Some((param_data, next_i)) = parse_parameter(&tokens, i, source) {
            parameters.push(param_data);
            i = next_i;
        } else {
            break;
        }
    }

    AnnotationData {
        label_text,
        label_byte_range,
        parameters,
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
pub(super) fn extract_foreign_block_data(
    subject_tokens: Vec<(Token, ByteRange<usize>)>,
    mut content_token_lines: Vec<Vec<(Token, ByteRange<usize>)>>,
    source: &str,
) -> ForeignBlockData {
    // Extract subject
    let subject_byte_range = compute_bounding_box(&subject_tokens);
    let subject_text = extract_text(subject_byte_range.clone(), source)
        .trim()
        .to_string();

    // Calculate and strip indentation wall
    let wall_depth = calculate_indentation_wall(&content_token_lines);

    // Strip wall from each line
    content_token_lines = content_token_lines
        .into_iter()
        .map(|tokens| strip_indentation_wall(tokens, wall_depth))
        .collect();

    // Extract content lines with their byte ranges
    let content_lines: Vec<(String, ByteRange<usize>)> = content_token_lines
        .into_iter()
        .map(|tokens| {
            if tokens.is_empty() {
                // Empty line
                (String::new(), 0..0)
            } else {
                let byte_range = compute_bounding_box(&tokens);
                let line_text = extract_text(byte_range.clone(), source);
                (line_text, byte_range)
            }
        })
        .collect();

    ForeignBlockData {
        subject_text,
        subject_byte_range,
        content_lines,
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
        assert_eq!(data.content_lines.len(), 2);
        assert_eq!(data.content_lines[0].0, "line1");
        assert_eq!(data.content_lines[1].0, "    line2");
    }
}
