//! Tree Builder - Builds hierarchical LineContainer tree from LineTokens
//!
//! This module builds a hierarchical tree structure from a flat list of classified
//! LineTokens. It uses a recursive descent approach to handle indentation.
//!
//! # Responsibilities
//!
//! 1. Build hierarchical tree based on Indent/Dedent markers
//! 2. Convert to LineContainer structure expected by parser

use crate::lex::token::{LineContainer, LineToken, LineType};
use std::iter::Peekable;

/// Build a LineContainer tree from already-grouped LineTokens.
///
/// This is the main entry point that builds a hierarchical structure from
/// line tokens that have already been grouped and classified by the
/// lexing pipeline.
///
/// # Arguments
///
/// * `line_tokens` - Vector of LineTokens from the lexing pipeline
///
/// # Returns
///
/// A LineContainer tree ready for the line-based parser
pub fn build_line_container(line_tokens: Vec<LineToken>) -> LineContainer {
    let mut tokens_iter = line_tokens.into_iter().peekable();
    let mut children = build_recursive(&mut tokens_iter, None);

    // Inject parent blank marker at document start if first element is not blank/indent
    inject_parent_blank_marker_at_start(&mut children);

    LineContainer::Container { children }
}

/// Recursively build a hierarchy of LineContainers from a stream of LineTokens.
///
/// This function processes tokens at the current indentation level. When it encounters
/// an `Indent`, it recursively calls itself to build a nested `Container`. It stops
/// processing the current level when it sees a `Dedent` (which belongs to the parent
/// level) or when the token stream is exhausted.
///
/// # Parameters
///
/// * `tokens` - The iterator of line tokens
/// * `parent_last_blank` - The last BlankLineGroup token from the parent level (if any)
fn build_recursive<I>(
    tokens: &mut Peekable<I>,
    parent_last_blank: Option<LineToken>,
) -> Vec<LineContainer>
where
    I: Iterator<Item = LineToken>,
{
    let mut children = Vec::new();
    let mut last_blank: Option<LineToken> = parent_last_blank;

    while let Some(token) = tokens.peek() {
        match token.line_type {
            LineType::Indent => {
                tokens.next(); // Consume Indent token
                let mut indented_children = build_recursive(tokens, last_blank.clone());

                // Inject parent blank marker if we have a last blank from this level
                if last_blank.is_some() {
                    inject_parent_blank_marker_at_start(&mut indented_children);
                }

                children.push(LineContainer::Container {
                    children: indented_children,
                });
            }
            LineType::Dedent => {
                // This Dedent signifies the end of the current level.
                // Consume it and return to the parent level.
                tokens.next();
                return children;
            }
            LineType::BlankLine => {
                // Track the last blank line token for potential injection into nested containers
                last_blank = Some(token.clone());
                if let Some(t) = tokens.next() {
                    children.push(LineContainer::Token(t));
                }
            }
            _ => {
                // Regular token, consume and add to the current level's children.
                if let Some(t) = tokens.next() {
                    children.push(LineContainer::Token(t));
                }
            }
        }
    }

    children
}

/// Inject a parent blank marker at the start of a container's children.
///
/// This function checks if the first token is not already a blank line or indent,
/// and if so, injects a ParentBlankMarker at the beginning.
fn inject_parent_blank_marker_at_start(children: &mut Vec<LineContainer>) {
    if children.is_empty() {
        return;
    }

    // Check if first child is a token and not already a blank/indent/parent-blank-marker
    let should_inject = match &children[0] {
        LineContainer::Token(t) => !matches!(
            t.line_type,
            LineType::BlankLine | LineType::Indent | LineType::ParentBlankMarker
        ),
        LineContainer::Container { .. } => false, // Don't inject before containers
    };

    if should_inject {
        let marker = LineToken {
            source_tokens: vec![],
            token_spans: vec![],
            line_type: LineType::ParentBlankMarker,
        };
        children.insert(0, LineContainer::Token(marker));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::token::Token;

    #[allow(clippy::single_range_in_vec_init)]
    #[test]
    fn test_build_hierarchy_simple() {
        // Test with already-grouped LineTokens
        let line_tokens = vec![LineToken {
            source_tokens: vec![
                Token::Text("Hello".to_string()),
                Token::Whitespace(1),
                Token::Text("world".to_string()),
                Token::BlankLine(Some("\n".to_string())),
            ],
            token_spans: vec![0..5, 5..6, 6..11, 11..12],
            line_type: LineType::ParagraphLine,
        }];

        let container = build_line_container(line_tokens);

        match container {
            LineContainer::Container { children } => {
                assert_eq!(
                    children.len(),
                    2,
                    "Should have ParentBlankMarker + ParagraphLine"
                );

                // First child should be ParentBlankMarker (injected at document start)
                match &children[0] {
                    LineContainer::Token(line_token) => {
                        assert_eq!(line_token.line_type, LineType::ParentBlankMarker);
                    }
                    _ => panic!("Expected ParentBlankMarker"),
                }

                // Second child should be the paragraph
                match &children[1] {
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
                    Token::BlankLine(Some("\n".to_string())),
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
                source_tokens: vec![
                    Token::Text("Content".to_string()),
                    Token::BlankLine(Some("\n".to_string())),
                ],
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

        // Expected structure: [ParentBlankMarker, Token(Title), Container([Token(Content)])]
        // ParentBlankMarker at document start, no ParentBlankMarker in nested container (no preceding blank)
        match container {
            LineContainer::Container { children } => {
                assert_eq!(
                    children.len(),
                    3,
                    "Should have three items at the root: ParentBlankMarker, title token, and content container"
                );

                // First child should be ParentBlankMarker (injected at document start)
                match &children[0] {
                    LineContainer::Token(line_token) => {
                        assert_eq!(line_token.line_type, LineType::ParentBlankMarker);
                    }
                    _ => panic!("Expected ParentBlankMarker"),
                }

                // Second child should be the title token
                match &children[1] {
                    LineContainer::Token(line_token) => {
                        assert_eq!(line_token.line_type, LineType::SubjectLine);
                    }
                    _ => panic!("Expected Token for title"),
                }

                // Third child should be the container for indented content
                match &children[2] {
                    LineContainer::Container {
                        children: nested_children,
                    } => {
                        assert_eq!(
                            nested_children.len(),
                            1,
                            "Nested container should have one item (no ParentBlankMarker because no preceding blank)"
                        );
                        match &nested_children[0] {
                            LineContainer::Token(line_token) => {
                                assert_eq!(line_token.line_type, LineType::ParagraphLine);
                            }
                            _ => panic!("Expected Token for content"),
                        }
                    }
                    _ => panic!("Expected Container for indented content"),
                }
            }
            _ => panic!("Expected Container at root"),
        }
    }
}
