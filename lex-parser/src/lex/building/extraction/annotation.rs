//! Annotation Data Extraction
//!
//! Extracts primitive data (text, byte ranges) from normalized token vectors
//! for building Annotation AST nodes. Orchestrates label and parameter parsing.

use super::parameter::{parse_parameter, ParameterData};
use crate::lex::annotation::split_label_tokens_with_ranges;
use crate::lex::token::normalization::utilities::{compute_bounding_box, extract_text};
use crate::lex::token::Token;
use std::ops::Range as ByteRange;

/// Extracted data for building an Annotation AST node.
///
/// Contains the label text, parameters, and their byte ranges.
#[derive(Debug, Clone)]
pub(in crate::lex::building) struct AnnotationData {
    /// The annotation label text
    pub label_text: String,
    /// Byte range of the label
    pub label_byte_range: ByteRange<usize>,
    /// Extracted parameter data
    pub parameters: Vec<ParameterData>,
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
pub(in crate::lex::building) fn extract_annotation_data(
    tokens: Vec<(Token, ByteRange<usize>)>,
    source: &str,
) -> AnnotationData {
    if tokens.is_empty() {
        panic!("Annotation header tokens cannot be empty; parser must ensure labels are present");
    }

    // 1. Parse label
    let (label_tokens, mut i, has_label) = split_label_tokens_with_ranges(&tokens);
    if !has_label {
        panic!("Annotation header must include a label before parameters");
    }

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
