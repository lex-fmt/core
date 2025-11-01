//! Lexer transformations for processing token streams
//!
//! This module contains the transformation pipeline that processes raw tokens from the lexer
//! into semantically meaningful tokens for the parser.
//!
//! The transformations are applied in order:
//! 1. tokenize() - creates raw tokens with location information
//! 2. process_whitespace_remainders() - handles txxt whitespace specification
//! 3. transform_indentation() - converts Indent tokens to IndentLevel/DedentLevel tokens
//! 4. transform_blank_lines() - converts consecutive Newline tokens to BlankLine tokens
//!
//! Experimental transformations (for the experimental 3-pass parser):
//! - experimental_transform_to_line_tokens() - flat tokens → line tokens (Pass 0)
//! - experimental_transform_indentation_to_token_tree() - line tokens → hierarchical tree (Pass 1)
//! - experimental_pipeline - orchestrates all transformations into cohesive pipeline

pub mod experimental_pipeline;
pub mod transform_blanklines;
pub mod transform_indentation;
pub mod transform_indentation_to_token_tree;
pub mod transform_to_line_tokens;
pub mod transform_whitespace;

pub use experimental_pipeline::{
    experimental_lex, experimental_lex_stage, PipelineOutput, PipelineStage,
};
pub use transform_blanklines::transform_blank_lines;
pub use transform_indentation::transform_indentation;
pub use transform_indentation_to_token_tree::experimental_transform_indentation_to_token_tree;
pub use transform_to_line_tokens::experimental_transform_to_line_tokens;
pub use transform_whitespace::process_whitespace_remainders;
