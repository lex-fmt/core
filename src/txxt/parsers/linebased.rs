//! Linebased Parser - Declarative Grammar Engine
//!
//! This module implements a unified declarative grammar parser that:
//! 1. Pattern-matches on LineToken types and opaque LineContainerToken structures
//! 2. Follows strict parse order from the specification
//! 3. Recursively descends into containers when building AST
//! 4. Reuses existing AST builder functions (unwrapper module)
//!
//! The parser operates on LineContainerToken trees from the linebased lexer,
//! with no regex-based pattern matching or complex intermediate representations.

pub mod declarative_grammar;
pub mod engine;
pub mod unwrapper;

pub use engine::parse_experimental_v2;
