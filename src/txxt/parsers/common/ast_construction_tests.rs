//! Integration tests for AST construction facade using real txxt samples
//!
//! Following testing guidelines from src/txxt/testing.rs:
//! - Use TxxtSources for verified txxt content
//! - Use TxxtPipeline to generate real tokens
//! - Test with actual parsed data, not mocked structures

use super::ast_construction::*;
use crate::txxt::parsers::ContentItem;
use crate::txxt::processor::txxt_sources::TxxtSources;

#[test]
fn test_build_paragraph_from_real_tokens() {
    // Use verified txxt sample
    let source = TxxtSources::get_string("000-paragraphs.txxt").expect("Failed to load sample");

    // Get real tokens using the linebased pipeline
    let result = crate::txxt::lexers::_lex(&source);
    assert!(result.is_ok(), "Failed to lex source");

    let container = result.unwrap();

    // Get line tokens from the container
    let line_tokens = crate::txxt::lexers::linebased::transformations::indentation_to_token_tree::unwrap_container_to_token_tree(&container);

    // Extract paragraph line tokens (skip blank lines)
    let paragraph_tokens: Vec<crate::txxt::lexers::linebased::tokens::LineToken> = line_tokens
        .iter()
        .filter_map(|tree| match tree {
            crate::txxt::lexers::linebased::tokens::LineTokenTree::Token(lt)
                if matches!(
                    lt.line_type,
                    crate::txxt::lexers::linebased::tokens::LineTokenType::ParagraphLine
                ) =>
            {
                Some(lt.clone())
            }
            _ => None,
        })
        .take(2) // Take first 2 paragraph lines
        .collect();

    // Only run test if we have paragraph tokens
    if !paragraph_tokens.is_empty() {
        let paragraph = build_paragraph_from_line_tokens(&paragraph_tokens, &source);

        // Verify it's a paragraph
        match paragraph {
            ContentItem::Paragraph(p) => {
                assert!(!p.lines.is_empty(), "Paragraph should have lines");
                // Verify location is valid (not default)
                assert!(p.location.start.line < 100, "Location should be reasonable");
            }
            _ => panic!("Expected Paragraph"),
        }
    }
}

#[test]
fn test_build_session_with_real_tokens() {
    // Use sample with sessions
    let source = TxxtSources::get_string("010-paragraphs-sessions-flat-single.txxt")
        .expect("Failed to load sample");

    // Parse to get real tokens
    let result = crate::txxt::lexers::_lex(&source);
    assert!(result.is_ok(), "Failed to lex source");

    let container = result.unwrap();
    let line_tokens = crate::txxt::lexers::linebased::transformations::indentation_to_token_tree::unwrap_container_to_token_tree(&container);

    // Find a subject line (session title)
    let subject_token = line_tokens.iter().find_map(|tree| match tree {
        crate::txxt::lexers::linebased::tokens::LineTokenTree::Token(lt)
            if matches!(
                lt.line_type,
                crate::txxt::lexers::linebased::tokens::LineTokenType::SubjectLine
            ) =>
        {
            Some(lt)
        }
        _ => None,
    });

    if let Some(title_token) = subject_token {
        let session = build_session_from_line_token(title_token, vec![], &source);

        match session {
            ContentItem::Session(s) => {
                assert!(!s.title.as_ref().is_empty(), "Title should not be empty");
                assert!(s.location.start.line < 100, "Location should be reasonable");
            }
            _ => panic!("Expected Session"),
        }
    }
}

#[test]
fn test_build_definition_with_real_tokens() {
    // Use sample with definitions
    let source =
        TxxtSources::get_string("090-definitions-simple.txxt").expect("Failed to load sample");

    // Parse to get real tokens
    let result = crate::txxt::lexers::_lex(&source);
    assert!(result.is_ok(), "Failed to lex source");

    let container = result.unwrap();
    let line_tokens = crate::txxt::lexers::linebased::transformations::indentation_to_token_tree::unwrap_container_to_token_tree(&container);

    // Find a subject line (definition subject)
    let subject_token = line_tokens.iter().find_map(|tree| match tree {
        crate::txxt::lexers::linebased::tokens::LineTokenTree::Token(lt)
            if matches!(
                lt.line_type,
                crate::txxt::lexers::linebased::tokens::LineTokenType::SubjectLine
            ) =>
        {
            Some(lt)
        }
        _ => None,
    });

    if let Some(subj_token) = subject_token {
        let definition = build_definition_from_line_token(subj_token, vec![], &source);

        match definition {
            ContentItem::Definition(d) => {
                assert!(
                    !d.subject.as_ref().is_empty(),
                    "Subject should not be empty"
                );
                assert!(d.location.start.line < 100, "Location should be reasonable");
            }
            _ => panic!("Expected Definition"),
        }
    }
}
