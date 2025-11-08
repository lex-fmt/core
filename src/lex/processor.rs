//! File processing API for lex format
//!
//! This module provides an extensible API for processing lex files with different
//! stages (token, ast) and formats (simple, json, xml, etc.).
//!
//! # DocumentLoader API
//!
//! For new code, consider using the `DocumentLoader` API in the `pipeline` module
//! which provides the primary interface for document processing:
//!
//! ```rust,ignore
//! use lex::lex::pipeline::DocumentLoader;
//!
//! let loader = DocumentLoader::new();
//!
//! // Parse files or strings
//! let doc = loader.load_and_parse("path/to/file.lex")?;
//! let doc = loader.parse(source)?;
//!
//! // Tokenize files or strings
//! let tokens = loader.load_and_tokenize("path/to/file.lex")?;
//! let tokens = loader.tokenize(source)?;
//! ```
//!
//! # Sample Sources
//!
//! The `Lexplore` module (in `lex::testing::lexplore`) provides access to verified lex sample files for testing.
//! These samples are the only canonical sources for lex content and should be used
//! instead of copying content to ensure tests use the latest specification.
//!
//! ## Example Usage
//!
//! ```rust,ignore
//! use lex::lex::testing::lexplore::{Lexplore, ElementType, DocumentType};
//!
//! // Get source for a specific element
//! let source = Lexplore::get_source_for(ElementType::Paragraph, 1).unwrap();
//!
//! // Get source for a document collection
//! let source = Lexplore::get_document_source_for(DocumentType::Trifecta, 0).unwrap();
//!
//! // Using the fluent API to parse
//! let paragraph = Lexplore::paragraph(1).parse().expect_paragraph();
//! ```

use crate::lex::formats::FormatRegistry;
use crate::lex::lexing::Token;
use crate::lex::pipeline::DocumentLoader;
use std::collections::HashMap;
use std::fmt;
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
    let loader = DocumentLoader::new();

    // Load source using DocumentLoader
    let content = loader
        .load_source(file_path)
        .map_err(|e| ProcessingError::IoError(e.to_string()))?;

    // Process according to stage and format
    match &spec.format {
        OutputFormat::TokenLine => {
            // Use DocumentLoader to execute tokens-linebased-flat config
            let tokens = loader
                .execute("tokens-linebased-flat", &content)
                .map_err(|e| ProcessingError::IoError(e.to_string()))
                .and_then(|output| match output {
                    crate::lex::pipeline::ExecutionOutput::Tokens(stream) => Ok(stream.unroll()),
                    _ => Err(ProcessingError::IoError(
                        "Expected Tokens output from tokens-linebased-flat config".to_string(),
                    )),
                })?;

            // Convert TokenStream to Vec<LineToken>
            let line_tokens =
                crate::lex::parsing::linebased::tree_builder::build_line_tokens(tokens);
            let json = serde_json::to_string_pretty(&line_tokens)
                .map_err(|e| ProcessingError::IoError(e.to_string()))?;
            Ok(json)
        }
        OutputFormat::TokenTree => {
            // Use DocumentLoader to execute tokens-linebased-tree config
            let tokens = loader
                .execute("tokens-linebased-tree", &content)
                .map_err(|e| ProcessingError::IoError(e.to_string()))
                .and_then(|output| match output {
                    crate::lex::pipeline::ExecutionOutput::Tokens(stream) => Ok(stream.unroll()),
                    _ => Err(ProcessingError::IoError(
                        "Expected Tokens output from tokens-linebased-tree config".to_string(),
                    )),
                })?;

            // Convert TokenStream to LineContainer
            let tree = crate::lex::parsing::linebased::tree_builder::build_line_container(tokens);
            let json = serde_json::to_string_pretty(&tree)
                .map_err(|e| ProcessingError::IoError(e.to_string()))?;
            Ok(json)
        }
        _ => {
            // Use DocumentLoader for standard processing
            match &spec.stage {
                ProcessingStage::Token => {
                    // Get tokens directly from DocumentLoader
                    let tokens = loader
                        .tokenize(&content)
                        .map_err(|e| ProcessingError::IoError(e.to_string()))?;

                    // Apply formatting
                    format_tokenss(&tokens, &spec.format)
                }
                ProcessingStage::Ast => {
                    // Parse with appropriate parser based on format
                    let doc = match &spec.format {
                        OutputFormat::AstLinebasedTag | OutputFormat::AstLinebasedTreeviz => {
                            // Use linebased parser
                            loader
                                .parse_with(&content, crate::lex::pipeline::Parser::Linebased)
                                .map_err(|e| ProcessingError::IoError(e.to_string()))?
                        }
                        _ => {
                            // Use default reference parser
                            loader
                                .parse(&content)
                                .map_err(|e| ProcessingError::IoError(e.to_string()))?
                        }
                    };

                    // Apply formatting based on output format
                    match &spec.format {
                        OutputFormat::AstTag | OutputFormat::AstLinebasedTag => {
                            let registry = FormatRegistry::with_defaults();
                            registry
                                .serialize(&doc, "tag")
                                .map_err(|e| ProcessingError::IoError(e.to_string()))
                        }
                        OutputFormat::AstTreeviz | OutputFormat::AstLinebasedTreeviz => {
                            let registry = FormatRegistry::with_defaults();
                            registry
                                .serialize(&doc, "treeviz")
                                .map_err(|e| ProcessingError::IoError(e.to_string()))
                        }
                        OutputFormat::AstPosition => {
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
                        _ => Err(ProcessingError::InvalidFormatType(format!(
                            "Unsupported AST format: {:?}",
                            spec.format
                        ))),
                    }
                }
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
