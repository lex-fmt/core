//! File processing API for txxt format
//!
//! This module provides an extensible API for processing txxt files with different
//! stages (token, ast) and formats (simple, json, xml, etc.).

use crate::txxt_nano::lexer::{tokenize, Token};
use std::fmt;
use std::fs;
use std::path::Path;

/// Represents the processing stage (what data to extract)
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessingStage {
    Token,
    Ast, // Future: AST processing
}

/// Represents the output format
#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Simple,
    Json,
    Xml, // Future: XML output
}

/// Represents a complete processing specification
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessingSpec {
    pub stage: ProcessingStage,
    pub format: OutputFormat,
}

impl ProcessingSpec {
    /// Parse a format string like "token-simple" or "ast-json"
    pub fn from_string(format_str: &str) -> Result<Self, ProcessingError> {
        let parts: Vec<&str> = format_str.split('-').collect();
        if parts.len() != 2 {
            return Err(ProcessingError::InvalidFormat(format_str.to_string()));
        }

        let stage = match parts[0] {
            "token" => ProcessingStage::Token,
            "ast" => return Err(ProcessingError::InvalidStage(parts[0].to_string())), // AST not implemented yet
            _ => return Err(ProcessingError::InvalidStage(parts[0].to_string())),
        };

        let format = match parts[1] {
            "simple" => OutputFormat::Simple,
            "json" => OutputFormat::Json,
            "xml" => return Err(ProcessingError::InvalidFormatType(parts[1].to_string())), // XML not implemented yet
            _ => return Err(ProcessingError::InvalidFormatType(parts[1].to_string())),
        };

        Ok(ProcessingSpec { stage, format })
    }

    /// Get all available processing specifications
    pub fn available_specs() -> Vec<ProcessingSpec> {
        vec![
            ProcessingSpec {
                stage: ProcessingStage::Token,
                format: OutputFormat::Simple,
            },
            ProcessingSpec {
                stage: ProcessingStage::Token,
                format: OutputFormat::Json,
            },
            // Future: AST formats
        ]
    }
}

/// Errors that can occur during processing
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessingError {
    FileNotFound(String),
    InvalidFormat(String),
    InvalidStage(String),
    InvalidFormatType(String),
    IoError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::FileNotFound(path) => write!(f, "File not found: {}", path),
            ProcessingError::InvalidFormat(format) => write!(f, "Invalid format: {}", format),
            ProcessingError::InvalidStage(stage) => write!(f, "Invalid stage: {}", stage),
            ProcessingError::InvalidFormatType(format_type) => {
                write!(f, "Invalid format type: {}", format_type)
            }
            ProcessingError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

/// Process a txxt file according to the given specification
pub fn process_file<P: AsRef<Path>>(
    file_path: P,
    spec: &ProcessingSpec,
) -> Result<String, ProcessingError> {
    let file_path = file_path.as_ref();

    // Read the file
    let content =
        fs::read_to_string(file_path).map_err(|e| ProcessingError::IoError(e.to_string()))?;

    // Process according to stage
    match spec.stage {
        ProcessingStage::Token => {
            let tokens = tokenize(&content);
            format_tokens(&tokens, &spec.format)
        }
        ProcessingStage::Ast => {
            // Future: AST processing
            Err(ProcessingError::InvalidStage("ast".to_string()))
        }
    }
}

/// Format tokens according to the specified format
fn format_tokens(tokens: &[Token], format: &OutputFormat) -> Result<String, ProcessingError> {
    match format {
        OutputFormat::Simple => {
            let mut result = String::new();
            for token in tokens {
                result.push_str(&format!("{}", token));
                if matches!(token, Token::Newline) {
                    result.push('\n');
                }
            }
            Ok(result)
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(tokens)
                .map_err(|e| ProcessingError::IoError(e.to_string()))?;
            Ok(json)
        }
        OutputFormat::Xml => {
            // Future: XML formatting
            Err(ProcessingError::InvalidFormatType("xml".to_string()))
        }
    }
}

/// Get all available format strings
pub fn available_formats() -> Vec<String> {
    ProcessingSpec::available_specs()
        .into_iter()
        .map(|spec| {
            format!(
                "{}-{}",
                match spec.stage {
                    ProcessingStage::Token => "token",
                    ProcessingStage::Ast => "ast",
                },
                match spec.format {
                    OutputFormat::Simple => "simple",
                    OutputFormat::Json => "json",
                    OutputFormat::Xml => "xml",
                }
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processing_spec_parsing() {
        let spec = ProcessingSpec::from_string("token-simple").unwrap();
        assert_eq!(spec.stage, ProcessingStage::Token);
        assert_eq!(spec.format, OutputFormat::Simple);

        let spec = ProcessingSpec::from_string("token-json").unwrap();
        assert_eq!(spec.stage, ProcessingStage::Token);
        assert_eq!(spec.format, OutputFormat::Json);

        assert!(ProcessingSpec::from_string("invalid").is_err());
        assert!(ProcessingSpec::from_string("token-invalid").is_err());
        assert!(ProcessingSpec::from_string("invalid-simple").is_err());
    }

    #[test]
    fn test_token_formatting() {
        let tokens = vec![Token::Text, Token::Whitespace, Token::Text, Token::Newline];

        let simple = format_tokens(&tokens, &OutputFormat::Simple).unwrap();
        assert_eq!(simple, "<text><whitespace><text><newline>\n");

        let json = format_tokens(&tokens, &OutputFormat::Json).unwrap();
        assert!(json.contains("\"Text\""));
        assert!(json.contains("\"Whitespace\""));
        assert!(json.contains("\"Newline\""));
    }

    #[test]
    fn test_available_formats() {
        let formats = available_formats();
        assert!(formats.contains(&"token-simple".to_string()));
        assert!(formats.contains(&"token-json".to_string()));
    }
}
