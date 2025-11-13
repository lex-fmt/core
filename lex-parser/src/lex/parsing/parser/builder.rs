//! AST Node Builder
//!
//! This module converts matched grammar patterns into ParseNode AST structures.
//! It handles the extraction of tokens from LineContainers and the recursive
//! descent into nested containers.

use crate::lex::parsing::ir::{NodeType, ParseNode};
use crate::lex::token::{LineContainer, LineToken, Token};
use std::ops::Range;

/// Type alias for the recursive parser function callback
type ParserFn = dyn Fn(Vec<LineContainer>, &str) -> Result<Vec<ParseNode>, String>;

/// Represents the result of pattern matching
#[derive(Debug, Clone)]
pub(super) enum PatternMatch {
    /// Verbatim block: one or more subject/content pairs followed by closing annotation
    VerbatimBlock {
        groups: Vec<VerbatimGroupMatch>,
        closing_idx: usize,
    },
    /// Annotation block: start + container + end
    AnnotationBlock {
        start_idx: usize,
        content_idx: usize,
    },
    /// Annotation single: just start line
    AnnotationSingle { start_idx: usize },
    /// List: preceding blank line + 2+ consecutive list items
    List { items: Vec<(usize, Option<usize>)> },
    /// Definition: subject + immediate indent + content
    Definition {
        subject_idx: usize,
        content_idx: usize,
    },
    /// Session: subject + blank line + indent + content
    Session {
        subject_idx: usize,
        content_idx: usize,
    },
    /// Paragraph: one or more consecutive non-blank, non-special lines
    Paragraph { start_idx: usize, end_idx: usize },
    /// Blank line group: one or more consecutive blank lines
    BlankLineGroup,
}

/// Represents a matched verbatim group (subject + optional content)
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct VerbatimGroupMatch {
    pub subject_idx: usize,
    pub content_idx: Option<usize>,
}

/// Convert a matched pattern to a ParseNode.
///
/// # Arguments
///
/// * `tokens` - The full token array
/// * `pattern` - The matched pattern with relative indices
/// * `pattern_offset` - Index where the pattern starts (converts relative to absolute indices)
/// * `source` - Original source text
/// * `parse_children` - Function to recursively parse nested containers
pub(super) fn convert_pattern_to_node(
    tokens: &[LineContainer],
    pattern: &PatternMatch,
    pattern_offset: usize,
    source: &str,
    parse_children: &ParserFn,
) -> Result<ParseNode, String> {
    match pattern {
        PatternMatch::VerbatimBlock {
            groups,
            closing_idx,
        } => build_verbatim_block(tokens, groups, *closing_idx),
        PatternMatch::AnnotationBlock {
            start_idx,
            content_idx,
        } => build_annotation_block(
            tokens,
            pattern_offset + start_idx,
            pattern_offset + content_idx,
            source,
            parse_children,
        ),
        PatternMatch::AnnotationSingle { start_idx } => {
            build_annotation_single(tokens, pattern_offset + start_idx)
        }
        PatternMatch::List { items } => {
            build_list(tokens, items, pattern_offset, source, parse_children)
        }
        PatternMatch::Definition {
            subject_idx,
            content_idx,
        } => build_definition(
            tokens,
            pattern_offset + subject_idx,
            pattern_offset + content_idx,
            source,
            parse_children,
        ),
        PatternMatch::Session {
            subject_idx,
            content_idx,
        } => build_session(
            tokens,
            pattern_offset + subject_idx,
            pattern_offset + content_idx,
            source,
            parse_children,
        ),
        PatternMatch::Paragraph { start_idx, end_idx } => {
            build_paragraph(tokens, pattern_offset + start_idx, pattern_offset + end_idx)
        }
        PatternMatch::BlankLineGroup => {
            Err("Internal error: BlankLineGroup reached convert_pattern_to_node".to_string())
        }
    }
}

/// Build a verbatim block node from matched groups
fn build_verbatim_block(
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

/// Build an annotation block node
fn build_annotation_block(
    tokens: &[LineContainer],
    start_idx: usize,
    content_idx: usize,
    source: &str,
    parse_children: &ParserFn,
) -> Result<ParseNode, String> {
    let start_token = extract_line_token(&tokens[start_idx])?;
    let header_tokens = extract_annotation_header_tokens(start_token);

    let children = if let Some(LineContainer::Container { children, .. }) = tokens.get(content_idx)
    {
        parse_children(children.clone(), source)?
    } else {
        vec![]
    };

    Ok(ParseNode::new(
        NodeType::Annotation,
        header_tokens,
        children,
    ))
}

/// Build an annotation single-line node
fn build_annotation_single(
    tokens: &[LineContainer],
    start_idx: usize,
) -> Result<ParseNode, String> {
    let start_token = extract_line_token(&tokens[start_idx])?;
    let (header_tokens, children) = extract_annotation_single_content(start_token);

    Ok(ParseNode::new(
        NodeType::Annotation,
        header_tokens,
        children,
    ))
}

/// Build a list node with list items
fn build_list(
    tokens: &[LineContainer],
    items: &[(usize, Option<usize>)],
    pattern_offset: usize,
    source: &str,
    parse_children: &ParserFn,
) -> Result<ParseNode, String> {
    let mut list_items = Vec::new();

    for (item_idx, content_idx) in items {
        let item_token = extract_line_token(&tokens[pattern_offset + item_idx])?;

        let children = if let Some(content_idx_val) = content_idx {
            if let Some(LineContainer::Container { children, .. }) =
                tokens.get(pattern_offset + content_idx_val)
            {
                parse_children(children.clone(), source)?
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        let list_item = ParseNode::new(
            NodeType::ListItem,
            item_token
                .source_tokens
                .clone()
                .into_iter()
                .zip(item_token.token_spans.clone())
                .collect(),
            children,
        );
        list_items.push(list_item);
    }

    Ok(ParseNode::new(NodeType::List, vec![], list_items))
}

/// Build a definition node
fn build_definition(
    tokens: &[LineContainer],
    subject_idx: usize,
    content_idx: usize,
    source: &str,
    parse_children: &ParserFn,
) -> Result<ParseNode, String> {
    let subject_token = extract_line_token(&tokens[subject_idx])?;

    let children = if let Some(LineContainer::Container { children, .. }) = tokens.get(content_idx)
    {
        parse_children(children.clone(), source)?
    } else {
        Vec::new()
    };

    // Filter out Colon, Whitespace, and BlankLine tokens from definition subject
    let subject_tokens: Vec<_> = subject_token
        .source_tokens
        .clone()
        .into_iter()
        .zip(subject_token.token_spans.clone())
        .filter(|(token, _)| {
            !matches!(
                token,
                crate::lex::lexing::Token::Colon
                    | crate::lex::lexing::Token::Whitespace
                    | crate::lex::lexing::Token::BlankLine(_)
            )
        })
        .collect();

    Ok(ParseNode::new(
        NodeType::Definition,
        subject_tokens,
        children,
    ))
}

/// Build a session node
fn build_session(
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
                crate::lex::lexing::Token::Whitespace | crate::lex::lexing::Token::BlankLine(_)
            )
        })
        .collect();

    Ok(ParseNode::new(
        NodeType::Session,
        subject_tokens,
        content_children,
    ))
}

/// Build a paragraph node
fn build_paragraph(
    tokens: &[LineContainer],
    start_idx: usize,
    end_idx: usize,
) -> Result<ParseNode, String> {
    let paragraph_tokens: Vec<LineToken> = (start_idx..=end_idx)
        .filter_map(|idx| extract_line_token(&tokens[idx]).ok().cloned())
        .collect();

    let mut all_tokens = Vec::new();
    for line in paragraph_tokens {
        all_tokens.extend(
            line.source_tokens
                .into_iter()
                .zip(line.token_spans.into_iter()),
        );
    }

    Ok(ParseNode::new(NodeType::Paragraph, all_tokens, vec![]))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Extract a LineToken from a LineContainer
fn extract_line_token(token: &LineContainer) -> Result<&LineToken, String> {
    match token {
        LineContainer::Token(t) => Ok(t),
        _ => Err("Expected LineToken, found Container".to_string()),
    }
}

/// Recursively gather all LineTokens contained within a LineContainer tree.
///
/// The tokenizer already encodes indentation structure via nested
/// `LineContainer::Container` nodes, so verbatim content that spans multiple
/// indentation levels needs to be flattened before we hand the tokens to the
/// shared AST builders. We keep every nested line (including those that contain
/// inline `::` markers) so verbatim blocks rely on dedent boundaries instead of
/// mistaking inline markers for closing annotations.
fn collect_line_tokens(container: &LineContainer, out: &mut Vec<LineToken>) {
    match container {
        LineContainer::Token(token) => out.push(token.clone()),
        LineContainer::Container { children } => {
            for child in children {
                collect_line_tokens(child, out);
            }
        }
    }
}

/// Extract header tokens from an annotation start line.
/// Header tokens are all tokens between the two :: markers (excluding the markers themselves).
fn extract_annotation_header_tokens(
    start_token: &LineToken,
) -> Vec<(Token, std::ops::Range<usize>)> {
    start_token
        .source_tokens
        .clone()
        .into_iter()
        .zip(start_token.token_spans.clone())
        .filter(|(token, _)| !matches!(token, Token::LexMarker))
        .collect()
}

/// Extract content from an annotation single-line form.
/// Returns (header_tokens, content_children) where content_children is either empty
/// or contains a single Paragraph node with the inline content.
fn extract_annotation_single_content(
    start_token: &LineToken,
) -> (Vec<(Token, Range<usize>)>, Vec<ParseNode>) {
    let all_tokens = start_token
        .source_tokens
        .clone()
        .into_iter()
        .zip(start_token.token_spans.clone())
        .collect::<Vec<_>>();

    let mut lex_marker_count = 0;
    let mut content_started = false;
    let mut header_tokens = Vec::new();
    let mut content_tokens = Vec::new();

    for (token, span) in all_tokens {
        if token == Token::LexMarker {
            lex_marker_count += 1;
            if lex_marker_count == 2 {
                content_started = true;
            }
            continue;
        }

        if !content_started {
            header_tokens.push((token, span));
        } else {
            content_tokens.push((token, span));
        }
    }

    // If there's content after the header, create a paragraph for it
    let children = if !content_tokens.is_empty() {
        vec![ParseNode::new(NodeType::Paragraph, content_tokens, vec![])]
    } else {
        vec![]
    };

    (header_tokens, children)
}
