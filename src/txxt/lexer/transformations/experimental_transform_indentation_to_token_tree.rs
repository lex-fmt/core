//! Experimental transformation: flat line tokens → hierarchical token tree
//!
//! This transformation converts a flat stream of line tokens (which include
//! IndentLevel/DedentLevel markers) into a hierarchical tree structure.
//!
//! The tree represents the indentation-based nesting structure:
//! - IndentLevel markers trigger creation of new nested blocks
//! - DedentLevel markers close nested blocks and return to parent level
//! - All other tokens are added to the current block
//!
//! This tree structure preserves all original LineTokens (including source_tokens),
//! and can later be consumed by pattern-matching parsers that work on named token types.

use crate::txxt::lexer::tokens::LineToken;

/// A tree node in the hierarchical token structure.
///
/// The tree is built by processing IndentLevel/DedentLevel markers:
/// - Token variant holds a single LineToken
/// - Block variant holds a vector of tree nodes (children at deeper indentation)
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum LineTokenTree {
    /// A single line token
    Token(LineToken),

    /// A block of nested tokens (represents indented content)
    Block(Vec<LineTokenTree>),
}

/// Transform flat line tokens into hierarchical token tree.
///
/// Groups line tokens based on IndentLevel/DedentLevel markers into a tree structure.
/// Each IndentLevel pushes a new nesting level, each DedentLevel pops back up.
///
/// Input: Flat sequence of LineTokens (with structural IndentLevel/DedentLevel tokens)
/// Output: Hierarchical tree where indented content becomes nested Block nodes
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
///     Token(SubjectLine),
///     Block([
///       Token(ParagraphLine),
///       Token(ParagraphLine),
///     ]),
///     Token(ParagraphLine),
///   ]
/// ```
pub fn experimental_transform_indentation_to_token_tree(
    tokens: Vec<LineToken>,
) -> Vec<LineTokenTree> {
    let mut stack: Vec<Vec<LineTokenTree>> = vec![Vec::new()]; // Start with root level

    for token in tokens {
        match &token.line_type {
            crate::txxt::lexer::tokens::LineTokenType::IndentLevel => {
                // Start a new nested level
                stack.push(Vec::new());
            }
            crate::txxt::lexer::tokens::LineTokenType::DedentLevel => {
                // Close current level and add it as a Block to parent
                if let Some(completed_block) = stack.pop() {
                    if !completed_block.is_empty() {
                        let current_level = stack.last_mut().expect("Stack should never be empty");
                        current_level.push(LineTokenTree::Block(completed_block));
                    }
                }
            }
            _ => {
                // Regular token - add to current level
                let current_level = stack.last_mut().expect("Stack should never be empty");
                current_level.push(LineTokenTree::Token(token));
            }
        }
    }

    // The root level should remain as the final result
    stack.pop().expect("Stack should never be empty")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::txxt::lexer::tokens::{LineTokenType, Token};

    fn make_line_token(line_type: LineTokenType, tokens: Vec<Token>) -> LineToken {
        LineToken {
            source_tokens: tokens,
            line_type,
        }
    }

    #[test]
    fn test_single_token_no_indentation() {
        let input = vec![make_line_token(
            LineTokenType::ParagraphLine,
            vec![Token::Text("hello".to_string())],
        )];

        let result = experimental_transform_indentation_to_token_tree(input);

        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], LineTokenTree::Token(_)));
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

        let result = experimental_transform_indentation_to_token_tree(input);

        assert_eq!(result.len(), 3);
        assert!(matches!(result[0], LineTokenTree::Token(_)));
        assert!(matches!(result[1], LineTokenTree::Token(_)));
        assert!(matches!(result[2], LineTokenTree::Token(_)));
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

        let result = experimental_transform_indentation_to_token_tree(input);

        assert_eq!(result.len(), 2);
        assert!(matches!(result[0], LineTokenTree::Token(_)));
        assert!(matches!(result[1], LineTokenTree::Block(_)));

        if let LineTokenTree::Block(inner) = &result[1] {
            assert_eq!(inner.len(), 1);
            assert!(matches!(inner[0], LineTokenTree::Token(_)));
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

        let result = experimental_transform_indentation_to_token_tree(input);

        assert_eq!(result.len(), 2);

        if let LineTokenTree::Block(inner) = &result[1] {
            assert_eq!(inner.len(), 3);
            assert!(matches!(inner[0], LineTokenTree::Token(_)));
            assert!(matches!(inner[1], LineTokenTree::Token(_)));
            assert!(matches!(inner[2], LineTokenTree::Token(_)));
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

        let result = experimental_transform_indentation_to_token_tree(input);

        assert_eq!(result.len(), 2);

        // First token is the subject line
        assert!(matches!(result[0], LineTokenTree::Token(_)));

        // Second is a block (level 1 indentation)
        if let LineTokenTree::Block(level1) = &result[1] {
            assert_eq!(level1.len(), 2);
            assert!(matches!(level1[0], LineTokenTree::Token(_)));
            assert!(matches!(level1[1], LineTokenTree::Block(_)));

            // Check level 2 (nested block)
            if let LineTokenTree::Block(level2) = &level1[1] {
                assert_eq!(level2.len(), 1);
                assert!(matches!(level2[0], LineTokenTree::Token(_)));
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

        let result = experimental_transform_indentation_to_token_tree(input);

        assert_eq!(result.len(), 4);
        assert!(matches!(result[0], LineTokenTree::Token(_))); // Title1
        assert!(matches!(result[1], LineTokenTree::Block(_))); // Block for Title1
        assert!(matches!(result[2], LineTokenTree::Token(_))); // Title2
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

        let result = experimental_transform_indentation_to_token_tree(input);

        assert_eq!(result.len(), 4);
        assert!(matches!(result[0], LineTokenTree::Token(_)));
        assert!(matches!(result[1], LineTokenTree::Token(_)));
        assert!(matches!(result[2], LineTokenTree::Block(_)));
        assert!(matches!(result[3], LineTokenTree::Token(_)));
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

        let result = experimental_transform_indentation_to_token_tree(input);

        // Check that first token preserved source_tokens
        if let LineTokenTree::Token(line_token) = &result[0] {
            assert_eq!(line_token.source_tokens, test_tokens);
        }
    }

    #[test]
    fn test_empty_blocks_not_created() {
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

        let result = experimental_transform_indentation_to_token_tree(input);

        // Empty blocks should not be created, just skip
        assert_eq!(result.len(), 2);
        assert!(matches!(result[0], LineTokenTree::Token(_)));
        assert!(matches!(result[1], LineTokenTree::Token(_)));
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

        let result = experimental_transform_indentation_to_token_tree(input);

        // Verify structure depth
        assert_eq!(result.len(), 2);

        if let LineTokenTree::Block(l1) = &result[1] {
            assert_eq!(l1.len(), 2);
            if let LineTokenTree::Block(l2) = &l1[1] {
                assert_eq!(l2.len(), 2);
                if let LineTokenTree::Block(l3) = &l2[1] {
                    assert_eq!(l3.len(), 1);
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

        let result = experimental_transform_indentation_to_token_tree(input);

        assert_eq!(result.len(), 2);
        assert!(matches!(result[0], LineTokenTree::Token(_)));
        assert!(matches!(result[1], LineTokenTree::Block(_)));

        if let LineTokenTree::Block(level1) = &result[1] {
            assert_eq!(level1.len(), 5); // Item1, Item2, Blank, Para, Block
            assert!(matches!(level1[4], LineTokenTree::Block(_)));

            if let LineTokenTree::Block(level2) = &level1[4] {
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

        let result = experimental_transform_indentation_to_token_tree(input);

        if let LineTokenTree::Block(block) = &result[1] {
            assert_eq!(block.len(), 3); // Para1, BlankLine, Para2
            assert!(matches!(block[0], LineTokenTree::Token(_)));
            assert!(matches!(block[1], LineTokenTree::Token(_)));
            assert!(matches!(block[2], LineTokenTree::Token(_)));
        }
    }
}
