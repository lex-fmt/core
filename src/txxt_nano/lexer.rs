//! Lexer module for the txxt format
//!
//! This module contains the tokenization logic for the txxt format,
//! including token definitions and the lexer implementation.

pub mod lexer_impl;
pub mod tokens;

pub use lexer_impl::{tokenize, tokenize_with_spans, TxxtLexer};
pub use tokens::Token;
