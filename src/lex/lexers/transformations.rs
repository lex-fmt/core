//! Lexer transformations for processing token streams
//!
//! This module contains legacy transformation implementations that are being migrated
//! to the new TokenStream architecture. The transformations process raw tokens from
//! the lexer into semantically meaningful tokens for the parser.
//!
//! The transformations are applied in order:
//! 1. tokenize() - creates raw tokens with location information
//! 2. normalize_whitespace - handles lex whitespace specification (MIGRATED to pipeline/mappers)
//! 3. sem_indentation - converts Indent tokens to Indent/Dedent tokens (MIGRATED to pipeline/mappers)
//! 4. transform_blank_lines - converts consecutive Newline tokens to BlankLine tokens (MIGRATED to pipeline/mappers)
//!
//! Linebased transformations (for the linebased 3-pass parser):
//! - _to_line_tokens() - flat tokens → line tokens (Pass 0)
//! - _indentation_to_token_tree() - line tokens → hierarchical tree (Pass 1)
//!
//! The line-based pipeline that orchestrates all transformations is now in the `linebased` module.
//! All flat transformations have been migrated to the TokenStream architecture in `pipeline/mappers`.

pub mod interface;

// Re-export the Transformation trait
pub use interface::Transformation;

// Re-export the linebased pipeline from the linebased module
pub use crate::lex::lexers::linebased::{
    PipelineError, PipelineOutput, PipelineStage, _lex, _lex_stage,
};
