//! Linebased transformation: flat line tokens → hierarchical container token tree
//!
//! This transformation converts a flat stream of line tokens (which include
//! Indent/Dedent markers) into a hierarchical tree structure.
//!
//! The output is a single LineContainerToken representing the root of the entire tree.
//! The tree represents the indentation-based nesting structure:
//! - Indent markers trigger creation of new nested container levels
//! - Dedent markers close nested containers and return to parent level
//! - All other tokens (LineTokens) are children of containers at the same level
//!
//! This tree structure preserves all original LineTokens (including source_tokens),
//! and can be consumed by pattern-matching parsers that check token types
//! (e.g., <container>, <subject-line>, <paragraph-line>).

use crate::lex::lexers::linebased::tokens::{LineContainer, LineToken, LineType};

/// Transform flat line tokens into a hierarchical container token tree.
///
/// Converts a flat sequence of LineTokens (with Indent/Dedent markers)
/// into a tree where every node is a LineContainerToken.
///
/// Groups line tokens based on Indent/Dedent markers:
/// - Each Indent triggers a new nested container
/// - Each Dedent closes the current container and returns to parent
/// - Line tokens at same indentation level become children of a container
///
/// Input: Flat sequence of LineTokens (with structural Indent/Dedent tokens)
/// Output: Root LineContainerToken containing the entire hierarchical tree
///
/// Example:
/// ```text
/// Input line tokens:
///   LineToken(SubjectLine),
///   LineToken(Indent),
///   LineToken(ParagraphLine),
///   LineToken(ParagraphLine),
///   LineToken(Dedent),
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
pub fn _indentation_to_token_tree(tokens: Vec<LineToken>) -> LineContainer {
    // Stack of pending children at each indentation level
    // Each level accumulates tokens/containers that become children of a container
    let mut stack: Vec<Vec<LineContainer>> = vec![Vec::new()];
    let mut pending_tokens: Vec<LineToken> = Vec::new();

    for token in tokens {
        match &token.line_type {
            LineType::Indent => {
                // Flush pending tokens before entering nested level
                if !pending_tokens.is_empty() {
                    for line_token in pending_tokens.drain(..) {
                        let current_level = stack.last_mut().expect("Stack never empty");
                        current_level.push(LineContainer::Token(line_token));
                    }
                }
                // Start a new nesting level
                stack.push(Vec::new());
            }
            LineType::Dedent => {
                // Flush pending tokens before closing level
                if !pending_tokens.is_empty() {
                    for line_token in pending_tokens.drain(..) {
                        let current_level = stack.last_mut().expect("Stack never empty");
                        current_level.push(LineContainer::Token(line_token));
                    }
                }
                // Close current level and add as nested container to parent
                if let Some(children) = stack.pop() {
                    let nested_container = LineContainer::Container { children };
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
            root_level.push(LineContainer::Token(line_token));
        }
    }

    // Create root container from accumulated children
    let root_children = stack.pop().expect("Stack should contain root level");

    LineContainer::Container {
        children: root_children,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::lexers::tokens::Token;

    fn make_line_token(line_type: LineType, tokens: Vec<Token>) -> LineToken {
        LineToken {
            source_tokens: tokens,
            token_spans: Vec::new(),
            line_type,
        }
    }

    #[test]
    fn test_single_token_no_indentation() {
        let input = vec![make_line_token(
            LineType::ParagraphLine,
            vec![Token::Text("hello".to_string())],
        )];

        let result = _indentation_to_token_tree(input);

        // Verify the result is a container with one child token
        match result {
            LineContainer::Container { children } => {
                assert_eq!(children.len(), 1);
                assert!(matches!(children[0], LineContainer::Token(_)));
            }
            _ => panic!("Expected Container"),
        }
    }

    #[test]
    fn test_multiple_tokens_same_level() {
        let input = vec![
            make_line_token(
                LineType::SubjectLine,
                vec![Token::Text("Title".to_string())],
            ),
            make_line_token(
                LineType::ParagraphLine,
                vec![Token::Text("Para1".to_string())],
            ),
            make_line_token(
                LineType::ParagraphLine,
                vec![Token::Text("Para2".to_string())],
            ),
        ];

        let result = _indentation_to_token_tree(input);

        // All tokens at same level should be children of root container
        match result {
            LineContainer::Container { children } => {
                assert_eq!(children.len(), 3);
                assert!(children
                    .iter()
                    .all(|c| matches!(c, LineContainer::Token(_))));
            }
            _ => panic!("Expected Container"),
        }
    }

    #[test]
    fn test_single_indent_level() {
        let input = vec![
            make_line_token(
                LineType::SubjectLine,
                vec![Token::Text("Title".to_string())],
            ),
            make_line_token(LineType::Indent, vec![Token::Indent(vec![])]),
            make_line_token(
                LineType::ParagraphLine,
                vec![Token::Text("Indented".to_string())],
            ),
            make_line_token(LineType::Dedent, vec![Token::Dedent(vec![])]),
        ];

        let result = _indentation_to_token_tree(input);

        // Root should have title token and nested container
        match result {
            LineContainer::Container { children } => {
                assert_eq!(children.len(), 2);
                assert!(matches!(children[0], LineContainer::Token(_)));
                assert!(matches!(children[1], LineContainer::Container { .. }));
            }
            _ => panic!("Expected Container"),
        }
    }
}
