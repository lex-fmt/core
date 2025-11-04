//! Indentation-based tree restructuring mapper for TokenStream pipeline
//!
//! This mapper converts a shallow tree of line nodes into a hierarchical tree
//! based on Indent/Dedent markers. It processes the indentation structure to
//! create proper parent-child relationships.
//!
//! # Logic
//!
//! 1. Maintains a stack of nesting levels
//! 2. When encountering Indent: starts a new nested level
//! 3. When encountering Dedent: closes current level and attaches to parent
//! 4. Regular line nodes accumulate at current level
//! 5. Builds hierarchical structure by moving nodes into children fields
//!
//! # Input/Output
//!
//! - **Input**: `TokenStream::Tree` - shallow tree from ToLineTokensMapper (one node per line)
//! - **Output**: `TokenStream::Tree` - nested tree with hierarchical parent-child relationships
//!
//! This is a pure adaptation of the existing indentation_to_token_tree transformation
//! to the TokenStream architecture.

use crate::lex::lexers::linebased::tokens::LineType;
use crate::lex::pipeline::mapper::{StreamMapper, TransformationError};
use crate::lex::pipeline::stream::{TokenStream, TokenStreamNode};

/// A mapper that restructures a shallow tree into a hierarchical tree based on indentation.
///
/// This transformation only operates on tree streams (expects output from ToLineTokensMapper)
/// and produces a nested tree structure based on Indent/Dedent tokens.
pub struct IndentationToTreeMapper;

impl IndentationToTreeMapper {
    /// Create a new IndentationToTreeMapper.
    pub fn new() -> Self {
        IndentationToTreeMapper
    }
}

impl Default for IndentationToTreeMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamMapper for IndentationToTreeMapper {
    fn map_flat(
        &mut self,
        _tokens: Vec<(crate::lex::lexers::tokens::Token, std::ops::Range<usize>)>,
    ) -> Result<TokenStream, TransformationError> {
        Err(TransformationError::Error(
            "IndentationToTreeMapper requires Tree input, not Flat".to_string(),
        ))
    }

    fn enter_node(
        &mut self,
        node: TokenStreamNode,
    ) -> Result<TokenStreamNode, TransformationError> {
        // This mapper works at the top level, not recursively on nodes
        Ok(node)
    }

    fn exit_node(&mut self, node: TokenStreamNode) -> Result<TokenStreamNode, TransformationError> {
        // This mapper works at the top level, not recursively on nodes
        Ok(node)
    }
}

impl IndentationToTreeMapper {
    /// Transform a shallow tree into a hierarchical tree based on Indent/Dedent tokens.
    ///
    /// This is the main entry point that should be called directly, not through the
    /// StreamMapper trait's walk_stream function (which is designed for recursive walking).
    pub fn transform(&mut self, stream: TokenStream) -> Result<TokenStream, TransformationError> {
        match stream {
            TokenStream::Flat(_) => Err(TransformationError::Error(
                "IndentationToTreeMapper requires Tree input, not Flat".to_string(),
            )),
            TokenStream::Tree(nodes) => {
                let restructured = self.build_hierarchy(nodes)?;
                Ok(TokenStream::Tree(restructured))
            }
        }
    }

    /// Build hierarchical structure from flat list of nodes based on Indent/Dedent.
    fn build_hierarchy(
        &mut self,
        nodes: Vec<TokenStreamNode>,
    ) -> Result<Vec<TokenStreamNode>, TransformationError> {
        // Stack of pending children at each indentation level
        // Each level accumulates nodes that become children of a container
        let mut stack: Vec<Vec<TokenStreamNode>> = vec![Vec::new()];
        let mut pending_nodes: Vec<TokenStreamNode> = Vec::new();

        for node in nodes {
            match node.line_type {
                Some(LineType::Indent) => {
                    // Flush pending nodes before entering nested level
                    if !pending_nodes.is_empty() {
                        let current_level = stack.last_mut().expect("Stack never empty");
                        current_level.append(&mut pending_nodes);
                    }
                    // Start a new nesting level
                    stack.push(Vec::new());
                }
                Some(LineType::Dedent) => {
                    // Flush pending nodes before closing level
                    if !pending_nodes.is_empty() {
                        let current_level = stack.last_mut().expect("Stack never empty");
                        current_level.append(&mut pending_nodes);
                    }
                    // Close current level and attach as children to last node in parent
                    if let Some(children_nodes) = stack.pop() {
                        let parent_level = stack.last_mut().expect("Stack never empty");

                        if let Some(last_parent_node) = parent_level.last_mut() {
                            // Attach the nested children to the last parent node
                            last_parent_node.children =
                                Some(Box::new(TokenStream::Tree(children_nodes)));
                        } else {
                            // If no parent node exists, this is malformed indentation
                            // In this case, we'll create a container node to hold the children
                            // This maintains the structure even with malformed input
                            parent_level.push(TokenStreamNode {
                                tokens: vec![],
                                children: Some(Box::new(TokenStream::Tree(children_nodes))),
                                line_type: None,
                            });
                        }
                    }
                }
                _ => {
                    // Accumulate regular line nodes at current level
                    pending_nodes.push(node);
                }
            }
        }

        // Flush any remaining pending nodes at root level
        if !pending_nodes.is_empty() {
            let root_level = stack.last_mut().expect("Stack never empty");
            root_level.append(&mut pending_nodes);
        }

        // Return the root level nodes
        let root_children = stack.pop().expect("Stack should contain root level");
        Ok(root_children)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::lexers::tokens::Token;

    fn make_node(
        line_type: LineType,
        tokens: Vec<(Token, std::ops::Range<usize>)>,
    ) -> TokenStreamNode {
        TokenStreamNode {
            tokens,
            children: None,
            line_type: Some(line_type),
        }
    }

    fn make_token_pair(token: Token, start: usize) -> (Token, std::ops::Range<usize>) {
        (token, start..start + 1)
    }

    #[test]
    fn test_single_node_no_indentation() {
        let input = vec![make_node(
            LineType::ParagraphLine,
            vec![make_token_pair(Token::Text("hello".to_string()), 0)],
        )];

        let mut mapper = IndentationToTreeMapper::new();
        let result = mapper.transform(TokenStream::Tree(input)).unwrap();

        // Verify the result is a tree with one node
        match result {
            TokenStream::Tree(nodes) => {
                assert_eq!(nodes.len(), 1);
                assert_eq!(nodes[0].line_type, Some(LineType::ParagraphLine));
                assert!(nodes[0].children.is_none());
            }
            _ => panic!("Expected Tree stream"),
        }
    }

    #[test]
    fn test_multiple_nodes_same_level() {
        let input = vec![
            make_node(
                LineType::SubjectLine,
                vec![make_token_pair(Token::Text("Title".to_string()), 0)],
            ),
            make_node(
                LineType::ParagraphLine,
                vec![make_token_pair(Token::Text("Para1".to_string()), 10)],
            ),
            make_node(
                LineType::ParagraphLine,
                vec![make_token_pair(Token::Text("Para2".to_string()), 20)],
            ),
        ];

        let mut mapper = IndentationToTreeMapper::new();
        let result = mapper.transform(TokenStream::Tree(input)).unwrap();

        // All nodes at same level should remain as siblings at root
        match result {
            TokenStream::Tree(nodes) => {
                assert_eq!(nodes.len(), 3);
                assert!(nodes.iter().all(|n| n.children.is_none()));
            }
            _ => panic!("Expected Tree stream"),
        }
    }

    #[test]
    fn test_single_indent_level() {
        let input = vec![
            make_node(
                LineType::SubjectLine,
                vec![make_token_pair(Token::Text("Title".to_string()), 0)],
            ),
            make_node(
                LineType::Indent,
                vec![make_token_pair(Token::Indentation, 10)],
            ),
            make_node(
                LineType::ParagraphLine,
                vec![make_token_pair(Token::Text("Indented".to_string()), 20)],
            ),
            make_node(
                LineType::Dedent,
                vec![make_token_pair(Token::Dedent(vec![]), 30)],
            ),
        ];

        let mut mapper = IndentationToTreeMapper::new();
        let result = mapper.transform(TokenStream::Tree(input)).unwrap();

        // Root should have title node with nested children
        match result {
            TokenStream::Tree(nodes) => {
                assert_eq!(nodes.len(), 1);
                assert_eq!(nodes[0].line_type, Some(LineType::SubjectLine));

                // Verify the title has children
                assert!(nodes[0].children.is_some());

                if let Some(ref children_stream) = nodes[0].children {
                    match children_stream.as_ref() {
                        TokenStream::Tree(child_nodes) => {
                            assert_eq!(child_nodes.len(), 1);
                            assert_eq!(child_nodes[0].line_type, Some(LineType::ParagraphLine));
                        }
                        _ => panic!("Expected Tree for children"),
                    }
                }
            }
            _ => panic!("Expected Tree stream"),
        }
    }

    #[test]
    fn test_multiple_indent_levels() {
        let input = vec![
            make_node(
                LineType::SubjectLine,
                vec![make_token_pair(Token::Text("Title".to_string()), 0)],
            ),
            make_node(
                LineType::Indent,
                vec![make_token_pair(Token::Indentation, 10)],
            ),
            make_node(
                LineType::ParagraphLine,
                vec![make_token_pair(Token::Text("Level1".to_string()), 20)],
            ),
            make_node(
                LineType::Indent,
                vec![make_token_pair(Token::Indentation, 30)],
            ),
            make_node(
                LineType::ParagraphLine,
                vec![make_token_pair(Token::Text("Level2".to_string()), 40)],
            ),
            make_node(
                LineType::Dedent,
                vec![make_token_pair(Token::Dedent(vec![]), 50)],
            ),
            make_node(
                LineType::Dedent,
                vec![make_token_pair(Token::Dedent(vec![]), 60)],
            ),
        ];

        let mut mapper = IndentationToTreeMapper::new();
        let result = mapper.transform(TokenStream::Tree(input)).unwrap();

        // Verify nested structure: Title -> Level1 -> Level2
        match result {
            TokenStream::Tree(nodes) => {
                assert_eq!(nodes.len(), 1);
                assert_eq!(nodes[0].line_type, Some(LineType::SubjectLine));

                // Check first level children
                if let Some(ref level1_stream) = nodes[0].children {
                    match level1_stream.as_ref() {
                        TokenStream::Tree(level1_nodes) => {
                            assert_eq!(level1_nodes.len(), 1);
                            assert_eq!(level1_nodes[0].line_type, Some(LineType::ParagraphLine));

                            // Check second level children
                            if let Some(ref level2_stream) = level1_nodes[0].children {
                                match level2_stream.as_ref() {
                                    TokenStream::Tree(level2_nodes) => {
                                        assert_eq!(level2_nodes.len(), 1);
                                        assert_eq!(
                                            level2_nodes[0].line_type,
                                            Some(LineType::ParagraphLine)
                                        );
                                    }
                                    _ => panic!("Expected Tree for level2"),
                                }
                            } else {
                                panic!("Expected children at level1");
                            }
                        }
                        _ => panic!("Expected Tree for level1"),
                    }
                } else {
                    panic!("Expected children at root");
                }
            }
            _ => panic!("Expected Tree stream"),
        }
    }

    #[test]
    fn test_multiple_children_at_same_level() {
        let input = vec![
            make_node(
                LineType::SubjectLine,
                vec![make_token_pair(Token::Text("Title".to_string()), 0)],
            ),
            make_node(
                LineType::Indent,
                vec![make_token_pair(Token::Indentation, 10)],
            ),
            make_node(
                LineType::ParagraphLine,
                vec![make_token_pair(Token::Text("Child1".to_string()), 20)],
            ),
            make_node(
                LineType::ParagraphLine,
                vec![make_token_pair(Token::Text("Child2".to_string()), 30)],
            ),
            make_node(
                LineType::ParagraphLine,
                vec![make_token_pair(Token::Text("Child3".to_string()), 40)],
            ),
            make_node(
                LineType::Dedent,
                vec![make_token_pair(Token::Dedent(vec![]), 50)],
            ),
        ];

        let mut mapper = IndentationToTreeMapper::new();
        let result = mapper.transform(TokenStream::Tree(input)).unwrap();

        // Title should have 3 children at the same level
        match result {
            TokenStream::Tree(nodes) => {
                assert_eq!(nodes.len(), 1);

                if let Some(ref children_stream) = nodes[0].children {
                    match children_stream.as_ref() {
                        TokenStream::Tree(child_nodes) => {
                            assert_eq!(child_nodes.len(), 3);
                            assert!(child_nodes
                                .iter()
                                .all(|n| n.line_type == Some(LineType::ParagraphLine)));
                        }
                        _ => panic!("Expected Tree for children"),
                    }
                }
            }
            _ => panic!("Expected Tree stream"),
        }
    }

    #[test]
    fn test_siblings_after_dedent() {
        let input = vec![
            make_node(
                LineType::SubjectLine,
                vec![make_token_pair(Token::Text("Title1".to_string()), 0)],
            ),
            make_node(
                LineType::Indent,
                vec![make_token_pair(Token::Indentation, 10)],
            ),
            make_node(
                LineType::ParagraphLine,
                vec![make_token_pair(Token::Text("Nested".to_string()), 20)],
            ),
            make_node(
                LineType::Dedent,
                vec![make_token_pair(Token::Dedent(vec![]), 30)],
            ),
            make_node(
                LineType::SubjectLine,
                vec![make_token_pair(Token::Text("Title2".to_string()), 40)],
            ),
        ];

        let mut mapper = IndentationToTreeMapper::new();
        let result = mapper.transform(TokenStream::Tree(input)).unwrap();

        // Should have 2 siblings at root level: Title1 (with children) and Title2
        match result {
            TokenStream::Tree(nodes) => {
                assert_eq!(nodes.len(), 2);
                assert_eq!(nodes[0].line_type, Some(LineType::SubjectLine));
                assert!(nodes[0].children.is_some());
                assert_eq!(nodes[1].line_type, Some(LineType::SubjectLine));
                assert!(nodes[1].children.is_none());
            }
            _ => panic!("Expected Tree stream"),
        }
    }

    #[test]
    fn test_preserves_token_ranges() {
        // Verify that byte ranges are preserved exactly through the transformation
        let input = vec![
            make_node(
                LineType::SubjectLine,
                vec![(Token::Text("Title".to_string()), 0..5)],
            ),
            make_node(LineType::Indent, vec![(Token::Indentation, 10..14)]),
            make_node(
                LineType::ParagraphLine,
                vec![(Token::Text("Content".to_string()), 20..27)],
            ),
            make_node(LineType::Dedent, vec![(Token::Dedent(vec![]), 30..30)]),
        ];

        let mut mapper = IndentationToTreeMapper::new();
        let result = mapper.transform(TokenStream::Tree(input)).unwrap();

        match result {
            TokenStream::Tree(nodes) => {
                // Verify root node token range
                assert_eq!(nodes[0].tokens[0].1, 0..5);

                // Verify nested child token range
                if let Some(ref children_stream) = nodes[0].children {
                    match children_stream.as_ref() {
                        TokenStream::Tree(child_nodes) => {
                            assert_eq!(child_nodes[0].tokens[0].1, 20..27);
                        }
                        _ => panic!("Expected Tree for children"),
                    }
                }
            }
            _ => panic!("Expected Tree stream"),
        }
    }

    #[test]
    fn test_preserves_line_type_information() {
        // Verify that LineType is preserved through the transformation
        let input = vec![
            make_node(
                LineType::SubjectLine,
                vec![make_token_pair(Token::Text("Subject".to_string()), 0)],
            ),
            make_node(
                LineType::Indent,
                vec![make_token_pair(Token::Indentation, 10)],
            ),
            make_node(LineType::ListLine, vec![make_token_pair(Token::Dash, 20)]),
            make_node(
                LineType::ParagraphLine,
                vec![make_token_pair(Token::Text("Para".to_string()), 30)],
            ),
            make_node(
                LineType::Dedent,
                vec![make_token_pair(Token::Dedent(vec![]), 40)],
            ),
        ];

        let mut mapper = IndentationToTreeMapper::new();
        let result = mapper.transform(TokenStream::Tree(input)).unwrap();

        match result {
            TokenStream::Tree(nodes) => {
                assert_eq!(nodes[0].line_type, Some(LineType::SubjectLine));

                if let Some(ref children_stream) = nodes[0].children {
                    match children_stream.as_ref() {
                        TokenStream::Tree(child_nodes) => {
                            assert_eq!(child_nodes.len(), 2);
                            assert_eq!(child_nodes[0].line_type, Some(LineType::ListLine));
                            assert_eq!(child_nodes[1].line_type, Some(LineType::ParagraphLine));
                        }
                        _ => panic!("Expected Tree for children"),
                    }
                }
            }
            _ => panic!("Expected Tree stream"),
        }
    }

    #[test]
    fn test_error_on_flat_input() {
        let tokens = vec![(Token::Text("hello".to_string()), 0..5)];
        let mut mapper = IndentationToTreeMapper::new();
        let result = mapper.transform(TokenStream::Flat(tokens));

        assert!(result.is_err());
        if let Err(TransformationError::Error(msg)) = result {
            assert!(msg.contains("requires Tree input"));
        }
    }
}
