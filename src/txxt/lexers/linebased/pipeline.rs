//! This module orchestrates the complete tokenization pipeline for the txxt format.
//!
//! Currently we are still running two parser designs side by side and the the newer parser requires
//! more preprocessing of the cst.
//! The pipeline consists of:
//! 1. Core tokenization using logos lexer
//! 2. Common Transformation pipeline:
//!    - Whitespace remainder processing ../transformations/normalize_whitespace.rs
//!    - Indentation transformation (Indent -> IndentLevel/DedentLevel) ../transformations/sem_indentation.rs
//!    - Blank line transformation (consecutive Newlines -> BlankLine) ../transformations/transform_blanklines.rs
//! 3. Line-based processing:
//!    - Flatten tokens into line tokens
//!    - Transform line tokens into a hierarchical tree
//!
//! This pipeline coexists with the existing lexer without modifying it.

use std::fmt;

use crate::txxt::lexers::ensure_source_ends_with_newline;
use crate::txxt::lexers::indentation::tokenize;
use crate::txxt::lexers::linebased::tokens::{LineContainerToken, LineToken};
use crate::txxt::lexers::linebased::transformations::{
    _indentation_to_token_tree, _to_line_tokens,
};
use crate::txxt::lexers::tokens::Token;
use crate::txxt::lexers::transformations::{
    process_whitespace_remainders, sem_indentation, transform_blank_lines,
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
    /// Token tree (root container with all line tokens and nested containers)
    TokenTree(LineContainerToken),
}

/// Main linebased lexer pipeline.
///
/// Runs all transformations in sequence and returns the final token tree.
///
/// # Arguments
/// * `source` - The input source text
///
/// # Returns
/// A Result containing a LineContainerToken (root node representing the entire hierarchical tree),
/// or a PipelineError if the pipeline stage returns an unexpected output type.
pub fn _lex(source: &str) -> Result<LineContainerToken, PipelineError> {
    let output = _lex_stage(source, PipelineStage::TokenTree);
    match output {
        PipelineOutput::TokenTree(tree) => Ok(tree),
        _ => Err(PipelineError::UnexpectedOutput(
            "TokenTree stage should return TokenTree output".to_string(),
        )),
    }
}

/// Attach source spans to line tokens by matching tokens in the original token stream.
///
/// This function pairs the line tokens (which have been transformed/grouped) with their
/// original source spans from the pipeline. Each line token gets assigned a span that covers
/// all the tokens that make up that line.
fn attach_spans_to_line_tokens(
    line_tokens: &mut [LineToken],
    tokens_with_spans: &[(Token, std::ops::Range<usize>)],
) {
    let mut source_idx = 0;

    for line_token in line_tokens.iter_mut() {
        // Find the start of this line's tokens in the original stream
        if source_idx >= tokens_with_spans.len() {
            break;
        }

        let line_start = tokens_with_spans[source_idx].1.start;
        let mut line_end = line_start;

        // Consume tokens from the source stream that match this line token's source_tokens
        for expected_token in &line_token.source_tokens {
            if source_idx < tokens_with_spans.len() {
                let (actual_token, span) = &tokens_with_spans[source_idx];
                // Check if tokens match (they should, since we derived line_tokens from these)
                if std::mem::discriminant(actual_token) == std::mem::discriminant(expected_token) {
                    line_end = span.end;
                    source_idx += 1;
                }
            }
        }

        // Attach the span to this line token
        line_token.source_span = Some(line_start..line_end);
    }
}

/// Linebased lexer pipeline with stage-based output.
///
/// Returns the pipeline output at any requested stage for debugging/testing.
///
/// # Arguments
/// * `source` - The input source text
/// * `stage` - The pipeline stage at which to return output
///
/// # Returns
/// Pipeline output at the requested stage
pub fn _lex_stage(source: &str, stage: PipelineStage) -> PipelineOutput {
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
    let after_indentation = sem_indentation(after_whitespace);

    if stage == PipelineStage::AfterIndentation {
        return PipelineOutput::Tokens(after_indentation);
    }

    // Stage 4: Blank line transformation
    let after_blank_lines = transform_blank_lines(after_indentation);

    if stage == PipelineStage::AfterBlankLines {
        return PipelineOutput::Tokens(after_blank_lines.clone());
    }

    // Stage 5: Line token transformation (linebased)
    // Extract tokens for transformation (spans are in after_blank_lines)
    let tokens_for_line_tokens: Vec<Token> =
        after_blank_lines.iter().map(|(t, _)| t.clone()).collect();
    let mut line_tokens = _to_line_tokens(tokens_for_line_tokens);

    // Now attach source spans to the line tokens we created
    // This is done here in the pipeline where we have access to both the tokens and their spans
    attach_spans_to_line_tokens(&mut line_tokens, &after_blank_lines);

    if stage == PipelineStage::LineTokens {
        return PipelineOutput::LineTokens(line_tokens.clone());
    }

    // Stage 6: Indentation-to-token-tree transformation (linebased)
    let token_tree = _indentation_to_token_tree(line_tokens);

    PipelineOutput::TokenTree(token_tree)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_empty_input() {
        let result = _lex("").expect("Pipeline should not fail");
        // Empty input should produce empty tree
        assert!(result.is_empty());
    }

    #[test]
    fn test_lex_single_paragraph() {
        let source = "Hello world";
        let result = _lex(source).expect("Pipeline should not fail");
        // Single paragraph should produce at least one token
        assert!(!result.is_empty());
    }

    #[test]
    fn test_lex_multiple_paragraphs() {
        let source = "First paragraph\n\nSecond paragraph";
        let result = _lex(source).expect("Pipeline should not fail");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_lex_with_indentation() {
        let source = "Title:\n    Indented content\n    More indented";
        let result = _lex(source).expect("Pipeline should not fail");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_lex_stage_raw_tokens() {
        let source = "Hello world";
        let output = _lex_stage(source, PipelineStage::RawTokens);
        match output {
            PipelineOutput::Tokens(tokens) => assert!(!tokens.is_empty()),
            _ => panic!("Expected Tokens output"),
        }
    }

    #[test]
    fn test_lex_stage_after_whitespace() {
        let source = "Hello world";
        let output = _lex_stage(source, PipelineStage::AfterWhitespace);
        match output {
            PipelineOutput::Tokens(tokens) => assert!(!tokens.is_empty()),
            _ => panic!("Expected Tokens output"),
        }
    }

    #[test]
    fn test_lex_stage_after_indentation() {
        let source = "Hello:\n    World";
        let output = _lex_stage(source, PipelineStage::AfterIndentation);
        match output {
            PipelineOutput::Tokens(tokens) => assert!(!tokens.is_empty()),
            _ => panic!("Expected Tokens output"),
        }
    }

    #[test]
    fn test_lex_stage_after_blank_lines() {
        let source = "Hello\n\nWorld";
        let output = _lex_stage(source, PipelineStage::AfterBlankLines);
        match output {
            PipelineOutput::Tokens(tokens) => assert!(!tokens.is_empty()),
            _ => panic!("Expected Tokens output"),
        }
    }

    #[test]
    fn test_lex_stage_line_tokens() {
        let source = "Title:\n    Content";
        let output = _lex_stage(source, PipelineStage::LineTokens);
        match output {
            PipelineOutput::LineTokens(tokens) => assert!(!tokens.is_empty()),
            _ => panic!("Expected LineTokens output"),
        }
    }

    #[test]
    fn test_lex_stage_token_tree() {
        let source = "Title:\n    Content";
        let output = _lex_stage(source, PipelineStage::TokenTree);
        match output {
            PipelineOutput::TokenTree(tree) => assert!(!tree.is_empty()),
            _ => panic!("Expected TokenTree output"),
        }
    }

    #[test]
    fn test_pipeline_consistency_across_stages() {
        let source = "Item 1\n\n- First\n- Second";

        // Verify all stages return successfully
        let raw = _lex_stage(source, PipelineStage::RawTokens);
        assert!(matches!(raw, PipelineOutput::Tokens(_)));

        let after_ws = _lex_stage(source, PipelineStage::AfterWhitespace);
        assert!(matches!(after_ws, PipelineOutput::Tokens(_)));

        let after_ind = _lex_stage(source, PipelineStage::AfterIndentation);
        assert!(matches!(after_ind, PipelineOutput::Tokens(_)));

        let after_blank = _lex_stage(source, PipelineStage::AfterBlankLines);
        assert!(matches!(after_blank, PipelineOutput::Tokens(_)));

        let line_tokens = _lex_stage(source, PipelineStage::LineTokens);
        assert!(matches!(line_tokens, PipelineOutput::LineTokens(_)));

        let tree = _lex_stage(source, PipelineStage::TokenTree);
        assert!(matches!(tree, PipelineOutput::TokenTree(_)));
    }

    #[test]
    fn test_lex_list_structure() {
        let source = "Items:\n    - First\n    - Second\n    - Third";
        let result = _lex(source).expect("Pipeline should not fail");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_lex_nested_indentation() {
        let source = "Level 1:\n    Level 2:\n        Level 3 content";
        let result = _lex(source).expect("Pipeline should not fail");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_lex_with_blank_lines() {
        let source = "Para 1\n\nPara 2\n\nPara 3";
        let result = _lex(source).expect("Pipeline should not fail");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_pipeline_blank_line_preservation() {
        use crate::txxt::lexers::linebased::transformations::unwrap_container_to_token_tree;
        use crate::txxt::lexers::LineTokenTree;

        let source = "Para 1\n\nPara 2";

        // Check LineTokens stage
        let line_tokens_output = _lex_stage(source, PipelineStage::LineTokens);
        if let PipelineOutput::LineTokens(tokens) = line_tokens_output {
            eprintln!("LineTokens (count={}): ", tokens.len());
            for (i, token) in tokens.iter().enumerate() {
                eprintln!("  [{}] {:?}", i, token.line_type);
            }
        }

        // Check TokenTree stage
        let tree_output = _lex_stage(source, PipelineStage::TokenTree);
        if let PipelineOutput::TokenTree(tree) = tree_output {
            let legacy_tree = unwrap_container_to_token_tree(&tree);
            eprintln!("TokenTree (count={}): ", legacy_tree.len());
            for (i, node) in legacy_tree.iter().enumerate() {
                match node {
                    LineTokenTree::Token(t) => {
                        eprintln!("  [{}] Token: {:?}", i, t.line_type);
                    }
                    LineTokenTree::Container(c) => {
                        eprintln!("  [{}] Container: {} tokens", i, c.source_tokens.len());
                    }
                    LineTokenTree::Block(_) => {
                        eprintln!("  [{}] Block", i);
                    }
                }
            }
        }
    }

    #[test]
    fn test_lex_mixed_content() {
        let source = "Title:\n\n    First paragraph\n\n    - List item 1\n    - List item 2";
        let result = _lex(source).expect("Pipeline should not fail");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_lex_with_annotations() {
        let source = ":: note ::\nSome text\n\n:: note :: with inline content";
        let result = _lex(source).expect("Pipeline should not fail");
        assert!(!result.is_empty());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_pipeline_with_000_paragraphs() {
        let content = std::fs::read_to_string("docs/specs/v1/samples/000-paragraphs.txxt")
            .expect("Could not read sample file");
        let tree = _lex(&content).expect("Pipeline should not fail");
        assert!(
            !tree.is_empty(),
            "Token tree should not be empty for paragraphs"
        );
    }

    #[test]
    fn test_pipeline_with_040_lists() {
        let content = std::fs::read_to_string("docs/specs/v1/samples/040-lists.txxt")
            .expect("Could not read sample file");
        let tree = _lex(&content).expect("Pipeline should not fail");
        assert!(!tree.is_empty(), "Token tree should not be empty for lists");
    }

    #[test]
    fn test_pipeline_with_050_paragraph_lists() {
        let content = std::fs::read_to_string("docs/specs/v1/samples/050-paragraph-lists.txxt")
            .expect("Could not read sample file");
        let tree = _lex(&content).expect("Pipeline should not fail");
        assert!(
            !tree.is_empty(),
            "Token tree should not be empty for mixed content"
        );
    }

    #[test]
    fn test_pipeline_with_090_definitions() {
        let content = std::fs::read_to_string("docs/specs/v1/samples/090-definitions-simple.txxt")
            .expect("Could not read sample file");
        let tree = _lex(&content).expect("Pipeline should not fail");
        assert!(
            !tree.is_empty(),
            "Token tree should not be empty for definitions"
        );
    }

    #[test]
    fn test_pipeline_with_120_annotations() {
        let content = std::fs::read_to_string("docs/specs/v1/samples/120-annotations-simple.txxt")
            .expect("Could not read sample file");
        let tree = _lex(&content).expect("Pipeline should not fail");
        assert!(
            !tree.is_empty(),
            "Token tree should not be empty for annotations"
        );
    }

    #[test]
    fn test_pipeline_with_030_nested_sessions() {
        let content = std::fs::read_to_string(
            "docs/specs/v1/samples/030-paragraphs-sessions-nested-multiple.txxt",
        )
        .expect("Could not read sample file");
        let tree = _lex(&content).expect("Pipeline should not fail");
        assert!(
            !tree.is_empty(),
            "Token tree should not be empty for nested sessions"
        );
    }

    #[test]
    fn test_pipeline_with_070_nested_lists() {
        let content = std::fs::read_to_string("docs/specs/v1/samples/070-nested-lists-simple.txxt")
            .expect("Could not read sample file");
        let tree = _lex(&content).expect("Pipeline should not fail");
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
        let raw = _lex_stage(&content, PipelineStage::RawTokens);
        assert!(matches!(raw, PipelineOutput::Tokens(_)));

        let after_ws = _lex_stage(&content, PipelineStage::AfterWhitespace);
        assert!(matches!(after_ws, PipelineOutput::Tokens(_)));

        let after_ind = _lex_stage(&content, PipelineStage::AfterIndentation);
        assert!(matches!(after_ind, PipelineOutput::Tokens(_)));

        let after_blank = _lex_stage(&content, PipelineStage::AfterBlankLines);
        assert!(matches!(after_blank, PipelineOutput::Tokens(_)));

        let line_tokens = _lex_stage(&content, PipelineStage::LineTokens);
        assert!(matches!(line_tokens, PipelineOutput::LineTokens(_)));

        let tree = _lex_stage(&content, PipelineStage::TokenTree);
        assert!(matches!(tree, PipelineOutput::TokenTree(_)));
    }
}
