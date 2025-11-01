//! Experimental lexer pipeline: orchestrates all transformations into a cohesive pipeline
//!
//! This module provides a complete experimental lexer pipeline that chains all transformations:
//!
//! ```text
//! source text
//!   ↓
//! tokenize_with_locations()                        [existing]
//!   ↓
//! process_whitespace_remainders()                  [existing]
//!   ↓
//! transform_indentation()                          [existing]
//!   ↓
//! transform_blank_lines()                          [existing]
//!   ↓
//! experimental_transform_to_line_tokens()          [Issue #111]
//!   ↓
//! experimental_transform_indentation_to_token_tree() [Issue #112]
//!   ↓
//! Token Tree (final output)
//! ```
//!
//! This pipeline coexists with the existing lexer without modifying it.

use std::fmt;

use crate::txxt::lexer::ensure_source_ends_with_newline;
use crate::txxt::lexer::lexer_impl::tokenize;
use crate::txxt::lexer::tokens::{LineToken, Token};
use crate::txxt::lexer::transformations::{
    experimental_transform_indentation_to_token_tree, experimental_transform_to_line_tokens,
    process_whitespace_remainders, transform_blank_lines, transform_indentation,
};

/// Error type for experimental pipeline operations
#[derive(Debug, Clone, PartialEq)]
pub enum PipelineError {
    /// Unexpected output type from a pipeline stage
    UnexpectedOutput(String),
}

impl fmt::Display for PipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PipelineError::UnexpectedOutput(msg) => write!(f, "Unexpected output: {}", msg),
        }
    }
}

impl std::error::Error for PipelineError {}

/// Represents a stage in the experimental pipeline for debugging/testing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineStage {
    /// Raw tokens from logos lexer
    RawTokens,
    /// After whitespace remainder processing
    AfterWhitespace,
    /// After indentation transformation (Indent -> IndentLevel/DedentLevel)
    AfterIndentation,
    /// After blank line transformation (consecutive Newlines -> BlankLine)
    AfterBlankLines,
    /// After line token transformation (semantic line tokens)
    LineTokens,
    /// Final token tree output
    TokenTree,
}

/// Wrapper enum for pipeline outputs at different stages
#[derive(Debug, Clone)]
pub enum PipelineOutput {
    /// Raw or transformed basic tokens with location information preserved
    /// This allows debugging and analysis of intermediate stages while maintaining
    /// the ability to map tokens back to their source locations.
    Tokens(Vec<(Token, std::ops::Range<usize>)>),
    /// Line tokens
    LineTokens(Vec<LineToken>),
    /// Token tree
    TokenTree(Vec<crate::txxt::lexer::LineTokenTree>),
}

/// Main experimental lexer pipeline.
///
/// Runs all transformations in sequence and returns the final token tree.
///
/// # Arguments
/// * `source` - The input source text
///
/// # Returns
/// A Result containing a vector of LineTokenTree nodes, or a PipelineError if the
/// pipeline stage returns an unexpected output type (which should not happen in normal usage).
pub fn experimental_lex(
    source: &str,
) -> Result<Vec<crate::txxt::lexer::LineTokenTree>, PipelineError> {
    let output = experimental_lex_stage(source, PipelineStage::TokenTree);
    match output {
        PipelineOutput::TokenTree(tree) => Ok(tree),
        _ => Err(PipelineError::UnexpectedOutput(
            "TokenTree stage should return TokenTree output".to_string(),
        )),
    }
}

/// Experimental lexer pipeline with stage-based output.
///
/// Returns the pipeline output at any requested stage for debugging/testing.
///
/// # Arguments
/// * `source` - The input source text
/// * `stage` - The pipeline stage at which to return output
///
/// # Returns
/// Pipeline output at the requested stage
pub fn experimental_lex_stage(source: &str, stage: PipelineStage) -> PipelineOutput {
    let source_with_newline = ensure_source_ends_with_newline(source);

    // Stage 1: Raw tokenization
    let raw_tokens = tokenize(&source_with_newline);
    if stage == PipelineStage::RawTokens {
        return PipelineOutput::Tokens(raw_tokens);
    }

    // Stage 2: Whitespace remainder processing
    let after_whitespace = process_whitespace_remainders(raw_tokens);

    if stage == PipelineStage::AfterWhitespace {
        return PipelineOutput::Tokens(after_whitespace);
    }

    // Stage 3: Indentation transformation
    let after_indentation = transform_indentation(after_whitespace);

    if stage == PipelineStage::AfterIndentation {
        return PipelineOutput::Tokens(after_indentation);
    }

    // Stage 4: Blank line transformation
    let after_blank_lines = transform_blank_lines(after_indentation);

    if stage == PipelineStage::AfterBlankLines {
        return PipelineOutput::Tokens(after_blank_lines.clone());
    }

    // Stage 5: Line token transformation (experimental)
    // Extract tokens for use in line token transformation (location info not needed downstream)
    let tokens_for_line_tokens: Vec<Token> =
        after_blank_lines.iter().map(|(t, _)| t.clone()).collect();
    let line_tokens = experimental_transform_to_line_tokens(tokens_for_line_tokens);

    if stage == PipelineStage::LineTokens {
        return PipelineOutput::LineTokens(line_tokens.clone());
    }

    // Stage 6: Indentation-to-token-tree transformation (experimental)
    let token_tree = experimental_transform_indentation_to_token_tree(line_tokens);

    PipelineOutput::TokenTree(token_tree)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_experimental_lex_empty_input() {
        let result = experimental_lex("").expect("Pipeline should not fail");
        // Empty input should produce empty tree
        assert!(result.is_empty());
    }

    #[test]
    fn test_experimental_lex_single_paragraph() {
        let source = "Hello world";
        let result = experimental_lex(source).expect("Pipeline should not fail");
        // Single paragraph should produce at least one token
        assert!(!result.is_empty());
    }

    #[test]
    fn test_experimental_lex_multiple_paragraphs() {
        let source = "First paragraph\n\nSecond paragraph";
        let result = experimental_lex(source).expect("Pipeline should not fail");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_experimental_lex_with_indentation() {
        let source = "Title:\n    Indented content\n    More indented";
        let result = experimental_lex(source).expect("Pipeline should not fail");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_experimental_lex_stage_raw_tokens() {
        let source = "Hello world";
        let output = experimental_lex_stage(source, PipelineStage::RawTokens);
        match output {
            PipelineOutput::Tokens(tokens) => assert!(!tokens.is_empty()),
            _ => panic!("Expected Tokens output"),
        }
    }

    #[test]
    fn test_experimental_lex_stage_after_whitespace() {
        let source = "Hello world";
        let output = experimental_lex_stage(source, PipelineStage::AfterWhitespace);
        match output {
            PipelineOutput::Tokens(tokens) => assert!(!tokens.is_empty()),
            _ => panic!("Expected Tokens output"),
        }
    }

    #[test]
    fn test_experimental_lex_stage_after_indentation() {
        let source = "Hello:\n    World";
        let output = experimental_lex_stage(source, PipelineStage::AfterIndentation);
        match output {
            PipelineOutput::Tokens(tokens) => assert!(!tokens.is_empty()),
            _ => panic!("Expected Tokens output"),
        }
    }

    #[test]
    fn test_experimental_lex_stage_after_blank_lines() {
        let source = "Hello\n\nWorld";
        let output = experimental_lex_stage(source, PipelineStage::AfterBlankLines);
        match output {
            PipelineOutput::Tokens(tokens) => assert!(!tokens.is_empty()),
            _ => panic!("Expected Tokens output"),
        }
    }

    #[test]
    fn test_experimental_lex_stage_line_tokens() {
        let source = "Title:\n    Content";
        let output = experimental_lex_stage(source, PipelineStage::LineTokens);
        match output {
            PipelineOutput::LineTokens(tokens) => assert!(!tokens.is_empty()),
            _ => panic!("Expected LineTokens output"),
        }
    }

    #[test]
    fn test_experimental_lex_stage_token_tree() {
        let source = "Title:\n    Content";
        let output = experimental_lex_stage(source, PipelineStage::TokenTree);
        match output {
            PipelineOutput::TokenTree(tree) => assert!(!tree.is_empty()),
            _ => panic!("Expected TokenTree output"),
        }
    }

    #[test]
    fn test_pipeline_consistency_across_stages() {
        let source = "Item 1\n\n- First\n- Second";

        // Verify all stages return successfully
        let raw = experimental_lex_stage(source, PipelineStage::RawTokens);
        assert!(matches!(raw, PipelineOutput::Tokens(_)));

        let after_ws = experimental_lex_stage(source, PipelineStage::AfterWhitespace);
        assert!(matches!(after_ws, PipelineOutput::Tokens(_)));

        let after_ind = experimental_lex_stage(source, PipelineStage::AfterIndentation);
        assert!(matches!(after_ind, PipelineOutput::Tokens(_)));

        let after_blank = experimental_lex_stage(source, PipelineStage::AfterBlankLines);
        assert!(matches!(after_blank, PipelineOutput::Tokens(_)));

        let line_tokens = experimental_lex_stage(source, PipelineStage::LineTokens);
        assert!(matches!(line_tokens, PipelineOutput::LineTokens(_)));

        let tree = experimental_lex_stage(source, PipelineStage::TokenTree);
        assert!(matches!(tree, PipelineOutput::TokenTree(_)));
    }

    #[test]
    fn test_experimental_lex_list_structure() {
        let source = "Items:\n    - First\n    - Second\n    - Third";
        let result = experimental_lex(source).expect("Pipeline should not fail");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_experimental_lex_nested_indentation() {
        let source = "Level 1:\n    Level 2:\n        Level 3 content";
        let result = experimental_lex(source).expect("Pipeline should not fail");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_experimental_lex_with_blank_lines() {
        let source = "Para 1\n\nPara 2\n\nPara 3";
        let result = experimental_lex(source).expect("Pipeline should not fail");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_pipeline_blank_line_preservation() {
        let source = "Para 1\n\nPara 2";

        // Check LineTokens stage
        let line_tokens_output = experimental_lex_stage(source, PipelineStage::LineTokens);
        if let PipelineOutput::LineTokens(tokens) = line_tokens_output {
            eprintln!("LineTokens (count={}): ", tokens.len());
            for (i, token) in tokens.iter().enumerate() {
                eprintln!("  [{}] {:?}", i, token.line_type);
            }
        }

        // Check TokenTree stage
        let tree_output = experimental_lex_stage(source, PipelineStage::TokenTree);
        if let PipelineOutput::TokenTree(tree) = tree_output {
            eprintln!("TokenTree (count={}): ", tree.len());
            for (i, node) in tree.iter().enumerate() {
                match node {
                    crate::txxt::lexer::LineTokenTree::Token(t) => {
                        eprintln!("  [{}] Token: {:?}", i, t.line_type);
                    }
                    crate::txxt::lexer::LineTokenTree::Block(_) => {
                        eprintln!("  [{}] Block", i);
                    }
                }
            }
        }
    }

    #[test]
    fn test_experimental_lex_mixed_content() {
        let source = "Title:\n\n    First paragraph\n\n    - List item 1\n    - List item 2";
        let result = experimental_lex(source).expect("Pipeline should not fail");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_experimental_lex_with_annotations() {
        let source = ":: note ::\nSome text\n\n:: note :: with inline content";
        let result = experimental_lex(source).expect("Pipeline should not fail");
        assert!(!result.is_empty());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_experimental_pipeline_with_000_paragraphs() {
        let content = std::fs::read_to_string("docs/specs/v1/samples/000-paragraphs.txxt")
            .expect("Could not read sample file");
        let tree = experimental_lex(&content).expect("Pipeline should not fail");
        assert!(
            !tree.is_empty(),
            "Token tree should not be empty for paragraphs"
        );
    }

    #[test]
    fn test_experimental_pipeline_with_040_lists() {
        let content = std::fs::read_to_string("docs/specs/v1/samples/040-lists.txxt")
            .expect("Could not read sample file");
        let tree = experimental_lex(&content).expect("Pipeline should not fail");
        assert!(!tree.is_empty(), "Token tree should not be empty for lists");
    }

    #[test]
    fn test_experimental_pipeline_with_050_paragraph_lists() {
        let content = std::fs::read_to_string("docs/specs/v1/samples/050-paragraph-lists.txxt")
            .expect("Could not read sample file");
        let tree = experimental_lex(&content).expect("Pipeline should not fail");
        assert!(
            !tree.is_empty(),
            "Token tree should not be empty for mixed content"
        );
    }

    #[test]
    fn test_experimental_pipeline_with_090_definitions() {
        let content = std::fs::read_to_string("docs/specs/v1/samples/090-definitions-simple.txxt")
            .expect("Could not read sample file");
        let tree = experimental_lex(&content).expect("Pipeline should not fail");
        assert!(
            !tree.is_empty(),
            "Token tree should not be empty for definitions"
        );
    }

    #[test]
    fn test_experimental_pipeline_with_120_annotations() {
        let content = std::fs::read_to_string("docs/specs/v1/samples/120-annotations-simple.txxt")
            .expect("Could not read sample file");
        let tree = experimental_lex(&content).expect("Pipeline should not fail");
        assert!(
            !tree.is_empty(),
            "Token tree should not be empty for annotations"
        );
    }

    #[test]
    fn test_experimental_pipeline_with_030_nested_sessions() {
        let content = std::fs::read_to_string(
            "docs/specs/v1/samples/030-paragraphs-sessions-nested-multiple.txxt",
        )
        .expect("Could not read sample file");
        let tree = experimental_lex(&content).expect("Pipeline should not fail");
        assert!(
            !tree.is_empty(),
            "Token tree should not be empty for nested sessions"
        );
    }

    #[test]
    fn test_experimental_pipeline_with_070_nested_lists() {
        let content = std::fs::read_to_string("docs/specs/v1/samples/070-nested-lists-simple.txxt")
            .expect("Could not read sample file");
        let tree = experimental_lex(&content).expect("Pipeline should not fail");
        assert!(
            !tree.is_empty(),
            "Token tree should not be empty for nested lists"
        );
    }

    #[test]
    fn test_pipeline_stage_consistency_with_real_file() {
        let content = std::fs::read_to_string("docs/specs/v1/samples/040-lists.txxt")
            .expect("Could not read sample file");

        // Verify all stages return successfully for real file
        let raw = experimental_lex_stage(&content, PipelineStage::RawTokens);
        assert!(matches!(raw, PipelineOutput::Tokens(_)));

        let after_ws = experimental_lex_stage(&content, PipelineStage::AfterWhitespace);
        assert!(matches!(after_ws, PipelineOutput::Tokens(_)));

        let after_ind = experimental_lex_stage(&content, PipelineStage::AfterIndentation);
        assert!(matches!(after_ind, PipelineOutput::Tokens(_)));

        let after_blank = experimental_lex_stage(&content, PipelineStage::AfterBlankLines);
        assert!(matches!(after_blank, PipelineOutput::Tokens(_)));

        let line_tokens = experimental_lex_stage(&content, PipelineStage::LineTokens);
        assert!(matches!(line_tokens, PipelineOutput::LineTokens(_)));

        let tree = experimental_lex_stage(&content, PipelineStage::TokenTree);
        assert!(matches!(tree, PipelineOutput::TokenTree(_)));
    }
}
