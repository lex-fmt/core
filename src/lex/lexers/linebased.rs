//! Line-based lexer pipeline module
//!
//! In this design we do two important things:
//! 1. We group tokens into lines.
//! 2. We have a token tree, that is where inner structures are represented as token vectors, since lex is a hierarchical format.
//!
//! This allows us, in the matching linebased parser to match patters much easier, since we can match
//! full lines and clearly see the structure, since the regext approach cannot count and keep tabs
//! of indent and dedent levels.

pub mod pipeline;
pub mod tokens;
pub mod transformations;

pub use pipeline::{PipelineError, PipelineOutput, PipelineStage, _lex, _lex_stage};
pub use tokens::{LineContainer, LineToken, LineType};
