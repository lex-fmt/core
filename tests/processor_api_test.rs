//! Unit tests for the txxt processor API

use std::fs;
use txxt_nano::txxt_nano::lexer::Token;
use txxt_nano::txxt_nano::processor::{
    process_file, OutputFormat, ProcessingError, ProcessingSpec, ProcessingStage,
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
        assert_eq!(specs.len(), 6);

        let token_simple = specs
            .iter()
            .find(|s| s.stage == ProcessingStage::Token && s.format == OutputFormat::Simple);
        assert!(token_simple.is_some());

        let token_json = specs
            .iter()
            .find(|s| s.stage == ProcessingStage::Token && s.format == OutputFormat::Json);
        assert!(token_json.is_some());
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
        assert_eq!(format!("{}", Token::Number), "<number>");
        assert_eq!(format!("{}", Token::Text), "<text>");
    }

    #[test]
    fn test_token_simple_formatting() {
        let tokens = vec![
            Token::Text,
            Token::Whitespace,
            Token::Text,
            Token::Newline,
            Token::Indent,
            Token::Dash,
        ];

        let spec = ProcessingSpec {
            stage: ProcessingStage::Token,
            format: OutputFormat::Simple,
        };

        let result = process_file_with_tokens(&tokens, &spec).unwrap();
        let expected = "<text><whitespace><text><newline>\n<indent><dash>";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_token_json_formatting() {
        let tokens = vec![Token::Text, Token::Whitespace, Token::Newline];

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

        assert!(result.contains("<number>"));
        assert!(result.contains("<period>"));
        assert!(result.contains("<text>"));
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
            Token::Text,
            Token::Newline,
            Token::Text,
            Token::Newline,
            Token::Text,
        ];

        let spec = ProcessingSpec {
            stage: ProcessingStage::Token,
            format: OutputFormat::Simple,
        };

        let result = process_file_with_tokens(&tokens, &spec).unwrap();
        let lines: Vec<&str> = result.split('\n').collect();

        // Should have 3 lines (2 newlines + 1 final line)
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "<text><newline>");
        assert_eq!(lines[1], "<text><newline>");
        assert_eq!(lines[2], "<text>");
    }

    // Helper function to test formatting without file I/O
    fn process_file_with_tokens(
        tokens: &[Token],
        spec: &ProcessingSpec,
    ) -> Result<String, ProcessingError> {
        match spec.stage {
            ProcessingStage::Token => match spec.format {
                OutputFormat::Simple | OutputFormat::RawSimple => {
                    let mut result = String::new();
                    for token in tokens {
                        result.push_str(&format!("{}", token));
                        if matches!(token, Token::Newline) {
                            result.push('\n');
                        }
                    }
                    Ok(result)
                }
                OutputFormat::Json | OutputFormat::RawJson => {
                    let json = serde_json::to_string_pretty(tokens)
                        .map_err(|e| ProcessingError::IoError(e.to_string()))?;
                    Ok(json)
                }
                OutputFormat::Xml => Err(ProcessingError::InvalidFormatType("xml".to_string())),
                OutputFormat::AstTag => Err(ProcessingError::InvalidFormatType(
                    "ast-tag format only works with ast stage".to_string(),
                )),
                OutputFormat::AstTreeviz => Err(ProcessingError::InvalidFormatType(
                    "ast-treeviz format only works with ast stage".to_string(),
                )),
            },
            ProcessingStage::Ast => Err(ProcessingError::InvalidStage("ast".to_string())),
        }
    }

    #[test]
    fn test_ast_treeviz_format_complex() {
        let file_path = "docs/specs/v1/samples/060-trifecta-nesting.txxt";
        let spec = ProcessingSpec::from_string("ast-treeviz").unwrap();
        let result = process_file(file_path, &spec).unwrap();
        let expected = "├─ Paragraph: Trifecta Nesting Test {{paragr...\n├─ Paragraph: This document tests the combin...\n├─ Session: 1. Root Session {{session-titl...\n│ ├─ Paragraph: This root session contains var...\n│ ├─ Session: 1.1. Sub-session with Paragrap...\n│ │ ├─ Paragraph: This sub-session starts with a...\n│ │ └─ List: 2 items\n│ │   ├─ ListItem: - Then has a list {{list-item}...\n│ │   └─ ListItem: - With multiple items {{list-i...\n│ ├─ Session: 1.2. Sub-session with List {{s...\n│ │ ├─ List: 2 items\n│ │ │ ├─ ListItem: - Starts with a list {{list-it...\n│ │ │ └─ ListItem: - Has multiple items {{list-it...\n│ │ ├─ Paragraph: Then has a paragraph. {{paragr...\n│ │ └─ Session: 1.2.1. Deeply Nested Session {...\n│ │   ├─ Paragraph: This is a deeply nested sessio...\n│ │   ├─ List: 2 items\n│ │   │ ├─ ListItem: - With its own list {{list-ite...\n│ │   │ └─ ListItem: - And multiple items {{list-it...\n│ │   └─ List: 2 items\n│ │     ├─ ListItem: - Another list follows {{list-...\n│ │     └─ ListItem: - In the same session {{list-i...\n│ ├─ Paragraph: Back to the root session level...\n│ └─ List: 2 items\n│   ├─ ListItem: - Root session can also have l...\n│   └─ ListItem: - At its own level {{list-item...\n├─ Session: 2. Another Root Session {{sess...\n│ ├─ Paragraph: This session demonstrates diff...\n│ └─ Session: 2.1. Mixed Content Sub-session...\n│   ├─ List: 2 items\n│   │ ├─ ListItem: - Starts with list {{list-item...\n│   │ └─ ListItem: - Multiple items {{list-item}}\n│   ├─ Paragraph: Paragraph in the middle. {{par...\n│   ├─ List: 2 items\n│   │ ├─ ListItem: - Ends with another list {{lis...\n│   │ └─ ListItem: - To complete the pattern {{li...\n│   └─ Session: 2.1.1. Even Deeper Nesting {{s...\n│     ├─ Paragraph: The deepest level contains par...\n│     ├─ List: 2 items\n│     │ ├─ ListItem: - First deep list {{list-item}...\n│     │ └─ ListItem: - Second deep item {{list-item...\n│     ├─ Paragraph: Another paragraph at deep leve...\n│     └─ List: 2 items\n│       ├─ ListItem: - Second deep list {{list-item...\n│       └─ ListItem: - Completing the deep structur...\n└─ Paragraph: Final root level paragraph. {{...\n";
        assert_eq!(result.replace("\r\n", "\n"), expected);
    }
}
