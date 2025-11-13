//! List Item Data Extraction
//!
//! Extracts primitive data (text, byte ranges) from normalized token vectors
//! for building ListItem AST nodes.

use crate::lex::token::normalization::utilities::{compute_bounding_box, extract_text};
use crate::lex::token::Token;
use std::ops::Range as ByteRange;

/// Extracted data for building a ListItem AST node.
///
/// Contains the marker text and its byte range.
#[derive(Debug, Clone)]
pub(in crate::lex::building) struct ListItemData {
    /// The list item marker text (e.g., "-", "1.", "a)")
    pub marker_text: String,
    /// Byte range of the marker
    pub marker_byte_range: ByteRange<usize>,
}

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
pub(in crate::lex::building) fn extract_list_item_data(
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
