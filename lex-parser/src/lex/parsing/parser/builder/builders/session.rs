//! Session builder
//!
//! Handles construction of session nodes.

use super::helpers::extract_line_token;
use crate::lex::parsing::ir::{NodeType, ParseNode};
use crate::lex::token::LineContainer;

/// Type alias for the recursive parser function callback
type ParserFn = dyn Fn(Vec<LineContainer>, &str) -> Result<Vec<ParseNode>, String>;

/// Build a session node
pub(in crate::lex::parsing::parser::builder) fn build_session(
    tokens: &[LineContainer],
    subject_idx: usize,
    content_idx: usize,
    source: &str,
    parse_children: &ParserFn,
) -> Result<ParseNode, String> {
    let subject_token = extract_line_token(&tokens[subject_idx])?;

    let content_children =
        if let Some(LineContainer::Container { children, .. }) = tokens.get(content_idx) {
            parse_children(children.clone(), source)?
        } else {
            vec![]
        };

    // Filter out trailing Whitespace and BlankLine tokens from session label
    let subject_tokens: Vec<_> = subject_token
        .source_tokens
        .clone()
        .into_iter()
        .zip(subject_token.token_spans.clone())
        .filter(|(token, _)| {
            !matches!(
                token,
                crate::lex::lexing::Token::Whitespace(_) | crate::lex::lexing::Token::BlankLine(_)
            )
        })
        .collect();

    Ok(ParseNode::new(
        NodeType::Session,
        subject_tokens,
        content_children,
    ))
}
