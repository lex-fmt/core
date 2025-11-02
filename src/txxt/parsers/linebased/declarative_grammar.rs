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
use crate::txxt::lexers::linebased::tokens::{LineContainerToken, LineToken};
use crate::txxt::parsers::ContentItem;
use regex::Regex;
use std::ops::Range;

/// Grammar patterns as regex rules with names and patterns
/// Order matters: patterns are tried in declaration order for correct disambiguation
/// Uses named capture groups to extract token positions directly from regex matches
const GRAMMAR_PATTERNS: &[(&str, &str)] = &[
    // Foreign Block: <subject-line>|<subject-or-list-item-line><blank-line>?<container>?<annotation-end-line>
    (
        "foreign_block",
        r"^(?P<subject><subject-line>|<subject-or-list-item-line>)(?P<blank><blank-line>)?(?P<container><container>)?(?P<closing><annotation-end-line>)",
    ),
    // Annotation (multi-line with markers): <annotation-start-line><container><annotation-end-line>
    (
        "annotation_block_with_end",
        r"^(?P<start><annotation-start-line>)(?P<container><container>)(?P<end><annotation-end-line>)",
    ),
    // Annotation (multi-line without end marker): <annotation-start-line><container>
    (
        "annotation_block",
        r"^(?P<start><annotation-start-line>)(?P<container><container>)",
    ),
    // Annotation (single-line): <annotation-start-line><content>
    // NOTE: <content> is implicit (the rest of the line), doesn't appear in token sequence
    ("annotation_single", r"^(?P<start><annotation-start-line>)"),
    // List: <blank-line><list-item-line><container>?<list-item-line><container>?{1,+}<blank-line>?
    // NOTE: We match the pattern and extract items via second regex pass on matched text
    // The pattern captures blank and optional trailing_blank, items are extracted from substring
    (
        "list",
        r"^(?P<blank><blank-line>)((<list-item-line>|<subject-or-list-item-line>)(<container>)?){2,}(?P<trailing_blank><blank-line>)?",
    ),
    // Session: <content-line><blank-line><container>
    // content-line = paragraph-line | subject-line | list-item-line | subject-or-list-item-line
    // NOTE: Must come before definition to take precedence (both have subject + container, but session has blank in between)
    (
        "session",
        r"^(?P<subject><paragraph-line>|<subject-line>|<list-item-line>|<subject-or-list-item-line>)(?P<blank><blank-line>)(?P<container><container>)",
    ),
    // Definition: <subject-line>|<subject-or-list-item-line><container>
    // NOTE: No blank line between subject and container
    (
        "definition",
        r"^(?P<subject><subject-line>|<subject-or-list-item-line>)(?P<container><container>)",
    ),
    // Paragraph: <content-line>+
    // content-line = paragraph-line | subject-line | list-item-line | subject-or-list-item-line
    (
        "paragraph",
        r"^(?P<lines>(<paragraph-line>|<subject-line>|<list-item-line>|<subject-or-list-item-line>)+)",
    ),
    // Blank lines: <blank-line-group>
    ("blank_line_group", r"^(?P<blanks>(<blank-line>)+)"),
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
                if let Some(captures) = regex.captures(&token_string) {
                    // Ensure match is at position 0 (all patterns start with ^)
                    let full_match = captures.get(0)?;
                    if full_match.start() != 0 {
                        continue;
                    }
                    
                    // Match found - extract pattern details from capture groups
                    let consumed_count = Self::count_consumed_tokens(&token_string[..full_match.end()]);

                    // Extract pattern details based on pattern name using capture groups
                    let pattern = match *pattern_name {
                        "foreign_block" => {
                            Self::parse_foreign_block_from_captures(&captures, &token_string)?
                        }
                        "annotation_block_with_end" => {
                            Self::parse_annotation_block_with_end_from_captures(&captures, &token_string)?
                        }
                        "annotation_block" => {
                            Self::parse_annotation_block_from_captures(&captures, &token_string)?
                        }
                        "annotation_single" => {
                            Self::parse_annotation_single_from_captures(&captures, &token_string)?
                        }
                        "list" => Self::parse_list_from_captures(&captures, &token_string, remaining_tokens)?,
                        "definition" => {
                            Self::parse_definition_from_captures(&captures, &token_string)?
                        }
                        "session" => Self::parse_session_from_captures(&captures, &token_string)?,
                        "paragraph" => Self::parse_paragraph_from_captures(&captures, &token_string)?,
                        "blank_line_group" => {
                            Self::parse_blank_line_group_from_captures(&captures, &token_string)?
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

    /// Map a position in the grammar string to the corresponding token index
    /// Counts tokens (angle brackets) up to the given position
    fn position_to_token_idx(grammar_str: &str, position: usize) -> usize {
        grammar_str[..position].matches('<').count()
    }

    /// Extract token index from a named capture group
    fn capture_group_to_token_idx(
        captures: &regex::Captures,
        grammar_str: &str,
        group_name: &str,
    ) -> Option<usize> {
        captures.name(group_name).map(|m| {
            Self::position_to_token_idx(grammar_str, m.start())
        })
    }

    /// Parse foreign block: extract indices from capture groups
    fn parse_foreign_block_from_captures(
        captures: &regex::Captures,
        grammar_str: &str,
    ) -> Option<PatternMatch> {
        let subject_idx = Self::capture_group_to_token_idx(captures, grammar_str, "subject")?;
        let blank_idx = Self::capture_group_to_token_idx(captures, grammar_str, "blank");
        let content_idx = Self::capture_group_to_token_idx(captures, grammar_str, "container");
        let closing_idx = Self::capture_group_to_token_idx(captures, grammar_str, "closing")?;

        Some(PatternMatch::ForeignBlock {
            subject_idx,
            blank_idx,
            content_idx,
            closing_idx,
        })
    }

    /// Parse annotation block with end marker: extract indices from capture groups
    fn parse_annotation_block_with_end_from_captures(
        captures: &regex::Captures,
        grammar_str: &str,
    ) -> Option<PatternMatch> {
        let start_idx = Self::capture_group_to_token_idx(captures, grammar_str, "start")?;
        let content_idx = Self::capture_group_to_token_idx(captures, grammar_str, "container")?;
        let end_idx = Self::capture_group_to_token_idx(captures, grammar_str, "end")?;

        Some(PatternMatch::AnnotationBlock {
            start_idx,
            content_idx,
            end_idx,
        })
    }

    /// Parse annotation block without end marker: extract indices from capture groups
    fn parse_annotation_block_from_captures(
        captures: &regex::Captures,
        grammar_str: &str,
    ) -> Option<PatternMatch> {
        let start_idx = Self::capture_group_to_token_idx(captures, grammar_str, "start")?;
        let content_idx = Self::capture_group_to_token_idx(captures, grammar_str, "container")?;

        Some(PatternMatch::AnnotationBlock {
            start_idx,
            content_idx,
            end_idx: content_idx, // No separate end marker
        })
    }

    /// Parse annotation single: extract indices from capture groups
    fn parse_annotation_single_from_captures(
        captures: &regex::Captures,
        grammar_str: &str,
    ) -> Option<PatternMatch> {
        let start_idx = Self::capture_group_to_token_idx(captures, grammar_str, "start")?;

        Some(PatternMatch::AnnotationSingle { start_idx })
    }

    /// Parse list: extract list items using a second regex pass on the matched items portion
    fn parse_list_from_captures(
        captures: &regex::Captures,
        grammar_str: &str,
        _tokens: &[LineContainerToken],
    ) -> Option<PatternMatch> {
        // Extract the blank line index
        let blank_idx = Self::capture_group_to_token_idx(captures, grammar_str, "blank")?;

        // Get the full match and extract the items portion (between blank and optional trailing_blank)
        let full_match = captures.get(0)?;
        let items_start = captures.name("blank")?.end();
        let items_end = if let Some(tb) = captures.name("trailing_blank") {
            tb.start()
        } else {
            full_match.end()
        };
        let items_grammar_str = &grammar_str[items_start..items_end];

        // Use a second regex to extract individual list items from the items group
        // Each item is: <list-item-line>|<subject-or-list-item-line> followed by optional <container>
        let item_pattern = Regex::new(
            r"(<list-item-line>|<subject-or-list-item-line>)(<container>)?"
        ).ok()?;

        // Calculate the starting token index for the items portion
        let items_start_token_idx = Self::position_to_token_idx(grammar_str, items_start);

        let mut items = Vec::new();

        // Iterate through all item matches in the items portion
        for mat in item_pattern.find_iter(items_grammar_str) {
            // Calculate the token index: items_start_token_idx + tokens before this match in items_grammar_str
            let item_token_idx = items_start_token_idx
                + Self::position_to_token_idx(items_grammar_str, mat.start());

            let matched_text = mat.as_str();
            let content_idx = if matched_text.contains("<container>") {
                Some(item_token_idx + 1)
            } else {
                None
            };

            items.push((item_token_idx, content_idx));
        }

        if items.len() >= 2 {
            Some(PatternMatch::List {
                blank_idx,
                items,
            })
        } else {
            None
        }
    }

    /// Parse definition: extract indices from capture groups
    fn parse_definition_from_captures(
        captures: &regex::Captures,
        grammar_str: &str,
    ) -> Option<PatternMatch> {
        let subject_idx = Self::capture_group_to_token_idx(captures, grammar_str, "subject")?;
        let content_idx = Self::capture_group_to_token_idx(captures, grammar_str, "container")?;

        Some(PatternMatch::Definition {
            subject_idx,
            content_idx,
        })
    }

    /// Parse session: extract indices from capture groups
    fn parse_session_from_captures(
        captures: &regex::Captures,
        grammar_str: &str,
    ) -> Option<PatternMatch> {
        let subject_idx = Self::capture_group_to_token_idx(captures, grammar_str, "subject")?;
        let blank_idx = Self::capture_group_to_token_idx(captures, grammar_str, "blank")?;
        let content_idx = Self::capture_group_to_token_idx(captures, grammar_str, "container")?;

        Some(PatternMatch::Session {
            subject_idx,
            blank_idx,
            content_idx,
        })
    }

    /// Parse paragraph: extract indices from capture groups
    fn parse_paragraph_from_captures(
        captures: &regex::Captures,
        grammar_str: &str,
    ) -> Option<PatternMatch> {
        // The "lines" capture group contains all the paragraph lines
        let lines_match = captures.name("lines")?;
        let start_idx = Self::position_to_token_idx(grammar_str, lines_match.start());
        let end_idx = Self::position_to_token_idx(grammar_str, lines_match.end()) - 1;

        Some(PatternMatch::Paragraph {
            start_idx,
            end_idx,
        })
    }

    /// Parse blank line group: extract indices from capture groups
    fn parse_blank_line_group_from_captures(
        captures: &regex::Captures,
        grammar_str: &str,
    ) -> Option<PatternMatch> {
        // The "blanks" capture group contains all the blank lines
        let blanks_match = captures.name("blanks")?;
        let start_idx = Self::position_to_token_idx(grammar_str, blanks_match.start());
        let end_idx = Self::position_to_token_idx(grammar_str, blanks_match.end()) - 1;

        Some(PatternMatch::BlankLineGroup {
            start_idx,
            end_idx,
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
