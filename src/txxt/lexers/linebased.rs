//! Line-based lexer pipeline module
//!
//! This module provides the experimental line-based lexer pipeline that:
//! - Flattens tokens into line tokens
//! - Transforms line tokens into a hierarchical tree

pub mod pipeline;
pub mod tokens;
pub mod transformations;

pub use pipeline::{PipelineError, PipelineOutput, PipelineStage, _lex, _lex_stage};
pub use tokens::{LineToken, LineTokenTree, LineTokenType};
