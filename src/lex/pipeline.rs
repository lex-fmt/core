//! Unified pipeline architecture for Lex processing
//!
//! This module provides:
//! - Config-based processing (`PipelineExecutor`) - for executing named configurations
//! - Low-level transformation pipeline (`Pipeline`) - for chaining TokenStream transformations
//! - Transformation infrastructure (`TokenStream`, `StreamMapper`)
//! - Adapters for architectural boundaries

pub mod adapters;
pub mod adapters_linebased;
pub mod builder;
pub mod config;
pub mod executor;
pub mod mapper;
pub mod mappers;
pub mod stream;

// Re-export low-level pipeline builder
pub use builder::{AnalyzerConfig, Pipeline, PipelineOutput};

// Re-export config-based processing API (primary interface)
pub use config::{ConfigRegistry, PipelineSpec, ProcessingConfig, TargetSpec};
pub use executor::{ExecutionError, ExecutionOutput, PipelineExecutor};

// Re-export concrete mapper implementations
pub use mappers::{
    BlankLinesMapper, IndentationToTreeMapper, NormalizeWhitespaceMapper,
    SemanticIndentationMapper, ToLineTokensMapper,
};
