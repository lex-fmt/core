//! AST building utilities for parsers
//!
//! This module provides utilities for building AST nodes from tokens.
//! It follows a three-layer architecture:
//!
//! 1. Token Normalization - Convert various token formats to standard vectors
//! 2. Data Extraction - Extract primitive data (text, byte ranges) from tokens
//! 3. AST Creation - Convert primitives to AST nodes with ast::Range
//!
//! Parsers should primarily use the `api` module which provides the public API.

pub mod api;
pub mod ast_builder;
pub mod location;
pub mod pipeline;

pub(super) mod builders;
pub(super) mod extraction;

// Re-export public API
pub use api::*;
