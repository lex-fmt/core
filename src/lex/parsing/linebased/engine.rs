//! Linebased Parser Engine - Tree Walker and Orchestrator
//!
//! This module implements the main parsing orchestrator that:
//! 1. Walks the semantic line token tree (from linebased lexer)
//! 2. Groups tokens at each level into flat sequences
//! 3. Applies pattern matching to recognize grammar elements
//! 4. Recursively processes indented blocks
//! 5. Delegates to unwrapper for pattern-to-AST conversion
//! 6. Returns final Document AST
//!
//! The tree walking is completely decoupled from grammar/pattern matching,
//! making it testable and maintainable independently.

use super::declarative_grammar;
use crate::lex::lexing::linebased::tokens_linebased::LineContainer;
use crate::lex::parsing::builder::AstBuilder;
use crate::lex::parsing::ir::{NodeType, ParseNode};
use crate::lex::parsing::Document;

/// Parse using the new declarative grammar engine (Delivery 2).
///
/// This is the main entry point for the new linebased parser using LineContainerToken.
/// It uses the declarative grammar matcher and recursive descent parser.
///
/// # Arguments
/// * `tree` - The token tree from the linebased lexer (LineContainerToken)
/// * `source` - The original source text (for location tracking)
///
/// # Returns
/// A Document AST if successful
pub fn parse_experimental_v2(tree: LineContainer, source: &str) -> Result<Document, String> {
    // Extract children from root container
    let children = match tree {
        LineContainer::Container { children, .. } => children,
        LineContainer::Token(_) => {
            return Err("Expected root container, found single token".to_string())
        }
    };

    // Use declarative grammar engine to parse
    let content = declarative_grammar::parse_with_declarative_grammar(children, source)?;
    let root_node = ParseNode::new(NodeType::Document, vec![], content);
    let builder = AstBuilder::new(source);
    Ok(builder.build(root_node))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::parsing::ContentItem;
    use crate::lex::pipeline::{ExecutionOutput, PipelineExecutor};

    // Helper to prepare token stream and call pipeline
    fn lex_helper(source: &str) -> Result<crate::lex::lexing::LineContainer, String> {
        let executor = PipelineExecutor::new();
        let output = executor
            .execute("tokens-linebased-tree", source)
            .map_err(|e| format!("Pipeline execution failed: {}", e))?;

        match output {
            ExecutionOutput::Tokens(stream) => {
                crate::lex::pipeline::adapters_linebased::token_stream_to_line_container(stream)
                    .map_err(|e| format!("Failed to convert to line container: {:?}", e))
            }
            _ => Err("Expected Tokens output from tokens-linebased-tree config".to_string()),
        }
    }

    #[test]
    fn test_parse_simple_paragraphs() {
        // Use tokens from the linebased lexer pipeline
        let source = "Simple paragraph\n";
        let container = lex_helper(source).expect("Failed to tokenize");

        let result = parse_experimental_v2(container, source);
        assert!(result.is_ok(), "Parser should succeed");

        let doc = result.unwrap();
        // Should have 1 paragraph with 1 line
        assert!(!doc.root.children.is_empty(), "Should have content");
        assert!(matches!(doc.root.children[0], ContentItem::Paragraph(_)));
    }

    #[test]
    fn test_parse_definition() {
        // Use tokens from the linebased lexer pipeline
        let source = "Definition:\n    This is the definition content\n";
        let container = lex_helper(source).expect("Failed to tokenize");

        let result = parse_experimental_v2(container, source);
        assert!(result.is_ok(), "Parser should succeed");

        let doc = result.unwrap();
        // Should have Definition at root level
        let has_definition = doc
            .root
            .children
            .iter()
            .any(|item| matches!(item, ContentItem::Definition(_)));
        assert!(has_definition, "Should contain Definition node");
    }

    #[test]
    fn test_parse_session() {
        // Use tokens from the linebased lexer pipeline
        let source = "Session:\n\n    Session content here\n";
        let container = lex_helper(source).expect("Failed to tokenize");

        let result = parse_experimental_v2(container, source);
        assert!(result.is_ok(), "Parser should succeed");

        let doc = result.unwrap();
        // Should have Session at root level (with blank line before content)
        let has_session = doc
            .root
            .children
            .iter()
            .any(|item| matches!(item, ContentItem::Session(_)));
        assert!(has_session, "Should contain a Session node");
    }

    #[test]
    fn test_parse_annotation() {
        // Use tokens from the linebased lexer pipeline
        let source = ":: note ::\n";
        let container = lex_helper(source).expect("Failed to tokenize");

        let result = parse_experimental_v2(container, source);
        assert!(result.is_ok(), "Parser should succeed");

        let doc = result.unwrap();
        // Should have Annotation at root level
        let has_annotation = doc
            .root
            .children
            .iter()
            .any(|item| matches!(item, ContentItem::Annotation(_)));
        assert!(has_annotation, "Should contain an Annotation node");
    }

    #[test]
    fn test_annotations_120_simple() {
        let source = std::fs::read_to_string("docs/specs/v1/samples/120-annotations-simple.lex")
            .expect("Could not read 120 sample");
        let container = lex_helper(&source).expect("Failed to tokenize");

        let doc = parse_experimental_v2(container, &source).expect("Parser failed");

        eprintln!("\n=== 120 ANNOTATIONS SIMPLE ===");
        eprintln!("Root items count: {}", doc.root.children.len());
        for (i, item) in doc.root.children.iter().enumerate() {
            match item {
                ContentItem::Paragraph(p) => {
                    eprintln!("  [{}] Paragraph: {} lines", i, p.lines.len())
                }
                ContentItem::Annotation(a) => {
                    eprintln!(
                        "  [{}] Annotation: label='{}' params={}",
                        i,
                        a.label.value,
                        a.parameters.len()
                    )
                }
                ContentItem::Session(s) => {
                    eprintln!("  [{}] Session: {} items", i, s.children.len())
                }
                ContentItem::List(l) => eprintln!("  [{}] List: {} items", i, l.items.len()),
                _ => eprintln!("  [{}] Other", i),
            }
        }

        // Verify we have paragraphs and annotations
        let has_annotations = doc
            .root
            .children
            .iter()
            .any(|item| matches!(item, ContentItem::Annotation(_)));
        let has_paragraphs = doc
            .root
            .children
            .iter()
            .any(|item| matches!(item, ContentItem::Paragraph(_)));

        assert!(has_annotations, "Should contain Annotation nodes");
        assert!(has_paragraphs, "Should contain Paragraph nodes");
    }

    #[test]
    fn test_annotations_130_block_content() {
        let source =
            std::fs::read_to_string("docs/specs/v1/samples/130-annotations-block-content.lex")
                .expect("Could not read 130 sample");
        let container = lex_helper(&source).expect("Failed to tokenize");

        let doc = parse_experimental_v2(container, &source).expect("Parser failed");

        eprintln!("\n=== 130 ANNOTATIONS BLOCK CONTENT ===");
        eprintln!("Root items count: {}", doc.root.children.len());
        for (i, item) in doc.root.children.iter().enumerate() {
            match item {
                ContentItem::Paragraph(p) => {
                    eprintln!("  [{}] Paragraph: {} lines", i, p.lines.len())
                }
                ContentItem::Annotation(a) => {
                    eprintln!(
                        "  [{}] Annotation: label='{}' params={} content={} items",
                        i,
                        a.label.value,
                        a.parameters.len(),
                        a.children.len()
                    )
                }
                ContentItem::Session(s) => {
                    eprintln!("  [{}] Session: {} items", i, s.children.len())
                }
                ContentItem::List(l) => eprintln!("  [{}] List: {} items", i, l.items.len()),
                _ => eprintln!("  [{}] Other", i),
            }
        }

        // Verify we have annotations with block content
        let annotations_with_content = doc
            .root
            .children
            .iter()
            .filter_map(|item| match item {
                ContentItem::Annotation(a) => Some(a),
                _ => None,
            })
            .filter(|a| !a.children.is_empty())
            .count();

        assert!(
            annotations_with_content > 0,
            "Should have annotations with block content"
        );
    }

    #[test]
    fn test_annotations_combined_trifecta() {
        // Test annotations combined with paragraphs, lists, and sessions
        let source = r#"Document with annotations and trifecta

:: info ::

Paragraph before session.

1. Session with annotation inside

    :: note author="system" ::
        This is an annotated note within a session
    ::

    - List item 1
    - List item 2

    Another paragraph in session.

:: warning severity=high ::
    - Item in annotated warning
    - Important item
::

Final paragraph.
"#;

        let container = lex_helper(source).expect("Failed to tokenize");

        let doc = parse_experimental_v2(container, source).expect("Parser failed");

        eprintln!("\n=== ANNOTATIONS + TRIFECTA COMBINED ===");
        eprintln!("Root items count: {}", doc.root.children.len());
        for (i, item) in doc.root.children.iter().enumerate() {
            match item {
                ContentItem::Paragraph(p) => {
                    eprintln!("  [{}] Paragraph: {} lines", i, p.lines.len())
                }
                ContentItem::Annotation(a) => {
                    eprintln!(
                        "  [{}] Annotation: label='{}' content={} items",
                        i,
                        a.label.value,
                        a.children.len()
                    )
                }
                ContentItem::Session(s) => {
                    eprintln!("  [{}] Session: {} items", i, s.children.len())
                }
                ContentItem::List(l) => eprintln!("  [{}] List: {} items", i, l.items.len()),
                _ => eprintln!("  [{}] Other", i),
            }
        }

        // Verify mixed content
        let has_annotations = doc
            .root
            .children
            .iter()
            .any(|item| matches!(item, ContentItem::Annotation(_)));
        let has_paragraphs = doc
            .root
            .children
            .iter()
            .any(|item| matches!(item, ContentItem::Paragraph(_)));
        let has_sessions = doc
            .root
            .children
            .iter()
            .any(|item| matches!(item, ContentItem::Session(_)));

        assert!(has_annotations, "Should contain annotations");
        assert!(has_paragraphs, "Should contain paragraphs");
        assert!(has_sessions, "Should contain sessions");
    }
}
