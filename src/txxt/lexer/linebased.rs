//! Line-based lexer pipeline module
//!
//! This module provides the experimental line-based lexer pipeline that:
//! - Flattens tokens into line tokens
//! - Transforms line tokens into a hierarchical tree

pub mod pipeline;

pub use pipeline::{
    experimental_lex, experimental_lex_stage, PipelineError, PipelineOutput, PipelineStage,
};
