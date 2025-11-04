//! Unified pipeline architecture for lexer transformations
//!
//! This module provides both:
//! - High-level pipeline orchestration (`LexPipeline`) - for selecting lexer/parser combinations
//! - Low-level transformation pipeline (`Pipeline`) - for chaining TokenStream transformations
//! - Transformation infrastructure (`TokenStream`, `StreamMapper`)
//! - Adapters for architectural boundaries

pub mod adapters;
pub mod adapters_linebased;
pub mod builder;
pub mod mapper;
pub mod mappers;
pub mod orchestration;
pub mod stream;

// Re-export high-level orchestration API
pub use orchestration::{LexPipeline, PipelineError};

// Re-export new TokenStream-based pipeline builder
pub use builder::Pipeline;

// Re-export concrete mapper implementations
pub use mappers::{
    BlankLinesMapper, IndentationToTreeMapper, NormalizeWhitespaceMapper,
    SemanticIndentationMapper, ToLineTokensMapper,
};
