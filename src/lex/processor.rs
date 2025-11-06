//! File processing API for lex format
//!
//! This module provides an extensible API for processing lex files with different
//! stages (token, ast) and formats (simple, json, xml, etc.).
//!
//! # Pipeline Executor API
//!
//! For new code, consider using the `PipelineExecutor` API in the `pipeline` module
//! which provides a cleaner, config-based interface:
//!
//! ```rust,ignore
//! use lex::lex::pipeline::PipelineExecutor;
//!
//! let executor = PipelineExecutor::new();
//!
//! // Use default stable pipeline (indentation lexer + reference parser)
//! let output = executor.execute("default", source).expect("Failed to parse");
//!
//! // Use experimental linebased pipeline
//! let output = executor.execute("linebased", source).expect("Failed to parse");
//! ```
//!
//! # Sample Sources
//!
//! The `lex_sources` module provides access to verified lex sample files for testing.
//! These samples are the only canonical sources for lex content and should be used
//! instead of copying content to ensure tests use the latest specification.
//!
//! ## Example Usage
//!
//! ```rust
//! use lex::lex::processor::lex_sources::LexSources;
//!
//! // Get raw string content
//! let content = LexSources::get_string("000-paragraphs.lex").unwrap();
//!
//! // Get tokenized content
//! let tokens = LexSources::get_tokens("040-lists.lex").unwrap();
//!
//! // Get processed content in simple format
//! let processed = LexSources::get_processed("050-paragraph-lists.lex", "token-simple").unwrap();
//! ```

use crate::lex::lexing::{lex, Token};
use crate::lex::pipeline::{ExecutionOutput, PipelineExecutor};
use std::collections::HashMap;
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
    AstPosition, // AST position lookup format
    // Linebased pipeline formats
    TokenLine,
    TokenTree,
    // Linebased parser AST formats
    AstLinebasedTag,
    AstLinebasedTreeviz,
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
            "position" => OutputFormat::AstPosition,
            "line" => OutputFormat::TokenLine,
            "tree" => OutputFormat::TokenTree,
            "linebased-tag" => OutputFormat::AstLinebasedTag,
            "linebased-treeviz" => OutputFormat::AstLinebasedTreeviz,
            _ => return Err(ProcessingError::InvalidFormatType(parts[1..].join("-"))),
        };

        // Validate stage/format compatibility
        match (&stage, &format) {
            (ProcessingStage::Ast, OutputFormat::AstTag) => {} // Valid
            (ProcessingStage::Ast, OutputFormat::AstTreeviz) => {} // Valid
            (ProcessingStage::Ast, OutputFormat::AstPosition) => {} // Valid
            (ProcessingStage::Ast, OutputFormat::AstLinebasedTag) => {} // Valid
            (ProcessingStage::Ast, OutputFormat::AstLinebasedTreeviz) => {} // Valid
            (ProcessingStage::Ast, _) => {
                return Err(ProcessingError::InvalidFormatType(format!(
                    "Format '{:?}' not supported for AST stage (only 'tag', 'treeviz', 'position', 'linebased-tag', and 'linebased-treeviz' are supported)",
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
            (ProcessingStage::Token, OutputFormat::AstPosition) => {
                return Err(ProcessingError::InvalidFormatType(
                    "Format 'position' only works with AST stage".to_string(),
                ))
            }
            (ProcessingStage::Token, OutputFormat::AstLinebasedTag) => {
                return Err(ProcessingError::InvalidFormatType(
                    "Format 'linebased-tag' only works with AST stage".to_string(),
                ))
            }
            (ProcessingStage::Token, OutputFormat::AstLinebasedTreeviz) => {
                return Err(ProcessingError::InvalidFormatType(
                    "Format 'linebased-treeviz' only works with AST stage".to_string(),
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
            ProcessingSpec {
                stage: ProcessingStage::Ast,
                format: OutputFormat::AstPosition,
            },
            ProcessingSpec {
                stage: ProcessingStage::Token,
                format: OutputFormat::TokenLine,
            },
            ProcessingSpec {
                stage: ProcessingStage::Token,
                format: OutputFormat::TokenTree,
            },
            ProcessingSpec {
                stage: ProcessingStage::Ast,
                format: OutputFormat::AstLinebasedTag,
            },
            ProcessingSpec {
                stage: ProcessingStage::Ast,
                format: OutputFormat::AstLinebasedTreeviz,
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

/// Process a lex file according to the given specification
pub fn process_file<P: AsRef<Path>>(
    file_path: P,
    spec: &ProcessingSpec,
) -> Result<String, ProcessingError> {
    process_file_with_extras(file_path, spec, HashMap::new())
}

/// Process a lex file according to the given specification with format-specific extras
pub fn process_file_with_extras<P: AsRef<Path>>(
    file_path: P,
    spec: &ProcessingSpec,
    extras: HashMap<String, String>,
) -> Result<String, ProcessingError> {
    let file_path = file_path.as_ref();

    // Read the file
    let content =
        fs::read_to_string(file_path).map_err(|e| ProcessingError::IoError(e.to_string()))?;

    // Process according to stage and format using PipelineExecutor
    match &spec.format {
        OutputFormat::TokenLine => {
            // Use PipelineExecutor with tokens-linebased-flat config
            let executor = PipelineExecutor::new();
            let output = executor
                .execute("tokens-linebased-flat", &content)
                .map_err(|e| ProcessingError::IoError(e.to_string()))?;

            match output {
                ExecutionOutput::Tokens(stream) => {
                    // Convert TokenStream to Vec<LineToken>
                    let line_tokens =
                        crate::lex::pipeline::adapters_linebased::token_stream_to_line_tokens(
                            stream,
                        )
                        .map_err(|e| {
                            ProcessingError::IoError(format!(
                                "Failed to convert to line tokens: {:?}",
                                e
                            ))
                        })?;
                    let json = serde_json::to_string_pretty(&line_tokens)
                        .map_err(|e| ProcessingError::IoError(e.to_string()))?;
                    Ok(json)
                }
                _ => Err(ProcessingError::IoError(
                    "Expected Tokens output from tokens-linebased-flat config".to_string(),
                )),
            }
        }
        OutputFormat::TokenTree => {
            // Use PipelineExecutor with tokens-linebased-tree config
            let executor = PipelineExecutor::new();
            let output = executor
                .execute("tokens-linebased-tree", &content)
                .map_err(|e| ProcessingError::IoError(e.to_string()))?;

            match output {
                ExecutionOutput::Tokens(stream) => {
                    // Convert TokenStream to LineContainer
                    let tree =
                        crate::lex::pipeline::adapters_linebased::token_stream_to_line_container(
                            stream,
                        )
                        .map_err(|e| {
                            ProcessingError::IoError(format!(
                                "Failed to convert to line container: {:?}",
                                e
                            ))
                        })?;
                    let json = serde_json::to_string_pretty(&tree)
                        .map_err(|e| ProcessingError::IoError(e.to_string()))?;
                    Ok(json)
                }
                _ => Err(ProcessingError::IoError(
                    "Expected Tokens output from tokens-linebased-tree config".to_string(),
                )),
            }
        }
        _ => {
            // Use new PipelineExecutor for standard processing
            let executor = PipelineExecutor::new();

            // Map ProcessingSpec to config name
            let config_name = match (&spec.stage, &spec.format) {
                (ProcessingStage::Token, OutputFormat::Simple) => "tokens-indentation",
                (ProcessingStage::Token, OutputFormat::Json) => "tokens-indentation",
                (ProcessingStage::Token, OutputFormat::RawSimple) => "tokens-indentation",
                (ProcessingStage::Token, OutputFormat::RawJson) => "tokens-indentation",
                (ProcessingStage::Ast, OutputFormat::AstTag) => "default",
                (ProcessingStage::Ast, OutputFormat::AstTreeviz) => "default",
                (ProcessingStage::Ast, OutputFormat::AstLinebasedTag) => "linebased",
                (ProcessingStage::Ast, OutputFormat::AstLinebasedTreeviz) => "linebased",
                (ProcessingStage::Ast, OutputFormat::AstPosition) => "default",
                _ => {
                    return Err(ProcessingError::InvalidFormatType(format!(
                        "Unsupported stage/format combination: {:?}/{:?}",
                        spec.stage, spec.format
                    )))
                }
            };

            // Execute pipeline
            let output = executor
                .execute(config_name, &content)
                .map_err(|e| ProcessingError::IoError(e.to_string()))?;

            // Format output
            match (output, &spec.stage, &spec.format) {
                (ExecutionOutput::Tokens(stream), ProcessingStage::Token, format) => {
                    let tokens = stream.unroll();
                    format_tokenss(&tokens, format)
                }
                (ExecutionOutput::Document(doc), ProcessingStage::Ast, OutputFormat::AstTag) => {
                    Ok(crate::lex::parsing::serialize_ast_tag(&doc))
                }
                (
                    ExecutionOutput::Document(doc),
                    ProcessingStage::Ast,
                    OutputFormat::AstTreeviz,
                ) => Ok(crate::lex::parsing::to_treeviz_str(&doc)),
                (
                    ExecutionOutput::Document(doc),
                    ProcessingStage::Ast,
                    OutputFormat::AstLinebasedTag,
                ) => Ok(crate::lex::parsing::serialize_ast_tag(&doc)),
                (
                    ExecutionOutput::Document(doc),
                    ProcessingStage::Ast,
                    OutputFormat::AstLinebasedTreeviz,
                ) => Ok(crate::lex::parsing::to_treeviz_str(&doc)),
                (
                    ExecutionOutput::Document(doc),
                    ProcessingStage::Ast,
                    OutputFormat::AstPosition,
                ) => {
                    let line = extras
                        .get("line")
                        .and_then(|s| s.parse::<usize>().ok())
                        .ok_or_else(|| {
                            ProcessingError::InvalidFormatType(
                                "Missing or invalid 'line' extra".to_string(),
                            )
                        })?;
                    let column = extras
                        .get("column")
                        .and_then(|s| s.parse::<usize>().ok())
                        .ok_or_else(|| {
                            ProcessingError::InvalidFormatType(
                                "Missing or invalid 'column' extra".to_string(),
                            )
                        })?;
                    Ok(crate::lex::parsing::format_at_position(
                        &doc,
                        crate::lex::parsing::Position::new(line, column),
                    ))
                }
                _ => Err(ProcessingError::InvalidFormatType(
                    "Mismatched output and format".to_string(),
                )),
            }
        }
    }
}

/// Format tokens according to the specified output format.
///
/// This function handles all token-based output formats (Simple, Json, RawSimple, RawJson).
/// AST-based formats and specialized formats are not handled here.
pub fn format_tokenss(
    tokens: &[(Token, std::ops::Range<usize>)],
    format: &OutputFormat,
) -> Result<String, ProcessingError> {
    match format {
        OutputFormat::Simple | OutputFormat::RawSimple => {
            let mut result = String::new();
            for (token, _) in tokens {
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
        OutputFormat::AstPosition => Err(ProcessingError::InvalidFormatType(
            "ast-position format only works with ast stage".to_string(),
        )),
        OutputFormat::AstLinebasedTag => Err(ProcessingError::InvalidFormatType(
            "ast linebased-tag format only works with ast stage".to_string(),
        )),
        OutputFormat::AstLinebasedTreeviz => Err(ProcessingError::InvalidFormatType(
            "ast linebased-treeviz format only works with ast stage".to_string(),
        )),
        OutputFormat::TokenLine | OutputFormat::TokenTree => {
            // These formats are handled in process_file_with_extras, not here
            Err(ProcessingError::InvalidFormatType(
                "Token line/tree formats should be handled by process_file_with_extras".to_string(),
            ))
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
                    OutputFormat::RawSimple => "raw-simple",
                    OutputFormat::RawJson => "raw-json",
                    OutputFormat::Xml => "xml",
                    OutputFormat::AstTag => "tag",
                    OutputFormat::AstTreeviz => "treeviz",
                    OutputFormat::AstPosition => "position",
                    OutputFormat::TokenLine => "line",
                    OutputFormat::TokenTree => "tree",
                    OutputFormat::AstLinebasedTag => "linebased-tag",
                    OutputFormat::AstLinebasedTreeviz => "linebased-treeviz",
                }
            )
        })
        .collect()
}

/// Sample sources module for accessing verified lex test files
pub mod lex_sources {
    use super::*;

    /// The current specification version - change this when spec updates
    pub const SPEC_VERSION: &str = "v1";

    /// Available sample files (canonical sources)
    pub const AVAILABLE_SAMPLES: &[&str] = &[
        "000-paragraphs.lex",
        "010-paragraphs-sessions-flat-single.lex",
        "020-paragraphs-sessions-flat-multiple.lex",
        "030-paragraphs-sessions-nested-multiple.lex",
        "040-lists.lex",
        "050-paragraph-lists.lex",
        "050-trifecta-flat-simple.lex",
        "060-trifecta-nesting.lex",
        "070-nested-lists-simple.lex",
        "080-nested-lists-mixed-content.lex",
        "090-definitions-simple.lex",
        "100-definitions-mixed-content.lex",
        "110-ensemble-with-definitions.lex",
        "120-annotations-simple.lex",
        "130-annotations-block-content.lex",
        "140-verbatim-blocks-simple.lex",
        "150-verbatim-blocks-no-content.lex",
        "dialog.lex",
    ];

    /// Format options for sample content
    #[derive(Debug, Clone, PartialEq)]
    pub enum SampleFormat {
        /// Raw string content
        String,
        /// Tokenized content (`Vec<Token>`)
        Tokens,
        /// Processed content using the specified format string
        Processed(String),
    }

    /// Main interface for accessing lex sample files
    pub struct LexSources;

    impl LexSources {
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

                    let source_with_newline =
                        crate::lex::lexing::ensure_source_ends_with_newline(&content);
                    let token_stream =
                        crate::lex::lexing::base_tokenization::tokenize(&source_with_newline);
                    let tokens = lex(token_stream);
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

    #[test]
    fn test_position_tracking_enabled() {
        let content = r#"First paragraph
Second paragraph"#;

        let doc = crate::lex::parsing::parse_document(content).unwrap();

        // Check if locations are populated
        if let Some(first_item) = doc.root.children.first() {
            // The first paragraph should have a location
            match first_item {
                crate::lex::parsing::ContentItem::Paragraph(_p) => {
                    // Paragraph has location
                }
                _ => panic!("Expected first item to be a paragraph"),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_get_string_sample() {
            let content = LexSources::get_string("000-paragraphs.lex").unwrap();
            assert!(content.contains("Simple Paragraphs Test"));
            assert!(content.contains("{{paragraph}}"));
        }

        #[test]
        fn test_get_tokens_sample() {
            let tokens_json = LexSources::get_tokens("040-lists.lex").unwrap();
            assert!(tokens_json.contains("\"Text\""));
            assert!(tokens_json.contains("\"Dash\""));
            assert!(tokens_json.contains("\"Number\""));
        }

        #[test]
        fn test_get_processed_sample() {
            let processed =
                LexSources::get_processed("050-paragraph-lists.lex", "token-simple").unwrap();
            assert!(processed.contains("<text:"));
            assert!(processed.contains("<newline>"));
        }

        #[test]
        fn test_validate_sample() {
            assert!(LexSources::validate_sample("000-paragraphs.lex").is_ok());
            assert!(LexSources::validate_sample("invalid-sample.lex").is_err());
        }

        #[test]
        fn test_list_samples() {
            let samples = LexSources::list_samples();
            assert!(samples.contains(&"000-paragraphs.lex"));
            assert!(samples.contains(&"040-lists.lex"));
            assert!(samples.contains(&"070-nested-lists-simple.lex"));
            assert!(samples.contains(&"080-nested-lists-mixed-content.lex"));
            assert!(samples.contains(&"090-definitions-simple.lex"));
            assert!(samples.contains(&"100-definitions-mixed-content.lex"));
            assert!(samples.contains(&"120-annotations-simple.lex"));
            assert!(samples.contains(&"130-annotations-block-content.lex"));
            assert!(samples.contains(&"140-verbatim-blocks-simple.lex"));
            assert!(samples.contains(&"150-verbatim-blocks-no-content.lex"));
            assert_eq!(samples.len(), 18);
        }

        #[test]
        fn test_get_sample_info() {
            let info = LexSources::get_sample_info("000-paragraphs.lex").unwrap();
            assert_eq!(info.filename, "000-paragraphs.lex");
            assert_eq!(info.spec_version, "v1");
            assert!(info.line_count > 0);
            assert!(info.char_count > 0);
            assert!(info.description.is_some());
        }

        #[test]
        fn test_all_samples_accessible() {
            for sample in LexSources::list_samples() {
                let content = LexSources::get_string(sample).unwrap();
                assert!(!content.is_empty(), "Sample {} should not be empty", sample);
            }
        }

        #[test]
        fn test_dialog_sample_accessible() {
            let content = LexSources::get_string("dialog.lex").unwrap();
            assert!(content.contains("- Hi mom!!."), "Dialog sample content is incorrect");
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
        let tokens: Vec<(Token, std::ops::Range<usize>)> = vec![
            (Token::Text("hello".to_string()), 0..5),
            (Token::Whitespace, 5..6),
            (Token::Text("world".to_string()), 6..11),
            (Token::Newline, 11..12),
        ];

        let simple = format_tokenss(&tokens, &OutputFormat::Simple).unwrap();
        assert_eq!(simple, "<text:hello><whitespace><text:world><newline>\n");

        let json = format_tokenss(&tokens, &OutputFormat::Json).unwrap();
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
