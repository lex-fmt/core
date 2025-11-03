//! Linebased transformation: flat line tokens → hierarchical container token tree
//!
//! This transformation converts a flat stream of line tokens (which include
//! IndentLevel/DedentLevel markers) into a hierarchical tree structure.
//!
//! The output is a single LineContainerToken representing the root of the entire tree.
//! The tree represents the indentation-based nesting structure:
//! - IndentLevel markers trigger creation of new nested container levels
//! - DedentLevel markers close nested containers and return to parent level
//! - All other tokens (LineTokens) are children of containers at the same level
//!
//! This tree structure preserves all original LineTokens (including source_tokens),
//! and can be consumed by pattern-matching parsers that check token types
//! (e.g., <container>, <subject-line>, <paragraph-line>).

use crate::txxt::lexers::linebased::tokens::{LineContainerToken, LineToken, LineTokenType};

/// Transform flat line tokens into a hierarchical container token tree.
///
/// Converts a flat sequence of LineTokens (with IndentLevel/DedentLevel markers)
/// into a tree where every node is a LineContainerToken.
///
/// Groups line tokens based on IndentLevel/DedentLevel markers:
/// - Each IndentLevel triggers a new nested container
/// - Each DedentLevel closes the current container and returns to parent
/// - Line tokens at same indentation level become children of a container
///
/// Input: Flat sequence of LineTokens (with structural IndentLevel/DedentLevel tokens)
/// Output: Root LineContainerToken containing the entire hierarchical tree
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
/// Output tree (single root):
///   Container {
///     children: [
///       Token(SubjectLine),
///       Container {
///         children: [
///           Token(ParagraphLine),
///           Token(ParagraphLine),
///         ]
///       },
///       Token(ParagraphLine),
///     ]
///   }
/// ```
pub fn _indentation_to_token_tree(tokens: Vec<LineToken>) -> LineContainerToken {
    // Stack of pending children at each indentation level
    // Each level accumulates tokens/containers that become children of a container
    let mut stack: Vec<Vec<LineContainerToken>> = vec![Vec::new()];
    let mut pending_tokens: Vec<LineToken> = Vec::new();

    for token in tokens {
        match &token.line_type {
            LineTokenType::IndentLevel => {
                // Flush pending tokens before entering nested level
                if !pending_tokens.is_empty() {
                    for line_token in pending_tokens.drain(..) {
                        let current_level = stack.last_mut().expect("Stack never empty");
                        current_level.push(LineContainerToken::Token(line_token));
                    }
                }
                // Start a new nesting level
                stack.push(Vec::new());
            }
            LineTokenType::DedentLevel => {
                // Flush pending tokens before closing level
                if !pending_tokens.is_empty() {
                    for line_token in pending_tokens.drain(..) {
                        let current_level = stack.last_mut().expect("Stack never empty");
                        current_level.push(LineContainerToken::Token(line_token));
                    }
                }
                // Close current level and add as nested container to parent
                if let Some(children) = stack.pop() {
                    let nested_container = LineContainerToken::Container { children };
                    let parent_level = stack.last_mut().expect("Stack never empty");
                    parent_level.push(nested_container);
                }
            }
            _ => {
                // Accumulate regular line tokens at current level
                pending_tokens.push(token);
            }
        }
    }

    // Flush any remaining pending tokens at root level
    if !pending_tokens.is_empty() {
        for line_token in pending_tokens.drain(..) {
            let root_level = stack.last_mut().expect("Stack never empty");
            root_level.push(LineContainerToken::Token(line_token));
        }
    }

    // Create root container from accumulated children
    let root_children = stack.pop().expect("Stack should contain root level");

    LineContainerToken::Container {
        children: root_children,
    }
}

/// TEMPORARY: Unwrap a LineContainerToken tree back into the old Vec<LineTokenTree> format.
///
/// This is a compatibility bridge for Delivery 1, allowing the parser to work unchanged
/// while the lexer outputs the new LineContainerToken tree structure.
///
/// In Delivery 2, this unwrapper will be removed and the parser will work directly
/// with LineContainerToken trees using the new declarative grammar.
pub fn unwrap_container_to_token_tree(
    root: &LineContainerToken,
) -> Vec<crate::txxt::lexers::linebased::tokens::LineTokenTree> {
    use crate::txxt::lexers::linebased::tokens::{LineContainerTokenLegacy, LineTokenTree};

    fn unwrap_recursive(container: &LineContainerToken) -> Vec<LineTokenTree> {
        match container {
            LineContainerToken::Token(token) => {
                vec![LineTokenTree::Token(token.clone())]
            }
            LineContainerToken::Container { children } => {
                let mut result = Vec::new();
                let mut pending_tokens = Vec::new();

                for child in children {
                    match child {
                        LineContainerToken::Token(token) => {
                            pending_tokens.push(token.clone());
                        }
                        LineContainerToken::Container { .. } => {
                            // Flush pending tokens as a Container before starting a Block
                            if !pending_tokens.is_empty() {
                                let legacy_container = LineContainerTokenLegacy {
                                    source_tokens: std::mem::take(&mut pending_tokens),
                                    source_span: None, // No aggregate span - will be computed by AST construction
                                };
                                result.push(LineTokenTree::Container(legacy_container));
                            }

                            // Add nested container as a Block
                            let block_contents = unwrap_recursive(child);
                            result.push(LineTokenTree::Block(block_contents));
                        }
                    }
                }

                // Flush remaining tokens
                if !pending_tokens.is_empty() {
                    let legacy_container = LineContainerTokenLegacy {
                        source_tokens: pending_tokens,
                        source_span: None, // No aggregate span - will be computed by AST construction
                    };
                    result.push(LineTokenTree::Container(legacy_container));
                }

                result
            }
        }
    }

    unwrap_recursive(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::txxt::lexers::linebased::tokens::LineTokenTree;
    use crate::txxt::lexers::tokens::Token;

    fn make_line_token(line_type: LineTokenType, tokens: Vec<Token>) -> LineToken {
        LineToken {
            source_tokens: tokens,
            token_spans: Vec::new(),
            line_type,
        }
    }

    #[test]
    fn test_single_token_no_indentation() {
        let input = vec![make_line_token(
            LineTokenType::ParagraphLine,
            vec![Token::Text("hello".to_string())],
        )];

        let result = _indentation_to_token_tree(input);
        let legacy = unwrap_container_to_token_tree(&result);

        assert_eq!(legacy.len(), 1);
        assert!(matches!(legacy[0], LineTokenTree::Container(_)));

        // Verify the container has the token
        if let LineTokenTree::Container(container) = &legacy[0] {
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
        let legacy = unwrap_container_to_token_tree(&result);

        // All tokens at same level should be in a single container
        assert_eq!(legacy.len(), 1);
        assert!(matches!(legacy[0], LineTokenTree::Container(_)));

        if let LineTokenTree::Container(container) = &legacy[0] {
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
            make_line_token(LineTokenType::IndentLevel, vec![Token::IndentLevel(vec![])]),
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Indented".to_string())],
            ),
            make_line_token(LineTokenType::DedentLevel, vec![Token::DedentLevel(vec![])]),
        ];

        let result = _indentation_to_token_tree(input);
        let legacy = unwrap_container_to_token_tree(&result);

        assert_eq!(legacy.len(), 2);
        assert!(matches!(legacy[0], LineTokenTree::Container(_)));
        assert!(matches!(legacy[1], LineTokenTree::Block(_)));

        if let LineTokenTree::Block(inner) = &legacy[1] {
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
            make_line_token(LineTokenType::IndentLevel, vec![Token::IndentLevel(vec![])]),
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
            make_line_token(LineTokenType::DedentLevel, vec![Token::DedentLevel(vec![])]),
        ];

        let result = _indentation_to_token_tree(input);
        let legacy = unwrap_container_to_token_tree(&result);

        assert_eq!(legacy.len(), 2);
        assert!(matches!(legacy[0], LineTokenTree::Container(_)));

        if let LineTokenTree::Block(inner) = &legacy[1] {
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
            make_line_token(LineTokenType::IndentLevel, vec![Token::IndentLevel(vec![])]),
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Level1".to_string())],
            ),
            make_line_token(LineTokenType::IndentLevel, vec![Token::IndentLevel(vec![])]),
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Level2".to_string())],
            ),
            make_line_token(LineTokenType::DedentLevel, vec![Token::DedentLevel(vec![])]),
            make_line_token(LineTokenType::DedentLevel, vec![Token::DedentLevel(vec![])]),
        ];

        let result = _indentation_to_token_tree(input);
        let legacy = unwrap_container_to_token_tree(&result);

        assert_eq!(legacy.len(), 2);

        // First is a container with subject line
        assert!(matches!(legacy[0], LineTokenTree::Container(_)));

        // Second is a block (level 1 indentation)
        if let LineTokenTree::Block(level1) = &legacy[1] {
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
            make_line_token(LineTokenType::IndentLevel, vec![Token::IndentLevel(vec![])]),
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Content1".to_string())],
            ),
            make_line_token(LineTokenType::DedentLevel, vec![Token::DedentLevel(vec![])]),
            make_line_token(
                LineTokenType::SubjectLine,
                vec![Token::Text("Title2".to_string())],
            ),
            make_line_token(LineTokenType::IndentLevel, vec![Token::IndentLevel(vec![])]),
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Content2".to_string())],
            ),
            make_line_token(LineTokenType::DedentLevel, vec![Token::DedentLevel(vec![])]),
        ];

        let result = _indentation_to_token_tree(input);
        let legacy = unwrap_container_to_token_tree(&result);

        assert_eq!(legacy.len(), 4);
        assert!(matches!(legacy[0], LineTokenTree::Container(_))); // Container with Title1
        assert!(matches!(legacy[1], LineTokenTree::Block(_))); // Block for Title1
        assert!(matches!(legacy[2], LineTokenTree::Container(_))); // Container with Title2
        assert!(matches!(legacy[3], LineTokenTree::Block(_))); // Block for Title2
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
            make_line_token(LineTokenType::IndentLevel, vec![Token::IndentLevel(vec![])]),
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Indented".to_string())],
            ),
            make_line_token(LineTokenType::DedentLevel, vec![Token::DedentLevel(vec![])]),
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Para2".to_string())],
            ),
        ];

        let result = _indentation_to_token_tree(input);
        let legacy = unwrap_container_to_token_tree(&result);

        assert_eq!(legacy.len(), 3);
        // First container has Para1 and Subject
        assert!(matches!(legacy[0], LineTokenTree::Container(_)));
        if let LineTokenTree::Container(container) = &legacy[0] {
            assert_eq!(container.source_tokens.len(), 2);
        }
        assert!(matches!(legacy[1], LineTokenTree::Block(_)));
        // Last container has Para2
        assert!(matches!(legacy[2], LineTokenTree::Container(_)));
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
            make_line_token(LineTokenType::IndentLevel, vec![Token::IndentLevel(vec![])]),
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Content".to_string())],
            ),
            make_line_token(LineTokenType::DedentLevel, vec![Token::DedentLevel(vec![])]),
        ];

        let result = _indentation_to_token_tree(input);
        let legacy = unwrap_container_to_token_tree(&result);

        // Check that the first container's source token preserved the test_tokens
        if let LineTokenTree::Container(container) = &legacy[0] {
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
            make_line_token(LineTokenType::IndentLevel, vec![Token::IndentLevel(vec![])]),
            make_line_token(LineTokenType::DedentLevel, vec![Token::DedentLevel(vec![])]),
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("After".to_string())],
            ),
        ];

        let result = _indentation_to_token_tree(input);
        let legacy = unwrap_container_to_token_tree(&result);

        // Empty blocks ARE preserved - they may be semantically meaningful
        assert_eq!(legacy.len(), 3);
        assert!(matches!(legacy[0], LineTokenTree::Container(_))); // Container with Title
        assert!(matches!(legacy[1], LineTokenTree::Block(_))); // Empty block
        assert!(matches!(legacy[2], LineTokenTree::Container(_))); // Container with After

        // Verify the empty block is indeed empty
        if let LineTokenTree::Block(inner) = &legacy[1] {
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
            make_line_token(LineTokenType::IndentLevel, vec![Token::IndentLevel(vec![])]),
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("L1".to_string())],
            ),
            make_line_token(LineTokenType::IndentLevel, vec![Token::IndentLevel(vec![])]),
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("L2".to_string())],
            ),
            make_line_token(LineTokenType::IndentLevel, vec![Token::IndentLevel(vec![])]),
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("L3".to_string())],
            ),
            make_line_token(LineTokenType::DedentLevel, vec![Token::DedentLevel(vec![])]),
            make_line_token(LineTokenType::DedentLevel, vec![Token::DedentLevel(vec![])]),
            make_line_token(LineTokenType::DedentLevel, vec![Token::DedentLevel(vec![])]),
        ];

        let result = _indentation_to_token_tree(input);
        let legacy = unwrap_container_to_token_tree(&result);

        // Verify structure depth
        assert_eq!(legacy.len(), 2);

        if let LineTokenTree::Block(l1) = &legacy[1] {
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
            make_line_token(LineTokenType::IndentLevel, vec![Token::IndentLevel(vec![])]),
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
            make_line_token(LineTokenType::IndentLevel, vec![Token::IndentLevel(vec![])]),
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Nested".to_string())],
            ),
            make_line_token(LineTokenType::DedentLevel, vec![Token::DedentLevel(vec![])]),
            make_line_token(LineTokenType::DedentLevel, vec![Token::DedentLevel(vec![])]),
        ];

        let result = _indentation_to_token_tree(input);
        let legacy = unwrap_container_to_token_tree(&result);

        assert_eq!(legacy.len(), 2);
        assert!(matches!(legacy[0], LineTokenTree::Container(_)));
        assert!(matches!(legacy[1], LineTokenTree::Block(_)));

        if let LineTokenTree::Block(level1) = &legacy[1] {
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
            make_line_token(LineTokenType::IndentLevel, vec![Token::IndentLevel(vec![])]),
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Para1".to_string())],
            ),
            make_line_token(LineTokenType::BlankLine, vec![Token::Newline]),
            make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Para2".to_string())],
            ),
            make_line_token(LineTokenType::DedentLevel, vec![Token::DedentLevel(vec![])]),
        ];

        let result = _indentation_to_token_tree(input);
        let legacy = unwrap_container_to_token_tree(&result);

        if let LineTokenTree::Block(block) = &legacy[1] {
            // All three lines at same level should be in one container
            assert_eq!(block.len(), 1);
            assert!(matches!(block[0], LineTokenTree::Container(_)));

            if let LineTokenTree::Container(container) = &block[0] {
                assert_eq!(container.source_tokens.len(), 3); // Para1, BlankLine, Para2
            }
        }
    }
}
