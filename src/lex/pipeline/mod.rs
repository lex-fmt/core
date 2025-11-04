//! Unified pipeline architecture for lexer transformations
//!
//! This module provides both:
//! - High-level pipeline orchestration (`LexPipeline`)
//! - Low-level transformation infrastructure (`TokenStream`, `StreamMapper`)

pub mod mapper;
pub mod orchestration;
pub mod stream;

// Re-export high-level API for backwards compatibility
pub use orchestration::{LexPipeline, PipelineError};
