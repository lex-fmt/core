//! Lexer module for the txxt format
//!
//! This module contains the tokenization logic for the txxt format,
//! including token definitions and the lexer implementation.
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

pub mod blank_line_transform;
pub mod detokenizer;
pub mod indentation_transform;
pub mod lexer_impl;
pub mod tokens;

pub use blank_line_transform::{transform_blank_lines, transform_blank_lines_with_spans};
pub use detokenizer::detokenize;
pub use indentation_transform::{transform_indentation, transform_indentation_with_spans};
pub use lexer_impl::{tokenize, tokenize_with_spans};
pub use tokens::Token;

/// Main lexer function that returns fully processed tokens
/// Processing pipeline:
/// 1. tokenize() - creates raw tokens with Indent and Newline tokens
/// 2. transform_indentation() - converts Indent tokens to semantic IndentLevel/DedentLevel tokens
/// 3. transform_blank_lines() - converts consecutive Newline tokens to BlankLine tokens
/// 4. Add document boundary tokens
pub fn lex(source: &str) -> Vec<Token> {
    // HACK: Ensure source ends with newline to work around Chumsky recursive/.repeated() issue
    // This helps when a paragraph is the last element in a recursive context
    let source_with_newline = if !source.is_empty() && !source.ends_with('\n') {
        format!("{}\n", source)
    } else {
        source.to_string()
    };

    let raw_tokens = tokenize(&source_with_newline);
    let tokens = transform_indentation(raw_tokens);
    let mut tokens = transform_blank_lines(tokens);

    // HACK: Add document boundary tokens to solve recursive/.repeated() EOF issue
    tokens.insert(0, Token::DocStart);
    tokens.push(Token::DocEnd);

    tokens
}

/// Lexing function that preserves source spans for parser
/// Returns tokens with their corresponding source spans
/// Synthetic tokens (IndentLevel, DedentLevel, BlankLine, DocStart, DocEnd) have empty spans (0..0)
/// Processing pipeline:
/// 1. tokenize_with_spans() - creates raw tokens with spans
/// 2. transform_indentation_with_spans() - converts Indent tokens to semantic IndentLevel/DedentLevel tokens
/// 3. transform_blank_lines_with_spans() - converts consecutive Newline tokens to BlankLine tokens
/// 4. Add document boundary tokens
pub fn lex_with_spans(source: &str) -> Vec<(Token, std::ops::Range<usize>)> {
    // HACK: Ensure source ends with newline to work around Chumsky recursive/.repeated() issue
    // This helps when a paragraph is the last element in a recursive context
    let source_with_newline = if !source.is_empty() && !source.ends_with('\n') {
        format!("{}\n", source)
    } else {
        source.to_string()
    };

    let raw_tokens_with_spans = tokenize_with_spans(&source_with_newline);
    let tokens = transform_indentation_with_spans(raw_tokens_with_spans);
    let mut tokens = transform_blank_lines_with_spans(tokens);

    // HACK: Add document boundary tokens to solve recursive/.repeated() EOF issue
    // DocStart at beginning, DocEnd at end
    // The recursive content parser will stop when it hits DocEnd (which it can't parse)
    // The document parser then consumes DocEnd to complete parsing
    tokens.insert(0, (Token::DocStart, 0..0));
    tokens.push((Token::DocEnd, 0..0));

    tokens
}
