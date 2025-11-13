//! Verbatim block builder
//!
//! Handles construction of verbatim block nodes from matched patterns.

use super::helpers::{collect_line_tokens, extract_annotation_header_tokens, extract_line_token};
use crate::lex::parsing::ir::{NodeType, ParseNode, ParseNodePayload};
use crate::lex::token::{LineContainer, LineType};
use std::ops::Range;

/// Build a verbatim block node from a subject, arbitrary content lines, and a closing annotation.
pub(in crate::lex::parsing::parser::builder) fn build_verbatim_block(
    tokens: &[LineContainer],
    subject_idx: usize,
    content_range: Range<usize>,
    closing_idx: usize,
) -> Result<ParseNode, String> {
    let subject_token = extract_line_token(&tokens[subject_idx])?.clone();

    let mut content_lines = Vec::new();
    for idx in content_range {
        if let Some(container) = tokens.get(idx) {
            collect_line_tokens(container, &mut content_lines);
        }
    }

    let closing_token = extract_line_token(&tokens[closing_idx])?;
    if closing_token.line_type != LineType::DataLine {
        return Err("Verbatim blocks must end with a data line (:: label params)".to_string());
    }
    let header_tokens = extract_annotation_header_tokens(closing_token)?;

    let verbatim_node = ParseNode::new(NodeType::VerbatimBlock, vec![], vec![]).with_payload(
        ParseNodePayload::VerbatimBlock {
            subject: subject_token,
            content_lines,
            closing_data_tokens: header_tokens,
        },
    );

    Ok(verbatim_node)
}
