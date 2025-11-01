//! Lexer module for the txxt format
//!
//! This module orchestrates the complete tokenization pipeline for the txxt format.
//! The pipeline consists of:
//! 1. Core tokenization using logos lexer
//! 2. Transformation pipeline:
//!    - Whitespace remainder processing
//!    - Indentation transformation (Indent -> IndentLevel/DedentLevel)
//!    - Blank line transformation (consecutive Newlines -> BlankLine)
//!
//! Indentation Handling
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

pub mod detokenizer;
pub mod lexer_impl;
pub mod tokens;
pub mod transformations;

pub use detokenizer::detokenize;
pub use lexer_impl::tokenize;
pub use tokens::Token;
pub use transformations::{
    process_whitespace_remainders, transform_blank_lines, transform_indentation,
};

/// Main lexer function that returns fully processed tokens with locations
/// Returns tokens with their corresponding source locations
/// Synthetic tokens (IndentLevel, DedentLevel, BlankLine) have meaningful locations
/// Processing pipeline:
/// 1. tokenize() - raw tokens with source locations
/// 2. process_whitespace_remainders() - handle whitespace with locations
/// 3. transform_indentation() - convert Indent tokens with location tracking
/// 4. transform_blank_lines() - convert Newline sequences with location tracking
pub fn lex(source: &str) -> Vec<(Token, std::ops::Range<usize>)> {
    // Ensure source ends with newline to help with paragraph parsing at EOF
    let source_with_newline = if !source.is_empty() && !source.ends_with('\n') {
        format!("{}\n", source)
    } else {
        source.to_string()
    };

    let tokenss = tokenize(&source_with_newline);
    let tokens_after_whitespace = process_whitespace_remainders(tokenss);
    let tokens_after_indentation = transform_indentation(tokens_after_whitespace);
    transform_blank_lines(tokens_after_indentation)
}
