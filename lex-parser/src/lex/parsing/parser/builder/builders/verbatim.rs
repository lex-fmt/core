//! Verbatim block builder
//!
//! Handles construction of verbatim block nodes from matched patterns.

use super::helpers::{collect_line_tokens, extract_annotation_single_content, extract_line_token};
use crate::lex::parsing::ir::{NodeType, ParseNode, ParseNodePayload};
use crate::lex::token::LineContainer;
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
    let (header_tokens, closing_children) = extract_annotation_single_content(closing_token);
    let closing_node = ParseNode::new(NodeType::Annotation, header_tokens, closing_children);

    let verbatim_node = ParseNode::new(NodeType::VerbatimBlock, vec![], vec![closing_node])
        .with_payload(ParseNodePayload::VerbatimBlock {
            subject: subject_token,
            content_lines,
        });

    Ok(verbatim_node)
}
