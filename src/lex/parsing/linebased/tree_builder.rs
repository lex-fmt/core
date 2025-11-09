//! Tree Builder - Builds hierarchical LineContainer tree from LineTokens
//!
//! This module builds hierarchical tree structure from already-grouped and classified
//! LineTokens. The grouping and classification now happens in the lexing pipeline
//! (LineTokenGroupingMapper transformation).
//!
//! # Responsibilities
//!
//! 1. Build hierarchical tree based on Indent/Dedent markers
//! 2. Convert to LineContainer structure expected by parser

use crate::lex::lexing::tokens_linebased::{LineContainer, LineToken, LineType};

/// Build a LineContainer tree from already-grouped LineTokens.
///
/// This is the main entry point that builds hierarchical structure from
/// line tokens that have already been grouped and classified by the
///lexing pipeline.
///
/// # Arguments
///
/// * `line_tokens` - Vector of LineTokens from lexing pipeline
///
/// # Returns
///
/// A LineContainer tree ready for the line-based parser
pub fn build_line_container(line_tokens: Vec<LineToken>) -> LineContainer {
    // Build hierarchical structure
    let children = build_hierarchy(line_tokens);

    // Wrap in root container
    LineContainer::Container { children }
}

/// Build hierarchical structure from flat list of LineTokens based on Indent/Dedent.
///
/// This implements the logic from IndentationToTreeMapper:
/// - Maintains a stack of nesting levels
/// - Indent starts a new nested level
/// - Dedent closes current level and attaches to parent
/// - Regular lines accumulate at current level
fn build_hierarchy(line_tokens: Vec<LineToken>) -> Vec<LineContainer> {
    // Stack of pending children at each indentation level
    let mut stack: Vec<Vec<LineContainer>> = vec![Vec::new()];
    let mut pending_containers: Vec<LineContainer> = Vec::new();

    for line_token in line_tokens {
        match line_token.line_type {
            LineType::Indent => {
                // Flush pending containers before entering nested level
                if !pending_containers.is_empty() {
                    let current_level = stack.last_mut().expect("Stack never empty");
                    current_level.append(&mut pending_containers);
                }
                // Start a new nesting level
                stack.push(Vec::new());
            }
            LineType::Dedent => {
                // Flush pending containers before closing level
                if !pending_containers.is_empty() {
                    let current_level = stack.last_mut().expect("Stack never empty");
                    current_level.append(&mut pending_containers);
                }
                // Close current level and attach as children to last container in parent
                if let Some(children_containers) = stack.pop() {
                    let parent_level = stack.last_mut().expect("Stack never empty");

                    if let Some(last_parent) = parent_level.last_mut() {
                        // Attach the nested children to the last parent container
                        // If the parent is already a Container, extend its children
                        // Otherwise, we need to convert Token to a structure that can hold children
                        match last_parent {
                            LineContainer::Container { ref mut children } => {
                                children.extend(children_containers);
                            }
                            LineContainer::Token(_) => {
                                // Need to replace Token with [Token, Container]
                                // Extract the token first
                                let token = std::mem::replace(
                                    last_parent,
                                    LineContainer::Container {
                                        children: Vec::new(),
                                    },
                                );
                                // Now last_parent is a placeholder Container
                                // Replace it with the proper sequence
                                *last_parent = token; // Put token back temporarily

                                // We need to insert a container after this token
                                // So we'll actually just append it to parent_level
                                parent_level.push(LineContainer::Container {
                                    children: children_containers,
                                });
                            }
                        }
                    } else {
                        // If no parent exists, create a container node to hold children
                        parent_level.push(LineContainer::Container {
                            children: children_containers,
                        });
                    }
                }
            }
            _ => {
                // Accumulate regular line tokens at current level
                pending_containers.push(LineContainer::Token(line_token));
            }
        }
    }

    // Flush any remaining pending containers at root level
    if !pending_containers.is_empty() {
        let root_level = stack.last_mut().expect("Stack never empty");
        root_level.append(&mut pending_containers);
    }

    // Return the root level containers
    stack.pop().expect("Stack should contain root level")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::lexing::tokens_core::Token;

    #[allow(clippy::single_range_in_vec_init)]
    #[test]
    fn test_build_hierarchy_simple() {
        // Test with already-grouped LineTokens
        let line_tokens = vec![LineToken {
            source_tokens: vec![
                Token::Text("Hello".to_string()),
                Token::Whitespace,
                Token::Text("world".to_string()),
                Token::Newline,
            ],
            token_spans: vec![0..5, 5..6, 6..11, 11..12],
            line_type: LineType::ParagraphLine,
        }];

        let container = build_line_container(line_tokens);

        match container {
            LineContainer::Container { children } => {
                assert_eq!(children.len(), 1);
                match &children[0] {
                    LineContainer::Token(line_token) => {
                        assert_eq!(line_token.line_type, LineType::ParagraphLine);
                        assert_eq!(line_token.source_tokens.len(), 4);
                    }
                    _ => panic!("Expected Token"),
                }
            }
            _ => panic!("Expected Container at root"),
        }
    }

    #[allow(clippy::single_range_in_vec_init)]
    #[test]
    fn test_build_hierarchy_with_indentation() {
        // Test hierarchy building with Indent/Dedent markers
        let line_tokens = vec![
            LineToken {
                source_tokens: vec![
                    Token::Text("Title".to_string()),
                    Token::Colon,
                    Token::Newline,
                ],
                token_spans: vec![0..5, 5..6, 6..7],
                line_type: LineType::SubjectLine,
            },
            LineToken {
                source_tokens: vec![Token::Indentation],
                token_spans: vec![7..11],
                line_type: LineType::Indent,
            },
            LineToken {
                source_tokens: vec![Token::Text("Content".to_string()), Token::Newline],
                token_spans: vec![11..18, 18..19],
                line_type: LineType::ParagraphLine,
            },
            LineToken {
                source_tokens: vec![Token::Dedent(vec![])],
                token_spans: vec![0..0],
                line_type: LineType::Dedent,
            },
        ];

        let container = build_line_container(line_tokens);

        match container {
            LineContainer::Container { children } => {
                // Should have title token and then a container with the indented content
                assert!(!children.is_empty());

                // First child should be the title
                match &children[0] {
                    LineContainer::Token(line_token) => {
                        assert_eq!(line_token.line_type, LineType::SubjectLine);
                    }
                    _ => panic!("Expected Token for title"),
                }
            }
            _ => panic!("Expected Container at root"),
        }
    }
}
