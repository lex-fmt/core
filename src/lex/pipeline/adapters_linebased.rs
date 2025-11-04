//! LineBased-specific adapters for TokenStream ↔ LineContainer conversion
//!
//! This module provides adapters for the linebased lexer and parser to work
//! with the unified TokenStream architecture.
//!
//! # Adapters
//!
//! - `line_container_to_token_stream()` - Convert LineContainer to TokenStream::Tree
//! - `token_stream_to_line_tokens()` - Convert TokenStream::Tree to Vec<LineToken>
//! - `token_stream_to_line_container()` - Convert TokenStream::Tree to LineContainer
//! - `adapt_linebased_parser()` - Wrapper for linebased parser using TokenStream input
//!
//! # Architectural Boundary
//!
//! These adapters exist at the lexer → parser boundary:
//! - **Lexer output**: TokenStream::Tree (from transformation pipeline)
//! - **Parser input**: LineContainer (semantic domain types)
//!
//! The transformation pipeline itself is adapter-free and works purely on TokenStream.

use crate::lex::pipeline::adapters::AdapterError;
use crate::lex::pipeline::stream::{TokenStream, TokenStreamNode};

/// Convert a LineContainer to a TokenStream::Tree.
///
/// This adapter converts the linebased lexer's hierarchical LineContainer structure
/// to the unified TokenStream::Tree format.
///
/// LineContainer structure:
/// - `LineContainer::Token(LineToken)` - A single line with tokens
/// - `LineContainer::Container { children }` - A container with nested children
///
/// Maps to TokenStream::Tree where each LineToken becomes a TokenStreamNode.
///
/// # Arguments
///
/// * `container` - The LineContainer to convert
///
/// # Returns
///
/// A TokenStream::Tree representing the hierarchical structure
///
/// # Examples
///
/// ```ignore
/// let line_container = LineContainer::Token(line_token);
/// let stream = line_container_to_token_stream(line_container);
/// assert!(matches!(stream, TokenStream::Tree(_)));
/// ```
pub fn line_container_to_token_stream(container: crate::lex::lexers::LineContainer) -> TokenStream {
    use crate::lex::lexers::LineContainer;

    fn convert_node(lc: LineContainer) -> TokenStreamNode {
        match lc {
            LineContainer::Token(line_token) => {
                // Convert LineToken to TokenStreamNode, preserving LineType
                let tokens = line_token.source_token_pairs();
                let line_type = Some(line_token.line_type);
                TokenStreamNode {
                    tokens,
                    children: None,
                    line_type,
                }
            }
            LineContainer::Container { children } => {
                // Container becomes a node with empty tokens and children as Tree
                // Containers don't have a LineType (they're structural, not classified lines)
                let child_nodes: Vec<TokenStreamNode> =
                    children.into_iter().map(convert_node).collect();
                TokenStreamNode {
                    tokens: vec![],
                    children: Some(Box::new(TokenStream::Tree(child_nodes))),
                    line_type: None,
                }
            }
        }
    }

    // Normalize root handling: treat both Token and Container uniformly
    // If the input is a single Token, treat it as a Container with one child
    // This makes round-trip conversion symmetrical with token_stream_to_line_container
    let children = match container {
        LineContainer::Token(line_token) => vec![LineContainer::Token(line_token)],
        LineContainer::Container { children } => children,
    };

    // Convert all children to TokenStreamNodes
    let nodes: Vec<TokenStreamNode> = children.into_iter().map(convert_node).collect();
    TokenStream::Tree(nodes)
}

/// Convert a TokenStream::Tree to a flat vector of LineTokens.
///
/// This adapter converts a TokenStream::Tree (shallow, from ToLineTokensMapper)
/// back to the Vec<LineToken> format for backward compatibility.
///
/// For safety during migration, this function rejects Flat variants and deeply
/// nested trees (only accepts shallow trees where each node is a line).
///
/// # Arguments
///
/// * `stream` - The TokenStream to convert (must be shallow Tree variant)
///
/// # Returns
///
/// The Vec<LineToken> structure
///
/// # Errors
///
/// Returns `AdapterError::ExpectedTree` if stream is Flat variant.
///
/// # Examples
///
/// ```ignore
/// let stream = TokenStream::Tree(vec![node]);
/// let line_tokens = token_stream_to_line_tokens(stream)?;
/// ```
pub fn token_stream_to_line_tokens(
    stream: TokenStream,
) -> Result<Vec<crate::lex::lexers::linebased::tokens_linebased::LineToken>, AdapterError> {
    use crate::lex::lexers::linebased::tokens_linebased::{LineToken, LineType};

    match stream {
        TokenStream::Flat(_) => Err(AdapterError::ExpectedTree),
        TokenStream::Tree(nodes) => {
            let line_tokens: Vec<LineToken> = nodes
                .into_iter()
                .map(|node| {
                    let (source_tokens, token_spans): (Vec<_>, Vec<_>) =
                        node.tokens.into_iter().unzip();

                    // Use the preserved LineType, or default to ParagraphLine if not set
                    let line_type = node.line_type.unwrap_or(LineType::ParagraphLine);

                    LineToken {
                        source_tokens,
                        token_spans,
                        line_type,
                    }
                })
                .collect();

            Ok(line_tokens)
        }
    }
}

/// Convert a TokenStream::Tree back to a LineContainer.
///
/// This adapter converts the unified TokenStream::Tree back to the linebased
/// lexer's LineContainer format.
///
/// For safety during migration, this function rejects Flat variants.
/// Use `flatten_token_stream()` if you need to explicitly flatten first.
///
/// # Arguments
///
/// * `stream` - The TokenStream to convert (must be Tree variant)
///
/// # Returns
///
/// The LineContainer structure
///
/// # Errors
///
/// Returns `AdapterError::ExpectedTree` if stream is Flat variant.
///
/// # Examples
///
/// ```ignore
/// let stream = TokenStream::Tree(vec![node]);
/// let container = token_stream_to_line_container(stream)?;
/// ```
pub fn token_stream_to_line_container(
    stream: TokenStream,
) -> Result<crate::lex::lexers::LineContainer, AdapterError> {
    use crate::lex::lexers::linebased::tokens_linebased::{LineContainer, LineToken, LineType};

    match stream {
        TokenStream::Flat(_) => Err(AdapterError::ExpectedTree),
        TokenStream::Tree(nodes) => {
            // Convert nodes into LineContainers, handling the special case where
            // a node has both tokens and children (which needs to become [Token, Container])
            let mut result_children = Vec::new();

            for node in nodes {
                if node.tokens.is_empty() && node.children.is_some() {
                    // This was a pure Container node (no tokens of its own)
                    let children_stream = node.children.unwrap();
                    match *children_stream {
                        TokenStream::Tree(child_nodes) => {
                            // Recursively convert children
                            let child_result =
                                token_stream_to_line_container(TokenStream::Tree(child_nodes))?;
                            result_children.push(child_result);
                        }
                        TokenStream::Flat(_) => {
                            // Shouldn't happen in well-formed tree
                            result_children.push(LineContainer::Container { children: vec![] });
                        }
                    }
                } else if node.children.is_some() {
                    // This node has BOTH tokens AND children
                    // In LineContainer, we need to add Token and then Container as siblings
                    let (source_tokens, token_spans): (Vec<_>, Vec<_>) =
                        node.tokens.into_iter().unzip();
                    let line_type = node.line_type.unwrap_or(LineType::ParagraphLine);
                    let line_token = LineToken {
                        source_tokens,
                        token_spans,
                        line_type,
                    };

                    // Add the token first
                    result_children.push(LineContainer::Token(line_token));

                    // Then convert and add the children container
                    let children_stream = node.children.unwrap();
                    match *children_stream {
                        TokenStream::Tree(child_nodes) => {
                            let child_result =
                                token_stream_to_line_container(TokenStream::Tree(child_nodes))?;
                            result_children.push(child_result);
                        }
                        TokenStream::Flat(_) => {
                            result_children.push(LineContainer::Container { children: vec![] });
                        }
                    }
                } else {
                    // This node has only tokens, no children - simple Token
                    let (source_tokens, token_spans): (Vec<_>, Vec<_>) =
                        node.tokens.into_iter().unzip();
                    let line_type = node.line_type.unwrap_or(LineType::ParagraphLine);
                    let line_token = LineToken {
                        source_tokens,
                        token_spans,
                        line_type,
                    };
                    result_children.push(LineContainer::Token(line_token));
                }
            }

            // Always wrap in a Container at the root level
            // The linebased parser expects the root to be a Container
            Ok(LineContainer::Container {
                children: result_children,
            })
        }
    }
}

/// Adapt the linebased parser to work with TokenStream input.
///
/// This function integrates the linebased parser into the TokenStream architecture
/// by adapting the input from `TokenStream::Tree` to `LineContainer`.
///
/// The linebased parser expects a hierarchical LineContainer structure, so this adapter:
/// 1. Converts TokenStream::Tree to LineContainer (safe, rejects Flat)
/// 2. Calls the linebased parser (parse_experimental_v2)
/// 3. Returns the Document (no output adapter needed - Document is final)
///
/// This leverages token_processing.rs which already provides the LineContainer → tokens
/// conversion for AST building. Once all transformations use TokenStream, this adapter
/// can be removed and the parser can consume TokenStream directly.
///
/// # Arguments
///
/// * `stream` - The TokenStream to parse (must be Tree variant)
/// * `source` - The original source text for location tracking
///
/// # Returns
///
/// The parsed Document or an adapter error
///
/// # Errors
///
/// Returns `AdapterError::ExpectedTree` if stream is Flat variant.
/// Returns `AdapterError::Error` if parsing fails.
///
/// # Examples
///
/// ```ignore
/// let stream = TokenStream::Tree(vec![node]);
/// let doc = adapt_linebased_parser(stream, "hello")?;
/// ```
pub fn adapt_linebased_parser(
    stream: TokenStream,
    source: &str,
) -> Result<crate::lex::parsers::Document, AdapterError> {
    // Adapt input: TokenStream::Tree -> LineContainer
    let container = token_stream_to_line_container(stream)?;

    // Call linebased parser
    crate::lex::parsers::linebased::parse_experimental_v2(container, source)
        .map_err(|error| AdapterError::Error(format!("LineBased parsing failed: {}", error)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::lexers::tokens_core::Token;

    // Tree adapter tests
    #[test]
    fn test_line_container_to_token_stream_single_token() {
        use crate::lex::lexers::linebased::tokens_linebased::{LineContainer, LineToken, LineType};

        // Create a simple LineContainer with one token
        #[allow(clippy::single_range_in_vec_init)]
        let line_token = LineToken {
            source_tokens: vec![Token::Text("hello".to_string())],
            token_spans: vec![0..5], // Single range in vec - intentional for LineToken
            line_type: LineType::ParagraphLine,
        };
        let container = LineContainer::Token(line_token);

        let stream = line_container_to_token_stream(container);

        match stream {
            TokenStream::Tree(nodes) => {
                assert_eq!(nodes.len(), 1);
                assert_eq!(nodes[0].tokens.len(), 1);
                assert_eq!(nodes[0].tokens[0].0, Token::Text("hello".to_string()));
                assert!(nodes[0].children.is_none());
            }
            _ => panic!("Expected TokenStream::Tree"),
        }
    }

    #[test]
    fn test_line_container_to_token_stream_container() {
        use crate::lex::lexers::linebased::tokens_linebased::{LineContainer, LineToken, LineType};

        // Create a container with multiple children
        #[allow(clippy::single_range_in_vec_init)]
        let child1 = LineContainer::Token(LineToken {
            source_tokens: vec![Token::Text("line1".to_string())],
            token_spans: vec![0..5], // Single range - intentional
            line_type: LineType::ParagraphLine,
        });
        #[allow(clippy::single_range_in_vec_init)]
        let child2 = LineContainer::Token(LineToken {
            source_tokens: vec![Token::Text("line2".to_string())],
            token_spans: vec![6..11], // Single range - intentional
            line_type: LineType::ParagraphLine,
        });

        let container = LineContainer::Container {
            children: vec![child1, child2],
        };

        let stream = line_container_to_token_stream(container);

        match stream {
            TokenStream::Tree(nodes) => {
                assert_eq!(nodes.len(), 2);
                assert_eq!(nodes[0].tokens[0].0, Token::Text("line1".to_string()));
                assert_eq!(nodes[1].tokens[0].0, Token::Text("line2".to_string()));
            }
            _ => panic!("Expected TokenStream::Tree"),
        }
    }

    #[test]
    fn test_token_stream_to_line_container_simple() {
        let node = TokenStreamNode {
            tokens: vec![(Token::Text("hello".to_string()), 0..5)],
            children: None,
            line_type: None,
        };
        let stream = TokenStream::Tree(vec![node]);

        let result = token_stream_to_line_container(stream);
        assert!(result.is_ok());

        let container = result.unwrap();
        // Result is always wrapped in Container at root
        match container {
            crate::lex::lexers::LineContainer::Container { children } => {
                assert_eq!(children.len(), 1);
                match &children[0] {
                    crate::lex::lexers::LineContainer::Token(line_token) => {
                        assert_eq!(line_token.source_tokens.len(), 1);
                        assert_eq!(
                            line_token.source_tokens[0],
                            Token::Text("hello".to_string())
                        );
                    }
                    _ => panic!("Expected LineContainer::Token inside"),
                }
            }
            _ => panic!("Expected LineContainer::Container at root"),
        }
    }

    #[test]
    fn test_token_stream_to_line_container_rejects_flat() {
        let stream = TokenStream::Flat(vec![(Token::Text("test".to_string()), 0..4)]);
        let result = token_stream_to_line_container(stream);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AdapterError::ExpectedTree);
    }

    #[test]
    fn test_tree_adapter_round_trip() {
        use crate::lex::lexers::linebased::tokens_linebased::{LineContainer, LineToken, LineType};

        // Create original LineContainer (wrapped in Container as root)
        #[allow(clippy::single_range_in_vec_init)]
        let original = LineContainer::Container {
            children: vec![LineContainer::Token(LineToken {
                source_tokens: vec![
                    Token::Text("hello".to_string()),
                    Token::Whitespace,
                    Token::Text("world".to_string()),
                ],
                token_spans: vec![0..5, 5..6, 6..11],
                line_type: LineType::ParagraphLine,
            })],
        };

        // Convert to TokenStream
        let stream = line_container_to_token_stream(original.clone());

        // Convert back to LineContainer
        let result = token_stream_to_line_container(stream).unwrap();

        // Verify tokens match (note: LineType will be ParagraphLine as default)
        match (original, result) {
            (
                LineContainer::Container {
                    children: orig_children,
                },
                LineContainer::Container {
                    children: res_children,
                },
            ) => {
                assert_eq!(orig_children.len(), res_children.len());
                match (&orig_children[0], &res_children[0]) {
                    (LineContainer::Token(orig), LineContainer::Token(converted)) => {
                        assert_eq!(orig.source_tokens, converted.source_tokens);
                        assert_eq!(orig.token_spans, converted.token_spans);
                    }
                    _ => panic!("Expected both to contain Token"),
                }
            }
            _ => panic!("Expected both to be Container variants"),
        }
    }

    #[test]
    fn test_tree_adapter_preserves_ranges() {
        use crate::lex::lexers::linebased::tokens_linebased::{LineContainer, LineToken, LineType};

        #[allow(clippy::single_range_in_vec_init)]
        let line_token = LineToken {
            source_tokens: vec![Token::Text("test".to_string()), Token::Newline],
            token_spans: vec![0..4, 4..5],
            line_type: LineType::ParagraphLine,
        };
        let container = LineContainer::Container {
            children: vec![LineContainer::Token(line_token)],
        };

        let stream = line_container_to_token_stream(container);
        let result = token_stream_to_line_container(stream).unwrap();

        match result {
            crate::lex::lexers::LineContainer::Container { children } => match &children[0] {
                crate::lex::lexers::LineContainer::Token(line_token) => {
                    assert_eq!(line_token.token_spans[0], 0..4);
                    assert_eq!(line_token.token_spans[1], 4..5);
                }
                _ => panic!("Expected Token inside Container"),
            },
            _ => panic!("Expected Container at root"),
        }
    }

    #[test]
    fn test_tree_adapter_nested_structure() {
        use crate::lex::lexers::linebased::tokens_linebased::{LineContainer, LineToken, LineType};

        // Create nested structure: container with children
        #[allow(clippy::single_range_in_vec_init)]
        let child = LineContainer::Token(LineToken {
            source_tokens: vec![Token::Text("child".to_string())],
            token_spans: vec![10..15], // Single range - intentional
            line_type: LineType::ParagraphLine,
        });

        let parent = LineContainer::Container {
            children: vec![child],
        };

        let stream = line_container_to_token_stream(parent);

        match stream {
            TokenStream::Tree(nodes) => {
                // Container becomes one node in the tree
                assert_eq!(nodes.len(), 1);
                // The node represents the child token
                assert_eq!(nodes[0].tokens.len(), 1);
                assert_eq!(nodes[0].tokens[0].0, Token::Text("child".to_string()));
            }
            _ => panic!("Expected TokenStream::Tree"),
        }
    }

    // LineBased parser adapter tests
    #[test]
    fn test_adapt_linebased_parser_simple() {
        use crate::lex::lexers::linebased::tokens_linebased::{LineContainer, LineToken, LineType};

        // Create a simple paragraph
        let source = "Hello world\n";

        #[allow(clippy::single_range_in_vec_init)]
        let line_token = LineToken {
            source_tokens: vec![
                Token::Text("Hello".to_string()),
                Token::Whitespace,
                Token::Text("world".to_string()),
                Token::Newline,
            ],
            token_spans: vec![0..5, 5..6, 6..11, 11..12],
            line_type: LineType::ParagraphLine,
        };

        let container = LineContainer::Container {
            children: vec![LineContainer::Token(line_token)],
        };

        // Convert to TokenStream
        let stream = line_container_to_token_stream(container);

        // Parse through adapter
        let result = adapt_linebased_parser(stream, source);

        assert!(result.is_ok(), "Failed to parse: {:?}", result);
        let doc = result.unwrap();
        assert_eq!(doc.root.content.len(), 1); // Should have one paragraph
    }

    #[test]
    fn test_adapt_linebased_parser_rejects_flat() {
        // Flat streams should be rejected
        let stream = TokenStream::Flat(vec![(Token::Text("test".to_string()), 0..4)]);

        let result = adapt_linebased_parser(stream, "test");

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AdapterError::ExpectedTree);
    }

    #[test]
    fn test_adapt_linebased_parser_round_trip() {
        use crate::lex::lexers::linebased::tokens_linebased::{LineContainer, LineToken, LineType};

        // Create a simple structure
        let source = "Paragraph text\n";

        #[allow(clippy::single_range_in_vec_init)]
        let line_token = LineToken {
            source_tokens: vec![
                Token::Text("Paragraph".to_string()),
                Token::Whitespace,
                Token::Text("text".to_string()),
                Token::Newline,
            ],
            token_spans: vec![0..9, 9..10, 10..14, 14..15],
            line_type: LineType::ParagraphLine,
        };

        let container = LineContainer::Container {
            children: vec![LineContainer::Token(line_token)],
        };

        // Original path
        let doc1 = crate::lex::parsers::linebased::parse_experimental_v2(container.clone(), source)
            .unwrap();

        // TokenStream path
        let stream = line_container_to_token_stream(container);
        let doc2 = adapt_linebased_parser(stream, source).unwrap();

        // Both should produce the same number of items
        assert_eq!(doc1.root.content.len(), doc2.root.content.len());
    }

    // LineType preservation tests
    #[test]
    fn test_line_type_preservation_paragraph() {
        use crate::lex::lexers::linebased::tokens_linebased::{LineContainer, LineToken, LineType};

        #[allow(clippy::single_range_in_vec_init)]
        let line_token = LineToken {
            source_tokens: vec![Token::Text("hello".to_string())],
            token_spans: vec![0..5],
            line_type: LineType::ParagraphLine,
        };

        let container = LineContainer::Token(line_token);

        // Convert to TokenStream
        let stream = line_container_to_token_stream(container);

        // Verify LineType is preserved in TokenStream
        match stream {
            TokenStream::Tree(nodes) => {
                assert_eq!(nodes.len(), 1);
                assert_eq!(nodes[0].line_type, Some(LineType::ParagraphLine));
            }
            _ => panic!("Expected Tree variant"),
        }
    }

    #[test]
    fn test_line_type_preservation_subject() {
        use crate::lex::lexers::linebased::tokens_linebased::{LineContainer, LineToken, LineType};

        #[allow(clippy::single_range_in_vec_init)]
        let line_token = LineToken {
            source_tokens: vec![Token::Text("Title:".to_string())],
            token_spans: vec![0..6],
            line_type: LineType::SubjectLine,
        };

        let container = LineContainer::Token(line_token);

        // Convert to TokenStream
        let stream = line_container_to_token_stream(container);

        // Verify SubjectLine is preserved
        match stream {
            TokenStream::Tree(nodes) => {
                assert_eq!(nodes.len(), 1);
                assert_eq!(nodes[0].line_type, Some(LineType::SubjectLine));
            }
            _ => panic!("Expected Tree variant"),
        }
    }

    #[test]
    fn test_line_type_preservation_list() {
        use crate::lex::lexers::linebased::tokens_linebased::{LineContainer, LineToken, LineType};

        #[allow(clippy::single_range_in_vec_init)]
        let line_token = LineToken {
            source_tokens: vec![Token::Text("- item".to_string())],
            token_spans: vec![0..6],
            line_type: LineType::ListLine,
        };

        let container = LineContainer::Token(line_token);

        // Convert to TokenStream
        let stream = line_container_to_token_stream(container);

        // Verify ListLine is preserved
        match stream {
            TokenStream::Tree(nodes) => {
                assert_eq!(nodes.len(), 1);
                assert_eq!(nodes[0].line_type, Some(LineType::ListLine));
            }
            _ => panic!("Expected Tree variant"),
        }
    }

    #[test]
    fn test_line_type_round_trip_preservation() {
        use crate::lex::lexers::linebased::tokens_linebased::{LineContainer, LineToken, LineType};

        // Test all LineType variants round-trip correctly
        let line_types = vec![
            LineType::ParagraphLine,
            LineType::SubjectLine,
            LineType::ListLine,
            LineType::BlankLine,
            LineType::AnnotationStartLine,
            LineType::AnnotationEndLine,
            LineType::SubjectOrListItemLine,
        ];

        for original_line_type in line_types {
            #[allow(clippy::single_range_in_vec_init)]
            let line_token = LineToken {
                source_tokens: vec![Token::Text("test".to_string())],
                token_spans: vec![0..4],
                line_type: original_line_type,
            };

            let original_container = LineContainer::Token(line_token);

            // Convert to TokenStream
            let stream = line_container_to_token_stream(original_container);

            // Convert back to LineContainer
            let result_container = token_stream_to_line_container(stream).unwrap();

            // Extract the LineType from the result
            match result_container {
                LineContainer::Container { children } => {
                    assert_eq!(children.len(), 1);
                    match &children[0] {
                        LineContainer::Token(line_token) => {
                            assert_eq!(
                                line_token.line_type, original_line_type,
                                "LineType {:?} was not preserved during round trip",
                                original_line_type
                            );
                        }
                        _ => panic!("Expected Token variant"),
                    }
                }
                _ => panic!("Expected Container at root"),
            }
        }
    }

    #[test]
    fn test_line_type_container_has_none() {
        use crate::lex::lexers::linebased::tokens_linebased::{LineContainer, LineToken, LineType};

        // When a Container has nested Containers, those inner containers should have line_type = None
        #[allow(clippy::single_range_in_vec_init)]
        let line_token = LineToken {
            source_tokens: vec![Token::Text("hello".to_string())],
            token_spans: vec![0..5],
            line_type: LineType::ParagraphLine,
        };

        // Create nested containers: Container -> Container -> Token
        let inner_container = LineContainer::Container {
            children: vec![LineContainer::Token(line_token)],
        };

        let outer_container = LineContainer::Container {
            children: vec![inner_container],
        };

        // Convert to TokenStream
        let stream = line_container_to_token_stream(outer_container);

        // Verify structure
        match stream {
            TokenStream::Tree(nodes) => {
                assert_eq!(nodes.len(), 1);
                // The container node should have line_type = None
                assert_eq!(nodes[0].line_type, None);
                assert_eq!(nodes[0].tokens.len(), 0); // Container nodes have no tokens

                // The inner child should have the ParagraphLine type
                match &nodes[0].children {
                    Some(children_stream) => match &**children_stream {
                        TokenStream::Tree(child_nodes) => {
                            assert_eq!(child_nodes.len(), 1);
                            assert_eq!(child_nodes[0].line_type, Some(LineType::ParagraphLine));
                        }
                        _ => panic!("Expected Tree children"),
                    },
                    None => panic!("Expected children for container"),
                }
            }
            _ => panic!("Expected Tree variant"),
        }
    }
}
