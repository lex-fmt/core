//! Declarative Grammar Engine - Regex-Based Parser for lex
//!
//! This module implements a unified parser using declarative regex grammar rules:
//! 1. Converts token sequences to grammar notation strings
//! 2. Matches against regex patterns in declaration order
//! 3. Extracts consumed token indices from regex match
//! 4. Recursively descends into containers when building AST
//! 5. No imperative pattern matching - grammar is data, not code
//!
//! The grammar patterns and AST building logic have been extracted to separate modules:
//! - `grammar.rs` - Pattern definitions and matching order
//! - `builder.rs` - AST node construction from matched patterns

use crate::lex::parsing::ir::ParseNode;
use crate::lex::token::{LineContainer, LineType};
use regex::Regex;
use std::ops::Range;

mod builder;
mod grammar;

use builder::{convert_pattern_to_node, PatternMatch};
use grammar::{GRAMMAR_PATTERNS, LIST_ITEM_REGEX};

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

        // Try verbatim block first (requires special imperative matching logic)
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

    /// Count how many tokens are represented in a grammar string.
    /// Each token type in angle brackets represents one token.
    fn count_consumed_tokens(grammar_str: &str) -> usize {
        grammar_str.matches('<').count()
    }

    /// Match verbatim blocks using imperative logic.
    ///
    /// Verbatim blocks consist of:
    /// 1. A subject line
    /// 2. Content that is either:
    ///    a) In a Container (inflow mode - content indented relative to subject)
    ///    b) Flat lines (fullwidth mode - content at fixed column, or groups)
    /// 3. A closing annotation marker (:: ... ::)
    ///
    /// This matcher handles both the original inflow case (subject + container + annotation)
    /// and the fullwidth case (subject + flat lines + annotation). To distinguish verbatim
    /// blocks from sessions followed by annotations, we require that either:
    /// - There's a Container immediately after the subject, OR
    /// - The closing annotation is at the SAME indentation as the subject
    ///
    /// Sessions have their title at the root level and content is indented. If we see
    /// a root-level annotation after a root-level subject with indented content between,
    /// that's NOT a verbatim block - it's a session followed by an annotation.
    fn match_verbatim_block(
        tokens: &[LineContainer],
        start_idx: usize,
    ) -> Option<(PatternMatch, Range<usize>)> {
        use LineType::{BlankLine, DataLine, SubjectLine, SubjectOrListItemLine};

        let len = tokens.len();
        if start_idx >= len {
            return None;
        }

        // Allow blank lines before the subject to be consumed as part of this match
        let mut idx = start_idx;
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

        // Must start with a subject line
        let first_subject_idx = match &tokens[idx] {
            LineContainer::Token(line)
                if matches!(line.line_type, SubjectLine | SubjectOrListItemLine) =>
            {
                idx
            }
            _ => return None,
        };

        let mut cursor = first_subject_idx + 1;

        // Try to match one or more subject+content pairs followed by closing annotation
        // This loop handles verbatim groups: multiple subjects sharing one closing annotation
        loop {
            // Skip blank lines
            while cursor < len {
                if let LineContainer::Token(line) = &tokens[cursor] {
                    if line.line_type == BlankLine {
                        cursor += 1;
                        continue;
                    }
                }
                break;
            }

            if cursor >= len {
                return None;
            }

            // Check what we have at cursor
            match &tokens[cursor] {
                LineContainer::Container { .. } => {
                    // Found a container - this is potentially inflow mode verbatim content
                    // But we need to verify the pattern:
                    // - Verbatim: subject + container + (annotation OR another subject+container)
                    // - Session: subject + container + (other content)
                    cursor += 1;

                    // Skip blank lines after container
                    while cursor < len {
                        if let LineContainer::Token(line) = &tokens[cursor] {
                            if line.line_type == BlankLine {
                                cursor += 1;
                                continue;
                            }
                        }
                        break;
                    }

                    // After container, check what follows
                    if cursor >= len {
                        return None; // Container at end - not a verbatim block
                    }

                    match &tokens[cursor] {
                        LineContainer::Token(line) => {
                            if matches!(line.line_type, DataLine) {
                                // Container followed by annotation - this IS verbatim!
                                // Continue loop to match it
                                continue;
                            }
                            if matches!(line.line_type, SubjectLine | SubjectOrListItemLine) {
                                // Container followed by another subject - this is a verbatim group!
                                // Continue loop to match more groups
                                continue;
                            }
                            // Container followed by something else - NOT a verbatim block
                            return None;
                        }
                        LineContainer::Container { .. } => {
                            // Container followed by another container - NOT verbatim pattern
                            return None;
                        }
                    }
                }
                LineContainer::Token(line) => {
                    if matches!(line.line_type, DataLine) {
                        // Found closing annotation - success!
                        // But only if we haven't mixed containers with flat content in a problematic way
                        return Some((
                            PatternMatch::VerbatimBlock {
                                subject_idx: first_subject_idx,
                                content_range: (first_subject_idx + 1)..cursor,
                                closing_idx: cursor,
                            },
                            start_idx..(cursor + 1),
                        ));
                    }

                    if matches!(line.line_type, SubjectLine | SubjectOrListItemLine) {
                        // Another subject - this is another group
                        cursor += 1;
                        continue;
                    }

                    // Any other flat token (paragraph line, etc.)
                    // This is fullwidth mode or group content
                    cursor += 1;
                }
            }
        }
    }
}

/// Main recursive descent parser using the declarative grammar.
///
/// This is the entry point for parsing a sequence of tokens at any level.
/// It iteratively tries to match patterns and recursively descends into containers.
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
                // Convert pattern to ParseNode
                let item = convert_pattern_to_node(
                    &tokens,
                    &pattern,
                    range.start,
                    source,
                    &|children, src| parse_with_declarative_grammar(children, src),
                )?;
                items.push(item);
            }
            idx = range.end;
        } else {
            idx += 1;
        }
    }

    Ok(items)
}
