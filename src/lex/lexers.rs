//! Lexer
//!
//! This module orchestrates the complete tokenization pipeline for the lex format.
//!
//! Structure:
//!     The tokenization is done through the logos lexer library, based on the grammar.lex file
//! Currently we are still running two parser designs side by side and the the newer parser requires
//! more preprocessing of the cst.
//!
//! The pipeline consists of:
//! 1. Core tokenization using logos lexer
//! 2. Common Transformation pipeline:
//!    - Whitespace remainder processing ./transformations/normalize_whitespace.rs
//!    - Indentation transformation (Indent -> Indent/Dedent) ./transformations/sem_indentation.rs
//!    - Blank line transformation (consecutive Newlines -> BlankLine) ./transformations/transform_blanklines.rs
//! 3. Line-based pipeline (linebased):
//!    - Flatten tokens into line tokens
//!    - Transform line tokens into a hierarchical tree
//!
//! Indentation Handling
//!
//!     In order to make indented blocks tractable by regular parser combinators libraries,
//!     indentation ultimately gets transformed into semantic indent and dedent tokens, which
//!     map nicely to brace tokens for more standard syntaxes. lex will work the same, but
//!     at this original lexing pass we only do simple 4 spaces / 1 tab substitutions for
//!     indentation blocks. This means that a line that is 2 levels indented will produce
//!     two indent tokens.
//!
//!     The rationale for this approach is:
//!     - This allows us to use a vanilla logos lexer, no custom code.
//!     - This isolates the logic for semantic indent and dedent tokens to a later
//!     transformation step, separate from all other tokenization, which helps a lot.
//!     - At some point in the spec, we will handle blocks much like markdown's fenced blocks,that
//! display non-lex strings. In these cases, while we may parse (for indentation)the lines, we never
//! want to emit the indent and dedent tokens. Having this happen two stages gives us more
//! flexibility on how to handle these cases.

pub mod base_tokenization;
pub mod common;
pub mod detokenizer;
pub mod linebased;
pub mod tokens;
pub mod transformations;

pub use base_tokenization::tokenize;
pub use common::{LexError, Lexer, LexerOutput, LexerRegistry};
pub use detokenizer::detokenize;
pub use tokens::Token;
pub use transformations::{
    _lex, _lex_stage, sem_indentation, transform_blank_lines, PipelineOutput, PipelineStage,
};

// Re-export line-based types for convenience
pub use linebased::{LineContainer, LineToken, LineType};

/// Preprocesses source text to ensure it ends with a newline.
///
/// This is required for proper paragraph parsing at EOF.
/// Returns the original string if it already ends with a newline, or empty string.
/// Otherwise, appends a newline.
pub fn ensure_source_ends_with_newline(source: &str) -> String {
    if !source.is_empty() && !source.ends_with('\n') {
        format!("{}\n", source)
    } else {
        source.to_string()
    }
}

/// Main indentation lexer pipeline that returns fully processed tokens with locations
/// Returns tokens with their corresponding source locations
/// Synthetic tokens (Indent, Dedent, BlankLine) have meaningful locations
/// Processing pipeline:
/// 1. Base tokenization (done by caller) - raw tokens with source locations
/// 2. NormalizeWhitespace - handle whitespace remainders with locations (uses new TokenStream mapper)
/// 3. SemanticIndentation - convert Indentation tokens with location tracking
/// 4. TransformBlankLines - convert Newline sequences with location tracking
pub fn lex(tokens: Vec<(Token, std::ops::Range<usize>)>) -> Vec<(Token, std::ops::Range<usize>)> {
    use crate::lex::pipeline::adapters::token_stream_to_flat;
    use crate::lex::pipeline::stream::TokenStream;
    use crate::lex::pipeline::NormalizeWhitespaceMapper;

    // Stage 1: NormalizeWhitespace using new TokenStream mapper
    let mut normalize_mapper = NormalizeWhitespaceMapper::new();
    let token_stream = TokenStream::Flat(tokens);
    let transformed_stream =
        crate::lex::pipeline::mapper::walk_stream(token_stream, &mut normalize_mapper)
            .expect("NormalizeWhitespace transformation failed");
    let current_tokens = token_stream_to_flat(transformed_stream)
        .expect("Expected Flat stream from NormalizeWhitespace");

    // Stages 2-3: Apply remaining transformations using legacy interface
    let remaining_transformations: Vec<Box<dyn transformations::Transformation>> = vec![
        Box::new(transformations::SemanticIndentation),
        Box::new(transformations::TransformBlankLines),
    ];

    // Apply remaining transformations in sequence
    remaining_transformations
        .iter()
        .fold(current_tokens, |acc, transform| transform.transform(acc))
}
