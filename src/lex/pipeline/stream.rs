//! Token stream representation for unified pipeline architecture
//!
//! This module provides a single, unified data structure that can represent tokens
//! in both flat and hierarchical forms. This `TokenStream` enum is the standard
//! input and output for all transformations in the lexer pipeline.
//!
//! # Design Principles
//!
//! 1. **Unified Interface**: All transformations use `TokenStream` -> `TokenStream`
//! 2. **Immutable Log**: Original token locations are never modified, only aggregated
//! 3. **Unrollable**: Any `TokenStream` can be unrolled back to flat `(Token, Range)` pairs
//!
//! # The Immutable Log Architecture
//!
//! The logos lexer produces `(Token, Range<usize>)` pairs - this is the ground truth.
//! Transformations create aggregate tokens that store these original pairs.
//! The `unroll()` method provides universal access back to the ground truth for AST building.

use crate::lex::lexers::linebased::tokens_linebased::LineType;
use crate::lex::lexers::tokens_core::Token;
use std::ops::Range as ByteRange;

/// A unified representation of a token collection that can be either a flat
/// sequence or a hierarchical tree.
///
/// This is the standard input and output for all transformations in the lexer pipeline.
///
/// # Variants
///
/// - `Flat`: A linear sequence of raw tokens, typically from the initial lexer
///   or simple transformations that don't create nesting.
/// - `Tree`: A hierarchical representation where each node can contain nested children,
///   used for indentation-based structures.
///
/// # Examples
///
/// ```ignore
/// // Flat stream from initial tokenization
/// let flat = TokenStream::Flat(vec![
///     (Token::Text("hello".into()), 0..5),
///     (Token::Newline, 5..6),
/// ]);
///
/// // Tree structure with nesting
/// let tree = TokenStream::Tree(vec![
///     TokenStreamNode {
///         tokens: vec![(Token::Text("title".into()), 0..5)],
///         children: Some(Box::new(TokenStream::Flat(vec![
///             (Token::Text("content".into()), 10..17),
///         ]))),
///     },
/// ]);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum TokenStream {
    /// A flat, linear sequence of raw tokens.
    ///
    /// This is the initial state from the lexer and the format for simple,
    /// non-structural transformations.
    Flat(Vec<(Token, ByteRange<usize>)>),

    /// A hierarchical representation of tokens.
    ///
    /// Typically used for indentation-based structures where each node can
    /// contain a nested `TokenStream` of its own.
    Tree(Vec<TokenStreamNode>),
}

/// A node within a hierarchical `TokenStream::Tree`.
///
/// Each node contains tokens for its own level (e.g., a line of text) and
/// optionally a nested `TokenStream` for its children (e.g., an indented block).
///
/// # Fields
///
/// - `tokens`: The original source tokens that constitute this node's level.
///   This list is always flat and preserves the ground truth from the lexer.
/// - `children`: Optional nested children for this node. `None` means no indented
///   block follows. `Some` contains the `TokenStream` for the entire nested block.
/// - `line_type`: Optional line type information from the linebased lexer pipeline.
///   Used to preserve LineType classification (SubjectLine, ListLine, etc.) when
///   converting between TokenStream and LineContainer. `None` for nodes that don't
///   represent a single line (e.g., intermediate container nodes, or nodes from
///   transformations that don't use LineType).
#[derive(Debug, Clone, PartialEq)]
pub struct TokenStreamNode {
    /// The original source tokens that constitute this node's specific line or level.
    ///
    /// This list is always flat and preserves the "ground truth" from the lexer.
    pub tokens: Vec<(Token, ByteRange<usize>)>,

    /// Optional nested children for this node.
    ///
    /// `None` signifies no indented block follows.
    /// `Some` contains the `TokenStream` for the entire nested block.
    pub children: Option<Box<TokenStream>>,

    /// Optional line type classification for this node.
    ///
    /// Preserves LineType information from the linebased lexer pipeline.
    /// `None` for nodes that don't represent a classified line (e.g., container nodes,
    /// or nodes from transformations that don't use the LineType system).
    pub line_type: Option<LineType>,
}

impl TokenStream {
    /// Unrolls the entire stream back to a flat list of original source tokens.
    ///
    /// This method is the cornerstone of the "Immutable Log" architecture,
    /// guaranteeing accurate location tracking by providing universal access
    /// to the original `(Token, Range<usize>)` pairs from the logos lexer.
    ///
    /// Whether the stream is flat or hierarchical, this method recursively
    /// extracts all original tokens in document order.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let flat = TokenStream::Flat(vec![
    ///     (Token::Text("hello".into()), 0..5),
    /// ]);
    /// assert_eq!(flat.unroll().len(), 1);
    ///
    /// let tree = TokenStream::Tree(vec![
    ///     TokenStreamNode {
    ///         tokens: vec![(Token::Text("a".into()), 0..1)],
    ///         children: Some(Box::new(TokenStream::Flat(vec![
    ///             (Token::Text("b".into()), 2..3),
    ///         ]))),
    ///     },
    /// ]);
    /// assert_eq!(tree.unroll().len(), 2); // "a" and "b"
    /// ```
    pub fn unroll(&self) -> Vec<(Token, ByteRange<usize>)> {
        match self {
            TokenStream::Flat(tokens) => tokens.clone(),
            TokenStream::Tree(nodes) => nodes.iter().flat_map(|node| node.unroll()).collect(),
        }
    }
}

impl TokenStreamNode {
    /// Unrolls a single node and recursively unrolls its children.
    ///
    /// Returns a flat list containing this node's tokens followed by all
    /// tokens from nested children (if any).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let node = TokenStreamNode {
    ///     tokens: vec![(Token::Text("parent".into()), 0..6)],
    ///     children: Some(Box::new(TokenStream::Flat(vec![
    ///         (Token::Text("child".into()), 10..15),
    ///     ]))),
    /// };
    ///
    /// let unrolled = node.unroll();
    /// assert_eq!(unrolled.len(), 2); // "parent" and "child"
    /// ```
    pub fn unroll(&self) -> Vec<(Token, ByteRange<usize>)> {
        let mut all_tokens = self.tokens.clone();
        if let Some(children_stream) = &self.children {
            all_tokens.extend(children_stream.unroll());
        }
        all_tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_stream_unroll() {
        let tokens = vec![
            (Token::Text("hello".to_string()), 0..5),
            (Token::Whitespace, 5..6),
            (Token::Text("world".to_string()), 6..11),
        ];

        let stream = TokenStream::Flat(tokens.clone());
        let unrolled = stream.unroll();

        assert_eq!(unrolled, tokens);
    }

    #[test]
    fn test_empty_flat_stream_unroll() {
        let stream = TokenStream::Flat(vec![]);
        let unrolled = stream.unroll();

        assert_eq!(unrolled, vec![]);
    }

    #[test]
    fn test_tree_stream_single_node_no_children() {
        let tokens = vec![(Token::Text("hello".to_string()), 0..5)];
        let node = TokenStreamNode {
            tokens: tokens.clone(),
            children: None,
            line_type: None,
        };

        let stream = TokenStream::Tree(vec![node]);
        let unrolled = stream.unroll();

        assert_eq!(unrolled, tokens);
    }

    #[test]
    fn test_tree_stream_single_node_with_flat_children() {
        let parent_tokens = vec![(Token::Text("parent".to_string()), 0..6)];
        let child_tokens = vec![(Token::Text("child".to_string()), 10..15)];

        let node = TokenStreamNode {
            tokens: parent_tokens.clone(),
            children: Some(Box::new(TokenStream::Flat(child_tokens.clone()))),
            line_type: None,
        };

        let stream = TokenStream::Tree(vec![node]);
        let unrolled = stream.unroll();

        // Should contain parent tokens followed by child tokens
        let mut expected = parent_tokens;
        expected.extend(child_tokens);
        assert_eq!(unrolled, expected);
    }

    #[test]
    fn test_tree_stream_multiple_nodes_no_children() {
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
        let unrolled = stream.unroll();

        assert_eq!(
            unrolled,
            vec![
                (Token::Text("first".to_string()), 0..5),
                (Token::Text("second".to_string()), 6..12),
            ]
        );
    }

    #[test]
    fn test_tree_stream_nested_children() {
        // Create a deeply nested structure:
        // root
        //   -> child1
        //        -> grandchild
        let grandchild_tokens = vec![(Token::Text("grandchild".to_string()), 20..30)];
        let grandchild_stream = TokenStream::Flat(grandchild_tokens.clone());

        let child1_tokens = vec![(Token::Text("child1".to_string()), 10..16)];
        let child1_node = TokenStreamNode {
            tokens: child1_tokens.clone(),
            children: Some(Box::new(grandchild_stream)),
            line_type: None,
        };

        let root_tokens = vec![(Token::Text("root".to_string()), 0..4)];
        let root_node = TokenStreamNode {
            tokens: root_tokens.clone(),
            children: Some(Box::new(TokenStream::Tree(vec![child1_node]))),
            line_type: None,
        };

        let stream = TokenStream::Tree(vec![root_node]);
        let unrolled = stream.unroll();

        // Should be: root, child1, grandchild (in order)
        let mut expected = root_tokens;
        expected.extend(child1_tokens);
        expected.extend(grandchild_tokens);
        assert_eq!(unrolled, expected);
    }

    #[test]
    fn test_tree_stream_mixed_nodes_with_and_without_children() {
        let node1 = TokenStreamNode {
            tokens: vec![(Token::Text("first".to_string()), 0..5)],
            children: None,
            line_type: None,
        };
        let node2 = TokenStreamNode {
            tokens: vec![(Token::Text("second".to_string()), 6..12)],
            children: Some(Box::new(TokenStream::Flat(vec![(
                Token::Text("nested".to_string()),
                20..26,
            )]))),
            line_type: None,
        };
        let node3 = TokenStreamNode {
            tokens: vec![(Token::Text("third".to_string()), 30..35)],
            children: None,
            line_type: None,
        };

        let stream = TokenStream::Tree(vec![node1, node2, node3]);
        let unrolled = stream.unroll();

        assert_eq!(
            unrolled,
            vec![
                (Token::Text("first".to_string()), 0..5),
                (Token::Text("second".to_string()), 6..12),
                (Token::Text("nested".to_string()), 20..26),
                (Token::Text("third".to_string()), 30..35),
            ]
        );
    }

    #[test]
    fn test_empty_tree_stream() {
        let stream = TokenStream::Tree(vec![]);
        let unrolled = stream.unroll();

        assert_eq!(unrolled, vec![]);
    }

    #[test]
    fn test_node_unroll_no_children() {
        let tokens = vec![(Token::Text("hello".to_string()), 0..5)];
        let node = TokenStreamNode {
            tokens: tokens.clone(),
            children: None,
            line_type: None,
        };

        let unrolled = node.unroll();
        assert_eq!(unrolled, tokens);
    }

    #[test]
    fn test_node_unroll_with_children() {
        let parent_tokens = vec![(Token::Text("parent".to_string()), 0..6)];
        let child_tokens = vec![(Token::Text("child".to_string()), 10..15)];

        let node = TokenStreamNode {
            tokens: parent_tokens.clone(),
            children: Some(Box::new(TokenStream::Flat(child_tokens.clone()))),
            line_type: None,
        };

        let unrolled = node.unroll();

        let mut expected = parent_tokens;
        expected.extend(child_tokens);
        assert_eq!(unrolled, expected);
    }

    #[test]
    fn test_preserves_token_ranges() {
        // Verify that unroll preserves exact byte ranges
        let tokens = vec![
            (Token::Text("hello".to_string()), 0..5),
            (Token::Whitespace, 5..6),
            (Token::Text("world".to_string()), 6..11),
        ];

        let node = TokenStreamNode {
            tokens: tokens.clone(),
            children: None,
            line_type: None,
        };

        let stream = TokenStream::Tree(vec![node]);
        let unrolled = stream.unroll();

        // Verify each range is preserved exactly
        assert_eq!(unrolled[0].1, 0..5);
        assert_eq!(unrolled[1].1, 5..6);
        assert_eq!(unrolled[2].1, 6..11);
    }

    #[test]
    fn test_complex_nested_structure() {
        // Test a realistic structure mimicking indented document
        // Line 1: "Title" (0..5)
        // Line 2: Indent
        //   Line 3: "Content" (10..17)
        //   Line 4: "More" (20..24)
        // Line 5: Dedent
        // Line 6: "End" (30..33)

        let content_node = TokenStreamNode {
            tokens: vec![
                (Token::Text("Content".to_string()), 10..17),
                (Token::Text("More".to_string()), 20..24),
            ],
            children: None,
            line_type: None,
        };

        let title_node = TokenStreamNode {
            tokens: vec![(Token::Text("Title".to_string()), 0..5)],
            children: Some(Box::new(TokenStream::Tree(vec![content_node]))),
            line_type: None,
        };

        let end_node = TokenStreamNode {
            tokens: vec![(Token::Text("End".to_string()), 30..33)],
            children: None,
            line_type: None,
        };

        let stream = TokenStream::Tree(vec![title_node, end_node]);
        let unrolled = stream.unroll();

        assert_eq!(
            unrolled,
            vec![
                (Token::Text("Title".to_string()), 0..5),
                (Token::Text("Content".to_string()), 10..17),
                (Token::Text("More".to_string()), 20..24),
                (Token::Text("End".to_string()), 30..33),
            ]
        );
    }
}
