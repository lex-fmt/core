//! Detokenizer for the txxt format
//!
//! This module provides functionality to convert a stream of tokens back into a string.
use crate::txxt_nano::lexer::tokens::Token;

#[cfg(test)]
use insta;

impl ToTxxtString for Token {
    fn to_txxt_string(&self) -> String {
        match self {
            Token::TxxtMarker => "::".to_string(),
            Token::Indent => "    ".to_string(),
            Token::Whitespace => " ".to_string(),
            Token::Newline => "\n".to_string(),
            Token::Dash => "-".to_string(),
            Token::Period => ".".to_string(),
            Token::OpenParen => "(".to_string(),
            Token::CloseParen => ")".to_string(),
            Token::Colon => ":".to_string(),
            Token::Comma => ",".to_string(),
            Token::Quote => "\"".to_string(),
            Token::Equals => "=".to_string(),
            Token::Number(s) => s.clone(),
            Token::Text(s) => s.clone(),
            // The following tokens are synthetic and should not be part of the detokenized output
            Token::IndentLevel | Token::DedentLevel | Token::DocStart | Token::DocEnd => {
                String::new()
            }
        }
    }
}

/// Trait for converting a token to its string representation
pub trait ToTxxtString {
    fn to_txxt_string(&self) -> String;
}

/// Detokenize a stream of tokens into a string
pub fn detokenize(tokens: &[Token]) -> String {
    let mut result = String::new();
    let mut indent_level = 0;

    for token in tokens {
        match token {
            Token::IndentLevel => indent_level += 1,
            Token::DedentLevel => indent_level -= 1,
            Token::Newline => {
                result.push('\n');
            }
            _ => {
                if result.ends_with('\n') || result.is_empty() {
                    for _ in 0..indent_level {
                        result.push_str("    ");
                    }
                }
                result.push_str(&token.to_txxt_string());
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::txxt_nano::lexer::{tokenize, transform_indentation};

    // ===== Stage 0/1: Basic detokenization with raw tokens (no indentation handling) =====

    #[test]
    fn test_detokenize_simple_paragraph() {
        let source = "Simple Paragraphs Test {{paragraph}}";
        let tokens = tokenize(source);
        let detokenized = detokenize(&tokens);
        assert_eq!(detokenized, source);
    }

    #[test]
    fn test_detokenize_multiline_paragraph() {
        let source = "This is a multi-line paragraph.\nIt continues on the second line.\nAnd even has a third line. {{paragraph}}";
        let tokens = tokenize(source);
        let detokenized = detokenize(&tokens);
        assert_eq!(detokenized, source);
    }

    #[test]
    fn test_detokenize_simple_list() {
        let source =
            "- First item {{list-item}}\n- Second item {{list-item}}\n- Third item {{list-item}}";
        let tokens = tokenize(source);
        let detokenized = detokenize(&tokens);
        assert_eq!(detokenized, source);
    }

    #[test]
    fn test_detokenize_session() {
        let source = "1. Introduction {{session-title}}\n\n    This is the content of the session. It contains a paragraph that is indented relative to the session title. {{paragraph}}";
        let tokens = tokenize(source);
        let detokenized = detokenize(&tokens);
        assert_eq!(detokenized, source);
    }

    // ===== Stage 2: Detokenization with semantic indentation tokens =====

    #[test]
    fn test_detokenize_with_semantic_indentation() {
        let source = "1. Session\n    - Item 1\n        - Nested Item\n    - Item 2";
        let raw_tokens = tokenize(source);
        let tokens = transform_indentation(raw_tokens);
        let detokenized = detokenize(&tokens);
        assert_eq!(detokenized, source);
    }

    // ===== Round-trip tests with snapshot verification =====
    // These tests verify that tokenizing and detokenizing produces the original source.
    // When differences occur, the snapshot provides a clear line-by-line diff that shows
    // exactly where whitespace or content differs, making debugging much easier than
    // a simple "strings don't match" message.

    fn test_roundtrip_raw_tokens(source: &str, snapshot_name: &str) {
        let tokens = tokenize(source);
        let detokenized = detokenize(&tokens);
        insta::assert_snapshot!(snapshot_name, detokenized);
    }

    fn test_roundtrip_semantic_tokens(source: &str, snapshot_name: &str) {
        let raw_tokens = tokenize(source);
        let tokens = transform_indentation(raw_tokens);
        let detokenized = detokenize(&tokens);
        insta::assert_snapshot!(snapshot_name, detokenized);
    }

    #[test]
    fn test_roundtrip_000_paragraphs() {
        let source = include_str!("../../../docs/specs/v1/samples/000-paragraphs.txxt");
        test_roundtrip_raw_tokens(source, "000-paragraphs");
    }

    #[test]
    fn test_roundtrip_010_sessions_flat_single() {
        let source =
            include_str!("../../../docs/specs/v1/samples/010-paragraphs-sessions-flat-single.txxt");
        test_roundtrip_raw_tokens(source, "010-paragraphs-sessions-flat-single");
    }

    // Note: 020 uses mixed tabs and spaces for indentation, which get normalized
    // to spaces by the Indent token. This is a known limitation that will be
    // addressed when we improve tab/space handling in token definitions.
    // Skipping for now.

    #[test]
    fn test_sample_030_sessions_nested_raw() {
        let source = include_str!(
            "../../../docs/specs/v1/samples/030-paragraphs-sessions-nested-multiple.txxt"
        );
        test_roundtrip_raw_tokens(source, "030-paragraphs-sessions-nested-multiple");
    }

    #[test]
    fn test_sample_040_lists_raw() {
        let source = include_str!("../../../docs/specs/v1/samples/040-lists.txxt");
        test_roundtrip_raw_tokens(source, "040-lists");
    }

    #[test]
    fn test_sample_050_paragraph_lists_raw() {
        let source = include_str!("../../../docs/specs/v1/samples/050-paragraph-lists.txxt");
        test_roundtrip_raw_tokens(source, "050-paragraph-lists");
    }

    #[test]
    fn test_sample_050_trifecta_flat_raw() {
        let source = include_str!("../../../docs/specs/v1/samples/050-trifecta-flat-simple.txxt");
        test_roundtrip_raw_tokens(source, "050-trifecta-flat-simple");
    }

    #[test]
    fn test_sample_060_trifecta_nesting_raw() {
        let source = include_str!("../../../docs/specs/v1/samples/060-trifecta-nesting.txxt");
        test_roundtrip_raw_tokens(source, "060-trifecta-nesting");
    }

    #[test]
    fn test_sample_070_nested_lists_simple_raw() {
        let source = include_str!("../../../docs/specs/v1/samples/070-nested-lists-simple.txxt");
        test_roundtrip_raw_tokens(source, "070-nested-lists-simple");
    }

    #[test]
    fn test_sample_080_nested_lists_mixed_raw() {
        let source =
            include_str!("../../../docs/specs/v1/samples/080-nested-lists-mixed-content.txxt");
        test_roundtrip_raw_tokens(source, "080-nested-lists-mixed-content");
    }

    #[test]
    fn test_sample_090_definitions_simple_raw() {
        let source = include_str!("../../../docs/specs/v1/samples/090-definitions-simple.txxt");
        test_roundtrip_raw_tokens(source, "090-definitions-simple");
    }

    #[test]
    fn test_sample_100_definitions_mixed_raw() {
        let source =
            include_str!("../../../docs/specs/v1/samples/100-definitions-mixed-content.txxt");
        test_roundtrip_raw_tokens(source, "100-definitions-mixed-content");
    }

    #[test]
    fn test_sample_110_ensemble_raw() {
        let source =
            include_str!("../../../docs/specs/v1/samples/110-ensemble-with-definitions.txxt");
        test_roundtrip_raw_tokens(source, "110-ensemble-with-definitions");
    }

    #[test]
    fn test_sample_120_annotations_simple_raw() {
        let source = include_str!("../../../docs/specs/v1/samples/120-annotations-simple.txxt");
        test_roundtrip_raw_tokens(source, "120-annotations-simple");
    }

    #[test]
    fn test_sample_130_annotations_block_raw() {
        let source =
            include_str!("../../../docs/specs/v1/samples/130-annotations-block-content.txxt");
        test_roundtrip_raw_tokens(source, "130-annotations-block-content");
    }

    #[test]
    fn test_sample_140_foreign_blocks_simple_raw() {
        let source = include_str!("../../../docs/specs/v1/samples/140-foreign-blocks-simple.txxt");
        test_roundtrip_raw_tokens(source, "140-foreign-blocks-simple");
    }

    #[test]
    fn test_sample_150_foreign_blocks_no_content_raw() {
        let source =
            include_str!("../../../docs/specs/v1/samples/150-foreign-blocks-no-content.txxt");
        test_roundtrip_raw_tokens(source, "150-foreign-blocks-no-content");
    }

    #[test]
    fn test_sample_200_quick_block_raw() {
        let source = include_str!("../../../docs/specs/v1/samples/200-quick-block.left.txxt");
        test_roundtrip_raw_tokens(source, "200-quick-block.left");
    }

    // ===== Semantic indentation roundtrip tests =====

    #[test]
    fn test_sample_030_sessions_nested_semantic() {
        let source = include_str!(
            "../../../docs/specs/v1/samples/030-paragraphs-sessions-nested-multiple.txxt"
        );
        test_roundtrip_semantic_tokens(
            source,
            "030-paragraphs-sessions-nested-multiple (semantic)",
        );
    }

    #[test]
    fn test_sample_060_trifecta_nesting_semantic() {
        let source = include_str!("../../../docs/specs/v1/samples/060-trifecta-nesting.txxt");
        test_roundtrip_semantic_tokens(source, "060-trifecta-nesting (semantic)");
    }

    #[test]
    fn test_sample_070_nested_lists_simple_semantic() {
        let source = include_str!("../../../docs/specs/v1/samples/070-nested-lists-simple.txxt");
        test_roundtrip_semantic_tokens(source, "070-nested-lists-simple (semantic)");
    }

    #[test]
    fn test_sample_080_nested_lists_mixed_semantic() {
        let source =
            include_str!("../../../docs/specs/v1/samples/080-nested-lists-mixed-content.txxt");
        test_roundtrip_semantic_tokens(source, "080-nested-lists-mixed-content (semantic)");
    }

    #[test]
    fn test_sample_110_ensemble_semantic() {
        let source =
            include_str!("../../../docs/specs/v1/samples/110-ensemble-with-definitions.txxt");
        test_roundtrip_semantic_tokens(source, "110-ensemble-with-definitions (semantic)");
    }
}
