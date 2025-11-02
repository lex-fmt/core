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

use super::unwrapper;
use crate::txxt::lexers::linebased::tokens::{LineContainerToken, LineToken, LineTokenType};
use crate::txxt::parsers::ContentItem;
use regex::Regex;
use std::ops::Range;

/// Grammar patterns as regex rules with names and patterns
/// Order matters: patterns are tried in declaration order for correct disambiguation
const GRAMMAR_PATTERNS: &[(&str, &str)] = &[
    // Foreign Block: <subject-line>|<subject-or-list-item-line><blank-line>?<container>?<annotation-end-line>
    (
        "foreign_block",
        r"^(<subject-line>|<subject-or-list-item-line>)(<blank-line>)?(<container>)?(<annotation-end-line>)",
    ),
    // Annotation (multi-line with markers): <annotation-start-line><container><annotation-end-line>
    (
        "annotation_block_with_end",
        r"^(<annotation-start-line>)(<container>)(<annotation-end-line>)",
    ),
    // Annotation (multi-line without end marker): <annotation-start-line><container>
    (
        "annotation_block",
        r"^(<annotation-start-line>)(<container>)",
    ),
    // Annotation (single-line): <annotation-start-line><content>
    // NOTE: <content> is implicit (the rest of the line), doesn't appear in token sequence
    ("annotation_single", r"^(<annotation-start-line>)"),
    // List: <blank-line><list-item-line><container>?<list-item-line><container>?{1,+}<blank-line>?
    // NOTE: Simplified to: blank + at least 2 list items (with optional containers)
    (
        "list",
        r"^(<blank-line>)((<list-item-line>)(<container>)?){2,}(<blank-line>)?",
    ),
    // Session: <content-line><blank-line><container>
    // content-line = paragraph-line | subject-line | list-item-line | subject-or-list-item-line
    // NOTE: Must come before definition to take precedence (both have subject + container, but session has blank in between)
    (
        "session",
        r"^(<paragraph-line>|<subject-line>|<list-item-line>|<subject-or-list-item-line>)(<blank-line>)(<container>)",
    ),
    // Definition: <subject-line>|<subject-or-list-item-line><container>
    // NOTE: No blank line between subject and container
    (
        "definition",
        r"^(<subject-line>|<subject-or-list-item-line>)(<container>)",
    ),
    // Paragraph: <content-line>+
    // content-line = paragraph-line | subject-line | list-item-line | subject-or-list-item-line
    (
        "paragraph",
        r"^(<paragraph-line>|<subject-line>|<list-item-line>|<subject-or-list-item-line>)+",
    ),
    // Blank lines: <blank-line-group>
    ("blank_line_group", r"^(<blank-line>)+"),
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
                if let Some(mat) = regex.find(&token_string) {
                    // Match found - now determine how many tokens were consumed
                    let consumed_count = Self::count_consumed_tokens(&token_string[..mat.end()]);

                    // Extract pattern details based on pattern name
                    let pattern = match *pattern_name {
                        "foreign_block" => {
                            Self::parse_foreign_block(remaining_tokens, consumed_count)?
                        }
                        "annotation_block_with_end" => {
                            Self::parse_annotation_block_with_end(remaining_tokens, consumed_count)?
                        }
                        "annotation_block" => {
                            Self::parse_annotation_block(remaining_tokens, consumed_count)?
                        }
                        "annotation_single" => {
                            Self::parse_annotation_single(remaining_tokens, consumed_count)?
                        }
                        "list" => Self::parse_list(remaining_tokens, consumed_count)?,
                        "definition" => Self::parse_definition(remaining_tokens, consumed_count)?,
                        "session" => Self::parse_session(remaining_tokens, consumed_count)?,
                        "paragraph" => Self::parse_paragraph(remaining_tokens, consumed_count)?,
                        "blank_line_group" => {
                            Self::parse_blank_line_group(remaining_tokens, consumed_count)?
                        }
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

    /// Parse foreign block: extract indices from consumed tokens
    fn parse_foreign_block(
        tokens: &[LineContainerToken],
        consumed_count: usize,
    ) -> Option<PatternMatch> {
        if !(2..=4).contains(&consumed_count) {
            return None;
        }

        let subject_idx = 0;
        let mut blank_idx = None;
        let mut content_idx = None;
        let mut closing_idx = subject_idx + 1;

        let mut current_idx = 1;

        // Check for optional blank line
        if current_idx < consumed_count {
            if let LineContainerToken::Token(t) = &tokens[current_idx] {
                if matches!(t.line_type, LineTokenType::BlankLine) {
                    blank_idx = Some(current_idx);
                    current_idx += 1;
                }
            }
        }

        // Check for optional container
        if current_idx < consumed_count {
            if let LineContainerToken::Container { .. } = &tokens[current_idx] {
                content_idx = Some(current_idx);
                current_idx += 1;
            }
        }

        // Must have closing annotation
        if current_idx < consumed_count {
            closing_idx = current_idx;
        }

        Some(PatternMatch::ForeignBlock {
            subject_idx,
            blank_idx,
            content_idx,
            closing_idx,
        })
    }

    /// Parse annotation block with end marker: extract indices from consumed tokens
    fn parse_annotation_block_with_end(
        _tokens: &[LineContainerToken],
        consumed_count: usize,
    ) -> Option<PatternMatch> {
        if consumed_count != 3 {
            return None;
        }

        Some(PatternMatch::AnnotationBlock {
            start_idx: 0,
            content_idx: 1,
            end_idx: 2,
        })
    }

    /// Parse annotation block without end marker: extract indices from consumed tokens
    fn parse_annotation_block(
        _tokens: &[LineContainerToken],
        consumed_count: usize,
    ) -> Option<PatternMatch> {
        if consumed_count != 2 {
            return None;
        }

        Some(PatternMatch::AnnotationBlock {
            start_idx: 0,
            content_idx: 1,
            end_idx: 1, // No separate end marker
        })
    }

    /// Parse annotation single: extract indices from consumed tokens
    fn parse_annotation_single(
        _tokens: &[LineContainerToken],
        consumed_count: usize,
    ) -> Option<PatternMatch> {
        if consumed_count != 1 {
            return None;
        }

        Some(PatternMatch::AnnotationSingle { start_idx: 0 })
    }

    /// Parse list: extract list items by scanning grammar string for item positions
    fn parse_list(tokens: &[LineContainerToken], consumed_count: usize) -> Option<PatternMatch> {
        if consumed_count < 3 {
            return None;
        }

        // Use regex to extract list item positions from grammar string
        let grammar_str = Self::tokens_to_grammar_string(&tokens[..consumed_count])?;

        // Find all <list-item-line> and <subject-or-list-item-line> positions in grammar string
        // Then map them to token indices
        let list_item_pattern =
            Regex::new(r"(<list-item-line>|<subject-or-list-item-line>)(<container>)?").ok()?;

        let mut items = Vec::new();
        let mut token_idx = 1; // Skip blank-line at index 0

        for mat in list_item_pattern.find_iter(&grammar_str) {
            let matched_text = mat.as_str();

            // This match represents one list item (+ optional container)
            let item_idx = token_idx;
            token_idx += 1; // Consumed the list item itself

            // Check if match includes a container
            let content_idx = if matched_text.contains("<container>") {
                let ci = Some(token_idx);
                token_idx += 1; // Consumed the container
                ci
            } else {
                None
            };

            items.push((item_idx, content_idx));
        }

        if items.len() >= 2 {
            Some(PatternMatch::List {
                blank_idx: 0,
                items,
            })
        } else {
            None
        }
    }

    /// Parse definition: extract indices from consumed tokens
    fn parse_definition(
        _tokens: &[LineContainerToken],
        consumed_count: usize,
    ) -> Option<PatternMatch> {
        if consumed_count != 2 {
            return None;
        }

        Some(PatternMatch::Definition {
            subject_idx: 0,
            content_idx: 1,
        })
    }

    /// Parse session: extract indices from consumed tokens
    fn parse_session(
        _tokens: &[LineContainerToken],
        consumed_count: usize,
    ) -> Option<PatternMatch> {
        if consumed_count != 3 {
            return None;
        }

        Some(PatternMatch::Session {
            subject_idx: 0,
            blank_idx: 1,
            content_idx: 2,
        })
    }

    /// Parse paragraph: extract indices from consumed tokens
    fn parse_paragraph(
        _tokens: &[LineContainerToken],
        consumed_count: usize,
    ) -> Option<PatternMatch> {
        if consumed_count < 1 {
            return None;
        }

        Some(PatternMatch::Paragraph {
            start_idx: 0,
            end_idx: consumed_count - 1,
        })
    }

    /// Parse blank line group: extract indices from consumed tokens
    fn parse_blank_line_group(
        _tokens: &[LineContainerToken],
        consumed_count: usize,
    ) -> Option<PatternMatch> {
        if consumed_count < 1 {
            return None;
        }

        Some(PatternMatch::BlankLineGroup {
            start_idx: 0,
            end_idx: consumed_count - 1,
        })
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

            unwrapper::unwrap_foreign_block(subject_token, content_lines, closing_token, source)
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
                unwrapper::unwrap_annotation_with_content(start_token, content, source)
            } else {
                // Fallback to single-line annotation if no container
                unwrapper::unwrap_annotation(start_token, source)
            }
        }
        PatternMatch::AnnotationSingle { start_idx } => {
            let start_token = extract_line_token(&tokens[pattern_offset + start_idx])?;
            unwrapper::unwrap_annotation(start_token, source)
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

                let list_item = unwrapper::unwrap_list_item(item_token, content, source)?;
                list_items.push(list_item);
            }

            unwrapper::unwrap_list(list_items, source)
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

            unwrapper::unwrap_definition(subject_token, content, source)
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

            unwrapper::unwrap_session(subject_token, content, source)
        }
        PatternMatch::Paragraph { start_idx, end_idx } => {
            let paragraph_tokens: Vec<LineToken> = ((pattern_offset + start_idx)
                ..=(pattern_offset + end_idx))
                .filter_map(|idx| extract_line_token(&tokens[idx]).ok().cloned())
                .collect();

            unwrapper::unwrap_tokens_to_paragraph(paragraph_tokens, source)
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
