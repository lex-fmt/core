//! Annotation Data Extraction
//!
//! Extracts primitive data (text, byte ranges) from normalized token vectors
//! for building Annotation AST nodes. Handles label and parameter parsing.

use crate::lex::token::normalization::utilities::{compute_bounding_box, extract_text};
use crate::lex::token::Token;
use std::ops::Range as ByteRange;

/// Extracted data for a parameter (key=value pair).
///
/// Contains primitive data (text and byte ranges) for constructing a Parameter AST node.
#[derive(Debug, Clone)]
pub(in crate::lex::building) struct ParameterData {
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
pub(in crate::lex::building) struct AnnotationData {
    /// The annotation label text
    pub label_text: String,
    /// Byte range of the label
    pub label_byte_range: ByteRange<usize>,
    /// Extracted parameter data
    pub parameters: Vec<ParameterData>,
}

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
            val_tokens.push(tokens[i].clone()); // Include opening quote
            i += 1;
            while i < tokens.len() && !matches!(tokens[i].0, Token::Quote) {
                val_tokens.push(tokens[i].clone());
                i += 1;
            }
            if i < tokens.len() && matches!(tokens[i].0, Token::Quote) {
                val_tokens.push(tokens[i].clone()); // Include closing quote
                i += 1;
            }
        } else {
            is_quoted = false;
            // Unquoted value - collect until comma, LexMarker, BlankLine, or end
            while i < tokens.len() {
                match &tokens[i].0 {
                    Token::Comma | Token::LexMarker | Token::BlankLine(_) => break,
                    Token::Whitespace => {
                        // Check if there's a comma, LexMarker, or BlankLine after whitespace
                        let mut peek = i + 1;
                        while peek < tokens.len() && matches!(tokens[peek].0, Token::Whitespace) {
                            peek += 1;
                        }
                        if peek < tokens.len()
                            && matches!(
                                tokens[peek].0,
                                Token::Comma | Token::LexMarker | Token::BlankLine(_)
                            )
                        {
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
pub(in crate::lex::building) fn extract_annotation_data(
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
