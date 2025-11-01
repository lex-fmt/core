//! Reference Parser Module
//!
//! This module contains the original combinator-based parser implementation for txxt.
//! It serves as the reference implementation and is maintained for backward compatibility.
//!
//! ## Structure
//!
//! - `parser.rs`: Main parser orchestration
//! - `combinators.rs`: Parser combinators and primitives
//! - `api.rs`: Public API entry points

pub mod api;
pub mod combinators;
#[allow(clippy::module_inception)]
pub mod parser;

// Re-export main API
pub use api::parse;
pub use parser::parse as parse_internal;
