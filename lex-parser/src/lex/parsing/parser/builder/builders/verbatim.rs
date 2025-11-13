//! Verbatim block builder
//!
//! Handles construction of verbatim block nodes from matched patterns.

use super::helpers::{collect_line_tokens, extract_annotation_single_content, extract_line_token};
use crate::lex::parsing::ir::{NodeType, ParseNode};
use crate::lex::token::LineContainer;

/// Represents a matched verbatim group (subject + optional content)
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(in crate::lex::parsing::parser) struct VerbatimGroupMatch {
    pub subject_idx: usize,
    pub content_idx: Option<usize>,
}

/// Build a verbatim block node from matched groups
pub(in crate::lex::parsing::parser::builder) fn build_verbatim_block(
    tokens: &[LineContainer],
    groups: &[VerbatimGroupMatch],
    closing_idx: usize,
) -> Result<ParseNode, String> {
    let mut child_nodes = Vec::new();

    for group in groups {
        let subject_token = extract_line_token(&tokens[group.subject_idx])?;
        let subject_tokens: Vec<_> = subject_token
            .source_tokens
            .clone()
            .into_iter()
            .zip(subject_token.token_spans.clone())
            .filter(|(token, _)| {
                !matches!(
                    token,
                    crate::lex::lexing::Token::Colon | crate::lex::lexing::Token::BlankLine(_)
                )
            })
            .collect();

        let mut content_tokens = Vec::new();
        if let Some(content_idx_val) = group.content_idx {
            if let Some(container) = tokens.get(content_idx_val) {
                let mut line_tokens = Vec::new();
                collect_line_tokens(container, &mut line_tokens);
                for line_token in line_tokens {
                    content_tokens.extend(
                        line_token
                            .source_tokens
                            .into_iter()
                            .zip(line_token.token_spans.into_iter()),
                    );
                }
            }
        }

        child_nodes.push(ParseNode::new(
            NodeType::VerbatimBlockkSubject,
            subject_tokens,
            vec![],
        ));
        child_nodes.push(ParseNode::new(
            NodeType::VerbatimBlockkContent,
            content_tokens,
            vec![],
        ));
    }

    let closing_token = extract_line_token(&tokens[closing_idx])?;
    let (header_tokens, closing_children) = extract_annotation_single_content(closing_token);
    child_nodes.push(ParseNode::new(
        NodeType::VerbatimBlockkClosing,
        header_tokens,
        closing_children,
    ));

    Ok(ParseNode::new(NodeType::VerbatimBlock, vec![], child_nodes))
}
