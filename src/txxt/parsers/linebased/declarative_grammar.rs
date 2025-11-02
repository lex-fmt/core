//! Declarative Grammar Engine - Regex-Based Parser for txxt
//!
//! This module implements a unified parser using declarative regex grammar rules:
//! 1. Converts token sequences to grammar notation strings
//! 2. Matches against regex patterns in declaration order
//! 3. Extracts consumed token indices from regex match
//! 4. Recursively descends into containers when building AST
//! 5. No imperative pattern matching - grammar is data, not code
//!
//! The grammar parse order (from grammar.txxt §4.7):
//! 1. foreign-block (requires closing annotation - try first for disambiguation)
//! 2. annotation_block (block with container between start and end markers)
//! 3. annotation_single (single-line annotation only)
//! 4. list (requires preceding blank line + 2+ list items)
//! 5. definition (requires subject + immediate indent)
//! 6. session (requires subject + blank line + indent)
//! 7. paragraph (any content-line or sequence thereof)
//! 8. blank_line_group (one or more consecutive blank lines)

use super::builders;
use crate::txxt::lexers::linebased::tokens::{LineContainerToken, LineToken};
use crate::txxt::parsers::ContentItem;
use once_cell::sync::Lazy;
use regex::Regex;
use std::ops::Range;

/// Lazy-compiled regex for extracting list items from the list group capture
static LIST_ITEM_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(<list-line>|<subject-or-list-item-line>)(<container>)?").unwrap());

/// Grammar patterns as regex rules with names and patterns
/// Order matters: patterns are tried in declaration order for correct disambiguation
const GRAMMAR_PATTERNS: &[(&str, &str)] = &[
    // Foreign Block: <subject-line>|<subject-or-list-item-line><blank-line>?<container>?<annotation-end-line>
    (
        "foreign_block",
        r"^(?P<subject><subject-line>|<subject-or-list-item-line>)(?P<blank><blank-line>)?(?P<content><container>)?(?P<closing><annotation-end-line>)",
    ),
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
    // List: <blank-line><list-line><container>?<list-line><container>?{1,+}<blank-line>?
    (
        "list",
        r"^(?P<blank><blank-line>)(?P<items>((<list-line>|<subject-or-list-item-line>)(<container>)?){2,})(?P<trailing_blank><blank-line>)?",
    ),
    // Session: <content-line><blank-line><container>
    (
        "session",
        r"^(?P<subject><paragraph-line>|<subject-line>|<list-line>|<subject-or-list-item-line>)(?P<blank><blank-line>)(?P<content><container>)",
    ),
    // Definition: <subject-line>|<subject-or-list-item-line><container>
    (
        "definition",
        r"^(?P<subject><subject-line>|<subject-or-list-item-line>)(?P<content><container>)",
    ),
    // Paragraph: <content-line>+
    (
        "paragraph",
        r"^(?P<lines>(<paragraph-line>|<subject-line>|<list-line>|<subject-or-list-item-line>)+)",
    ),
    // Blank lines: <blank-line-group>
    ("blank_line_group", r"^(?P<lines>(<blank-line>)+)"),
];

/// Represents the result of pattern matching at one level
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum PatternMatch {
    /// Foreign block: subject + optional blank + optional content + closing annotation
    ForeignBlock {
        subject_idx: usize,
        blank_idx: Option<usize>,
        content_idx: Option<usize>,
        closing_idx: usize,
    },
    /// Annotation block: start + container + end
    AnnotationBlock {
        start_idx: usize,
        content_idx: usize,
        end_idx: usize,
    },
    /// Annotation single: just start line
    AnnotationSingle { start_idx: usize },
    /// List: preceding blank line + 2+ consecutive list items
    List {
        blank_idx: usize,
        items: Vec<(usize, Option<usize>)>,
    },
    /// Definition: subject + immediate indent + content
    Definition {
        subject_idx: usize,
        content_idx: usize,
    },
    /// Session: subject + blank line + indent + content
    Session {
        subject_idx: usize,
        blank_idx: usize,
        content_idx: usize,
    },
    /// Paragraph: one or more consecutive non-blank, non-special lines
    Paragraph { start_idx: usize, end_idx: usize },
    /// Blank line group: one or more consecutive blank lines
    BlankLineGroup { start_idx: usize, end_idx: usize },
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
        tokens: &[LineContainerToken],
        start_idx: usize,
    ) -> Option<(PatternMatch, Range<usize>)> {
        if start_idx >= tokens.len() {
            return None;
        }

        // Convert remaining tokens to grammar string
        let remaining_tokens = &tokens[start_idx..];
        let token_string = Self::tokens_to_grammar_string(remaining_tokens)?;

        // Try each pattern in order
        for (pattern_name, pattern_regex_str) in GRAMMAR_PATTERNS {
            if let Ok(regex) = Regex::new(pattern_regex_str) {
                if let Some(caps) = regex.captures(&token_string) {
                    let full_match = caps.get(0).unwrap();
                    let consumed_count = Self::count_consumed_tokens(full_match.as_str());

                    // Use captures to extract indices and build the pattern
                    let pattern = match *pattern_name {
                        "foreign_block" => PatternMatch::ForeignBlock {
                            subject_idx: 0,
                            blank_idx: caps.name("blank").map(|_| 1),
                            content_idx: caps
                                .name("content")
                                .map(|_| caps.name("blank").map_or(1, |_| 2)),
                            closing_idx: consumed_count - 1,
                        },
                        "annotation_block_with_end" => PatternMatch::AnnotationBlock {
                            start_idx: 0,
                            content_idx: 1,
                            end_idx: 2,
                        },
                        "annotation_block" => PatternMatch::AnnotationBlock {
                            start_idx: 0,
                            content_idx: 1,
                            end_idx: 1,
                        },
                        "annotation_single" => PatternMatch::AnnotationSingle { start_idx: 0 },
                        "list" => {
                            let items_str = caps.name("items").unwrap().as_str();
                            let mut items = Vec::new();
                            let mut token_idx = 1; // Start after the blank line
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
                            PatternMatch::List {
                                blank_idx: 0,
                                items,
                            }
                        }
                        "session" => PatternMatch::Session {
                            subject_idx: 0,
                            blank_idx: 1,
                            content_idx: 2,
                        },
                        "definition" => PatternMatch::Definition {
                            subject_idx: 0,
                            content_idx: 1,
                        },
                        "paragraph" => PatternMatch::Paragraph {
                            start_idx: 0,
                            end_idx: consumed_count - 1,
                        },
                        "blank_line_group" => PatternMatch::BlankLineGroup {
                            start_idx: 0,
                            end_idx: consumed_count - 1,
                        },
                        _ => continue,
                    };

                    return Some((pattern, start_idx..start_idx + consumed_count));
                }
            }
        }

        None
    }

    /// Convert remaining tokens to grammar notation string
    fn tokens_to_grammar_string(tokens: &[LineContainerToken]) -> Option<String> {
        let mut result = String::new();
        for token in tokens {
            match token {
                LineContainerToken::Token(t) => {
                    result.push_str(&t.line_type.to_grammar_string());
                }
                LineContainerToken::Container { .. } => {
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
}

/// Main recursive descent parser using the declarative grammar
pub fn parse_with_declarative_grammar(
    tokens: Vec<LineContainerToken>,
    source: &str,
) -> Result<Vec<ContentItem>, String> {
    let mut items = Vec::new();
    let mut idx = 0;

    while idx < tokens.len() {
        if let Some((pattern, range)) = GrammarMatcher::try_match(&tokens, idx) {
            // Skip blank line groups (they're structural, not content)
            if !matches!(pattern, PatternMatch::BlankLineGroup { .. }) {
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
    tokens: &[LineContainerToken],
    pattern: &PatternMatch,
    pattern_offset: usize,
    source: &str,
) -> Result<ContentItem, String> {
    match pattern {
        PatternMatch::ForeignBlock {
            subject_idx,
            blank_idx: _,
            content_idx,
            closing_idx,
        } => {
            let subject_token = extract_line_token(&tokens[pattern_offset + subject_idx])?;
            let closing_token = extract_line_token(&tokens[pattern_offset + closing_idx])?;

            // Extract content lines from container if present
            let content_lines = if let Some(content_idx_val) = content_idx {
                if let LineContainerToken::Container { children, .. } =
                    &tokens[pattern_offset + content_idx_val]
                {
                    children
                        .iter()
                        .filter_map(|t| extract_line_token(t).ok())
                        .collect::<Vec<_>>()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };

            builders::unwrap_foreign_block(subject_token, content_lines, closing_token, source)
        }
        PatternMatch::AnnotationBlock {
            start_idx,
            content_idx,
            end_idx: _,
        } => {
            let start_token = extract_line_token(&tokens[pattern_offset + start_idx])?;

            // Extract content from container
            if let LineContainerToken::Container { children, .. } =
                &tokens[pattern_offset + content_idx]
            {
                let content = parse_with_declarative_grammar(children.clone(), source)?;
                builders::unwrap_annotation_with_content(start_token, content, source)
            } else {
                // Fallback to single-line annotation if no container
                builders::unwrap_annotation(start_token, source)
            }
        }
        PatternMatch::AnnotationSingle { start_idx } => {
            let start_token = extract_line_token(&tokens[pattern_offset + start_idx])?;
            builders::unwrap_annotation(start_token, source)
        }
        PatternMatch::List {
            blank_idx: _,
            items,
        } => {
            let mut list_items = Vec::new();

            for (item_idx, content_idx) in items {
                let item_token = extract_line_token(&tokens[pattern_offset + item_idx])?;

                let content = if let Some(content_idx_val) = content_idx {
                    if let LineContainerToken::Container { children, .. } =
                        &tokens[pattern_offset + content_idx_val]
                    {
                        parse_with_declarative_grammar(children.clone(), source)?
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                };

                let list_item = builders::unwrap_list_item(item_token, content, source)?;
                list_items.push(list_item);
            }

            builders::unwrap_list(list_items, source)
        }
        PatternMatch::Definition {
            subject_idx,
            content_idx,
        } => {
            let subject_token = extract_line_token(&tokens[pattern_offset + subject_idx])?;

            let content = if let LineContainerToken::Container { children, .. } =
                &tokens[pattern_offset + content_idx]
            {
                parse_with_declarative_grammar(children.clone(), source)?
            } else {
                Vec::new()
            };

            builders::unwrap_definition(subject_token, content, source)
        }
        PatternMatch::Session {
            subject_idx,
            blank_idx: _,
            content_idx,
        } => {
            let subject_token = extract_line_token(&tokens[pattern_offset + subject_idx])?;

            let content = if let LineContainerToken::Container { children, .. } =
                &tokens[pattern_offset + content_idx]
            {
                parse_with_declarative_grammar(children.clone(), source)?
            } else {
                Vec::new()
            };

            builders::unwrap_session(subject_token, content, source)
        }
        PatternMatch::Paragraph { start_idx, end_idx } => {
            let paragraph_tokens: Vec<LineToken> = ((pattern_offset + start_idx)
                ..=(pattern_offset + end_idx))
                .filter_map(|idx| extract_line_token(&tokens[idx]).ok().cloned())
                .collect();

            builders::unwrap_tokens_to_paragraph(paragraph_tokens, source)
        }
        PatternMatch::BlankLineGroup { .. } => {
            // BlankLineGroups should have been filtered out in parse_with_declarative_grammar
            // If we reach here, something went wrong
            Err("Internal error: BlankLineGroup reached convert_pattern_to_item".to_string())
        }
    }
}

/// Helper to extract a LineToken from a LineContainerToken
fn extract_line_token(token: &LineContainerToken) -> Result<&LineToken, String> {
    match token {
        LineContainerToken::Token(t) => Ok(t),
        _ => Err("Expected LineToken, found Container".to_string()),
    }
}
