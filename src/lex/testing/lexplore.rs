//! Test harness for per-element testing
//!
//! This module provides utilities for testing individual element variations
//! using the per-element library in `docs/specs/v1/elements/`.
//!
//! # Module Organization
//!
//! - `loader`: File loading, parsing, and tokenization infrastructure
//! - `extraction`: AST node extraction and assertion helpers
//!
//! # Usage
//!
//! ```rust,ignore
//! use lex::lex::testing::lexplore::*;
//!
//! // Load and parse elements
//! let parsed = Lexplore::paragraph(1).parse();
//! let paragraph = parsed.expect_paragraph();
//!
//! // Load and tokenize
//! let tokens = Lexplore::paragraph(1).tokenize();
//!
//! // Load from arbitrary paths
//! let doc = Lexplore::from_path("path/to/file.lex").parse();
//!
//! // Use extraction helpers
//! assert!(paragraph_text_starts_with(&paragraph, "This is"));
//! ```

mod extraction;
mod loader;

// Re-export everything public from submodules
pub use extraction::*;
pub use loader::*;
