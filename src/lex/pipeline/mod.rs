//! Unified pipeline architecture for lexer transformations
//!
//! This module provides both:
//! - High-level pipeline orchestration (`LexPipeline`)
//! - New TokenStream-based pipeline (`Pipeline`)
//! - Low-level transformation infrastructure (`TokenStream`, `StreamMapper`)
//! - Adapters for migration (`flat_to_token_stream`, `token_stream_to_flat`)

pub mod adapters;
pub mod adapters_linebased;
pub mod builder;
pub mod legacy_orchestration;
pub mod mapper;
pub mod mappers;
pub mod stream;

// Re-export legacy orchestration API for backwards compatibility
pub use legacy_orchestration::{LexPipeline, PipelineError};

// Re-export new TokenStream-based pipeline builder
pub use builder::Pipeline;

// Re-export concrete mapper implementations
pub use mappers::{
    BlankLinesMapper, IndentationToTreeMapper, NormalizeWhitespaceMapper,
    SemanticIndentationMapper, ToLineTokensMapper,
};
