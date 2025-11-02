//! Indentation-based lexer module
//!
//! This module contains the core tokenization implementation for the txxt lexer.
//! The tokenization is handled by the logos lexer library, which produces raw tokens
//! with location information.

pub mod pipeline;

pub use pipeline::tokenize;
