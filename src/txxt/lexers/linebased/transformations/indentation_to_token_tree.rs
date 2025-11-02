//! Linebased transformation: flat line tokens → hierarchical token tree
//!
//! This transformation converts a flat stream of line tokens (which include
//! IndentLevel/DedentLevel markers) into a hierarchical tree structure.
//!
//! The tree represents the indentation-based nesting structure:
//! - IndentLevel markers trigger creation of new nested blocks
//! - DedentLevel markers close nested blocks and return to parent level
//! - All other tokens are grouped into LineContainerTokens at the same level
//!
//! This tree structure creates TWO levels of grouping:
//! 1. LineTokens grouped by line (preserves raw tokens for location tracking)
//! 2. LineTokens at same indentation grouped into LineContainerTokens (preserves block structure)
//!
//! This tree structure preserves all original LineTokens (including source_tokens),
//! and can later be consumed by pattern-matching parsers that work on named token types.

use crate::txxt::lexers::linebased::tokens::{
    LineContainerToken, LineToken, LineTokenTree, LineTokenType,
};

/// Transform flat line tokens into hierarchical token tree with two levels of grouping.
///
/// Groups line tokens based on IndentLevel/DedentLevel markers into a tree structure.
/// Each IndentLevel pushes a new nesting level, each DedentLevel pops back up.
/// Lines at the same level are grouped into LineContainerTokens.
///
/// Input: Flat sequence of LineTokens (with structural IndentLevel/DedentLevel tokens)
/// Output: Hierarchical tree where:
///   - Tokens at same level are grouped into Container nodes
///   - Indented content becomes nested Block nodes
///
/// Example:
/// ```text
/// Input line tokens:
///   LineToken(SubjectLine),
///   LineToken(IndentLevel),
///   LineToken(ParagraphLine),
///   LineToken(ParagraphLine),
///   LineToken(DedentLevel),
///   LineToken(ParagraphLine),
///
/// Output tree:
///   [
///     Container([SubjectLine]),
///     Block([
///       Container([ParagraphLine, ParagraphLine]),
///     ]),
///     Container([ParagraphLine]),
///   ]
/// ```
pub fn _indentation_to_token_tree(tokens: Vec<LineToken>) -> Vec<LineTokenTree> {
    let mut stack: Vec<Vec<LineTokenTree>> = vec![Vec::new()]; // Start with root level
    let mut pending_tokens: Vec<LineToken> = Vec::new(); // Accumulate tokens at current level

    for token in tokens {
        match &token.line_type {
            LineTokenType::IndentLevel => {
                // Flush any pending tokens as a container before starting new level
                if !pending_tokens.is_empty() {
                    let container = create_line_container_token(pending_tokens);
                    let current_level = stack.last_mut().expect("Stack should never be empty");
                    current_level.push(LineTokenTree::Container(container));
                    pending_tokens = Vec::new();
                }
                // Start a new nested level
                stack.push(Vec::new());
            }
            LineTokenType::DedentLevel => {
                // Flush any pending tokens before closing level
                if !pending_tokens.is_empty() {
                    let container = create_line_container_token(pending_tokens);
                    let current_level = stack.last_mut().expect("Stack should never be empty");
                    current_level.push(LineTokenTree::Container(container));
                    pending_tokens = Vec::new();
                }
                // Close current level and add it as a Block to parent
                // Even empty blocks are preserved - they may be semantically meaningful
                // (e.g., an empty session body, though this shouldn't occur in valid txxt)
                if let Some(completed_block) = stack.pop() {
                    let current_level = stack.last_mut().expect("Stack should never be empty");
                    current_level.push(LineTokenTree::Block(completed_block));
                }
            }
            _ => {
                // Accumulate regular tokens at current level
                pending_tokens.push(token);
            }
        }
    }

    // Flush any remaining pending tokens
    if !pending_tokens.is_empty() {
        let container = create_line_container_token(pending_tokens);
        let current_level = stack.last_mut().expect("Stack should never be empty");
        current_level.push(LineTokenTree::Container(container));
    }

    // The root level should remain as the final result
    stack.pop().expect("Stack should never be empty")
}

/// Create a LineContainerToken from a vec of LineTokens.
///
/// Computes the combined source span from all tokens in the container.
fn create_line_container_token(tokens: Vec<LineToken>) -> LineContainerToken {
    let source_span = if tokens.is_empty() {
        None
    } else {
        // Combine all source spans into one that covers all tokens
        let mut start: Option<usize> = None;
        let mut end: Option<usize> = None;

        for token in &tokens {
            if let Some(ref span) = token.source_span {
                start = Some(start.map_or(span.start, |s| s.min(span.start)));
                end = Some(end.map_or(span.end, |e| e.max(span.end)));
            }
        }

        match (start, end) {
            (Some(s), Some(e)) => Some(s..e),
            _ => None,
        }
    };

    LineContainerToken {
        source_tokens: tokens,
        source_span,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::txxt::lexers::tokens::Token;

    fn make_line_token(line_type: LineTokenType, tokens: Vec<Token>) -> LineToken {
        LineToken {
            source_tokens: tokens,
            line_type,
            source_span: None,
        }
    }

    #[test]
    fn test_single_token_no_indentation() {
        let input = vec![make_line_token(
            LineTokenType::ParagraphLine,
            vec![Token::Text("hello".to_string())],
        )];

        let result = _indentation_to_token_tree(input);

        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], LineTokenTree::Container(_)));

        // Verify the container has the token
        if let LineTokenTree::Container(container) = &result[0] {
            assert_eq!(container.source_tokens.len(), 1);
        }
    }

    #[test]
    fn test_multiple_tokens_same_level() {
        let input = vec![
            make_line_token(
                LineTokenType::SubjectLine,
                vec![Token::Text("Title".to_string())],
            ),
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Para1".to_string())],
            ),
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Para2".to_string())],
            ),
        ];

        let result = _indentation_to_token_tree(input);

        // All tokens at same level should be in a single container
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], LineTokenTree::Container(_)));

        if let LineTokenTree::Container(container) = &result[0] {
            assert_eq!(container.source_tokens.len(), 3);
        }
    }

    #[test]
    fn test_single_indent_level() {
        let input = vec![
            make_line_token(
                LineTokenType::SubjectLine,
                vec![Token::Text("Title".to_string())],
            ),
            make_line_token(LineTokenType::IndentLevel, vec![Token::IndentLevel]),
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Indented".to_string())],
            ),
            make_line_token(LineTokenType::DedentLevel, vec![Token::DedentLevel]),
        ];

        let result = _indentation_to_token_tree(input);

        assert_eq!(result.len(), 2);
        assert!(matches!(result[0], LineTokenTree::Container(_)));
        assert!(matches!(result[1], LineTokenTree::Block(_)));

        if let LineTokenTree::Block(inner) = &result[1] {
            assert_eq!(inner.len(), 1);
            assert!(matches!(inner[0], LineTokenTree::Container(_)));

            // Verify the inner container has the indented token
            if let LineTokenTree::Container(container) = &inner[0] {
                assert_eq!(container.source_tokens.len(), 1);
            }
        }
    }

    #[test]
    fn test_multiple_lines_in_block() {
        let input = vec![
            make_line_token(
                LineTokenType::SubjectLine,
                vec![Token::Text("Title".to_string())],
            ),
            make_line_token(LineTokenType::IndentLevel, vec![Token::IndentLevel]),
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Line1".to_string())],
            ),
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Line2".to_string())],
            ),
            make_line_token(
                LineTokenType::ListLine,
                vec![
                    Token::Dash,
                    Token::Whitespace,
                    Token::Text("Item".to_string()),
                ],
            ),
            make_line_token(LineTokenType::DedentLevel, vec![Token::DedentLevel]),
        ];

        let result = _indentation_to_token_tree(input);

        assert_eq!(result.len(), 2);
        assert!(matches!(result[0], LineTokenTree::Container(_)));

        if let LineTokenTree::Block(inner) = &result[1] {
            // All three lines at same level should be in one container
            assert_eq!(inner.len(), 1);
            assert!(matches!(inner[0], LineTokenTree::Container(_)));

            if let LineTokenTree::Container(container) = &inner[0] {
                assert_eq!(container.source_tokens.len(), 3);
            }
        }
    }

    #[test]
    fn test_nested_indentation_two_levels() {
        let input = vec![
            make_line_token(
                LineTokenType::SubjectLine,
                vec![Token::Text("Title".to_string())],
            ),
            make_line_token(LineTokenType::IndentLevel, vec![Token::IndentLevel]),
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Level1".to_string())],
            ),
            make_line_token(LineTokenType::IndentLevel, vec![Token::IndentLevel]),
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Level2".to_string())],
            ),
            make_line_token(LineTokenType::DedentLevel, vec![Token::DedentLevel]),
            make_line_token(LineTokenType::DedentLevel, vec![Token::DedentLevel]),
        ];

        let result = _indentation_to_token_tree(input);

        assert_eq!(result.len(), 2);

        // First is a container with subject line
        assert!(matches!(result[0], LineTokenTree::Container(_)));

        // Second is a block (level 1 indentation)
        if let LineTokenTree::Block(level1) = &result[1] {
            assert_eq!(level1.len(), 2);
            assert!(matches!(level1[0], LineTokenTree::Container(_)));
            assert!(matches!(level1[1], LineTokenTree::Block(_)));

            // Check level 2 (nested block)
            if let LineTokenTree::Block(level2) = &level1[1] {
                assert_eq!(level2.len(), 1);
                assert!(matches!(level2[0], LineTokenTree::Container(_)));
            }
        }
    }

    #[test]
    fn test_multiple_blocks_same_level() {
        let input = vec![
            make_line_token(
                LineTokenType::SubjectLine,
                vec![Token::Text("Title1".to_string())],
            ),
            make_line_token(LineTokenType::IndentLevel, vec![Token::IndentLevel]),
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Content1".to_string())],
            ),
            make_line_token(LineTokenType::DedentLevel, vec![Token::DedentLevel]),
            make_line_token(
                LineTokenType::SubjectLine,
                vec![Token::Text("Title2".to_string())],
            ),
            make_line_token(LineTokenType::IndentLevel, vec![Token::IndentLevel]),
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Content2".to_string())],
            ),
            make_line_token(LineTokenType::DedentLevel, vec![Token::DedentLevel]),
        ];

        let result = _indentation_to_token_tree(input);

        assert_eq!(result.len(), 4);
        assert!(matches!(result[0], LineTokenTree::Container(_))); // Container with Title1
        assert!(matches!(result[1], LineTokenTree::Block(_))); // Block for Title1
        assert!(matches!(result[2], LineTokenTree::Container(_))); // Container with Title2
        assert!(matches!(result[3], LineTokenTree::Block(_))); // Block for Title2
    }

    #[test]
    fn test_mixed_tokens_and_blocks() {
        let input = vec![
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Para1".to_string())],
            ),
            make_line_token(
                LineTokenType::SubjectLine,
                vec![Token::Text("Subject".to_string())],
            ),
            make_line_token(LineTokenType::IndentLevel, vec![Token::IndentLevel]),
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Indented".to_string())],
            ),
            make_line_token(LineTokenType::DedentLevel, vec![Token::DedentLevel]),
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Para2".to_string())],
            ),
        ];

        let result = _indentation_to_token_tree(input);

        assert_eq!(result.len(), 3);
        // First container has Para1 and Subject
        assert!(matches!(result[0], LineTokenTree::Container(_)));
        if let LineTokenTree::Container(container) = &result[0] {
            assert_eq!(container.source_tokens.len(), 2);
        }
        assert!(matches!(result[1], LineTokenTree::Block(_)));
        // Last container has Para2
        assert!(matches!(result[2], LineTokenTree::Container(_)));
    }

    #[test]
    fn test_preserves_all_source_tokens() {
        let test_tokens = vec![
            Token::Text("Title".to_string()),
            Token::Colon,
            Token::Newline,
        ];
        let input = vec![
            make_line_token(LineTokenType::SubjectLine, test_tokens.clone()),
            make_line_token(LineTokenType::IndentLevel, vec![Token::IndentLevel]),
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Content".to_string())],
            ),
            make_line_token(LineTokenType::DedentLevel, vec![Token::DedentLevel]),
        ];

        let result = _indentation_to_token_tree(input);

        // Check that the first container's source token preserved the test_tokens
        if let LineTokenTree::Container(container) = &result[0] {
            assert_eq!(container.source_tokens[0].source_tokens, test_tokens);
        }
    }

    #[test]
    fn test_empty_blocks_are_preserved() {
        let input = vec![
            make_line_token(
                LineTokenType::SubjectLine,
                vec![Token::Text("Title".to_string())],
            ),
            make_line_token(LineTokenType::IndentLevel, vec![Token::IndentLevel]),
            make_line_token(LineTokenType::DedentLevel, vec![Token::DedentLevel]),
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("After".to_string())],
            ),
        ];

        let result = _indentation_to_token_tree(input);

        // Empty blocks ARE preserved - they may be semantically meaningful
        assert_eq!(result.len(), 3);
        assert!(matches!(result[0], LineTokenTree::Container(_))); // Container with Title
        assert!(matches!(result[1], LineTokenTree::Block(_))); // Empty block
        assert!(matches!(result[2], LineTokenTree::Container(_))); // Container with After

        // Verify the empty block is indeed empty
        if let LineTokenTree::Block(inner) = &result[1] {
            assert_eq!(inner.len(), 0);
        }
    }

    #[test]
    fn test_three_level_nesting() {
        let input = vec![
            make_line_token(
                LineTokenType::SubjectLine,
                vec![Token::Text("L0".to_string())],
            ),
            make_line_token(LineTokenType::IndentLevel, vec![Token::IndentLevel]),
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("L1".to_string())],
            ),
            make_line_token(LineTokenType::IndentLevel, vec![Token::IndentLevel]),
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("L2".to_string())],
            ),
            make_line_token(LineTokenType::IndentLevel, vec![Token::IndentLevel]),
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("L3".to_string())],
            ),
            make_line_token(LineTokenType::DedentLevel, vec![Token::DedentLevel]),
            make_line_token(LineTokenType::DedentLevel, vec![Token::DedentLevel]),
            make_line_token(LineTokenType::DedentLevel, vec![Token::DedentLevel]),
        ];

        let result = _indentation_to_token_tree(input);

        // Verify structure depth
        assert_eq!(result.len(), 2);

        if let LineTokenTree::Block(l1) = &result[1] {
            assert_eq!(l1.len(), 2);
            assert!(matches!(l1[0], LineTokenTree::Container(_)));
            if let LineTokenTree::Block(l2) = &l1[1] {
                assert_eq!(l2.len(), 2);
                assert!(matches!(l2[0], LineTokenTree::Container(_)));
                if let LineTokenTree::Block(l3) = &l2[1] {
                    assert_eq!(l3.len(), 1);
                    assert!(matches!(l3[0], LineTokenTree::Container(_)));
                }
            }
        }
    }

    #[test]
    fn test_complex_mixed_structure() {
        let input = vec![
            make_line_token(
                LineTokenType::SubjectLine,
                vec![Token::Text("Subject".to_string())],
            ),
            make_line_token(LineTokenType::IndentLevel, vec![Token::IndentLevel]),
            make_line_token(
                LineTokenType::ListLine,
                vec![
                    Token::Dash,
                    Token::Whitespace,
                    Token::Text("Item1".to_string()),
                ],
            ),
            make_line_token(
                LineTokenType::ListLine,
                vec![
                    Token::Dash,
                    Token::Whitespace,
                    Token::Text("Item2".to_string()),
                ],
            ),
            make_line_token(LineTokenType::BlankLine, vec![Token::Newline]),
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Paragraph".to_string())],
            ),
            make_line_token(LineTokenType::IndentLevel, vec![Token::IndentLevel]),
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Nested".to_string())],
            ),
            make_line_token(LineTokenType::DedentLevel, vec![Token::DedentLevel]),
            make_line_token(LineTokenType::DedentLevel, vec![Token::DedentLevel]),
        ];

        let result = _indentation_to_token_tree(input);

        assert_eq!(result.len(), 2);
        assert!(matches!(result[0], LineTokenTree::Container(_)));
        assert!(matches!(result[1], LineTokenTree::Block(_)));

        if let LineTokenTree::Block(level1) = &result[1] {
            // Item1, Item2, Blank, Para grouped in one container, plus Block for nested
            assert_eq!(level1.len(), 2);
            assert!(matches!(level1[0], LineTokenTree::Container(_)));
            assert!(matches!(level1[1], LineTokenTree::Block(_)));

            if let LineTokenTree::Container(container) = &level1[0] {
                assert_eq!(container.source_tokens.len(), 4); // Item1, Item2, Blank, Para
            }

            if let LineTokenTree::Block(level2) = &level1[1] {
                assert_eq!(level2.len(), 1); // Just the nested content
            }
        }
    }

    #[test]
    fn test_blank_line_preserved_in_block() {
        let input = vec![
            make_line_token(
                LineTokenType::SubjectLine,
                vec![Token::Text("Subject".to_string())],
            ),
            make_line_token(LineTokenType::IndentLevel, vec![Token::IndentLevel]),
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Para1".to_string())],
            ),
            make_line_token(LineTokenType::BlankLine, vec![Token::Newline]),
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Para2".to_string())],
            ),
            make_line_token(LineTokenType::DedentLevel, vec![Token::DedentLevel]),
        ];

        let result = _indentation_to_token_tree(input);

        if let LineTokenTree::Block(block) = &result[1] {
            // All three lines at same level should be in one container
            assert_eq!(block.len(), 1);
            assert!(matches!(block[0], LineTokenTree::Container(_)));

            if let LineTokenTree::Container(container) = &block[0] {
                assert_eq!(container.source_tokens.len(), 3); // Para1, BlankLine, Para2
            }
        }
    }
}
