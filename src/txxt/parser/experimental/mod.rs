//! Experimental Parser Module
//!
//! This module implements a multi-pass parsing approach that separates concerns:
//! - Tree Walking (orchestration and recursion handling)
//! - Pattern Matching (grammar recognition - stubbed for now)
//! - AST Construction (converting patterns to nodes - stubbed for now)
//!
//! ## Design
//!
//! The parser operates in phases:
//! 1. Receive a LineTokenTree from the experimental lexer
//! 2. Walk the tree recursively, flattening tokens at each level
//! 3. Apply pattern matching to recognize grammar elements (stubbed initially)
//! 4. Convert matched patterns to AST nodes via the unwrapper
//! 5. Return final Document
//!
//! This separation makes the tree walking testable independently of the grammar rules.

pub mod engine;
pub mod unwrapper;

pub use engine::parse_experimental;
