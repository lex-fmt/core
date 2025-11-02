//! Declarative Grammar Engine + Recursive Descent Parser for txxt
//!
//! This module implements a unified parser that:
//! 1. Pattern-matches on LineToken types and opaque LineContainerToken structures
//! 2. Follows the strict parse order from grammar.txxt
//! 3. Recursively descends into containers when needed
//! 4. No peeking inside containers - just matches on current level
//!
//! The grammar parse order (from grammar.txxt §4.7):
//! 1. foreign-block (requires closing annotation - try first for disambiguation)
//! 2. annotation (single-line annotations with ::)
//! 3. list (requires preceding blank line)
//! 4. definition (requires subject + immediate indent)
//! 5. session (requires subject + blank line + indent)
//! 6. paragraph (fallback - catches everything else)

use super::unwrapper;
use crate::txxt::lexers::linebased::tokens::{LineContainerToken, LineToken, LineTokenType};
use crate::txxt::parsers::ContentItem;
use std::ops::Range;

/// Represents the result of pattern matching at one level
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum PatternMatch {
    /// Foreign block: subject + optional blank + optional content + closing annotation
    ForeignBlock {
        #[allow(dead_code)]
        subject_idx: usize, // Index of subject line
        #[allow(dead_code)]
        blank_idx: Option<usize>, // Index of blank line if present
        #[allow(dead_code)]
        content_idx: Option<usize>, // Index of container with content if present
        #[allow(dead_code)]
        closing_idx: usize, // Index of closing annotation
    },
    /// Annotation: marker form, single-line, or block
    Annotation {
        #[allow(dead_code)]
        start_idx: usize,
        #[allow(dead_code)]
        end_idx: usize,
    },
    /// List: preceding blank line + 2+ consecutive list items
    List {
        #[allow(dead_code)]
        blank_idx: usize,
        #[allow(dead_code)]
        items: Vec<(usize, Option<usize>)>, // (item_idx, optional_content_block_idx)
    },
    /// Definition: subject + immediate indent + content
    Definition {
        #[allow(dead_code)]
        subject_idx: usize,
        #[allow(dead_code)]
        content_idx: usize,
    },
    /// Session: subject + blank line + indent + content
    Session {
        #[allow(dead_code)]
        subject_idx: usize,
        #[allow(dead_code)]
        blank_idx: usize,
        #[allow(dead_code)]
        content_idx: usize,
    },
    /// Paragraph: one or more consecutive non-blank, non-special lines
    Paragraph {
        #[allow(dead_code)]
        start_idx: usize,
        #[allow(dead_code)]
        end_idx: usize,
    },
    /// Blank line group: one or more consecutive blank lines
    BlankLineGroup {
        #[allow(dead_code)]
        start_idx: usize,
        #[allow(dead_code)]
        end_idx: usize,
    },
}

/// Pattern matcher for declarative grammar
pub struct GrammarMatcher;

impl GrammarMatcher {
    /// Try to match a pattern at the current level.
    ///
    /// Returns (matched_pattern, consumed_indices)
    fn try_match(
        tokens: &[LineContainerToken],
        start_idx: usize,
    ) -> Option<(PatternMatch, Range<usize>)> {
        if start_idx >= tokens.len() {
            return None;
        }

        // 1. Try foreign block (requires closing annotation)
        if let Some((pattern, range)) = Self::try_foreign_block(tokens, start_idx) {
            return Some((pattern, range));
        }

        // 2. Try annotation (single-line or block with ::)
        if let Some((pattern, range)) = Self::try_annotation(tokens, start_idx) {
            return Some((pattern, range));
        }

        // 3. Try list (requires preceding blank line)
        if let Some((pattern, range)) = Self::try_list(tokens, start_idx) {
            return Some((pattern, range));
        }

        // 4. Try definition (subject + immediate indent)
        if let Some((pattern, range)) = Self::try_definition(tokens, start_idx) {
            return Some((pattern, range));
        }

        // 5. Try session (subject + blank + indent)
        if let Some((pattern, range)) = Self::try_session(tokens, start_idx) {
            return Some((pattern, range));
        }

        // 6. Fallback: paragraph or blank lines
        if let Some((pattern, range)) = Self::try_blank_lines(tokens, start_idx) {
            return Some((pattern, range));
        }

        if let Some((pattern, range)) = Self::try_paragraph(tokens, start_idx) {
            return Some((pattern, range));
        }

        None
    }

    /// Try to match a foreign block
    fn try_foreign_block(
        tokens: &[LineContainerToken],
        start_idx: usize,
    ) -> Option<(PatternMatch, Range<usize>)> {
        // Pattern: subject-line [blank?] [block?] annotation-end-line
        // Must have: subject + closing annotation
        if start_idx >= tokens.len() {
            return None;
        }

        // Check if starts with subject line
        let subject_token = Self::get_line_token(&tokens[start_idx])?;
        if !matches!(
            subject_token.line_type,
            LineTokenType::SubjectLine | LineTokenType::SubjectOrListItemLine
        ) {
            return None;
        }

        let mut idx = start_idx + 1;
        let mut blank_idx = None;
        let mut content_idx = None;

        // Look for optional blank line
        if idx < tokens.len() {
            if let LineContainerToken::Token(token) = &tokens[idx] {
                if matches!(token.line_type, LineTokenType::BlankLine) {
                    blank_idx = Some(idx);
                    idx += 1;
                }
            }
        }

        // Look for optional content block (indented)
        if idx < tokens.len() {
            if let LineContainerToken::Container { .. } = &tokens[idx] {
                content_idx = Some(idx);
                idx += 1;
            }
        }

        // Must have closing annotation at current level
        if idx < tokens.len() {
            if let LineContainerToken::Token(token) = &tokens[idx] {
                if matches!(token.line_type, LineTokenType::AnnotationEndLine) {
                    return Some((
                        PatternMatch::ForeignBlock {
                            subject_idx: start_idx,
                            blank_idx,
                            content_idx,
                            closing_idx: idx,
                        },
                        start_idx..idx + 1,
                    ));
                }
            }
        }

        None
    }

    /// Try to match an annotation
    fn try_annotation(
        tokens: &[LineContainerToken],
        start_idx: usize,
    ) -> Option<(PatternMatch, Range<usize>)> {
        // Pattern: annotation-start-line [block?] [annotation-end-line?]
        let subject_token = Self::get_line_token(&tokens[start_idx])?;
        if !matches!(subject_token.line_type, LineTokenType::AnnotationStartLine) {
            return None;
        }

        let mut end_idx = start_idx;

        // Look for optional block after annotation start
        if start_idx + 1 < tokens.len() {
            if let LineContainerToken::Container { .. } = &tokens[start_idx + 1] {
                end_idx = start_idx + 1;
            }
        }

        // Look for optional closing marker
        if end_idx + 1 < tokens.len() {
            if let LineContainerToken::Token(token) = &tokens[end_idx + 1] {
                if matches!(token.line_type, LineTokenType::AnnotationEndLine) {
                    end_idx += 1;
                }
            }
        }

        Some((
            PatternMatch::Annotation { start_idx, end_idx },
            start_idx..end_idx + 1,
        ))
    }

    /// Try to match a list
    fn try_list(
        tokens: &[LineContainerToken],
        start_idx: usize,
    ) -> Option<(PatternMatch, Range<usize>)> {
        // Pattern: blank-line list-item [block?] (list-item [block?])+
        if start_idx >= tokens.len() {
            return None;
        }

        // Must start with blank line
        let blank_token = Self::get_line_token(&tokens[start_idx])?;
        if !matches!(blank_token.line_type, LineTokenType::BlankLine) {
            return None;
        }

        let mut idx = start_idx + 1;
        let mut items = Vec::new();

        // Collect 2+ list items
        loop {
            if idx >= tokens.len() {
                break;
            }

            // Try to match list item
            if let LineContainerToken::Token(token) = &tokens[idx] {
                if !matches!(
                    token.line_type,
                    LineTokenType::ListLine | LineTokenType::SubjectOrListItemLine
                ) {
                    break;
                }
            } else {
                break;
            }

            let item_idx = idx;
            idx += 1;

            // Check for optional block content after list item
            let content_idx = if idx < tokens.len() {
                if let LineContainerToken::Container { .. } = &tokens[idx] {
                    let ci = Some(idx);
                    idx += 1;
                    ci
                } else {
                    None
                }
            } else {
                None
            };

            items.push((item_idx, content_idx));

            // Must have at least 2 items
            if items.len() >= 2 {
                // Continue collecting items or stop
                if idx >= tokens.len()
                    || !matches!(&tokens[idx], LineContainerToken::Token(t) if matches!(t.line_type, LineTokenType::ListLine | LineTokenType::SubjectOrListItemLine))
                {
                    break;
                }
            }
        }

        if items.len() >= 2 {
            return Some((
                PatternMatch::List {
                    blank_idx: start_idx,
                    items,
                },
                start_idx..idx,
            ));
        }

        None
    }

    /// Try to match a definition
    fn try_definition(
        tokens: &[LineContainerToken],
        start_idx: usize,
    ) -> Option<(PatternMatch, Range<usize>)> {
        // Pattern: subject-line indent content dedent
        // NO blank line between subject and indent
        let subject_token = Self::get_line_token(&tokens[start_idx])?;
        if !matches!(subject_token.line_type, LineTokenType::SubjectLine) {
            return None;
        }

        // Must have container immediately after (no blank line)
        if start_idx + 1 >= tokens.len() {
            return None;
        }

        if let LineContainerToken::Container { .. } = &tokens[start_idx + 1] {
            return Some((
                PatternMatch::Definition {
                    subject_idx: start_idx,
                    content_idx: start_idx + 1,
                },
                start_idx..start_idx + 2,
            ));
        }

        None
    }

    /// Try to match a session
    fn try_session(
        tokens: &[LineContainerToken],
        start_idx: usize,
    ) -> Option<(PatternMatch, Range<usize>)> {
        // Pattern: <any-line> <blank-line> <indent><container>
        // Sessions can start with ANY line type (including ParagraphLine, SubjectOrListItemLine, etc.)
        // The distinguishing feature is the blank line followed by a container
        let subject_token = Self::get_line_token(&tokens[start_idx])?;

        // Sessions must NOT start with blank lines or annotations
        if matches!(
            subject_token.line_type,
            LineTokenType::BlankLine
                | LineTokenType::AnnotationStartLine
                | LineTokenType::AnnotationEndLine
        ) {
            return None;
        }

        if start_idx + 2 >= tokens.len() {
            return None;
        }

        // Must have blank line after subject
        let blank_token = Self::get_line_token(&tokens[start_idx + 1])?;
        if !matches!(blank_token.line_type, LineTokenType::BlankLine) {
            return None;
        }

        // Must have container after blank line
        if let LineContainerToken::Container { .. } = &tokens[start_idx + 2] {
            return Some((
                PatternMatch::Session {
                    subject_idx: start_idx,
                    blank_idx: start_idx + 1,
                    content_idx: start_idx + 2,
                },
                start_idx..start_idx + 3,
            ));
        }

        None
    }

    /// Try to match blank lines
    fn try_blank_lines(
        tokens: &[LineContainerToken],
        start_idx: usize,
    ) -> Option<(PatternMatch, Range<usize>)> {
        let token = Self::get_line_token(&tokens[start_idx])?;
        if !matches!(token.line_type, LineTokenType::BlankLine) {
            return None;
        }

        let mut idx = start_idx;
        while idx < tokens.len() {
            if let LineContainerToken::Token(t) = &tokens[idx] {
                if matches!(t.line_type, LineTokenType::BlankLine) {
                    idx += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        Some((
            PatternMatch::BlankLineGroup {
                start_idx,
                end_idx: idx - 1,
            },
            start_idx..idx,
        ))
    }

    /// Try to match a paragraph
    fn try_paragraph(
        tokens: &[LineContainerToken],
        start_idx: usize,
    ) -> Option<(PatternMatch, Range<usize>)> {
        let token = Self::get_line_token(&tokens[start_idx])?;

        // Paragraph is any non-blank, non-special line
        if matches!(
            token.line_type,
            LineTokenType::BlankLine | LineTokenType::IndentLevel | LineTokenType::DedentLevel
        ) {
            return None;
        }

        let mut idx = start_idx;
        while idx < tokens.len() {
            match &tokens[idx] {
                LineContainerToken::Token(t) => {
                    if matches!(
                        t.line_type,
                        LineTokenType::BlankLine
                            | LineTokenType::IndentLevel
                            | LineTokenType::DedentLevel
                    ) {
                        break;
                    }
                    idx += 1;
                }
                LineContainerToken::Container { .. } => {
                    break;
                }
            }
        }

        if idx > start_idx {
            return Some((
                PatternMatch::Paragraph {
                    start_idx,
                    end_idx: idx - 1,
                },
                start_idx..idx,
            ));
        }

        None
    }

    /// Helper to extract LineToken from LineContainerToken
    fn get_line_token(token: &LineContainerToken) -> Option<&LineToken> {
        match token {
            LineContainerToken::Token(t) => Some(t),
            _ => None,
        }
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
                let item = convert_pattern_to_item(&tokens, &pattern, source)?;
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
fn convert_pattern_to_item(
    tokens: &[LineContainerToken],
    pattern: &PatternMatch,
    source: &str,
) -> Result<ContentItem, String> {
    match pattern {
        PatternMatch::ForeignBlock {
            subject_idx,
            blank_idx: _,
            content_idx,
            closing_idx,
        } => {
            let subject_token = extract_line_token(&tokens[*subject_idx])?;
            let closing_token = extract_line_token(&tokens[*closing_idx])?;

            // Extract content lines from container if present
            let content_lines = if let Some(content_idx_val) = content_idx {
                if let LineContainerToken::Container { children, .. } = &tokens[*content_idx_val] {
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
        PatternMatch::Annotation { start_idx, end_idx } => {
            let start_token = extract_line_token(&tokens[*start_idx])?;

            // Check if there's content between start and end markers
            if *end_idx > *start_idx {
                // Has block content
                if let LineContainerToken::Container { children, .. } = &tokens[*start_idx + 1] {
                    let content = parse_with_declarative_grammar(children.clone(), source)?;
                    unwrapper::unwrap_annotation_with_content(start_token, content, source)
                } else {
                    // No container, just single-line annotation
                    unwrapper::unwrap_annotation(start_token, source)
                }
            } else {
                // Single-line annotation only
                unwrapper::unwrap_annotation(start_token, source)
            }
        }
        PatternMatch::List {
            blank_idx: _,
            items,
        } => {
            let mut list_items = Vec::new();

            for (item_idx, content_idx) in items {
                let item_token = extract_line_token(&tokens[*item_idx])?;

                let content = if let Some(content_idx_val) = content_idx {
                    if let LineContainerToken::Container { children, .. } =
                        &tokens[*content_idx_val]
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
            let subject_token = extract_line_token(&tokens[*subject_idx])?;

            let content =
                if let LineContainerToken::Container { children, .. } = &tokens[*content_idx] {
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
            let subject_token = extract_line_token(&tokens[*subject_idx])?;

            let content =
                if let LineContainerToken::Container { children, .. } = &tokens[*content_idx] {
                    parse_with_declarative_grammar(children.clone(), source)?
                } else {
                    Vec::new()
                };

            unwrapper::unwrap_session(subject_token, content, source)
        }
        PatternMatch::Paragraph { start_idx, end_idx } => {
            let paragraph_tokens: Vec<LineToken> = ((*start_idx)..=(*end_idx))
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
