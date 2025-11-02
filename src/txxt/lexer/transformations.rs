//! Lexer transformations for processing token streams
//!
//! This module contains the transformation pipeline that processes raw tokens from the lexer
//! into semantically meaningful tokens for the parser.
//!
//! The transformations are applied in order:
//! 1. tokenize() - creates raw tokens with location information
//! 2. process_whitespace_remainders() - handles txxt whitespace specification
//! 3. sem_indentation() - converts Indent tokens to IndentLevel/DedentLevel tokens
//! 4. transform_blank_lines() - converts consecutive Newline tokens to BlankLine tokens
//!
//! Experimental transformations (for the experimental 3-pass parser):
//! - experimental_to_line_tokens() - flat tokens → line tokens (Pass 0)
//! - experimental_indentation_to_token_tree() - line tokens → hierarchical tree (Pass 1)
//!
//! The line-based pipeline that orchestrates all transformations is now in the `linebased` module.

pub mod blanklines;
pub mod normalize_whitespace;
pub mod sem_indentation;

// Re-export the experimental pipeline from the linebased module
pub use crate::txxt::lexer::linebased::{
    experimental_lex, experimental_lex_stage, PipelineError, PipelineOutput, PipelineStage,
};
pub use blanklines::transform_blank_lines;
pub use normalize_whitespace::process_whitespace_remainders;
pub use sem_indentation::sem_indentation;
