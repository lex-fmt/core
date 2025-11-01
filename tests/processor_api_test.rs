//! Unit tests for the txxt processor API

use std::fs;
use txxt::txxt::lexer::Token;
use txxt::txxt::processor::{
    format_tokenss, process_file, OutputFormat, ProcessingError, ProcessingSpec, ProcessingStage,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processing_spec_parsing() {
        // Test valid specs
        let spec = ProcessingSpec::from_string("token-simple").unwrap();
        assert_eq!(spec.stage, ProcessingStage::Token);
        assert_eq!(spec.format, OutputFormat::Simple);

        let spec = ProcessingSpec::from_string("token-json").unwrap();
        assert_eq!(spec.stage, ProcessingStage::Token);
        assert_eq!(spec.format, OutputFormat::Json);

        // Test AST stage
        let spec = ProcessingSpec::from_string("ast-tag").unwrap();
        assert_eq!(spec.stage, ProcessingStage::Ast);
        assert_eq!(spec.format, OutputFormat::AstTag);

        // Test invalid specs
        assert!(ProcessingSpec::from_string("invalid").is_err());
        assert!(ProcessingSpec::from_string("token-invalid").is_err());
        assert!(ProcessingSpec::from_string("invalid-simple").is_err());
        assert!(ProcessingSpec::from_string("ast-simple").is_err()); // Simple format not supported for AST
    }

    #[test]
    fn test_available_specs() {
        let specs = ProcessingSpec::available_specs();
        assert_eq!(specs.len(), 11); // Added ast-position and 2 experimental formats

        let token_simple = specs
            .iter()
            .find(|s| s.stage == ProcessingStage::Token && s.format == OutputFormat::Simple);
        assert!(token_simple.is_some());

        let token_json = specs
            .iter()
            .find(|s| s.stage == ProcessingStage::Token && s.format == OutputFormat::Json);
        assert!(token_json.is_some());

        let ast_position = specs
            .iter()
            .find(|s| s.stage == ProcessingStage::Ast && s.format == OutputFormat::AstPosition);
        assert!(ast_position.is_some());
    }

    #[test]
    fn test_token_display_format() {
        // Test that tokens display with lowercase dash-separated names
        assert_eq!(format!("{}", Token::TxxtMarker), "<txxt-marker>");
        assert_eq!(format!("{}", Token::Indent), "<indent>");
        assert_eq!(format!("{}", Token::Whitespace), "<whitespace>");
        assert_eq!(format!("{}", Token::Newline), "<newline>");
        assert_eq!(format!("{}", Token::Dash), "<dash>");
        assert_eq!(format!("{}", Token::Period), "<period>");
        assert_eq!(format!("{}", Token::OpenParen), "<open-paren>");
        assert_eq!(format!("{}", Token::CloseParen), "<close-paren>");
        assert_eq!(format!("{}", Token::Colon), "<colon>");
        assert_eq!(
            format!("{}", Token::Number("123".to_string())),
            "<number:123>"
        );
        assert_eq!(
            format!("{}", Token::Text("hello".to_string())),
            "<text:hello>"
        );
    }

    #[test]
    fn test_token_simple_formatting() {
        let tokens = vec![
            Token::Text("a".to_string()),
            Token::Whitespace,
            Token::Text("b".to_string()),
            Token::Newline,
            Token::Indent,
            Token::Dash,
        ];

        let spec = ProcessingSpec {
            stage: ProcessingStage::Token,
            format: OutputFormat::Simple,
        };

        let result = process_file_with_tokens(&tokens, &spec).unwrap();
        let expected = "<text:a><whitespace><text:b><newline>\n<indent><dash>";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_token_json_formatting() {
        let tokens = vec![
            Token::Text("a".to_string()),
            Token::Whitespace,
            Token::Newline,
        ];

        let spec = ProcessingSpec {
            stage: ProcessingStage::Token,
            format: OutputFormat::Json,
        };

        let result = process_file_with_tokens(&tokens, &spec).unwrap();
        assert!(result.contains("\"Text\""));
        assert!(result.contains("\"Whitespace\""));
        assert!(result.contains("\"Newline\""));
        assert!(result.starts_with('['));
        assert!(result.ends_with(']'));
    }

    #[test]
    fn test_file_processing() {
        // Create a temporary test file
        let test_content = "1. Hello world\n    - Item 1";
        let test_file = "test_api.txxt";

        fs::write(test_file, test_content).unwrap();

        // Test token-simple processing
        let spec = ProcessingSpec::from_string("token-simple").unwrap();
        let result = process_file(test_file, &spec).unwrap();

        assert!(result.contains("<number:"));
        assert!(result.contains("<period>"));
        assert!(result.contains("<text:"));
        assert!(result.contains("<newline>"));
        assert!(result.contains("<indent-level>"));
        assert!(result.contains("<dash>"));

        // Test token-json processing
        let spec = ProcessingSpec::from_string("token-json").unwrap();
        let result = process_file(test_file, &spec).unwrap();

        assert!(result.contains("\"Number\""));
        assert!(result.contains("\"Period\""));
        assert!(result.contains("\"Text\""));
        assert!(result.contains("\"Newline\""));
        assert!(result.contains("\"IndentLevel\""));
        assert!(result.contains("\"Dash\""));

        // Clean up
        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_file_not_found_error() {
        let spec = ProcessingSpec::from_string("token-simple").unwrap();
        let result = process_file("nonexistent.txxt", &spec);

        assert!(result.is_err());
        match result.unwrap_err() {
            ProcessingError::IoError(_) => {} // Expected
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_ast_tag_format() {
        // AST tag format is now implemented
        let result = ProcessingSpec::from_string("ast-tag");
        assert!(result.is_ok());
        let spec = result.unwrap();
        assert_eq!(spec.stage, ProcessingStage::Ast);
        assert_eq!(spec.format, OutputFormat::AstTag);

        // But ast-simple is still not supported
        let result = ProcessingSpec::from_string("ast-simple");
        assert!(result.is_err());
        match result.unwrap_err() {
            ProcessingError::InvalidFormatType(_) => {} // Expected
            _ => panic!("Expected InvalidFormatType error"),
        }
    }

    #[test]
    fn test_xml_format_not_implemented() {
        let result = ProcessingSpec::from_string("token-xml");
        assert!(result.is_err());
        match result.unwrap_err() {
            ProcessingError::InvalidFormatType(_) => {} // Expected
            _ => panic!("Expected InvalidFormatType error"),
        }
    }

    #[test]
    fn test_line_break_handling_in_simple_format() {
        let tokens = vec![
            Token::Text("a".to_string()),
            Token::Newline,
            Token::Text("b".to_string()),
            Token::Newline,
            Token::Text("c".to_string()),
        ];

        let spec = ProcessingSpec {
            stage: ProcessingStage::Token,
            format: OutputFormat::Simple,
        };

        let result = process_file_with_tokens(&tokens, &spec).unwrap();
        let lines: Vec<&str> = result.split('\n').collect();

        // Should have 3 lines (2 newlines + 1 final line)
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "<text:a><newline>");
        assert_eq!(lines[1], "<text:b><newline>");
        assert_eq!(lines[2], "<text:c>");
    }

    // Helper function to test formatting without file I/O
    // Wraps tokens with dummy location information and delegates to format_tokenss
    fn process_file_with_tokens(
        tokens: &[Token],
        spec: &ProcessingSpec,
    ) -> Result<String, ProcessingError> {
        match spec.stage {
            ProcessingStage::Token => {
                // Convert tokens to include dummy location information
                let tokens_with_loc: Vec<(Token, std::ops::Range<usize>)> = tokens
                    .iter()
                    .enumerate()
                    .map(|(i, t)| (t.clone(), i..i + 1))
                    .collect();
                format_tokenss(&tokens_with_loc, &spec.format)
            }
            ProcessingStage::Ast => Err(ProcessingError::InvalidStage("ast".to_string())),
        }
    }

    #[test]
    fn test_ast_treeviz_format_complex() {
        let file_path = "docs/specs/v1/samples/060-trifecta-nesting.txxt";
        let spec = ProcessingSpec::from_string("ast-treeviz").unwrap();
        let result = process_file(file_path, &spec).unwrap();

        let expected = format!(
            "{}\n{}",
            "⧉ Document (0 metadata, 5 items)",
            include_str!("fixtures/treeviz_expected_body.txt").trim_end_matches('\n')
        );

        let normalized = result
            .replace("\r\n", "\n")
            .trim_end_matches('\n')
            .to_string();
        let expected_normalized = expected
            .replace("\r\n", "\n")
            .trim_end_matches('\n')
            .to_string();

        assert_eq!(
            normalized, expected_normalized,
            "treeviz output doesn't match expected format"
        );
    }
}
