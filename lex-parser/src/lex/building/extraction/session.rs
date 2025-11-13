//! Session Data Extraction
//!
//! Extracts primitive data (text, byte ranges) from normalized token vectors
//! for building Session AST nodes.

use crate::lex::token::normalization::utilities::{compute_bounding_box, extract_text};
use crate::lex::token::Token;
use std::ops::Range as ByteRange;

/// Extracted data for building a Session AST node.
///
/// Contains the title text and its byte range.
#[derive(Debug, Clone)]
pub(in crate::lex::building) struct SessionData {
    /// The session title text
    pub title_text: String,
    /// Byte range of the title
    pub title_byte_range: ByteRange<usize>,
}

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
pub(in crate::lex::building) fn extract_session_data(
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
