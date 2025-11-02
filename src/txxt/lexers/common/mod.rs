//! Common lexer module
//!
//! This module contains shared interfaces and utilities for lexer implementations.

pub mod interface;

pub use interface::{
    IndentationLexerImpl, LexError, Lexer, LexerOutput, LexerRegistry, LinebasedLexerImpl,
};
