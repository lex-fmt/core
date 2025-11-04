Proposal: Unified Token Stream and Transformation API

	This document proposes a new, unified architecture for the lexer pipeline to address the irregularity of transformation inputs and outputs. The goal is to create a robust, composable system where transformations can be chained predictably while strictly adhering to the "Immutable Log" principle for location tracking.

1. The Problem: Irregular Transformation Interfaces

	The current lexer pipeline uses a series of transformations to process the initial token stream. However, these transformations lack a common interface. Some operate on flat vectors of tokens, others on vectors of lines, and others on tree-like `LineContainer` structures.

	This irregularity creates several pain points:
		- Brittleness: Chaining transformations is difficult and order-dependent. A transformation designed to work on a flat list cannot be placed after one that produces a tree.
		- Complexity: Each transformation must implement its own logic for handling its specific input data structure, leading to duplicated effort and complex manual iteration or recursion.
		- Maintenance: Adding or reordering transformations is a significant effort, requiring careful adaptation of the surrounding pipeline stages.

2. The Solution Part 1: A Unified TokenStream

	The first step is to introduce a single, unified data structure that can represent tokens in both flat and hierarchical forms. This `TokenStream` enum will be the standard input and output for every transformation in the pipeline.

	2.1 The `TokenStream` Enum

		This enum explicitly handles both simple token lists and the nested structures required for indentation and line grouping.

			// Suggested location: src/lex/pipeline/stream.rs

			use crate::lex::lexers::tokens::Token;
			use std::ops::Range as ByteRange;

			/// A unified representation of a token collection that can be either a flat
			/// sequence or a hierarchical tree. This is the standard input and output
			/// for all transformations in the lexer pipeline.
			#[derive(Debug, Clone)]
			pub enum TokenStream {
			    /// A flat, linear sequence of raw tokens. This is the initial state from
			    /// the lexer and the format for simple, non-structural transformations.
			    Flat(Vec<(Token, ByteRange<usize>)>),

			    /// A hierarchical representation of tokens, typically used for indentation,
			    /// where each node can contain a nested `TokenStream` of its own.
			    Tree(Vec<TokenStreamNode>),
			}

			/// A node within a hierarchical `TokenStream::Tree`. It contains the tokens
			/// for its own level (e.g., a line of text) and an optional `TokenStream`
			/// for its children (e.g., an indented block).
			#[derive(Debug, Clone)]
			pub struct TokenStreamNode {
			    /// The original source tokens that constitute this node's specific line or level.
			    /// This list is always flat and preserves the "ground truth" from the lexer.
			    pub tokens: Vec<(Token, ByteRange<usize>)>,

			    /// Optional nested children for this node. `None` signifies no indented block follows.
			    /// `Some` contains the `TokenStream` for the entire nested block.
			    pub children: Option<Box<TokenStream>>,
			}
		:: rust ::

	2.2 Upholding the "Immutable Log" Principle

		To guarantee that we never lose the original source token information, `TokenStream` provides a universal `unroll` method. This becomes the single, reliable way to get back to the "ground truth" for AST building, regardless of the stream's complexity.

			// In the same file as TokenStream

			impl TokenStream {
			    /// Unrolls the entire stream, whether flat or hierarchical, back into a single,
			    /// flat list of the original source tokens. This method is the cornerstone of
			    /// the "Immutable Log" architecture, guaranteeing accurate location tracking.
			    pub fn unroll(&self) -> Vec<(Token, ByteRange<usize>)> {
			        match self {
			            TokenStream::Flat(tokens) => tokens.clone(),
			            TokenStream::Tree(nodes) => {
			                nodes.iter().flat_map(|node| node.unroll()).collect()
			            }
			        }
			    }
			}

			impl TokenStreamNode {
			    /// Unrolls a single node and recursively unrolls its children.
			    pub fn unroll(&self) -> Vec<(Token, ByteRange<usize>)> {
			        let mut all_tokens = self.tokens.clone();
			        if let Some(children_stream) = &self.children {
			            all_tokens.extend(children_stream.unroll());
			        }
			        all_tokens
			    }
			}
		:: rust ::

3. The Solution Part 2: Abstracting Traversal with a StreamMapper

	To prevent each transformation from implementing its own traversal logic, we introduce a `StreamMapper` trait (a Visitor pattern). A generic "walker" function will handle the traversal, applying the mapper's logic at each step.

	3.1 The `StreamMapper` Trait and Walker Function

		The trait defines methods that the walker calls during its traversal. The walker itself contains the core recursive logic, keeping transformations simple and focused.

			// Suggested location: src/lex/pipeline/mapper.rs

			pub trait StreamMapper {
			    /// Transforms a `TokenStream::Flat`.
			    fn map_flat(&mut self, tokens: Vec<(Token, ByteRange<usize>)>) -> Result<TokenStream, TransformationError> {
			        Ok(TokenStream::Flat(tokens))
			    }

			    /// Called in pre-order (before visiting children).
			    fn enter_node(&mut self, node: TokenStreamNode) -> Result<TokenStreamNode, TransformationError> {
			        Ok(node)
			    }

			    /// Called in post-order (after visiting children).
			    fn exit_node(&mut self, node: TokenStreamNode) -> Result<TokenStreamNode, TransformationError> {
			        Ok(node)
			    }
			}

			/// The walker function that drives the transformation process.
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

			/// Helper function to recursively walk a single `TokenStreamNode`.
			fn walk_node(
			    node: TokenStreamNode,
			    mapper: &mut impl StreamMapper,
			) -> Result<TokenStreamNode, TransformationError> {
			    let mut node_after_enter = mapper.enter_node(node)?;

			    if let Some(children_stream) = node_after_enter.children.take() {
			        let new_children = walk_stream(*children_stream, mapper)?;
			        node_after_enter.children = Some(Box::new(new_children));
			    }

			    let final_node = mapper.exit_node(node_after_enter)?;
			    Ok(final_node)
			}
		:: rust ::

4. Benefits of this Approach

	- Regularity & Composability: All transformations share an identical interface (`TokenStream` -> `TokenStream`), making them easy to chain, reorder, or disable in a pipeline.
	- Guaranteed Location Accuracy: The `unroll()` method ensures the "Immutable Log" of original source tokens is always accessible, preserving perfect location tracking for the final AST.
	- Simplicity: Transformation logic is focused purely on the "what," not the "how" of traversal. The complex recursive logic is written once in the walker.
	- Robustness: The traversal logic is centralized, tested, and verified in one place, reducing the chance of bugs in individual transformations.
	- Flexibility: The pre-order (`enter_node`) and post-order (`exit_node`) hooks provide complete control for a wide variety of transformations, from simple token replacement to complex tree restructuring.
