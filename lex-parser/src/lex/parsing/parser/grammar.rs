//! Grammar Pattern Definitions
//!
//! This module defines the declarative grammar patterns used by the parser.
//! Patterns are defined as regex rules and are tried in declaration order
//! for correct disambiguation according to the grammar specification.
//!
//! # Grammar Parse Order (from grammar.lex §4.7)
//!
//! 1. verbatim-block - requires closing annotation, tried first for disambiguation
//! 2. annotation_block - block with container between start and end markers
//! 3. annotation_single - single-line annotation only
//! 4. list_no_blank - 2+ list items without preceding blank (inside containers)
//! 5. list - requires preceding blank line + 2+ list items (at root level)
//! 6. definition - requires subject + immediate indent
//! 7. session_no_blank - requires subject + blank + indent (at container start or after separator)
//! 8. paragraph - any content-line or sequence thereof
//! 9. blank_line_group - one or more consecutive blank lines

use once_cell::sync::Lazy;
use regex::Regex;

/// Lazy-compiled regex for extracting list items from the list group capture.
///
/// This regex identifies individual list items and their optional nested containers
/// within the matched list pattern.
pub(super) static LIST_ITEM_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(<list-line>|<subject-or-list-item-line>)(<container>)?").unwrap());

/// Grammar patterns as regex rules with names and patterns.
///
/// Order matters: patterns are tried in declaration order for correct disambiguation.
/// Each pattern is a tuple of (pattern_name, regex_pattern_string).
///
/// # Pattern Structure
///
/// - Named capture groups (e.g., `(?P<start>...)`) allow extracting specific parts
/// - Token types in angle brackets (e.g., `<annotation-start-line>`) match grammar symbols
/// - `<container>` represents a nested indented block
/// - Quantifiers like `+` (one or more) and `{2,}` (two or more) enforce grammar rules
pub(super) const GRAMMAR_PATTERNS: &[(&str, &str)] = &[
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
    // Definition: <subject-line>|<subject-or-list-item-line>|<paragraph-line><container>
    (
        "definition",
        r"^(?P<subject><subject-line>|<subject-or-list-item-line>|<paragraph-line>)(?P<content><container>)",
    ),
    // Session without preceding blank line (for sessions at container start)
    (
        "session_no_blank",
        r"^(?P<subject><paragraph-line>|<subject-line>|<list-line>|<subject-or-list-item-line>)(?P<blank><blank-line>+)(?P<content><container>)",
    ),
    // Paragraph: <content-line>+
    (
        "paragraph",
        r"^(?P<lines>(<paragraph-line>|<subject-line>|<list-line>|<subject-or-list-item-line>|<dialog-line>)+)",
    ),
    // Blank lines: <blank-line-group>
    ("blank_line_group", r"^(?P<lines>(<blank-line>)+)"),
];
