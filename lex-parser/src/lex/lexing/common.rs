//! Common lexer module
//!
//! This module contains shared interfaces and utilities for lexer implementations.

use crate::lex::lexing::tokens_core::Token;
use std::fmt;
use std::ops::Range;

/// Output from a lexer
#[derive(Debug, Clone)]
pub enum LexerOutput {
    /// Flat sequence of tokens
    Flat(Vec<(Token, Range<usize>)>),
}

/// Errors that can occur during lexing
#[derive(Debug, Clone)]
pub enum LexError {
    /// Generic error message
    Error(String),
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexError::Error(msg) => write!(f, "Lexing error: {}", msg),
        }
    }
}

impl std::error::Error for LexError {}

/// Trait for lexer implementations
pub trait Lexer {
    /// Lex the source text
    fn lex(&self, source: &str) -> Result<LexerOutput, LexError>;
}
