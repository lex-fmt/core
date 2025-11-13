//! Declarative Grammar Engine - Regex-Based Parser for lex
//!
//! This module implements a unified parser using declarative regex grammar rules:
//! 1. Converts token sequences to grammar notation strings
//! 2. Matches against regex patterns in declaration order
//! 3. Extracts consumed token indices from regex match
//! 4. Recursively descends into containers when building AST
//! 5. No imperative pattern matching - grammar is data, not code
//!
//! The grammar parse order (from grammar.lex §4.7):
//! 1. verbatim-block (requires closing annotation - try first for disambiguation)
//! 2. annotation_block (block with container between start and end markers)
//! 3. annotation_single (single-line annotation only)
//! 4. list (requires preceding blank line + 2+ list items)
//! 5. definition (requires subject + immediate indent)
//! 6. session (requires subject + blank line + indent)
//! 7. paragraph (any content-line or sequence thereof)
//! 8. blank_line_group (one or more consecutive blank lines)

use crate::lex::parsing::ir::{NodeType, ParseNode};
use crate::lex::token::{LineContainer, LineToken, LineType, Token};
use once_cell::sync::Lazy;
use regex::Regex;
use std::ops::Range;

/// Lazy-compiled regex for extracting list items from the list group capture
static LIST_ITEM_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(<list-line>|<subject-or-list-item-line>)(<container>)?").unwrap());

/// Grammar patterns as regex rules with names and patterns
/// Order matters: patterns are tried in declaration order for correct disambiguation
const GRAMMAR_PATTERNS: &[(&str, &str)] = &[
    // Annotation (multi-line with markers): <annotation-start-line><container><annotation-end-line>
    (
        "annotation_block_with_end",
        r"^(?P<start><annotation-start-line>)(?P<content><container>)(?P<end><annotation-end-line>)",
    ),
    // Annotation (multi-line without end marker): <annotation-start-line><container>
    (
        "annotation_block",
        r"^(?P<start><annotation-start-line>)(?P<content><container>)",
    ),
    // Annotation (single-line): <annotation-start-line><content>
    ("annotation_single", r"^(?P<start><annotation-start-line>)"),
    // List without preceding blank line (for lists inside containers)
    (
        "list_no_blank",
        r"^(?P<items>((<list-line>|<subject-or-list-item-line>)(<container>)?){2,})(?P<trailing_blank><blank-line>)?",
    ),
    // List with preceding blank line (for lists at root level)
    (
        "list",
        r"^(?P<blank><blank-line>+)(?P<items>((<list-line>|<subject-or-list-item-line>)(<container>)?){2,})(?P<trailing_blank><blank-line>)?",
    ),
    // Session: <content-line><blank-line><container>
    (
        "session",
        r"^(?P<subject><paragraph-line>|<subject-line>|<list-line>|<subject-or-list-item-line>)(?P<blank><blank-line>+)(?P<content><container>)",
    ),
    // Definition: <subject-line>|<subject-or-list-item-line>|<paragraph-line><container>
    (
        "definition",
        r"^(?P<subject><subject-line>|<subject-or-list-item-line>|<paragraph-line>)(?P<content><container>)",
    ),
    // Paragraph: <content-line>+
    (
        "paragraph",
        r"^(?P<lines>(<paragraph-line>|<subject-line>|<list-line>|<subject-or-list-item-line>|<dialog-line>)+)",
    ),
    // Blank lines: <blank-line-group>
    ("blank_line_group", r"^(?P<lines>(<blank-line>)+)"),
];

/// Represents the result of pattern matching at one level
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct VerbatimGroupMatch {
    pub subject_idx: usize,
    pub content_idx: Option<usize>,
}

enum PatternMatch {
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

/// Pattern matcher for declarative grammar using regex-based matching
pub struct GrammarMatcher;

impl GrammarMatcher {
    /// Try to match a pattern at the current level using regex patterns.
    ///
    /// Converts the current token sequence to a grammar string, matches against
    /// regex patterns in declaration order, and returns the matched pattern with
    /// consumed token indices.
    ///
    /// Returns (matched_pattern, consumed_indices)
    fn try_match(
        tokens: &[LineContainer],
        start_idx: usize,
    ) -> Option<(PatternMatch, Range<usize>)> {
        if start_idx >= tokens.len() {
            return None;
        }

        if let Some(result) = Self::match_verbatim_block(tokens, start_idx) {
            return Some(result);
        }

        // Convert remaining tokens to grammar string
        let remaining_tokens = &tokens[start_idx..];
        let token_string = Self::tokens_to_grammar_string(remaining_tokens)?;

        // Try each pattern in order
        for (pattern_name, pattern_regex_str) in GRAMMAR_PATTERNS {
            if *pattern_name == "verbatim_block" {
                continue;
            }
            if let Ok(regex) = Regex::new(pattern_regex_str) {
                if let Some(caps) = regex.captures(&token_string) {
                    let full_match = caps.get(0).unwrap();
                    let consumed_count = Self::count_consumed_tokens(full_match.as_str());

                    // Use captures to extract indices and build the pattern
                    let pattern = match *pattern_name {
                        "annotation_block_with_end" => PatternMatch::AnnotationBlock {
                            start_idx: 0,
                            content_idx: 1,
                        },
                        "annotation_block" => PatternMatch::AnnotationBlock {
                            start_idx: 0,
                            content_idx: 1,
                        },
                        "annotation_single" => PatternMatch::AnnotationSingle { start_idx: 0 },
                        "list_no_blank" => {
                            // List without preceding blank line
                            let items_str = caps.name("items").unwrap().as_str();
                            let mut items = Vec::new();
                            let mut token_idx = 0; // No blank line, so start at 0
                            for item_cap in LIST_ITEM_REGEX.find_iter(items_str) {
                                let has_container = item_cap.as_str().contains("<container>");
                                items.push((
                                    token_idx,
                                    if has_container {
                                        Some(token_idx + 1)
                                    } else {
                                        None
                                    },
                                ));
                                token_idx += if has_container { 2 } else { 1 };
                            }
                            PatternMatch::List { items }
                        }
                        "list" => {
                            let blank_count = caps
                                .name("blank")
                                .map(|m| Self::count_consumed_tokens(m.as_str()))
                                .unwrap_or(0);
                            let items_str = caps.name("items").unwrap().as_str();
                            let mut items = Vec::new();
                            let mut token_idx = blank_count;
                            for item_cap in LIST_ITEM_REGEX.find_iter(items_str) {
                                let has_container = item_cap.as_str().contains("<container>");
                                items.push((
                                    token_idx,
                                    if has_container {
                                        Some(token_idx + 1)
                                    } else {
                                        None
                                    },
                                ));
                                token_idx += if has_container { 2 } else { 1 };
                            }
                            PatternMatch::List { items }
                        }
                        "session" => {
                            let blank_str = caps.name("blank").unwrap().as_str();
                            let blank_count = Self::count_consumed_tokens(blank_str);
                            PatternMatch::Session {
                                subject_idx: 0,
                                content_idx: 1 + blank_count,
                            }
                        }
                        "definition" => PatternMatch::Definition {
                            subject_idx: 0,
                            content_idx: 1,
                        },
                        "paragraph" => PatternMatch::Paragraph {
                            start_idx: 0,
                            end_idx: consumed_count - 1,
                        },
                        "blank_line_group" => PatternMatch::BlankLineGroup,
                        _ => continue,
                    };

                    return Some((pattern, start_idx..start_idx + consumed_count));
                }
            }
        }

        None
    }

    /// Convert remaining tokens to grammar notation string
    fn tokens_to_grammar_string(tokens: &[LineContainer]) -> Option<String> {
        let mut result = String::new();
        for token in tokens {
            match token {
                LineContainer::Token(t) => {
                    result.push_str(&t.line_type.to_grammar_string());
                }
                LineContainer::Container { .. } => {
                    result.push_str("<container>");
                }
            }
        }
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    /// Count how many tokens are represented in a grammar string
    /// Each token type in angle brackets represents one token
    fn count_consumed_tokens(grammar_str: &str) -> usize {
        grammar_str.matches('<').count()
    }

    fn match_verbatim_block(
        tokens: &[LineContainer],
        start_idx: usize,
    ) -> Option<(PatternMatch, Range<usize>)> {
        use LineType::{
            AnnotationEndLine, AnnotationStartLine, BlankLine, SubjectLine, SubjectOrListItemLine,
        };

        let mut idx = start_idx;
        let len = tokens.len();
        let mut groups = Vec::new();

        // Allow blank lines before the first subject as part of the match
        while idx < len {
            if let LineContainer::Token(line) = &tokens[idx] {
                if line.line_type == BlankLine {
                    idx += 1;
                    continue;
                }
            }
            break;
        }

        if idx >= len {
            return None;
        }

        let mut current = idx;

        loop {
            let subject_idx = match &tokens.get(current)? {
                LineContainer::Token(line)
                    if matches!(line.line_type, SubjectLine | SubjectOrListItemLine) =>
                {
                    let idx_val = current;
                    current += 1;
                    idx_val
                }
                _ => return None,
            };

            while current < len {
                if let LineContainer::Token(line) = &tokens[current] {
                    if line.line_type == BlankLine {
                        current += 1;
                        continue;
                    }
                }
                break;
            }

            let mut content_idx = None;
            if current < len && matches!(tokens[current], LineContainer::Container { .. }) {
                content_idx = Some(current);
                current += 1;
            }

            groups.push(VerbatimGroupMatch {
                subject_idx,
                content_idx,
            });

            while current < len {
                if let LineContainer::Token(line) = &tokens[current] {
                    if line.line_type == BlankLine {
                        current += 1;
                        continue;
                    }
                }
                break;
            }

            if current < len {
                if let LineContainer::Token(line) = &tokens[current] {
                    if matches!(line.line_type, AnnotationStartLine | AnnotationEndLine) {
                        let closing_idx = current;
                        return Some((
                            PatternMatch::VerbatimBlock {
                                groups,
                                closing_idx,
                            },
                            start_idx..(closing_idx + 1),
                        ));
                    }
                } else {
                    return None;
                }
            } else {
                return None;
            }

            // Continue loop expecting another subject
        }
    }
}

/// Main recursive descent parser using the declarative grammar
pub fn parse_with_declarative_grammar(
    tokens: Vec<LineContainer>,
    source: &str,
) -> Result<Vec<ParseNode>, String> {
    let mut items = Vec::new();
    let mut idx = 0;

    while idx < tokens.len() {
        if let Some((pattern, range)) = GrammarMatcher::try_match(&tokens, idx) {
            // Skip blank line groups (they're structural, not content)
            if !matches!(pattern, PatternMatch::BlankLineGroup) {
                // Convert pattern to ContentItem
                let item = convert_pattern_to_item(&tokens, &pattern, range.start, source)?;
                items.push(item);
            }
            idx = range.end;
        } else {
            idx += 1;
        }
    }

    Ok(items)
}

/// Convert a matched pattern to a ContentItem
///
/// pattern_offset: the index where the pattern starts in the tokens array
/// (used to convert relative indices in the pattern to absolute indices)
fn convert_pattern_to_item(
    tokens: &[LineContainer],
    pattern: &PatternMatch,
    pattern_offset: usize,
    source: &str,
) -> Result<ParseNode, String> {
    match pattern {
        PatternMatch::VerbatimBlock {
            groups,
            closing_idx,
        } => {
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
                            crate::lex::lexing::Token::Colon
                                | crate::lex::lexing::Token::BlankLine(_)
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

            let closing_token = extract_line_token(&tokens[*closing_idx])?;
            let (header_tokens, closing_children) =
                extract_annotation_single_content(closing_token);
            child_nodes.push(ParseNode::new(
                NodeType::VerbatimBlockkClosing,
                header_tokens,
                closing_children,
            ));

            Ok(ParseNode::new(NodeType::VerbatimBlock, vec![], child_nodes))
        }
        PatternMatch::AnnotationBlock {
            start_idx,
            content_idx,
        } => {
            let start_token = extract_line_token(&tokens[pattern_offset + start_idx])?;

            // Extract header tokens using shared helper function
            let header_tokens = extract_annotation_header_tokens(start_token);

            // Extract content from container using shared helper function
            // This ensures block forms use the same content extraction path
            // Note: content_idx is relative to pattern_offset, so we need to add it
            let children =
                extract_annotation_block_content(tokens, pattern_offset + content_idx, source)?;

            Ok(ParseNode::new(
                NodeType::Annotation,
                header_tokens,
                children,
            ))
        }
        PatternMatch::AnnotationSingle { start_idx } => {
            let start_token = extract_line_token(&tokens[pattern_offset + start_idx])?;

            // Extract header tokens and content using shared helper function
            // This ensures single-line form uses the same extraction logic as block forms
            let (header_tokens, children) = extract_annotation_single_content(start_token);

            Ok(ParseNode::new(
                NodeType::Annotation,
                header_tokens,
                children,
            ))
        }
        PatternMatch::List { items } => {
            let mut list_items = Vec::new();

            for (item_idx, content_idx) in items {
                let item_token = extract_line_token(&tokens[pattern_offset + item_idx])?;

                let children = if let Some(content_idx_val) = content_idx {
                    if let Some(LineContainer::Container { children, .. }) =
                        tokens.get(pattern_offset + content_idx_val)
                    {
                        parse_with_declarative_grammar(children.clone(), source)?
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
        PatternMatch::Definition {
            subject_idx,
            content_idx,
        } => {
            let subject_token = extract_line_token(&tokens[pattern_offset + subject_idx])?;

            let children = if let Some(LineContainer::Container { children, .. }) =
                tokens.get(pattern_offset + content_idx)
            {
                parse_with_declarative_grammar(children.clone(), source)?
            } else {
                Vec::new()
            };

            // Filter out Colon, Whitespace, and BlankLine tokens from definition subject
            // Definition subject should only contain the text before the colon
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
        PatternMatch::Session {
            subject_idx,
            content_idx,
        } => {
            let subject_token = extract_line_token(&tokens[pattern_offset + subject_idx])?;
            let mut content_children = vec![];
            if let Some(LineContainer::Container { children, .. }) =
                tokens.get(pattern_offset + content_idx)
            {
                content_children = parse_with_declarative_grammar(children.clone(), source)?;
            }

            // Filter out trailing Whitespace and BlankLine tokens from session label
            let subject_tokens: Vec<_> = subject_token
                .source_tokens
                .clone()
                .into_iter()
                .zip(subject_token.token_spans.clone())
                .filter(|(token, _)| {
                    !matches!(
                        token,
                        crate::lex::lexing::Token::Whitespace
                            | crate::lex::lexing::Token::BlankLine(_)
                    )
                })
                .collect();

            Ok(ParseNode::new(
                NodeType::Session,
                subject_tokens,
                content_children,
            ))
        }
        PatternMatch::Paragraph { start_idx, end_idx } => {
            let paragraph_tokens: Vec<LineToken> = ((pattern_offset + start_idx)
                ..=(pattern_offset + end_idx))
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
        PatternMatch::BlankLineGroup => {
            // BlankLineGroups should have been filtered out in parse_with_declarative_grammar
            // If we reach here, something went wrong
            Err("Internal error: BlankLineGroup reached convert_pattern_to_item".to_string())
        }
    }
}

/// Helper to extract a LineToken from a LineContainerToken
fn extract_line_token(token: &LineContainer) -> Result<&LineToken, String> {
    match token {
        LineContainer::Token(t) => Ok(t),
        _ => Err("Expected LineToken, found Container".to_string()),
    }
}

/// Recursively gather all LineTokens contained within a LineContainer tree.
///
/// The linebased tokenizer already encodes indentation structure via nested
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
) -> (Vec<(Token, std::ops::Range<usize>)>, Vec<ParseNode>) {
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
            // Don't include LexMarker tokens in header_tokens
            continue;
        }

        if !content_started {
            // Collect tokens between the two :: markers (excluding the markers themselves)
            header_tokens.push((token, span));
        } else {
            // Collect tokens after the second :: marker
            content_tokens.push((token, span));
        }
    }

    // If there's content after the header, create a paragraph for it
    // This ensures single-line form content goes through the same structure as block forms
    let children = if !content_tokens.is_empty() {
        vec![ParseNode::new(NodeType::Paragraph, content_tokens, vec![])]
    } else {
        vec![]
    };

    (header_tokens, children)
}

/// Extract content from an annotation block form.
/// Returns the parsed children from the container, or empty vector if no container.
fn extract_annotation_block_content(
    tokens: &[LineContainer],
    content_idx: usize,
    source: &str,
) -> Result<Vec<ParseNode>, String> {
    if let Some(LineContainer::Container { children, .. }) = tokens.get(content_idx) {
        parse_with_declarative_grammar(children.clone(), source)
    } else {
        Ok(vec![])
    }
}
