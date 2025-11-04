//! This module orchestrates the complete tokenization pipeline for the lex format.
//!
//! Currently we are still running two parser designs side by side and the the newer parser requires
//! more preprocessing of the cst.
//! The pipeline consists of:
//! 1. Core tokenization using logos lexer
//! 2. Common Transformation pipeline:
//!    - Whitespace remainder processing ../transformations/normalize_whitespace.rs
//!    - Indentation transformation (Indent -> Indent/Dedent) ../transformations/sem_indentation.rs
//!    - Blank line transformation (consecutive Newlines -> BlankLine) ../transformations/transform_blanklines.rs
//! 3. Line-based processing:
//!    - Flatten tokens into line tokens
//!    - Transform line tokens into a hierarchical tree
//!
//! This pipeline coexists with the existing lexer without modifying it.

use std::fmt;

use crate::lex::lexers::linebased::tokens_linebased::{LineContainer, LineToken};
use crate::lex::lexers::tokens_core::Token;
use crate::lex::pipeline::adapters_linebased::{
    token_stream_to_line_container, token_stream_to_line_tokens,
};
use crate::lex::pipeline::{
    BlankLinesMapper, IndentationToTreeMapper, NormalizeWhitespaceMapper,
    SemanticIndentationMapper, ToLineTokensMapper,
};

/// Error type for linebased pipeline operations
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

/// Represents a stage in the linebased pipeline for debugging/testing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineStage {
    /// Raw tokens from logos lexer
    RawTokens,
    /// After whitespace remainder processing
    AfterWhitespace,
    /// After indentation transformation (Indent -> Indent/Dedent)
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
    /// Token tree (root container with all line tokens and nested containers)
    TokenTree(LineContainer),
}

/// Main linebased lexer pipeline.
///
/// Runs all transformations in sequence and returns the final token tree.
///
/// # Arguments
/// * `tokens` - The input token stream from base tokenization
///
/// # Returns
/// A Result containing a LineContainerToken (root node representing the entire hierarchical tree),
/// or a PipelineError if the pipeline stage returns an unexpected output type.
pub fn _lex(tokens: Vec<(Token, std::ops::Range<usize>)>) -> Result<LineContainer, PipelineError> {
    let output = _lex_stage(tokens, PipelineStage::TokenTree);
    match output {
        PipelineOutput::TokenTree(tree) => Ok(tree),
        _ => Err(PipelineError::UnexpectedOutput(
            "TokenTree stage should return TokenTree output".to_string(),
        )),
    }
}

/// Linebased lexer pipeline with stage-based output.
///
/// Returns the pipeline output at any requested stage for debugging/testing.
///
/// # Arguments
/// * `tokens` - The input token stream from base tokenization
/// * `stage` - The pipeline stage at which to return output
///
/// # Returns
/// Pipeline output at the requested stage
pub fn _lex_stage(
    tokens: Vec<(Token, std::ops::Range<usize>)>,
    stage: PipelineStage,
) -> PipelineOutput {
    use crate::lex::pipeline::stream::TokenStream;

    // Stage 1: Raw tokenization (already done by caller)
    if stage == PipelineStage::RawTokens {
        return PipelineOutput::Tokens(tokens);
    }

    // Start with TokenStream::Flat
    let mut current_stream = TokenStream::Flat(tokens);

    // Stage 2: NormalizeWhitespace mapper
    let mut normalize_mapper = NormalizeWhitespaceMapper::new();
    current_stream =
        crate::lex::pipeline::mapper::walk_stream(current_stream, &mut normalize_mapper)
            .expect("NormalizeWhitespace transformation failed");

    if stage == PipelineStage::AfterWhitespace {
        // Convert back to flat tokens for this output stage
        return PipelineOutput::Tokens(current_stream.unroll());
    }

    // Stage 3: SemanticIndentation mapper
    let mut semantic_indent_mapper = SemanticIndentationMapper::new();
    current_stream =
        crate::lex::pipeline::mapper::walk_stream(current_stream, &mut semantic_indent_mapper)
            .expect("SemanticIndentation transformation failed");

    if stage == PipelineStage::AfterIndentation {
        return PipelineOutput::Tokens(current_stream.unroll());
    }

    // Stage 4: BlankLines mapper
    let mut blank_lines_mapper = BlankLinesMapper::new();
    current_stream =
        crate::lex::pipeline::mapper::walk_stream(current_stream, &mut blank_lines_mapper)
            .expect("BlankLines transformation failed");

    if stage == PipelineStage::AfterBlankLines {
        return PipelineOutput::Tokens(current_stream.unroll());
    }

    // Stage 5: ToLineTokens mapper (Flat → Shallow Tree with LineType)
    let mut to_line_tokens_mapper = ToLineTokensMapper::new();
    current_stream =
        crate::lex::pipeline::mapper::walk_stream(current_stream, &mut to_line_tokens_mapper)
            .expect("ToLineTokens transformation failed");

    if stage == PipelineStage::LineTokens {
        // Convert TokenStream::Tree to Vec<LineToken> for backward compatibility
        let line_tokens = token_stream_to_line_tokens(current_stream.clone())
            .expect("Expected Tree stream from ToLineTokens");
        return PipelineOutput::LineTokens(line_tokens);
    }

    // Stage 6: IndentationToTree mapper (Shallow Tree → Nested Tree)
    let mut indentation_mapper = IndentationToTreeMapper::new();
    current_stream = indentation_mapper
        .transform(current_stream)
        .expect("IndentationToTree transformation failed");

    // Convert final TokenStream::Tree to LineContainer for backward compatibility
    let token_tree = token_stream_to_line_container(current_stream)
        .expect("Expected Tree stream from IndentationToTree");

    PipelineOutput::TokenTree(token_tree)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::lexers::base_tokenization::tokenize;
    use crate::lex::lexers::ensure_source_ends_with_newline;

    // Helper to prepare token stream from source
    fn prepare_tokens(source: &str) -> Vec<(Token, std::ops::Range<usize>)> {
        let source_with_newline = ensure_source_ends_with_newline(source);
        tokenize(&source_with_newline)
    }

    // @audit: hardcoded_source
    #[test]
    fn test_lex_empty_input() {
        let tokens = prepare_tokens("");
        let result = _lex(tokens).expect("Pipeline should not fail");
        // Empty input should produce empty tree
        assert!(result.is_empty());
    }

    // @audit: hardcoded_source
    #[test]
    fn test_lex_single_paragraph() {
        let source = "Hello world";
        let tokens = prepare_tokens(source);
        let result = _lex(tokens).expect("Pipeline should not fail");
        // Single paragraph should produce at least one token
        assert!(!result.is_empty());
    }

    // @audit: hardcoded_source
    #[test]
    fn test_lex_multiple_paragraphs() {
        let source = "First paragraph\n\nSecond paragraph";
        let tokens = prepare_tokens(source);
        let result = _lex(tokens).expect("Pipeline should not fail");
        assert!(!result.is_empty());
    }

    // @audit: hardcoded_source
    #[test]
    fn test_lex_with_indentation() {
        let source = "Title:\n    Indented content\n    More indented";
        let tokens = prepare_tokens(source);
        let result = _lex(tokens).expect("Pipeline should not fail");
        assert!(!result.is_empty());
    }

    // @audit: hardcoded_source
    #[test]
    fn test_lex_stage_raw_tokens() {
        let source = "Hello world";
        let tokens = prepare_tokens(source);
        let output = _lex_stage(tokens, PipelineStage::RawTokens);
        match output {
            PipelineOutput::Tokens(tokens) => assert!(!tokens.is_empty()),
            _ => panic!("Expected Tokens output"),
        }
    }

    // @audit: hardcoded_source
    #[test]
    fn test_lex_stage_after_whitespace() {
        let source = "Hello world";
        let tokens = prepare_tokens(source);
        let output = _lex_stage(tokens, PipelineStage::AfterWhitespace);
        match output {
            PipelineOutput::Tokens(tokens) => assert!(!tokens.is_empty()),
            _ => panic!("Expected Tokens output"),
        }
    }

    // @audit: hardcoded_source
    #[test]
    fn test_lex_stage_after_indentation() {
        let source = "Hello:\n    World";
        let tokens = prepare_tokens(source);
        let output = _lex_stage(tokens, PipelineStage::AfterIndentation);
        match output {
            PipelineOutput::Tokens(tokens) => assert!(!tokens.is_empty()),
            _ => panic!("Expected Tokens output"),
        }
    }

    // @audit: hardcoded_source
    #[test]
    fn test_lex_stage_after_blank_lines() {
        let source = "Hello\n\nWorld";
        let tokens = prepare_tokens(source);
        let output = _lex_stage(tokens, PipelineStage::AfterBlankLines);
        match output {
            PipelineOutput::Tokens(tokens) => assert!(!tokens.is_empty()),
            _ => panic!("Expected Tokens output"),
        }
    }

    // @audit: hardcoded_source
    #[test]
    fn test_lex_stage_line_tokens() {
        let source = "Title:\n    Content";
        let tokens = prepare_tokens(source);
        let output = _lex_stage(tokens, PipelineStage::LineTokens);
        match output {
            PipelineOutput::LineTokens(tokens) => assert!(!tokens.is_empty()),
            _ => panic!("Expected LineTokens output"),
        }
    }

    // @audit: hardcoded_source
    #[test]
    fn test_lex_stage_token_tree() {
        let source = "Title:\n    Content";
        let tokens = prepare_tokens(source);
        let output = _lex_stage(tokens, PipelineStage::TokenTree);
        match output {
            PipelineOutput::TokenTree(tree) => assert!(!tree.is_empty()),
            _ => panic!("Expected TokenTree output"),
        }
    }

    // @audit: hardcoded_source
    #[test]
    fn test_pipeline_consistency_across_stages() {
        let source = "Item 1\n\n- First\n- Second";

        // Verify all stages return successfully
        let raw = _lex_stage(prepare_tokens(source), PipelineStage::RawTokens);
        assert!(matches!(raw, PipelineOutput::Tokens(_)));

        let after_ws = _lex_stage(prepare_tokens(source), PipelineStage::AfterWhitespace);
        assert!(matches!(after_ws, PipelineOutput::Tokens(_)));

        let after_ind = _lex_stage(prepare_tokens(source), PipelineStage::AfterIndentation);
        assert!(matches!(after_ind, PipelineOutput::Tokens(_)));

        let after_blank = _lex_stage(prepare_tokens(source), PipelineStage::AfterBlankLines);
        assert!(matches!(after_blank, PipelineOutput::Tokens(_)));

        let line_tokens = _lex_stage(prepare_tokens(source), PipelineStage::LineTokens);
        assert!(matches!(line_tokens, PipelineOutput::LineTokens(_)));

        let tree = _lex_stage(prepare_tokens(source), PipelineStage::TokenTree);
        assert!(matches!(tree, PipelineOutput::TokenTree(_)));
    }

    // @audit: hardcoded_source
    #[test]
    fn test_lex_list_structure() {
        let source = "Items:\n    - First\n    - Second\n    - Third";
        let tokens = prepare_tokens(source);
        let result = _lex(tokens).expect("Pipeline should not fail");
        assert!(!result.is_empty());
    }

    // @audit: hardcoded_source
    #[test]
    fn test_lex_nested_indentation() {
        let source = "Level 1:\n    Level 2:\n        Level 3 content";
        let tokens = prepare_tokens(source);
        let result = _lex(tokens).expect("Pipeline should not fail");
        assert!(!result.is_empty());
    }

    // @audit: hardcoded_source
    #[test]
    fn test_lex_with_blank_lines() {
        let source = "Para 1\n\nPara 2\n\nPara 3";
        let tokens = prepare_tokens(source);
        let result = _lex(tokens).expect("Pipeline should not fail");
        assert!(!result.is_empty());
    }

    // @audit: hardcoded_source
    #[test]
    fn test_pipeline_blank_line_preservation() {
        let source = "Para 1\n\nPara 2";

        // Check LineTokens stage
        let line_tokens_output = _lex_stage(prepare_tokens(source), PipelineStage::LineTokens);
        if let PipelineOutput::LineTokens(tokens) = line_tokens_output {
            // Should have 3 line tokens: Para1, BlankLine, Para2
            assert_eq!(tokens.len(), 3);
        }

        // Check TokenTree stage
        let tree_output = _lex_stage(prepare_tokens(source), PipelineStage::TokenTree);
        if let PipelineOutput::TokenTree(tree) = tree_output {
            // Verify we get a container with children
            assert!(!tree.is_empty());
        }
    }

    // @audit: hardcoded_source
    #[test]
    fn test_lex_mixed_content() {
        let source = "Title:\n\n    First paragraph\n\n    - List item 1\n    - List item 2";
        let tokens = prepare_tokens(source);
        let result = _lex(tokens).expect("Pipeline should not fail");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_lex_with_annotations() {
        let source = ":: note ::\nSome text\n\n:: note :: with inline content";
        let tokens = prepare_tokens(source);
        let result = _lex(tokens).expect("Pipeline should not fail");
        assert!(!result.is_empty());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::lex::lexers::base_tokenization::tokenize;
    use crate::lex::lexers::ensure_source_ends_with_newline;

    // Helper to prepare token stream from source
    fn prepare_tokens(source: &str) -> Vec<(Token, std::ops::Range<usize>)> {
        let source_with_newline = ensure_source_ends_with_newline(source);
        tokenize(&source_with_newline)
    }

    #[test]
    fn test_pipeline_with_000_paragraphs() {
        let content = std::fs::read_to_string("docs/specs/v1/samples/000-paragraphs.lex")
            .expect("Could not read sample file");
        let tokens = prepare_tokens(&content);
        let tree = _lex(tokens).expect("Pipeline should not fail");
        assert!(
            !tree.is_empty(),
            "Token tree should not be empty for paragraphs"
        );
    }

    #[test]
    fn test_pipeline_with_040_lists() {
        let content = std::fs::read_to_string("docs/specs/v1/samples/040-lists.lex")
            .expect("Could not read sample file");
        let tokens = prepare_tokens(&content);
        let tree = _lex(tokens).expect("Pipeline should not fail");
        assert!(!tree.is_empty(), "Token tree should not be empty for lists");
    }

    #[test]
    fn test_pipeline_with_050_paragraph_lists() {
        let content = std::fs::read_to_string("docs/specs/v1/samples/050-paragraph-lists.lex")
            .expect("Could not read sample file");
        let tokens = prepare_tokens(&content);
        let tree = _lex(tokens).expect("Pipeline should not fail");
        assert!(
            !tree.is_empty(),
            "Token tree should not be empty for mixed content"
        );
    }

    #[test]
    fn test_pipeline_with_090_definitions() {
        let content = std::fs::read_to_string("docs/specs/v1/samples/090-definitions-simple.lex")
            .expect("Could not read sample file");
        let tokens = prepare_tokens(&content);
        let tree = _lex(tokens).expect("Pipeline should not fail");
        assert!(
            !tree.is_empty(),
            "Token tree should not be empty for definitions"
        );
    }

    #[test]
    fn test_pipeline_with_120_annotations() {
        let content = std::fs::read_to_string("docs/specs/v1/samples/120-annotations-simple.lex")
            .expect("Could not read sample file");
        let tokens = prepare_tokens(&content);
        let tree = _lex(tokens).expect("Pipeline should not fail");
        assert!(
            !tree.is_empty(),
            "Token tree should not be empty for annotations"
        );
    }

    #[test]
    fn test_pipeline_with_030_nested_sessions() {
        let content = std::fs::read_to_string(
            "docs/specs/v1/samples/030-paragraphs-sessions-nested-multiple.lex",
        )
        .expect("Could not read sample file");
        let tokens = prepare_tokens(&content);
        let tree = _lex(tokens).expect("Pipeline should not fail");
        assert!(
            !tree.is_empty(),
            "Token tree should not be empty for nested sessions"
        );
    }

    #[test]
    fn test_pipeline_with_070_nested_lists() {
        let content = std::fs::read_to_string("docs/specs/v1/samples/070-nested-lists-simple.lex")
            .expect("Could not read sample file");
        let tokens = prepare_tokens(&content);
        let tree = _lex(tokens).expect("Pipeline should not fail");
        assert!(
            !tree.is_empty(),
            "Token tree should not be empty for nested lists"
        );
    }

    #[test]
    fn test_pipeline_stage_consistency_with_real_file() {
        let content = std::fs::read_to_string("docs/specs/v1/samples/040-lists.lex")
            .expect("Could not read sample file");

        // Verify all stages return successfully for real file
        let raw = _lex_stage(prepare_tokens(&content), PipelineStage::RawTokens);
        assert!(matches!(raw, PipelineOutput::Tokens(_)));

        let after_ws = _lex_stage(prepare_tokens(&content), PipelineStage::AfterWhitespace);
        assert!(matches!(after_ws, PipelineOutput::Tokens(_)));

        let after_ind = _lex_stage(prepare_tokens(&content), PipelineStage::AfterIndentation);
        assert!(matches!(after_ind, PipelineOutput::Tokens(_)));

        let after_blank = _lex_stage(prepare_tokens(&content), PipelineStage::AfterBlankLines);
        assert!(matches!(after_blank, PipelineOutput::Tokens(_)));

        let line_tokens = _lex_stage(prepare_tokens(&content), PipelineStage::LineTokens);
        assert!(matches!(line_tokens, PipelineOutput::LineTokens(_)));

        let tree = _lex_stage(prepare_tokens(&content), PipelineStage::TokenTree);
        assert!(matches!(tree, PipelineOutput::TokenTree(_)));
    }
}
