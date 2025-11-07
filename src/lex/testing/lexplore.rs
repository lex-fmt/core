//! Test harness for per-element testing
//!
//! This module provides utilities for testing individual element variations
//! using the per-element library in `docs/specs/v1/elements/`.
//!
//! # Module Organization
//!
//! - `loader`: File loading, parsing, and tokenization infrastructure
//! - `extraction`: AST node extraction and assertion helpers
//!
//! # Usage
//!
//! ```rust,ignore
//! use lex::lex::testing::lexplore::*;
//!
//! // Load and parse elements
//! let parsed = Lexplore::paragraph(1).parse();
//! let paragraph = parsed.expect_paragraph();
//!
//! // Load and tokenize
//! let tokens = Lexplore::paragraph(1).tokenize();
//!
//! // Load from arbitrary paths
//! let doc = Lexplore::from_path("path/to/file.lex").parse();
//!
//! // Use extraction helpers
//! assert!(paragraph_text_starts_with(&paragraph, "This is"));
//! ```

mod extraction;
mod loader;

// Re-export everything public from submodules
pub use extraction::*;
pub use loader::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::ast::traits::Container;
    use crate::lex::lexing::Token;

    #[test]
    fn test_get_source_for_paragraph() {
        let source = Lexplore::get_source_for(ElementType::Paragraph, 1);
        assert!(source.is_ok(), "Should find paragraph-01 file");
        let content = source.unwrap();
        assert!(!content.is_empty());
    }

    #[test]
    fn test_list_numbers_for_paragraphs() {
        let numbers = Lexplore::list_numbers_for(ElementType::Paragraph).unwrap();
        assert!(!numbers.is_empty());
        assert!(numbers.contains(&1));
    }

    #[test]
    fn test_parse_with_reference_parser() {
        let source = Lexplore::get_source_for(ElementType::Paragraph, 1).unwrap();
        let doc = parse_with_parser(&source, Parser::Reference);
        assert!(doc.is_ok(), "Reference parser should parse successfully");
    }

    #[test]
    fn test_get_first_paragraph() {
        let source = Lexplore::get_source_for(ElementType::Paragraph, 1).unwrap();
        let doc = parse_with_parser(&source, Parser::Reference).unwrap();
        let paragraph = get_first_paragraph(&doc);
        assert!(paragraph.is_some());
    }

    #[test]
    fn test_paragraph_assertions() {
        let source = Lexplore::get_source_for(ElementType::Paragraph, 1).unwrap();
        let doc = parse_with_parser(&source, Parser::Reference).unwrap();
        let paragraph = get_first_paragraph(&doc).unwrap();

        // Note: parsers may have bugs, we're just testing the infrastructure
        // The important part is that we can get a paragraph and check its content
        assert!(paragraph_text_starts_with(paragraph, "This is a simple"));
    }

    // ===== Fluent API Tests =====

    #[test]
    fn test_fluent_api_basic() {
        // Test: Lexplore::paragraph(1).parse().expect_paragraph()
        let parsed = Lexplore::paragraph(1).parse();
        let paragraph = parsed.expect_paragraph();

        assert!(paragraph_text_starts_with(paragraph, "This is a simple"));
    }

    #[test]
    fn test_fluent_api_with_parser_selection() {
        // Test with explicit parser selection
        let parsed = Lexplore::paragraph(1).parse_with(Parser::Reference);
        let paragraph = parsed.expect_paragraph();

        assert!(paragraph_text_starts_with(paragraph, "This is a simple"));
    }

    #[test]
    fn test_fluent_api_source_only() {
        // Get just the source without parsing
        let source = Lexplore::paragraph(1).source();
        assert!(source.contains("simple"));
    }

    #[test]
    fn test_fluent_api_list() {
        let parsed = Lexplore::list(1).parse();
        let list = parsed.expect_list();

        assert!(!list.items.is_empty());
    }

    #[test]
    fn test_fluent_api_session() {
        let parsed = Lexplore::session(1).parse();
        let session = parsed.expect_session();

        assert!(!session.label().is_empty());
    }

    #[test]
    fn test_fluent_api_definition() {
        let parsed = Lexplore::definition(1).parse();
        let definition = parsed.expect_definition();

        assert!(!definition.label().is_empty());
    }

    #[test]
    fn test_must_methods() {
        // Test must_get_source_for
        let source = Lexplore::must_get_source_for(ElementType::Paragraph, 1);
        assert!(!source.is_empty());

        // Test must_get_ast_for
        let doc = Lexplore::must_get_ast_for(ElementType::Paragraph, 1, Parser::Reference);
        assert!(!doc.root.children.is_empty());
    }

    // ===== Document Collection Tests =====

    #[test]
    fn test_benchmark_fluent_api() {
        // Test: Lexplore::benchmark(10).parse()
        let parsed = Lexplore::benchmark(10).parse();
        let doc = parsed.document();

        // Benchmark documents should have multiple elements
        assert!(!doc.root.children.is_empty());
    }

    #[test]
    fn test_trifecta_fluent_api() {
        // Test: Lexplore::trifecta(0).parse()
        let parsed = Lexplore::trifecta(0).parse();
        let doc = parsed.document();

        // Trifecta documents should have content
        assert!(!doc.root.children.is_empty());
    }

    #[test]
    fn test_benchmark_source_only() {
        // Get just the source without parsing
        let source = Lexplore::benchmark(10).source();
        assert!(!source.is_empty());
    }

    #[test]
    fn test_trifecta_source_only() {
        // Get just the source without parsing
        let source = Lexplore::trifecta(0).source();
        assert!(!source.is_empty());
    }

    #[test]
    fn test_get_document_source_for() {
        let source = Lexplore::get_document_source_for(DocumentType::Benchmark, 10);
        assert!(source.is_ok(), "Should find benchmark-010 file");
        let content = source.unwrap();
        assert!(!content.is_empty());
    }

    #[test]
    fn test_must_get_document_source_for() {
        let source = Lexplore::must_get_document_source_for(DocumentType::Trifecta, 0);
        assert!(!source.is_empty());
    }

    #[test]
    fn test_get_document_ast_for() {
        let doc = Lexplore::get_document_ast_for(DocumentType::Benchmark, 10, Parser::Reference);
        assert!(doc.is_ok(), "Should parse benchmark document");
        assert!(!doc.unwrap().root.children.is_empty());
    }

    #[test]
    fn test_must_get_document_ast_for() {
        let doc = Lexplore::must_get_document_ast_for(DocumentType::Trifecta, 0, Parser::Reference);
        assert!(!doc.root.children.is_empty());
    }

    // ===== Tokenization Tests =====

    #[test]
    fn test_tokenize_paragraph() {
        // Test: Lexplore::paragraph(1).tokenize()
        let parsed_tokens = Lexplore::paragraph(1).tokenize();

        assert!(!parsed_tokens.is_empty());
        assert!(!parsed_tokens.is_empty());
    }

    #[test]
    fn test_tokenize_with_parser() {
        // Test with explicit parser selection
        let parsed_tokens = Lexplore::paragraph(1).tokenize_with(Parser::Reference);

        assert!(!parsed_tokens.is_empty());
        // Should have text tokens
        assert!(parsed_tokens.has_token(|t| matches!(t, Token::Text(_))));
    }

    #[test]
    fn test_tokenize_list() {
        let parsed_tokens = Lexplore::list(1).tokenize();

        // Lists should have dash or number tokens
        assert!(
            parsed_tokens.has_token(|t| matches!(t, Token::Dash))
                || parsed_tokens.has_token(|t| matches!(t, Token::Number(_)))
        );
    }

    #[test]
    fn test_tokenize_benchmark() {
        let parsed_tokens = Lexplore::benchmark(10).tokenize();

        assert!(!parsed_tokens.is_empty());
        // Benchmark should have multiple types of tokens
        assert!(parsed_tokens.len() > 10);
    }

    #[test]
    fn test_tokenize_trifecta() {
        let parsed_tokens = Lexplore::trifecta(0).tokenize();

        assert!(!parsed_tokens.is_empty());
        assert!(parsed_tokens.has_token(|t| matches!(t, Token::Text(_))));
    }

    #[test]
    fn test_get_tokens_for() {
        let tokens = Lexplore::get_tokens_for(ElementType::Paragraph, 1, Parser::Reference);
        assert!(tokens.is_ok());
        assert!(!tokens.unwrap().is_empty());
    }

    #[test]
    fn test_must_get_tokens_for() {
        let tokens = Lexplore::must_get_tokens_for(ElementType::Paragraph, 1, Parser::Reference);
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_get_document_tokens_for() {
        let tokens =
            Lexplore::get_document_tokens_for(DocumentType::Benchmark, 10, Parser::Reference);
        assert!(tokens.is_ok());
        assert!(!tokens.unwrap().is_empty());
    }

    #[test]
    fn test_must_get_document_tokens_for() {
        let tokens =
            Lexplore::must_get_document_tokens_for(DocumentType::Trifecta, 0, Parser::Reference);
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_parsed_tokens_methods() {
        let parsed_tokens = Lexplore::paragraph(1).tokenize();

        // Test len/is_empty
        assert!(!parsed_tokens.is_empty());
        assert!(!parsed_tokens.is_empty());

        // Test tokens()
        let tokens = parsed_tokens.tokens();
        assert!(!tokens.is_empty());

        // Test find_token
        let text_token = parsed_tokens.find_token(|t| matches!(t, Token::Text(_)));
        assert!(text_token.is_some());

        // Test count_tokens
        let text_count = parsed_tokens.count_tokens(|t| matches!(t, Token::Text(_)));
        assert!(text_count > 0);

        // Test has_token
        assert!(parsed_tokens.has_token(|t| matches!(t, Token::Text(_))));
        assert!(parsed_tokens.has_token(|t| matches!(t, Token::Newline)));
    }

    #[test]
    fn test_tokenize_with_parser_function() {
        let source = Lexplore::must_get_source_for(ElementType::Paragraph, 1);
        let tokens = tokenize_with_parser(&source, Parser::Reference);

        assert!(tokens.is_ok());
        let tokens = tokens.unwrap();
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_tokenize_linebased_parser() {
        let parsed_tokens = Lexplore::paragraph(1).tokenize_with(Parser::Linebased);

        assert!(!parsed_tokens.is_empty());
        // Linebased should also produce tokens
        assert!(parsed_tokens.has_token(|t| matches!(t, Token::Text(_))));
    }

    // ===== Path-based Loading Tests =====

    #[test]
    fn test_from_path_parse() {
        // Load a paragraph file by path
        let path = "docs/specs/v1/elements/paragraph/paragraph-01-flat-oneline.lex";
        let parsed = Lexplore::from_path(path).parse();

        let paragraph = parsed.expect_paragraph();
        assert!(!paragraph.text().is_empty());
    }

    #[test]
    fn test_from_path_tokenize() {
        // Load and tokenize a file by path
        let path = "docs/specs/v1/elements/paragraph/paragraph-01-flat-oneline.lex";
        let tokens = Lexplore::from_path(path).tokenize();

        assert!(!tokens.is_empty());
        assert!(tokens.has_token(|t| matches!(t, Token::Text(_))));
    }

    #[test]
    fn test_from_path_source() {
        // Get just the source string
        let path = "docs/specs/v1/elements/paragraph/paragraph-01-flat-oneline.lex";
        let source = Lexplore::from_path(path).source();

        assert!(!source.is_empty());
    }

    #[test]
    fn test_from_path_with_parser() {
        // Test with explicit parser selection
        let path = "docs/specs/v1/elements/list/list-01-flat-simple-dash.lex";
        let parsed = Lexplore::from_path(path).parse_with(Parser::Reference);

        let list = parsed.expect_list();
        assert!(!list.items.is_empty());
    }

    #[test]
    fn test_from_path_tokenize_with_parser() {
        // Test tokenization with explicit parser
        let path = "docs/specs/v1/elements/paragraph/paragraph-01-flat-oneline.lex";
        let tokens = Lexplore::from_path(path).tokenize_with(Parser::Linebased);

        assert!(!tokens.is_empty());
        assert!(tokens.has_token(|t| matches!(t, Token::Text(_))));
    }

    #[test]
    fn test_get_source_from_path() {
        let path = "docs/specs/v1/elements/paragraph/paragraph-01-flat-oneline.lex";
        let source = Lexplore::get_source_from_path(path);

        assert!(source.is_ok());
        assert!(!source.unwrap().is_empty());
    }

    #[test]
    fn test_must_get_source_from_path() {
        let path = "docs/specs/v1/elements/paragraph/paragraph-01-flat-oneline.lex";
        let source = Lexplore::must_get_source_from_path(path);

        assert!(!source.is_empty());
    }

    #[test]
    fn test_get_ast_from_path() {
        let path = "docs/specs/v1/elements/paragraph/paragraph-01-flat-oneline.lex";
        let doc = Lexplore::get_ast_from_path(path, Parser::Reference);

        assert!(doc.is_ok());
        assert!(!doc.unwrap().root.children.is_empty());
    }

    #[test]
    fn test_must_get_ast_from_path() {
        let path = "docs/specs/v1/elements/paragraph/paragraph-01-flat-oneline.lex";
        let doc = Lexplore::must_get_ast_from_path(path, Parser::Reference);

        assert!(!doc.root.children.is_empty());
    }

    #[test]
    fn test_get_tokens_from_path() {
        let path = "docs/specs/v1/elements/paragraph/paragraph-01-flat-oneline.lex";
        let tokens = Lexplore::get_tokens_from_path(path, Parser::Reference);

        assert!(tokens.is_ok());
        assert!(!tokens.unwrap().is_empty());
    }

    #[test]
    fn test_must_get_tokens_from_path() {
        let path = "docs/specs/v1/elements/paragraph/paragraph-01-flat-oneline.lex";
        let tokens = Lexplore::must_get_tokens_from_path(path, Parser::Reference);

        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_from_path_with_benchmark() {
        // Load a benchmark document by path
        let path = "docs/specs/v1/benchmark/010-kitchensink.lex";
        let parsed = Lexplore::from_path(path).parse();

        let doc = parsed.document();
        assert!(!doc.root.children.is_empty());
    }

    #[test]
    fn test_from_path_with_trifecta() {
        // Load a trifecta document by path
        let path = "docs/specs/v1/trifecta/000-paragraphs.lex";
        let parsed = Lexplore::from_path(path).parse();

        let doc = parsed.document();
        assert!(!doc.root.children.is_empty());
    }
}
