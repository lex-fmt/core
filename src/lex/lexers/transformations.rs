//! Lexer transformations for processing token streams
//!
//! This module contains the transformation pipeline that processes raw tokens from the lexer
//! into semantically meaningful tokens for the parser.
//!
//! The transformations are applied in order:
//! 1. tokenize() - creates raw tokens with location information
//! 2. process_whitespace_remainders() - handles lex whitespace specification
//! 3. sem_indentation() - converts Indent tokens to Indent/Dedent tokens
//! 4. transform_blank_lines() - converts consecutive Newline tokens to BlankLine tokens
//!
//! Linebased transformations (for the linebased 3-pass parser):
//! - _to_line_tokens() - flat tokens → line tokens (Pass 0)
//! - _indentation_to_token_tree() - line tokens → hierarchical tree (Pass 1)
//!
//! The line-based pipeline that orchestrates all transformations is now in the `linebased` module.

pub mod blanklines;
pub mod normalize_whitespace;
pub mod sem_indentation;

// Re-export the linebased pipeline from the linebased module
pub use crate::lex::lexers::linebased::{
    PipelineError, PipelineOutput, PipelineStage, _lex, _lex_stage,
};
pub use blanklines::transform_blank_lines;
pub use normalize_whitespace::process_whitespace_remainders;
pub use sem_indentation::sem_indentation;
