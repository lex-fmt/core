//! Reference Parser Module
//!
//! This module contains the original combinator-based parser implementation for txxt.
//! It serves as the reference implementation and is maintained for backward compatibility.
//!
//! ## Structure
//!
//! - `builders.rs`: Consolidated AST node building functions and location utilities
//! - `parser.rs`: Main parser orchestration
//! - `combinators.rs`: Parser combinators and primitives (legacy, to be deprecated)
//! - `api.rs`: Public API entry points
//! - Element parsers (annotations, definitions, document, etc.): Build element-specific parsers
//!   and construct AST nodes for each txxt element type

pub mod annotations;
pub mod api;
pub mod builders;
pub mod combinators;
pub mod definitions;
pub mod document;
pub mod foreign;
pub mod labels;
pub mod lists;
pub mod parameters;
#[allow(clippy::module_inception)]
pub mod parser;
pub mod sessions;

// Re-export main API
pub use api::parse;
pub use document::document;
pub use parser::parse as parse_internal;
