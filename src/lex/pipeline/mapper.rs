//! Stream mapper trait and traversal infrastructure
//!
//! This module provides the visitor pattern infrastructure for transforming token streams.
//! The `StreamMapper` trait defines the interface for transformations, and the `walk_stream`
//! function handles the recursive traversal logic.
//!
//! # Design
//!
//! Transformations implement `StreamMapper` and provide logic for what to do at each node.
//! The walker (`walk_stream`) handles the complexity of traversing the tree structure,
//! allowing transformations to focus purely on their transformation logic.
//!
//! # Examples
//!
//! ```ignore
//! struct MyMapper;
//!
//! impl StreamMapper for MyMapper {
//!     fn map_flat(&mut self, tokens: Vec<(Token, Range<usize>)>) -> Result<TokenStream, TransformationError> {
//!         // Transform flat tokens
//!         Ok(TokenStream::Flat(tokens))
//!     }
//! }
//!
//! let stream = TokenStream::Flat(vec![...]);
//! let mut mapper = MyMapper;
//! let result = walk_stream(stream, &mut mapper)?;
//! ```

use crate::lex::lexers::tokens::Token;
use crate::lex::pipeline::stream::{TokenStream, TokenStreamNode};
use std::fmt;
use std::ops::Range as ByteRange;

/// Errors that can occur during transformation
#[derive(Debug, Clone, PartialEq)]
pub enum TransformationError {
    /// Generic transformation error with message
    Error(String),
}

impl fmt::Display for TransformationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransformationError::Error(msg) => write!(f, "Transformation error: {}", msg),
        }
    }
}

impl std::error::Error for TransformationError {}

/// Trait for transforming token streams using the visitor pattern.
///
/// Implementations provide logic for transforming tokens at different points
/// in the traversal. The walker handles the recursive traversal, calling these
/// methods at appropriate times.
///
/// # Methods
///
/// - `map_flat`: Transform a flat token stream
/// - `enter_node`: Called before visiting a node's children (pre-order)
/// - `exit_node`: Called after visiting a node's children (post-order)
///
/// All methods have default implementations that pass data through unchanged,
/// so implementations only need to override the methods they care about.
///
/// # Examples
///
/// ```ignore
/// // Simple mapper that counts tokens
/// struct TokenCounter {
///     count: usize,
/// }
///
/// impl StreamMapper for TokenCounter {
///     fn map_flat(&mut self, tokens: Vec<(Token, Range<usize>)>) -> Result<TokenStream, TransformationError> {
///         self.count += tokens.len();
///         Ok(TokenStream::Flat(tokens))
///     }
/// }
/// ```
pub trait StreamMapper {
    /// Transform a flat token stream.
    ///
    /// Called when the walker encounters a `TokenStream::Flat` variant.
    /// Default implementation passes tokens through unchanged.
    ///
    /// # Arguments
    ///
    /// * `tokens` - The flat list of tokens to transform
    ///
    /// # Returns
    ///
    /// The transformed token stream (can be flat or tree)
    fn map_flat(
        &mut self,
        tokens: Vec<(Token, ByteRange<usize>)>,
    ) -> Result<TokenStream, TransformationError> {
        Ok(TokenStream::Flat(tokens))
    }

    /// Called in pre-order (before visiting children).
    ///
    /// This method is called when entering a node, before its children are visited.
    /// Useful for transformations that need to modify node tokens before processing children.
    ///
    /// Default implementation passes the node through unchanged.
    ///
    /// # Arguments
    ///
    /// * `node` - The node being entered
    ///
    /// # Returns
    ///
    /// The potentially modified node
    fn enter_node(
        &mut self,
        node: TokenStreamNode,
    ) -> Result<TokenStreamNode, TransformationError> {
        Ok(node)
    }

    /// Called in post-order (after visiting children).
    ///
    /// This method is called when exiting a node, after its children have been visited.
    /// Useful for transformations that need to modify nodes based on their processed children.
    ///
    /// Default implementation passes the node through unchanged.
    ///
    /// # Arguments
    ///
    /// * `node` - The node being exited (with already-processed children)
    ///
    /// # Returns
    ///
    /// The potentially modified node
    fn exit_node(&mut self, node: TokenStreamNode) -> Result<TokenStreamNode, TransformationError> {
        Ok(node)
    }
}

/// Walk a token stream, applying a mapper's transformations.
///
/// This function drives the transformation process, handling all traversal logic.
/// It calls the mapper's methods at appropriate times during the traversal.
///
/// For `TokenStream::Flat`, calls `mapper.map_flat()`.
/// For `TokenStream::Tree`, recursively walks each node calling `enter_node`,
/// recursing on children, then calling `exit_node`.
///
/// # Arguments
///
/// * `stream` - The token stream to transform
/// * `mapper` - The mapper implementing the transformation logic
///
/// # Returns
///
/// The transformed token stream
///
/// # Examples
///
/// ```ignore
/// let stream = TokenStream::Flat(vec![(Token::Text("hello".into()), 0..5)]);
/// let mut mapper = MyMapper;
/// let result = walk_stream(stream, &mut mapper)?;
/// ```
pub fn walk_stream(
    stream: TokenStream,
    mapper: &mut impl StreamMapper,
) -> Result<TokenStream, TransformationError> {
    match stream {
        TokenStream::Flat(tokens) => mapper.map_flat(tokens),
        TokenStream::Tree(nodes) => {
            let new_nodes = nodes
                .into_iter()
                .map(|node| walk_node(node, mapper))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(TokenStream::Tree(new_nodes))
        }
    }
}

/// Walk a single node, applying mapper transformations.
///
/// Helper function for walking individual nodes in a tree. Called by `walk_stream`
/// when processing tree structures.
///
/// Process order:
/// 1. Call `mapper.enter_node(node)` (pre-order)
/// 2. If node has children, recursively walk them with `walk_stream`
/// 3. Call `mapper.exit_node(node)` (post-order)
///
/// # Arguments
///
/// * `node` - The node to walk
/// * `mapper` - The mapper implementing the transformation logic
///
/// # Returns
///
/// The transformed node
fn walk_node(
    node: TokenStreamNode,
    mapper: &mut impl StreamMapper,
) -> Result<TokenStreamNode, TransformationError> {
    // Pre-order: enter the node
    let mut node_after_enter = mapper.enter_node(node)?;

    // Recursively walk children if present
    if let Some(children_stream) = node_after_enter.children.take() {
        let new_children = walk_stream(*children_stream, mapper)?;
        node_after_enter.children = Some(Box::new(new_children));
    }

    // Post-order: exit the node
    let final_node = mapper.exit_node(node_after_enter)?;
    Ok(final_node)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test mapper that counts tokens
    struct TokenCounter {
        flat_count: usize,
        node_count: usize,
    }

    impl StreamMapper for TokenCounter {
        fn map_flat(
            &mut self,
            tokens: Vec<(Token, ByteRange<usize>)>,
        ) -> Result<TokenStream, TransformationError> {
            self.flat_count += tokens.len();
            Ok(TokenStream::Flat(tokens))
        }

        fn enter_node(
            &mut self,
            node: TokenStreamNode,
        ) -> Result<TokenStreamNode, TransformationError> {
            self.node_count += 1;
            Ok(node)
        }
    }

    // Test mapper that transforms text tokens
    struct TextTransformer;

    impl StreamMapper for TextTransformer {
        fn map_flat(
            &mut self,
            tokens: Vec<(Token, ByteRange<usize>)>,
        ) -> Result<TokenStream, TransformationError> {
            let transformed: Vec<_> = tokens
                .into_iter()
                .map(|(token, range)| match token {
                    Token::Text(s) => (Token::Text(s.to_uppercase()), range),
                    other => (other, range),
                })
                .collect();
            Ok(TokenStream::Flat(transformed))
        }
    }

    // Test mapper that adds tokens in post-order
    struct PostOrderMarker;

    impl StreamMapper for PostOrderMarker {
        fn exit_node(
            &mut self,
            mut node: TokenStreamNode,
        ) -> Result<TokenStreamNode, TransformationError> {
            // Add a marker token at the end
            node.tokens.push((Token::Text("_marked".to_string()), 0..0));
            Ok(node)
        }
    }

    #[test]
    fn test_walk_flat_stream() {
        let tokens = vec![
            (Token::Text("hello".to_string()), 0..5),
            (Token::Whitespace, 5..6),
            (Token::Text("world".to_string()), 6..11),
        ];
        let stream = TokenStream::Flat(tokens.clone());

        let mut counter = TokenCounter {
            flat_count: 0,
            node_count: 0,
        };
        let result = walk_stream(stream, &mut counter).unwrap();

        assert_eq!(counter.flat_count, 3);
        assert_eq!(counter.node_count, 0);
        assert_eq!(result, TokenStream::Flat(tokens));
    }

    #[test]
    fn test_walk_empty_flat_stream() {
        let stream = TokenStream::Flat(vec![]);
        let mut counter = TokenCounter {
            flat_count: 0,
            node_count: 0,
        };
        let result = walk_stream(stream, &mut counter).unwrap();

        assert_eq!(counter.flat_count, 0);
        assert_eq!(result, TokenStream::Flat(vec![]));
    }

    #[test]
    fn test_walk_tree_single_node_no_children() {
        let node = TokenStreamNode {
            tokens: vec![(Token::Text("hello".to_string()), 0..5)],
            children: None,
            line_type: None,
        };
        let stream = TokenStream::Tree(vec![node]);

        let mut counter = TokenCounter {
            flat_count: 0,
            node_count: 0,
        };
        let result = walk_stream(stream, &mut counter).unwrap();

        assert_eq!(counter.flat_count, 0);
        assert_eq!(counter.node_count, 1); // One node visited
        assert!(matches!(result, TokenStream::Tree(_)));
    }

    #[test]
    fn test_walk_tree_single_node_with_flat_children() {
        let node = TokenStreamNode {
            tokens: vec![(Token::Text("parent".to_string()), 0..6)],
            children: Some(Box::new(TokenStream::Flat(vec![(
                Token::Text("child".to_string()),
                10..15,
            )]))),
            line_type: None,
        };
        let stream = TokenStream::Tree(vec![node]);

        let mut counter = TokenCounter {
            flat_count: 0,
            node_count: 0,
        };
        let result = walk_stream(stream, &mut counter).unwrap();

        assert_eq!(counter.flat_count, 1); // Child flat stream
        assert_eq!(counter.node_count, 1); // Parent node
        assert!(matches!(result, TokenStream::Tree(_)));
    }

    #[test]
    fn test_walk_tree_nested_nodes() {
        // Create nested structure: root -> child -> grandchild
        let grandchild_stream =
            TokenStream::Flat(vec![(Token::Text("grandchild".to_string()), 20..30)]);

        let child_node = TokenStreamNode {
            tokens: vec![(Token::Text("child".to_string()), 10..15)],
            children: Some(Box::new(grandchild_stream)),
            line_type: None,
        };

        let root_node = TokenStreamNode {
            tokens: vec![(Token::Text("root".to_string()), 0..4)],
            children: Some(Box::new(TokenStream::Tree(vec![child_node]))),
            line_type: None,
        };

        let stream = TokenStream::Tree(vec![root_node]);

        let mut counter = TokenCounter {
            flat_count: 0,
            node_count: 0,
        };
        let result = walk_stream(stream, &mut counter).unwrap();

        assert_eq!(counter.flat_count, 1); // Grandchild flat stream
        assert_eq!(counter.node_count, 2); // Root and child nodes
        assert!(matches!(result, TokenStream::Tree(_)));
    }

    #[test]
    fn test_transform_flat_tokens() {
        let tokens = vec![
            (Token::Text("hello".to_string()), 0..5),
            (Token::Text("world".to_string()), 6..11),
        ];
        let stream = TokenStream::Flat(tokens);

        let mut transformer = TextTransformer;
        let result = walk_stream(stream, &mut transformer).unwrap();

        match result {
            TokenStream::Flat(transformed) => {
                assert_eq!(transformed.len(), 2);
                assert_eq!(transformed[0].0, Token::Text("HELLO".to_string()));
                assert_eq!(transformed[1].0, Token::Text("WORLD".to_string()));
            }
            _ => panic!("Expected flat stream"),
        }
    }

    #[test]
    fn test_transform_tree_tokens() {
        let node = TokenStreamNode {
            tokens: vec![(Token::Text("parent".to_string()), 0..6)],
            children: Some(Box::new(TokenStream::Flat(vec![(
                Token::Text("child".to_string()),
                10..15,
            )]))),
            line_type: None,
        };
        let stream = TokenStream::Tree(vec![node]);

        let mut transformer = TextTransformer;
        let result = walk_stream(stream, &mut transformer).unwrap();

        match result {
            TokenStream::Tree(nodes) => {
                assert_eq!(nodes.len(), 1);
                // Parent tokens are not in a flat stream, so not transformed by map_flat
                // Only the child flat stream is transformed
                if let Some(children) = &nodes[0].children {
                    match children.as_ref() {
                        TokenStream::Flat(child_tokens) => {
                            assert_eq!(child_tokens[0].0, Token::Text("CHILD".to_string()));
                        }
                        _ => panic!("Expected flat child stream"),
                    }
                }
            }
            _ => panic!("Expected tree stream"),
        }
    }

    #[test]
    fn test_post_order_modification() {
        let node = TokenStreamNode {
            tokens: vec![(Token::Text("hello".to_string()), 0..5)],
            children: None,
            line_type: None,
        };
        let stream = TokenStream::Tree(vec![node]);

        let mut marker = PostOrderMarker;
        let result = walk_stream(stream, &mut marker).unwrap();

        match result {
            TokenStream::Tree(nodes) => {
                assert_eq!(nodes.len(), 1);
                assert_eq!(nodes[0].tokens.len(), 2); // Original + marker
                assert_eq!(nodes[0].tokens[1].0, Token::Text("_marked".to_string()));
            }
            _ => panic!("Expected tree stream"),
        }
    }

    #[test]
    fn test_multiple_nodes_at_same_level() {
        let node1 = TokenStreamNode {
            tokens: vec![(Token::Text("first".to_string()), 0..5)],
            children: None,
            line_type: None,
        };
        let node2 = TokenStreamNode {
            tokens: vec![(Token::Text("second".to_string()), 6..12)],
            children: None,
            line_type: None,
        };
        let stream = TokenStream::Tree(vec![node1, node2]);

        let mut counter = TokenCounter {
            flat_count: 0,
            node_count: 0,
        };
        let result = walk_stream(stream, &mut counter).unwrap();

        assert_eq!(counter.node_count, 2); // Both nodes visited
        assert!(matches!(result, TokenStream::Tree(_)));
    }

    #[test]
    fn test_preserves_ranges() {
        let tokens = vec![
            (Token::Text("hello".to_string()), 0..5),
            (Token::Whitespace, 5..6),
        ];
        let stream = TokenStream::Flat(tokens.clone());

        let mut counter = TokenCounter {
            flat_count: 0,
            node_count: 0,
        };
        let result = walk_stream(stream, &mut counter).unwrap();

        match result {
            TokenStream::Flat(result_tokens) => {
                assert_eq!(result_tokens[0].1, 0..5);
                assert_eq!(result_tokens[1].1, 5..6);
            }
            _ => panic!("Expected flat stream"),
        }
    }

    #[test]
    fn test_empty_tree() {
        let stream = TokenStream::Tree(vec![]);
        let mut counter = TokenCounter {
            flat_count: 0,
            node_count: 0,
        };
        let result = walk_stream(stream, &mut counter).unwrap();

        assert_eq!(counter.node_count, 0);
        assert_eq!(result, TokenStream::Tree(vec![]));
    }

    #[test]
    fn test_error_propagation() {
        struct FailingMapper;

        impl StreamMapper for FailingMapper {
            fn map_flat(
                &mut self,
                _tokens: Vec<(Token, ByteRange<usize>)>,
            ) -> Result<TokenStream, TransformationError> {
                Err(TransformationError::Error("test error".to_string()))
            }
        }

        let stream = TokenStream::Flat(vec![(Token::Text("test".to_string()), 0..4)]);
        let mut mapper = FailingMapper;
        let result = walk_stream(stream, &mut mapper);

        assert!(result.is_err());
        match result {
            Err(TransformationError::Error(msg)) => assert_eq!(msg, "test error"),
            _ => panic!("Expected error"),
        }
    }

    #[test]
    fn test_error_propagation_from_enter_node() {
        struct FailingEnterMapper;

        impl StreamMapper for FailingEnterMapper {
            fn enter_node(
                &mut self,
                _node: TokenStreamNode,
            ) -> Result<TokenStreamNode, TransformationError> {
                Err(TransformationError::Error("enter_node error".to_string()))
            }
        }

        let node = TokenStreamNode {
            tokens: vec![(Token::Text("test".to_string()), 0..4)],
            children: None,
            line_type: None,
        };
        let stream = TokenStream::Tree(vec![node]);
        let mut mapper = FailingEnterMapper;
        let result = walk_stream(stream, &mut mapper);

        assert!(result.is_err());
        match result {
            Err(TransformationError::Error(msg)) => assert_eq!(msg, "enter_node error"),
            _ => panic!("Expected error from enter_node"),
        }
    }

    #[test]
    fn test_error_propagation_from_exit_node() {
        struct FailingExitMapper;

        impl StreamMapper for FailingExitMapper {
            fn exit_node(
                &mut self,
                _node: TokenStreamNode,
            ) -> Result<TokenStreamNode, TransformationError> {
                Err(TransformationError::Error("exit_node error".to_string()))
            }
        }

        let node = TokenStreamNode {
            tokens: vec![(Token::Text("test".to_string()), 0..4)],
            children: None,
            line_type: None,
        };
        let stream = TokenStream::Tree(vec![node]);
        let mut mapper = FailingExitMapper;
        let result = walk_stream(stream, &mut mapper);

        assert!(result.is_err());
        match result {
            Err(TransformationError::Error(msg)) => assert_eq!(msg, "exit_node error"),
            _ => panic!("Expected error from exit_node"),
        }
    }

    #[test]
    fn test_error_propagation_from_nested_children() {
        // Test that errors from deeply nested children propagate correctly
        struct FailingFlatMapper;

        impl StreamMapper for FailingFlatMapper {
            fn map_flat(
                &mut self,
                _tokens: Vec<(Token, ByteRange<usize>)>,
            ) -> Result<TokenStream, TransformationError> {
                Err(TransformationError::Error("nested error".to_string()))
            }
        }

        // Create a tree with nested children containing a flat stream
        let node = TokenStreamNode {
            tokens: vec![(Token::Text("parent".to_string()), 0..6)],
            children: Some(Box::new(TokenStream::Flat(vec![(
                Token::Text("child".to_string()),
                10..15,
            )]))),
            line_type: None,
        };
        let stream = TokenStream::Tree(vec![node]);
        let mut mapper = FailingFlatMapper;
        let result = walk_stream(stream, &mut mapper);

        assert!(result.is_err());
        match result {
            Err(TransformationError::Error(msg)) => assert_eq!(msg, "nested error"),
            _ => panic!("Expected error from nested flat stream"),
        }
    }
}
