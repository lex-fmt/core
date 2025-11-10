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
pub mod line_classification;
pub mod line_grouping;
pub mod pipeline;
pub mod tokens_core;
pub mod tokens_linebased;
pub mod transformations;

pub use base_tokenization::tokenize;
pub use common::{LexError, Lexer, LexerOutput};
pub use tokens_core::Token;
// Re-export line-based types for convenience
pub use tokens_linebased::{LineContainer, LineToken, LineType};

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
pub fn lex(tokens: Vec<(Token, std::ops::Range<usize>)>) -> Vec<(Token, std::ops::Range<usize>)> {
    use crate::lex::lexing::transformations::{SemanticIndentationMapper};
    use crate::lex::pipeline::stream::TokenStream;

    // Start with TokenStream::Flat and chain transformations
    let mut current_stream = TokenStream::Flat(tokens);

    // Stage 2: SemanticIndentation
    let mut semantic_indent_mapper = SemanticIndentationMapper::new();
    current_stream =
        crate::lex::pipeline::mapper::walk_stream(current_stream, &mut semantic_indent_mapper)
            .expect("SemanticIndentation transformation failed");

    // Unroll the final stream to get flat tokens for backward compatibility
    current_stream.unroll()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::testing::factories::mk_tokens;

    /// Helper to prepare token stream and call lex pipeline
    fn lex_helper(source: &str) -> Vec<(Token, std::ops::Range<usize>)> {
        let source_with_newline = ensure_source_ends_with_newline(source);
        let token_stream = base_tokenization::tokenize(&source_with_newline);
        lex(token_stream)
    }

    #[test]
    fn test_paragraph_pattern() {
        let input = "This is a paragraph.\nIt has multiple lines.";
        let tokens = lex_helper(input);

        // Exact token sequence validation
        // lex() adds a trailing newline and applies full transformations
        assert_eq!(
            tokens,
            mk_tokens(&[
                (Token::Text("This".to_string()), 0, 4),
                (Token::Whitespace, 4, 5),
                (Token::Text("is".to_string()), 5, 7),
                (Token::Whitespace, 7, 8),
                (Token::Text("a".to_string()), 8, 9),
                (Token::Whitespace, 9, 10),
                (Token::Text("paragraph".to_string()), 10, 19),
                (Token::Period, 19, 20),
                (Token::BlankLine(Some("\n".to_string())), 20, 21),
                (Token::Text("It".to_string()), 21, 23),
                (Token::Whitespace, 23, 24),
                (Token::Text("has".to_string()), 24, 27),
                (Token::Whitespace, 27, 28),
                (Token::Text("multiple".to_string()), 28, 36),
                (Token::Whitespace, 36, 37),
                (Token::Text("lines".to_string()), 37, 42),
                (Token::Period, 42, 43),
                (Token::BlankLine(Some("\n".to_string())), 43, 44),
            ])
        );
    }

    #[test]
    fn test_list_pattern() {
        let input = "- First item\n- Second item";
        let tokens = lex_helper(input);

        // Exact token sequence validation
        // lex() adds a trailing newline and applies full transformations
        assert_eq!(
            tokens,
            mk_tokens(&[
                (Token::Dash, 0, 1),
                (Token::Whitespace, 1, 2),
                (Token::Text("First".to_string()), 2, 7),
                (Token::Whitespace, 7, 8),
                (Token::Text("item".to_string()), 8, 12),
                (Token::BlankLine(Some("\n".to_string())), 12, 13),
                (Token::Dash, 13, 14),
                (Token::Whitespace, 14, 15),
                (Token::Text("Second".to_string()), 15, 21),
                (Token::Whitespace, 21, 22),
                (Token::Text("item".to_string()), 22, 26),
                (Token::BlankLine(Some("\n".to_string())), 26, 27),
            ])
        );
    }

    #[test]
    fn test_session_pattern() {
        let input = "1. Session Title\n    Content here";
        let tokens = lex_helper(input);

        // Exact token sequence validation
        // lex() transforms Indent -> Indent and adds trailing newline
        assert_eq!(
            tokens,
            mk_tokens(&[
                (Token::Number("1".to_string()), 0, 1),
                (Token::Period, 1, 2),
                (Token::Whitespace, 2, 3),
                (Token::Text("Session".to_string()), 3, 10),
                (Token::Whitespace, 10, 11),
                (Token::Text("Title".to_string()), 11, 16),
                (Token::BlankLine(Some("\n".to_string())), 16, 17),
                (Token::Indent(vec![(Token::Indentation, 17..21)]), 0, 0),
                (Token::Text("Content".to_string()), 21, 28),
                (Token::Whitespace, 28, 29),
                (Token::Text("here".to_string()), 29, 33),
                (Token::BlankLine(Some("\n".to_string())), 33, 34),
                (Token::Dedent(vec![]), 0, 0),
            ])
        );
    }

    #[test]
    fn test_lex_marker_pattern() {
        let input = "Some text :: marker";
        let tokens = lex_helper(input);

        // Exact token sequence validation
        // lex() adds a trailing newline
        assert_eq!(
            tokens,
            mk_tokens(&[
                (Token::Text("Some".to_string()), 0, 4),
                (Token::Whitespace, 4, 5),
                (Token::Text("text".to_string()), 5, 9),
                (Token::Whitespace, 9, 10),
                (Token::LexMarker, 10, 12),
                (Token::Whitespace, 12, 13),
                (Token::Text("marker".to_string()), 13, 19),
                (Token::BlankLine(Some("\n".to_string())), 19, 20),
            ])
        );
    }

    #[test]
    fn test_mixed_content_pattern() {
        let input = "1. Session\n    - Item 1\n    - Item 2\n\nParagraph after.";
        let tokens = lex_helper(input);

        // Exact token sequence validation
        // lex() transforms Indent -> Indent and consecutive Newlines -> BlankLine
        assert_eq!(
            tokens,
            mk_tokens(&[
                (Token::Number("1".to_string()), 0, 1),
                (Token::Period, 1, 2),
                (Token::Whitespace, 2, 3),
                (Token::Text("Session".to_string()), 3, 10),
                (Token::BlankLine(Some("\n".to_string())), 10, 11),
                (Token::Indent(vec![(Token::Indentation, 11..15)]), 0, 0),
                (Token::Dash, 15, 16),
                (Token::Whitespace, 16, 17),
                (Token::Text("Item".to_string()), 17, 21),
                (Token::Whitespace, 21, 22),
                (Token::Number("1".to_string()), 22, 23),
                (Token::BlankLine(Some("\n".to_string())), 23, 24),
                (Token::Dash, 28, 29),
                (Token::Whitespace, 29, 30),
                (Token::Text("Item".to_string()), 30, 34),
                (Token::Whitespace, 34, 35),
                (Token::Number("2".to_string()), 35, 36),
                (Token::BlankLine(Some("\n".to_string())), 36, 37),
                (Token::BlankLine(Some("\n".to_string())), 37, 38),
                (Token::Dedent(vec![]), 0, 0),
                (Token::Text("Paragraph".to_string()), 38, 47),
                (Token::Whitespace, 47, 48),
                (Token::Text("after".to_string()), 48, 53),
                (Token::Period, 53, 54),
                (Token::BlankLine(Some("\n".to_string())), 54, 55),
            ])
        );
    }
}
