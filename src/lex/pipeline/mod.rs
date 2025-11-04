//! Unified pipeline architecture for lexer transformations
//!
//! This module provides both:
//! - High-level pipeline orchestration (`LexPipeline`)
//! - New TokenStream-based pipeline (`Pipeline`)
//! - Low-level transformation infrastructure (`TokenStream`, `StreamMapper`)
//! - Adapters for migration (`flat_to_token_stream`, `token_stream_to_flat`)

pub mod adapters;
pub mod mapper;
pub mod mappers;
pub mod orchestration;
pub mod stream;
pub mod transformation_pipeline;

// Re-export high-level API for backwards compatibility
pub use orchestration::{LexPipeline, PipelineError};

// Re-export new TokenStream-based pipeline
pub use transformation_pipeline::Pipeline;

// Re-export concrete mapper implementations
pub use mappers::{BlankLinesMapper, NormalizeWhitespaceMapper, SemanticIndentationMapper};
