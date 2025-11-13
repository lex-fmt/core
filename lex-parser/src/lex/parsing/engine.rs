//! Parser Engine - Tree Walker and Orchestrator
//!
//! This module implements the main parsing orchestrator that:
//! 1. Walks the semantic line token tree (from the lexer)
//! 2. Groups tokens at each level into flat sequences
//! 3. Applies pattern matching to recognize grammar elements
//! 4. Recursively processes indented blocks
//! 5. Delegates to unwrapper for pattern-to-AST conversion
//! 6. Returns final Document AST
//!
//! The tree walking is completely decoupled from grammar/pattern matching,
//! making it testable and maintainable independently.
use super::parser;
use crate::lex::building::ast_builder::AstBuilder;
use crate::lex::parsing::ir::{NodeType, ParseNode};
use crate::lex::parsing::Document;
use crate::lex::token::{to_line_container, LineContainer, LineToken, Token};
use std::ops::Range as ByteRange;

/// Parse from grouped token stream (main entry point).
///
/// This entry point accepts TokenStream::Grouped from the lexing pipeline.
/// The pipeline should have applied LineTokenGroupingMapper to group tokens into lines.
///
/// # Arguments
/// * `stream` - TokenStream::Grouped from lexing pipeline
/// * `source` - The original source text (for location tracking)
///
/// # Returns
/// A Document AST if successful
use crate::lex::lexing::transformations::line_token_grouping::{GroupType, GroupedTokens};

pub fn parse_from_grouped_stream(
    grouped_tokens: Vec<GroupedTokens>,
    source: &str,
) -> Result<Document, String> {
    // Convert grouped tokens to line tokens
    let line_tokens = grouped_tokens
        .into_iter()
        .map(|g| {
            let (source_tokens, token_spans): (Vec<_>, Vec<_>) =
                g.source_tokens.into_iter().unzip();
            let GroupType::Line(line_type) = g.group_type;
            LineToken {
                source_tokens,
                token_spans,
                line_type,
            }
        })
        .collect();

    // Build LineContainer tree from line tokens
    let tree = to_line_container::build_line_container(line_tokens);

    // Parse using existing logic
    parse_experimental_v2(tree, source)
}

/// Parse from flat token stream (legacy/test entry point).
///
/// This entry point is kept for backward compatibility with existing tests.
/// Production code should use parse_from_grouped_stream instead.
///
/// # Arguments
/// * `tokens` - Flat vector of (Token, Range) pairs
/// * `source` - The original source text (for location tracking)
///
/// # Returns
/// A Document AST if successful
pub fn parse_from_flat_tokens(
    tokens: Vec<(Token, ByteRange<usize>)>,
    source: &str,
) -> Result<Document, String> {
    // Apply grouping transformation inline for tests/legacy code
    use crate::lex::lexing::transformations::LineTokenGroupingMapper;

    let mut mapper = LineTokenGroupingMapper::new();
    let grouped_tokens = mapper.map(tokens);

    parse_from_grouped_stream(grouped_tokens, source)
}

/// Parse using the new declarative grammar engine (Delivery 2).
///
/// This is the main entry point for the parser using LineContainerToken.
/// It uses the declarative grammar matcher and recursive descent parser.
///
/// # Arguments
/// * `tree` - The token tree from the lexer (LineContainerToken)
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
    let content = parser::parse_with_declarative_grammar(children, source)?;
    let root_node = ParseNode::new(NodeType::Document, vec![], content);
    let builder = AstBuilder::new(source);
    Ok(builder.build(root_node))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::parsing::ContentItem;

    // Helper to prepare flat token stream
    fn lex_helper(
        source: &str,
    ) -> Result<Vec<(crate::lex::token::Token, std::ops::Range<usize>)>, String> {
        let tokens = crate::lex::lexing::tokenize(source);
        Ok(crate::lex::lexing::lex(tokens))
    }

    #[test]
    fn test_parse_simple_paragraphs() {
        // Use tokens from the lexer pipeline
        let source = "Simple paragraph\n";
        let tokens = lex_helper(source).expect("Failed to tokenize");

        let result = parse_from_flat_tokens(tokens, source);
        assert!(result.is_ok(), "Parser should succeed");

        let doc = result.unwrap();
        // Should have 1 paragraph with 1 line
        assert!(!doc.root.children.is_empty(), "Should have content");
        assert!(matches!(doc.root.children[0], ContentItem::Paragraph(_)));
    }

    #[test]
    fn test_parse_definition() {
        // Use tokens from the lexer pipeline
        let source = "Definition:\n    This is the definition content\n";
        let tokens = lex_helper(source).expect("Failed to tokenize");

        let result = parse_from_flat_tokens(tokens, source);
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
        // Use tokens from the lexer pipeline
        let source = "Session:\n\n    Session content here\n";
        let tokens = lex_helper(source).expect("Failed to tokenize");

        let result = parse_from_flat_tokens(tokens, source);
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
        // Use tokens from the lexer pipeline
        let source = ":: note ::\n";
        let tokens = lex_helper(source).expect("Failed to tokenize");

        let result = parse_from_flat_tokens(tokens, source);
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

        let tokens = lex_helper(source).expect("Failed to tokenize");

        let doc = parse_from_flat_tokens(tokens, source).expect("Parser failed");

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
