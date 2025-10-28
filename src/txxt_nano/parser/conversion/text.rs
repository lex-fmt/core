//! Text extraction utilities for converting token spans to text
//!
//! These functions handle the conversion from token spans to actual text content,
//! dealing with edge cases like empty spans and synthetic tokens.

use std::ops::Range;

/// Helper to extract text from source using a span
pub(crate) fn extract_text(source: &str, span: &Range<usize>) -> String {
    if span.start >= span.end || span.end > source.len() {
        // Empty or synthetic span (like for IndentLevel/DedentLevel)
        return String::new();
    }
    source[span.start..span.end].to_string()
}

/// Helper to extract and concatenate text from multiple spans
pub(crate) fn extract_line_text(source: &str, spans: &[Range<usize>]) -> String {
    if spans.is_empty() {
        return String::new();
    }

    // Find the overall span from first to last
    let start = spans.first().map(|s| s.start).unwrap_or(0);
    let end = spans.last().map(|s| s.end).unwrap_or(0);

    if start >= end || end > source.len() {
        return String::new();
    }

    source[start..end].trim().to_string()
}

/// Reconstruct raw content from token spans, preserving indentation
///
/// For foreign blocks, we need to include the leading indentation.
/// This looks backwards from the first span to find the previous newline.
pub(crate) fn reconstruct_raw_content(source: &str, spans: &[Range<usize>]) -> String {
    if spans.is_empty() {
        return String::new();
    }
    // Find the overall span from first to last
    let first_start = spans.first().map(|s| s.start).unwrap_or(0);
    let last_end = spans.last().map(|s| s.end).unwrap_or(0);

    if first_start >= last_end || last_end > source.len() {
        return String::new();
    }

    // For foreign blocks, we need to include the leading indentation.
    // Look backwards from first_start to find the previous newline.
    // Everything from after the newline to last_end is the content.
    let mut start = first_start;

    // Scan backwards to find the beginning of this line (after previous newline)
    if first_start > 0 {
        let bytes = source.as_bytes();
        // Look for the previous newline
        for i in (0..first_start).rev() {
            if bytes[i] == b'\n' {
                // Found the newline, content starts after it
                start = i + 1;
                break;
            }
        }
        // If no newline found, start from the beginning of the source
        // (This handles the first line case)
    }

    source[start..last_end].to_string()
}
