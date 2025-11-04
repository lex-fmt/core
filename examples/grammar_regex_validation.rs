//! Grammar Rules as Regex Patterns - Validation Test
//!
//! This file translates the lex grammar rules (from docs/specs/v1/grammar.lex)
//! into regex patterns and validates them against sample token sequences.
//!
//! No parser engine, no AST construction - just grammar validation.
//!
//! Run with: cargo run --example grammar_regex_validation

use regex::Regex;

// ============================================================================
// GRAMMAR RULE DEFINITIONS (as Regex Patterns)
// ============================================================================
//
// IMPORTANT: Line type definitions from grammar.lex:
// - content-line: any-line that is NOT (annotation-start-line OR annotation-end-line)
// - any-line: any non-blank line
// - This means: content-line = paragraph-line | subject-line | list-item-line | subject-or-list-item-line

const GRAMMAR_RULES: &[(&str, &str)] = &[
    // Foreign Block: <subject-line>|<subject-or-list-item-line><blank-line>?<container>?<annotation-end-line>
    (
        "foreign_block",
        r"^(<subject-line>|<subject-or-list-item-line>)(<blank-line>)?(<container>)?(<annotation-end-line>)$",
    ),
    // Annotation (multi-line): <annotation-start-line><container><annotation-end-line>
    (
        "annotation_block",
        r"^(<annotation-start-line>)(<container>)(<annotation-end-line>)$",
    ),
    // Annotation (single-line): <annotation-start-line><content>
    // NOTE: <content> is implicit (the rest of the line), doesn't appear in token sequence
    ("annotation_single", r"^(<annotation-start-line>)$"),
    // List: <blank-line><list-item-line><container>?<list-item-line><container>?{1,+}<blank-line>?
    // NOTE: Simplified to: blank + at least 2 list items (with optional containers)
    (
        "list",
        r"^(<blank-line>)((<list-item-line>)(<container>)?){2,}(<blank-line>)?$",
    ),
    // Definition: <subject-line>|<subject-or-list-item-line><container>
    // NOTE: No blank line between subject and container
    (
        "definition",
        r"^(<subject-line>|<subject-or-list-item-line>)(<container>)$",
    ),
    // Session: <blank-line><content-line><blank-line><container>
    // content-line = paragraph-line | subject-line | list-item-line | subject-or-list-item-line
    (
        "session",
        r"^(<blank-line>)(<paragraph-line>|<subject-line>|<list-item-line>|<subject-or-list-item-line>)(<blank-line>)(<container>)$",
    ),
    // Paragraph: <content-line>+
    // content-line = paragraph-line | subject-line | list-item-line | subject-or-list-item-line
    (
        "paragraph",
        r"^(<paragraph-line>|<subject-line>|<list-item-line>|<subject-or-list-item-line>)+$",
    ),
    // Blank lines: <blank-line-group>
    ("blank_line_group", r"^(<blank-line>)+$"),
];

// ============================================================================
// TEST CASES: Sample Token Sequences
// ============================================================================

struct TestCase {
    name: &'static str,
    tokens: &'static str,
    expected_match: Option<&'static str>, // None = no match, Some = rule name
}

const TEST_CASES: &[TestCase] = &[
    // DEFINITION TESTS
    TestCase {
        name: "definition: subject + container",
        tokens: "<subject-line><container>",
        expected_match: Some("definition"),
    },
    TestCase {
        name: "definition: subject-or-list-item + container",
        tokens: "<subject-or-list-item-line><container>",
        expected_match: Some("definition"),
    },
    // SESSION TESTS
    // NOTE: <content-line> is not a token itself, it's a family (paragraph-line, subject-line, etc)
    TestCase {
        name: "session: blank + subject-line + blank + container",
        tokens: "<blank-line><subject-line><blank-line><container>",
        expected_match: Some("session"),
    },
    TestCase {
        name: "session: blank + paragraph-line + blank + container",
        tokens: "<blank-line><paragraph-line><blank-line><container>",
        expected_match: Some("session"),
    },
    TestCase {
        name: "NOT session: missing blank before content",
        tokens: "<paragraph-line><blank-line><container>",
        expected_match: None,
    },
    TestCase {
        name: "NOT session: missing blank between content and container",
        tokens: "<blank-line><paragraph-line><container>",
        expected_match: None,
    },
    // FOREIGN BLOCK TESTS
    TestCase {
        name: "foreign_block: subject + annotation-end",
        tokens: "<subject-line><annotation-end-line>",
        expected_match: Some("foreign_block"),
    },
    TestCase {
        name: "foreign_block: subject + blank + annotation-end",
        tokens: "<subject-line><blank-line><annotation-end-line>",
        expected_match: Some("foreign_block"),
    },
    TestCase {
        name: "foreign_block: subject + container + annotation-end",
        tokens: "<subject-line><container><annotation-end-line>",
        expected_match: Some("foreign_block"),
    },
    TestCase {
        name: "foreign_block: subject + blank + container + annotation-end",
        tokens: "<subject-line><blank-line><container><annotation-end-line>",
        expected_match: Some("foreign_block"),
    },
    TestCase {
        name: "foreign_block: subject-or-list-item + annotation-end",
        tokens: "<subject-or-list-item-line><annotation-end-line>",
        expected_match: Some("foreign_block"),
    },
    // LIST TESTS
    TestCase {
        name: "list: blank + 2 list items",
        tokens: "<blank-line><list-item-line><list-item-line>",
        expected_match: Some("list"),
    },
    TestCase {
        name: "list: blank + 2 list items with containers",
        tokens: "<blank-line><list-item-line><container><list-item-line><container>",
        expected_match: Some("list"),
    },
    TestCase {
        name: "list: blank + 3 list items mixed containers",
        tokens: "<blank-line><list-item-line><list-item-line><container><list-item-line>",
        expected_match: Some("list"),
    },
    TestCase {
        name: "list: blank + 2 list items + trailing blank",
        tokens: "<blank-line><list-item-line><list-item-line><blank-line>",
        expected_match: Some("list"),
    },
    TestCase {
        name: "NOT list: only 1 list item",
        tokens: "<blank-line><list-item-line>",
        expected_match: None,
    },
    TestCase {
        name: "NOT list: missing leading blank (matches paragraph instead)",
        tokens: "<list-item-line><list-item-line>",
        expected_match: Some("paragraph"),
    },
    // ANNOTATION TESTS
    TestCase {
        name: "annotation_block: start + container + end",
        tokens: "<annotation-start-line><container><annotation-end-line>",
        expected_match: Some("annotation_block"),
    },
    TestCase {
        name: "annotation_single: just start line",
        tokens: "<annotation-start-line>",
        expected_match: Some("annotation_single"),
    },
    TestCase {
        name: "NOT annotation: start without end",
        tokens: "<annotation-start-line><container>",
        expected_match: None,
    },
    // PARAGRAPH TESTS
    TestCase {
        name: "paragraph: single paragraph-line",
        tokens: "<paragraph-line>",
        expected_match: Some("paragraph"),
    },
    TestCase {
        name: "paragraph: multiple paragraph-lines",
        tokens: "<paragraph-line><paragraph-line><paragraph-line>",
        expected_match: Some("paragraph"),
    },
    TestCase {
        name: "paragraph: subject-line alone",
        tokens: "<subject-line>",
        expected_match: Some("paragraph"),
    },
    TestCase {
        name: "paragraph: list-item-line (is content-line)",
        tokens: "<list-item-line>",
        expected_match: Some("paragraph"),
    },
    // BLANK LINE GROUP TESTS
    TestCase {
        name: "blank_line_group: single blank",
        tokens: "<blank-line>",
        expected_match: Some("blank_line_group"),
    },
    TestCase {
        name: "blank_line_group: multiple blanks",
        tokens: "<blank-line><blank-line><blank-line>",
        expected_match: Some("blank_line_group"),
    },
];

// ============================================================================
// TEST RUNNER
// ============================================================================

fn main() {
    println!("=== GRAMMAR REGEX VALIDATION ===\n");

    let mut passed = 0;
    let mut failed = 0;

    for test in TEST_CASES {
        let matched_rule = find_matching_rule(test.tokens);
        let success = matched_rule == test.expected_match;

        if success {
            passed += 1;
            println!("✓ {}", test.name);
        } else {
            failed += 1;
            println!("✗ {}", test.name);
            println!("  Tokens: {}", test.tokens);
            println!("  Expected: {:?}", test.expected_match);
            println!("  Got: {:?}", matched_rule);
        }
    }

    println!("\n=== RESULTS ===");
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);
    println!("Total:  {}", passed + failed);

    if failed == 0 {
        println!("\n✓ All tests passed!");
    } else {
        println!("\n✗ Some tests failed");
        std::process::exit(1);
    }
}

/// Try to match a token sequence against grammar rules in order.
/// Returns the name of the first matching rule, or None if no match.
fn find_matching_rule(tokens: &str) -> Option<&'static str> {
    for (rule_name, pattern) in GRAMMAR_RULES {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(tokens) {
                return Some(rule_name);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_rules_compile() {
        for (name, pattern) in GRAMMAR_RULES {
            assert!(
                Regex::new(pattern).is_ok(),
                "Rule '{}' has invalid regex: {}",
                name,
                pattern
            );
        }
    }

    #[test]
    fn test_definition_rule() {
        let re = Regex::new(GRAMMAR_RULES[4].1).unwrap(); // definition rule
        assert!(re.is_match("<subject-line><container>"));
        assert!(!re.is_match("<subject-line>"));
        assert!(!re.is_match("<container>"));
    }

    #[test]
    fn test_session_rule() {
        let re = Regex::new(GRAMMAR_RULES[5].1).unwrap(); // session rule
        assert!(re.is_match("<blank-line><content-line><blank-line><container>"));
        assert!(!re.is_match("<blank-line><content-line><container>"));
        assert!(!re.is_match("<content-line><blank-line><container>"));
    }

    #[test]
    fn test_list_rule() {
        let re = Regex::new(GRAMMAR_RULES[3].1).unwrap(); // list rule
        assert!(re.is_match("<blank-line><list-item-line><list-item-line>"));
        assert!(re.is_match("<blank-line><list-item-line><container><list-item-line>"));
        assert!(!re.is_match("<list-item-line><list-item-line>"));
    }
}
