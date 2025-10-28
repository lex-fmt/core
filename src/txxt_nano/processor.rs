//! File processing API for txxt format
//!
//! This module provides an extensible API for processing txxt files with different
//! stages (token, ast) and formats (simple, json, xml, etc.).
//!
//! # Sample Sources
//!
//! The `txxt_sources` module provides access to verified txxt sample files for testing.
//! These samples are the only canonical sources for txxt content and should be used
//! instead of copying content to ensure tests use the latest specification.
//!
//! ## Example Usage
//!
//! ```rust
//! use txxt_nano::txxt_nano::processor::txxt_sources::TxxtSources;
//!
//! // Get raw string content
//! let content = TxxtSources::get_string("000-paragraphs.txxt").unwrap();
//!
//! // Get tokenized content
//! let tokens = TxxtSources::get_tokens("040-lists.txxt").unwrap();
//!
//! // Get processed content in simple format
//! let processed = TxxtSources::get_processed("050-paragraph-lists.txxt", "token-simple").unwrap();
//! ```

use crate::txxt_nano::lexer::{lex, tokenize, Token};
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
    RawSimple,
    RawJson,
    Xml,    // Future: XML output
    AstTag, // AST XML-like tag format
    AstTreeviz,
}

/// Represents a complete processing specification
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessingSpec {
    pub stage: ProcessingStage,
    pub format: OutputFormat,
}

impl ProcessingSpec {
    /// Parse a format string like "token-simple" or "token-raw-simple"
    pub fn from_string(format_str: &str) -> Result<Self, ProcessingError> {
        let parts: Vec<&str> = format_str.split('-').collect();
        if parts.len() < 2 {
            return Err(ProcessingError::InvalidFormat(format_str.to_string()));
        }

        let stage = match parts[0] {
            "token" => ProcessingStage::Token,
            "ast" => ProcessingStage::Ast,
            _ => return Err(ProcessingError::InvalidStage(parts[0].to_string())),
        };

        let format = match parts[1..].join("-").as_str() {
            "simple" => OutputFormat::Simple,
            "json" => OutputFormat::Json,
            "raw-simple" => OutputFormat::RawSimple,
            "raw-json" => OutputFormat::RawJson,
            "xml" => return Err(ProcessingError::InvalidFormatType("xml".to_string())), // XML not implemented yet
            "tag" => OutputFormat::AstTag,
            "treeviz" => OutputFormat::AstTreeviz,
            _ => return Err(ProcessingError::InvalidFormatType(parts[1..].join("-"))),
        };

        // Validate stage/format compatibility
        match (&stage, &format) {
            (ProcessingStage::Ast, OutputFormat::AstTag) => {} // Valid
            (ProcessingStage::Ast, OutputFormat::AstTreeviz) => {} // Valid
            (ProcessingStage::Ast, _) => {
                return Err(ProcessingError::InvalidFormatType(format!(
                    "Format '{:?}' not supported for AST stage (only 'tag' and 'treeviz' are supported)",
                    format
                )))
            }
            (ProcessingStage::Token, OutputFormat::AstTag) => {
                return Err(ProcessingError::InvalidFormatType(
                    "Format 'tag' only works with AST stage".to_string(),
                ))
            }
            (ProcessingStage::Token, OutputFormat::AstTreeviz) => {
                return Err(ProcessingError::InvalidFormatType(
                    "Format 'treeviz' only works with AST stage".to_string(),
                ))
            }
            _ => {} // Token stage with other formats is fine
        }

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
            ProcessingSpec {
                stage: ProcessingStage::Token,
                format: OutputFormat::RawSimple,
            },
            ProcessingSpec {
                stage: ProcessingStage::Token,
                format: OutputFormat::RawJson,
            },
            ProcessingSpec {
                stage: ProcessingStage::Ast,
                format: OutputFormat::AstTag,
            },
            ProcessingSpec {
                stage: ProcessingStage::Ast,
                format: OutputFormat::AstTreeviz,
            },
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

impl std::error::Error for ProcessingError {}

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
            let tokens = match spec.format {
                OutputFormat::RawSimple | OutputFormat::RawJson => tokenize(&content),
                _ => lex(&content),
            };
            format_tokens(&tokens, &spec.format)
        }
        ProcessingStage::Ast => {
            // Parse the document
            let doc = crate::txxt_nano::parser::parse_document(&content).map_err(|errs| {
                let error_details = errs
                    .iter()
                    .map(|e| {
                        format!(
                            "  Parse error at span {:?}: reason={:?}, found={:?}",
                            e.span(),
                            e.reason(),
                            e.found()
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                ProcessingError::IoError(format!("Failed to parse document:\n{}", error_details))
            })?;

            // Format according to output format
            match spec.format {
                OutputFormat::AstTag => Ok(crate::txxt_nano::parser::serialize_ast_tag(&doc)),
                OutputFormat::AstTreeviz => Ok(crate::txxt_nano::parser::to_treeviz_str(&doc)),
                _ => Err(ProcessingError::InvalidFormatType(
                    "Only ast-tag and ast-treeviz formats are supported for AST stage".to_string(),
                )),
            }
        }
    }
}

/// Format tokens according to the specified format
fn format_tokens(tokens: &[Token], format: &OutputFormat) -> Result<String, ProcessingError> {
    match format {
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
        OutputFormat::Xml => {
            // Future: XML formatting
            Err(ProcessingError::InvalidFormatType("xml".to_string()))
        }
        OutputFormat::AstTag => {
            // AstTag only works with AST stage, not Token stage
            Err(ProcessingError::InvalidFormatType(
                "ast-tag format only works with ast stage".to_string(),
            ))
        }
        OutputFormat::AstTreeviz => Err(ProcessingError::InvalidFormatType(
            "ast-treeviz format only works with ast stage".to_string(),
        )),
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
                    OutputFormat::RawSimple => "raw-simple",
                    OutputFormat::RawJson => "raw-json",
                    OutputFormat::Xml => "xml",
                    OutputFormat::AstTag => "tag",
                    OutputFormat::AstTreeviz => "treeviz",
                }
            )
        })
        .collect()
}

/// Sample sources module for accessing verified txxt test files
pub mod txxt_sources {
    use super::*;

    /// The current specification version - change this when spec updates
    pub const SPEC_VERSION: &str = "v1";

    /// Available sample files (canonical sources)
    pub const AVAILABLE_SAMPLES: &[&str] = &[
        "000-paragraphs.txxt",
        "010-paragraphs-sessions-flat-single.txxt",
        "020-paragraphs-sessions-flat-multiple.txxt",
        "030-paragraphs-sessions-nested-multiple.txxt",
        "040-lists.txxt",
        "050-paragraph-lists.txxt",
        "050-trifecta-flat-simple.txxt",
        "060-trifecta-nesting.txxt",
        "070-nested-lists-simple.txxt",
        "080-nested-lists-mixed-content.txxt",
        "090-definitions-simple.txxt",
        "100-definitions-mixed-content.txxt",
        "110-ensemble-with-definitions.txxt",
        "120-annotations-simple.txxt",
        "130-annotations-block-content.txxt",
        "140-foreign-blocks-simple.txxt",
        "150-foreign-blocks-no-content.txxt",
    ];

    /// Format options for sample content
    #[derive(Debug, Clone, PartialEq)]
    pub enum SampleFormat {
        /// Raw string content
        String,
        /// Tokenized content (Vec<Token>)
        Tokens,
        /// Processed content using the specified format string
        Processed(String),
    }

    /// Main interface for accessing txxt sample files
    pub struct TxxtSources;

    impl TxxtSources {
        /// Get the path to the samples directory
        fn samples_dir() -> String {
            format!("docs/specs/{}/samples", SPEC_VERSION)
        }

        /// Get the full path to a sample file
        fn sample_path(filename: &str) -> String {
            format!("{}/{}", Self::samples_dir(), filename)
        }

        /// Validate that a sample file exists and is available
        fn validate_sample(filename: &str) -> Result<(), ProcessingError> {
            if !AVAILABLE_SAMPLES.contains(&filename) {
                return Err(ProcessingError::FileNotFound(format!(
                    "Sample '{}' is not available. Available samples: {:?}",
                    filename, AVAILABLE_SAMPLES
                )));
            }
            Ok(())
        }

        /// Get sample content in the specified format
        pub fn get_sample(filename: &str, format: SampleFormat) -> Result<String, ProcessingError> {
            Self::validate_sample(filename)?;

            let path = Self::sample_path(filename);

            match format {
                SampleFormat::String => fs::read_to_string(&path).map_err(|e| {
                    ProcessingError::IoError(format!("Failed to read {}: {}", path, e))
                }),
                SampleFormat::Tokens => {
                    let content = fs::read_to_string(&path).map_err(|e| {
                        ProcessingError::IoError(format!("Failed to read {}: {}", path, e))
                    })?;

                    let tokens = lex(&content);
                    let json = serde_json::to_string_pretty(&tokens).map_err(|e| {
                        ProcessingError::IoError(format!("Failed to serialize tokens: {}", e))
                    })?;

                    Ok(json)
                }
                SampleFormat::Processed(format_str) => {
                    let spec = ProcessingSpec::from_string(&format_str)?;
                    process_file(&path, &spec)
                }
            }
        }

        /// Get sample content as raw string
        pub fn get_string(filename: &str) -> Result<String, ProcessingError> {
            Self::get_sample(filename, SampleFormat::String)
        }

        /// Get sample content as tokens (JSON format)
        pub fn get_tokens(filename: &str) -> Result<String, ProcessingError> {
            Self::get_sample(filename, SampleFormat::Tokens)
        }

        /// Get sample content processed with the specified format
        pub fn get_processed(filename: &str, format: &str) -> Result<String, ProcessingError> {
            Self::get_sample(filename, SampleFormat::Processed(format.to_string()))
        }

        /// List all available sample files
        pub fn list_samples() -> Vec<&'static str> {
            AVAILABLE_SAMPLES.to_vec()
        }

        /// Get sample metadata
        pub fn get_sample_info(filename: &str) -> Result<SampleInfo, ProcessingError> {
            Self::validate_sample(filename)?;

            let path = Self::sample_path(filename);
            let content = fs::read_to_string(&path)
                .map_err(|e| ProcessingError::IoError(format!("Failed to read {}: {}", path, e)))?;

            let lines: Vec<&str> = content.lines().collect();
            let line_count = lines.len();
            let char_count = content.len();

            Ok(SampleInfo {
                filename: filename.to_string(),
                spec_version: SPEC_VERSION.to_string(),
                line_count,
                char_count,
                description: Self::extract_description(&content),
            })
        }

        /// Extract description from sample content (first line or comment)
        fn extract_description(content: &str) -> Option<String> {
            let first_line = content.lines().next()?;
            if first_line.contains("{{paragraph}}") {
                Some(first_line.replace("{{paragraph}}", "").trim().to_string())
            } else {
                Some(first_line.to_string())
            }
        }
    }

    /// Information about a sample file
    #[derive(Debug, Clone, PartialEq)]
    pub struct SampleInfo {
        pub filename: String,
        pub spec_version: String,
        pub line_count: usize,
        pub char_count: usize,
        pub description: Option<String>,
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_get_string_sample() {
            let content = TxxtSources::get_string("000-paragraphs.txxt").unwrap();
            assert!(content.contains("Simple Paragraphs Test"));
            assert!(content.contains("{{paragraph}}"));
        }

        #[test]
        fn test_get_tokens_sample() {
            let tokens_json = TxxtSources::get_tokens("040-lists.txxt").unwrap();
            assert!(tokens_json.contains("\"Text\""));
            assert!(tokens_json.contains("\"Dash\""));
            assert!(tokens_json.contains("\"Number\""));
        }

        #[test]
        fn test_get_processed_sample() {
            let processed =
                TxxtSources::get_processed("050-paragraph-lists.txxt", "token-simple").unwrap();
            assert!(processed.contains("<text:"));
            assert!(processed.contains("<newline>"));
        }

        #[test]
        fn test_validate_sample() {
            assert!(TxxtSources::validate_sample("000-paragraphs.txxt").is_ok());
            assert!(TxxtSources::validate_sample("invalid-sample.txxt").is_err());
        }

        #[test]
        fn test_list_samples() {
            let samples = TxxtSources::list_samples();
            assert!(samples.contains(&"000-paragraphs.txxt"));
            assert!(samples.contains(&"040-lists.txxt"));
            assert!(samples.contains(&"070-nested-lists-simple.txxt"));
            assert!(samples.contains(&"080-nested-lists-mixed-content.txxt"));
            assert!(samples.contains(&"090-definitions-simple.txxt"));
            assert!(samples.contains(&"100-definitions-mixed-content.txxt"));
            assert!(samples.contains(&"120-annotations-simple.txxt"));
            assert!(samples.contains(&"130-annotations-block-content.txxt"));
            assert!(samples.contains(&"140-foreign-blocks-simple.txxt"));
            assert!(samples.contains(&"150-foreign-blocks-no-content.txxt"));
            assert_eq!(samples.len(), 17); // Updated for foreign block samples 140 and 150
        }

        #[test]
        fn test_get_sample_info() {
            let info = TxxtSources::get_sample_info("000-paragraphs.txxt").unwrap();
            assert_eq!(info.filename, "000-paragraphs.txxt");
            assert_eq!(info.spec_version, "v1");
            assert!(info.line_count > 0);
            assert!(info.char_count > 0);
            assert!(info.description.is_some());
        }

        #[test]
        fn test_all_samples_accessible() {
            for sample in TxxtSources::list_samples() {
                let content = TxxtSources::get_string(sample).unwrap();
                assert!(!content.is_empty(), "Sample {} should not be empty", sample);
            }
        }
    }
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
        let tokens = vec![
            Token::Text("hello".to_string()),
            Token::Whitespace,
            Token::Text("world".to_string()),
            Token::Newline,
        ];

        let simple = format_tokens(&tokens, &OutputFormat::Simple).unwrap();
        assert_eq!(
            simple,
            "<text:hello><whitespace><text:world><newline>\n"
        );

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
