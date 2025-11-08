//! Unified pipeline architecture for Lex processing
//!
//! This module provides:
//! - Document loading and processing (`DocumentLoader`) - primary API for all pipeline operations
//! - Config-based processing (`PipelineExecutor`) - for executing named configurations
//! - Low-level transformation pipeline (`Pipeline`) - for chaining TokenStream transformations
//! - Transformation infrastructure (`TokenStream`, `StreamMapper`)
//! - Adapters for architectural boundaries

pub mod adapters;
pub mod builder;
pub mod config;
pub mod executor;
pub mod loader;
pub mod mapper;
pub mod mappers;
pub mod stream;

// Re-export low-level pipeline builder
pub use builder::{AnalyzerConfig, Pipeline, PipelineOutput};

// Re-export config-based processing API (primary interface)
pub use config::{ConfigRegistry, PipelineSpec, ProcessingConfig, TargetSpec};
pub use executor::{ExecutionError, ExecutionOutput, PipelineExecutor};

// Re-export document loading API (recommended entry point for most use cases)
pub use loader::{DocumentLoader, Parser};

// Re-export concrete mapper implementations
pub use mappers::{BlankLinesMapper, NormalizeWhitespaceMapper, SemanticIndentationMapper};
