//! Lexer
pub mod base_tokenization;
pub mod common;
pub mod line_classification;
pub mod line_grouping;
pub mod transformations;
pub use base_tokenization::tokenize;
pub use common::{LexError, Lexer, LexerOutput};
// Re-export token types for consumers that still import them from `lexing`
pub use crate::lex::token::{LineContainer, LineToken, LineType, Token};
/// Preprocesses source text to ensure it ends with a newline.
pub fn ensure_source_ends_with_newline(source: &str) -> String {
    if !source.is_empty() && !source.ends_with('\n') {
        format!("{}\n", source)
    } else {
        source.to_string()
    }
}
pub fn lex(tokens: Vec<(Token, std::ops::Range<usize>)>) -> Vec<(Token, std::ops::Range<usize>)> {
    use crate::lex::lexing::transformations::semantic_indentation::SemanticIndentationMapper;
    let line_tokens = line_grouping::group_into_lines(tokens);
    // Flatten the LineTokens back into the old format.
    let flat_tokens: Vec<(Token, std::ops::Range<usize>)> = line_tokens
        .into_iter()
        .flat_map(|line_token| {
            line_token
                .source_tokens
                .into_iter()
                .zip(line_token.token_spans.into_iter())
        })
        .collect();
    let mut mapper = SemanticIndentationMapper::new();
    mapper
        .map(flat_tokens)
        .expect("SemanticIndentation transformation failed")
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
        let text_tokens: Vec<_> = tokens
            .into_iter()
            .filter(|(t, _)| matches!(t, Token::Text(_)))
            .collect();
        assert_eq!(text_tokens.len(), 6);
    }
}
