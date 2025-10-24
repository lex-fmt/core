//! Lexer module for the txxt format
//!
//! This module contains the tokenization logic for the txxt format,
//! including token definitions and the lexer implementation.
//!
//! ## Indentation Handling
//!
//! In order to make indented blocks tractable by regular parser combinators libraries,
//! indentation ultimately gets transformed into semantic indent and dedent tokens, which
//! map nicely to brace tokens for more standard syntaxes. txxt will work the same, but
//! at this original lexing pass we only do simple 4 spaces / 1 tab substitutions for
//! indentation blocks. This means that a line that is 2 levels indented will produce
//! two indent tokens.
//!
//! The rationale for this approach is:
//! - This allows us to use a vanilla logos lexer, no custom code.
//! - This isolates the logic for semantic indent and dedent tokens to a later
//!   transformation step, separate from all other tokenization, which helps a lot.
//! - At some point in the spec, we will handle blocks much like markdown's fenced blocks,that display non-txxt strings. In these cases, while we may parse (for indentation)the lines, we never want to emit the indent and dedent tokens. Having this happen two stages gives us more flexibility on how to handle these cases.

pub mod lexer_impl;
pub mod tokens;

pub use lexer_impl::{tokenize, tokenize_with_spans};
pub use tokens::Token;
