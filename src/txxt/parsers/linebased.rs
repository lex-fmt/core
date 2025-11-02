//! Linebased Parser - Regex-Based Declarative Grammar Engine
//!
//! This module implements a unified regex-driven declarative grammar parser that:
//! 1. Converts LineToken sequences to grammar notation strings (e.g., <subject-line><blank-line>)
//! 2. Matches token sequences against regex patterns that define grammar rules
//! 3. Follows strict parse order from the specification for correct disambiguation
//! 4. Recursively descends into containers when building AST
//! 5. Reuses existing AST builder functions (unwrapper module)
//!
//! Grammar patterns are declarative data, not imperative code, making them:
//! - Easy to understand and modify
//! - Maintainable and debuggable
//! - Less error-prone than hand-written matching logic

pub mod declarative_grammar;
pub mod engine;
pub mod unwrapper;

pub use engine::parse_experimental_v2;
