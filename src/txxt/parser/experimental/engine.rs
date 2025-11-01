//! Experimental Parser Engine - Tree Walker and Orchestrator
//!
//! This module implements the main parsing orchestrator that:
//! 1. Walks the semantic line token tree (from experimental lexer)
//! 2. Groups tokens at each level into flat sequences
//! 3. Applies pattern matching to recognize grammar elements
//! 4. Recursively processes indented blocks
//! 5. Delegates to unwrapper for pattern-to-AST conversion
//! 6. Returns final Document AST
//!
//! The tree walking is completely decoupled from grammar/pattern matching,
//! making it testable and maintainable independently.

use super::txxt_grammar::TxxtGrammarRules;
use crate::txxt::ast::TextContent;
use crate::txxt::lexer::tokens::LineTokenType;
use crate::txxt::lexer::transformations::experimental_transform_indentation_to_token_tree::LineTokenTree;
use crate::txxt::parser::{ContentItem, Document, Location, Position, Session};

/// Parse a semantic line token tree into an AST Document.
///
/// This is the main entry point for the experimental parser.
/// It orchestrates the tree walking and pattern matching process.
///
/// # Arguments
/// * `tree` - The token tree from the experimental lexer
/// * `source` - The original source text (for location tracking)
///
/// # Returns
/// A Document AST if successful
pub fn parse_experimental(tree: Vec<LineTokenTree>, source: &str) -> Result<Document, String> {
    // Walk the tree and convert to content items
    let content = walk_and_parse(&tree, source)?;

    // Create the root session containing all top-level content
    let root = Session {
        title: TextContent::from_string("root".to_string(), None),
        content,
        location: Location {
            start: Position { line: 0, column: 0 },
            end: Position { line: 0, column: 0 },
        },
    };

    Ok(Document {
        metadata: vec![],
        root,
    })
}

/// Recursively walk the token tree and parse content at each level.
///
/// Algorithm:
/// 1. Convert tree nodes to token types at current level
/// 2. Apply pattern matching using grammar rules
/// 3. For each matched pattern:
///    - If it includes a nested block, recursively parse it
///    - Use unwrapper to convert pattern + tokens → AST node
/// 4. Return the list of content items
fn walk_and_parse(tree: &[LineTokenTree], source: &str) -> Result<Vec<ContentItem>, String> {
    let grammar =
        TxxtGrammarRules::new().map_err(|e| format!("Failed to create grammar rules: {}", e))?;

    let mut content_items = Vec::new();
    let mut i = 0;

    while i < tree.len() {
        // Extract token types at current level
        let remaining_tree = &tree[i..];
        let token_types: Vec<LineTokenType> = remaining_tree
            .iter()
            .map_while(|node| {
                match node {
                    LineTokenTree::Token(line_token) => Some(line_token.line_type),
                    LineTokenTree::Block(_) => None, // Stop at blocks
                }
            })
            .collect();

        // Try to match a pattern
        let (item, consumed) = parse_node_at_level(remaining_tree, &token_types, &grammar, source)?;
        content_items.push(item);
        i += consumed;
    }

    Ok(content_items)
}

/// Parse a single node or pattern starting at the current position in the tree.
///
/// Tries patterns in order of specificity, returns the matched pattern and number of tree items consumed.
fn parse_node_at_level(
    tree: &[LineTokenTree],
    token_types: &[LineTokenType],
    grammar: &TxxtGrammarRules,
    source: &str,
) -> Result<(ContentItem, usize), String> {
    // Handle blank lines: create a simple paragraph with no content
    if !token_types.is_empty() && token_types[0] == LineTokenType::BlankLine {
        if let LineTokenTree::Token(line_token) = &tree[0] {
            let item = super::unwrapper::unwrap_token_to_paragraph(line_token, source)?;
            return Ok((item, 1));
        }
    }

    // Try annotation pattern first (most specific)
    if let Some(_consumed) = grammar.try_annotation(token_types) {
        if let LineTokenTree::Token(line_token) = &tree[0] {
            let item = super::unwrapper::unwrap_annotation(line_token, source)?;
            return Ok((item, 1));
        }
    }

    // Check if we have a block following tokens (potential definition/session/list)
    let mut tokens_before_block = 0;
    for (idx, node) in tree.iter().enumerate() {
        match node {
            LineTokenTree::Token(_) => tokens_before_block = idx + 1,
            LineTokenTree::Block(_) => break,
        }
    }

    // If we found a block, we have a structured element (needs pattern matching with blocks)
    // For now, just consume tokens until block and wrap the block
    if tokens_before_block > 0 && tokens_before_block < tree.len() {
        // Consume all tokens before the block
        if let LineTokenTree::Token(line_token) = &tree[0] {
            let item = super::unwrapper::unwrap_token_to_paragraph(line_token, source)?;
            return Ok((item, 1));
        }
    }

    // Default: try paragraph (fallback pattern)
    if let Some(_consumed) = grammar.try_paragraph(token_types) {
        if let LineTokenTree::Token(line_token) = &tree[0] {
            let item = super::unwrapper::unwrap_token_to_paragraph(line_token, source)?;
            return Ok((item, 1));
        }
    }

    // If block is next, recursively parse its content and wrap in a session
    if let LineTokenTree::Block(children) = &tree[0] {
        let block_content = walk_and_parse(children, source)?;
        let container = Session {
            title: TextContent::from_string("container".to_string(), None),
            content: block_content,
            location: Location {
                start: Position { line: 0, column: 0 },
                end: Position { line: 0, column: 0 },
            },
        };
        return Ok((ContentItem::Session(container), 1));
    }

    Err("No pattern matched and no block found".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::txxt::lexer::tokens::{LineToken, LineTokenType, Token};

    fn make_line_token(line_type: LineTokenType, tokens: Vec<Token>) -> LineToken {
        LineToken {
            source_tokens: tokens,
            line_type,
        }
    }

    #[test]
    fn test_engine_parses_simple_paragraph() {
        let tree = vec![LineTokenTree::Token(make_line_token(
            LineTokenType::ParagraphLine,
            vec![Token::Text("Hello world".to_string())],
        ))];

        let result = parse_experimental(tree, "Hello world\n");
        assert!(result.is_ok());

        let doc = result.unwrap();
        assert_eq!(doc.root.content.len(), 1);
        assert!(matches!(doc.root.content[0], ContentItem::Paragraph(_)));
    }

    #[test]
    fn test_engine_parses_multiple_paragraphs() {
        let tree = vec![
            LineTokenTree::Token(make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Line 1".to_string())],
            )),
            LineTokenTree::Token(make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Line 2".to_string())],
            )),
            LineTokenTree::Token(make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Line 3".to_string())],
            )),
        ];

        let result = parse_experimental(tree, "Line 1\nLine 2\nLine 3\n");
        assert!(result.is_ok());

        let doc = result.unwrap();
        assert_eq!(doc.root.content.len(), 3);
        assert!(matches!(doc.root.content[0], ContentItem::Paragraph(_)));
        assert!(matches!(doc.root.content[1], ContentItem::Paragraph(_)));
        assert!(matches!(doc.root.content[2], ContentItem::Paragraph(_)));
    }

    #[test]
    fn test_engine_parses_simple_block() {
        let tree = vec![
            LineTokenTree::Token(make_line_token(
                LineTokenType::SubjectLine,
                vec![Token::Text("Title".to_string()), Token::Colon],
            )),
            LineTokenTree::Block(vec![LineTokenTree::Token(make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Content".to_string())],
            ))]),
        ];

        let result = parse_experimental(tree, "Title:\n    Content\n");
        assert!(result.is_ok());

        let doc = result.unwrap();
        assert_eq!(doc.root.content.len(), 2);
        assert!(matches!(doc.root.content[0], ContentItem::Paragraph(_))); // Subject line as paragraph
        assert!(matches!(doc.root.content[1], ContentItem::Session(_))); // Block as container
    }

    #[test]
    fn test_engine_parses_nested_blocks() {
        let tree = vec![
            LineTokenTree::Token(make_line_token(
                LineTokenType::SubjectLine,
                vec![Token::Text("Level0".to_string()), Token::Colon],
            )),
            LineTokenTree::Block(vec![
                LineTokenTree::Token(make_line_token(
                    LineTokenType::ParagraphLine,
                    vec![Token::Text("Content0".to_string())],
                )),
                LineTokenTree::Token(make_line_token(
                    LineTokenType::SubjectLine,
                    vec![Token::Text("Level1".to_string()), Token::Colon],
                )),
                LineTokenTree::Block(vec![LineTokenTree::Token(make_line_token(
                    LineTokenType::ParagraphLine,
                    vec![Token::Text("Content1".to_string())],
                ))]),
            ]),
        ];

        let result = parse_experimental(
            tree,
            "Level0:\n    Content0\n    Level1:\n        Content1\n",
        );
        assert!(result.is_ok());

        let doc = result.unwrap();
        // Root has: paragraph (Level0) + container (block)
        assert_eq!(doc.root.content.len(), 2);
        assert!(matches!(doc.root.content[1], ContentItem::Session(_)));

        // Inside the container: paragraph (Content0) + paragraph (Level1) + container (block)
        if let ContentItem::Session(container) = &doc.root.content[1] {
            assert_eq!(container.content.len(), 3);
        }
    }

    #[test]
    fn test_engine_parses_empty_block() {
        let tree = vec![
            LineTokenTree::Token(make_line_token(
                LineTokenType::SubjectLine,
                vec![Token::Text("Title".to_string()), Token::Colon],
            )),
            LineTokenTree::Block(vec![]), // Empty block
        ];

        let result = parse_experimental(tree, "Title:\n");
        assert!(result.is_ok());

        let doc = result.unwrap();
        assert_eq!(doc.root.content.len(), 2);
        // The empty block should still create a session (even if empty)
        assert!(matches!(doc.root.content[1], ContentItem::Session(_)));

        if let ContentItem::Session(container) = &doc.root.content[1] {
            assert_eq!(container.content.len(), 0);
        }
    }

    #[test]
    fn test_engine_parses_multiple_blocks_at_same_level() {
        let tree = vec![
            LineTokenTree::Token(make_line_token(
                LineTokenType::SubjectLine,
                vec![Token::Text("Title1".to_string()), Token::Colon],
            )),
            LineTokenTree::Block(vec![LineTokenTree::Token(make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Content1".to_string())],
            ))]),
            LineTokenTree::Token(make_line_token(
                LineTokenType::SubjectLine,
                vec![Token::Text("Title2".to_string()), Token::Colon],
            )),
            LineTokenTree::Block(vec![LineTokenTree::Token(make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Content2".to_string())],
            ))]),
        ];

        let result = parse_experimental(tree, "Title1:\n    Content1\nTitle2:\n    Content2\n");
        assert!(result.is_ok());

        let doc = result.unwrap();
        // Should have: para(Title1) + container + para(Title2) + container
        assert_eq!(doc.root.content.len(), 4);
        assert!(matches!(doc.root.content[0], ContentItem::Paragraph(_)));
        assert!(matches!(doc.root.content[1], ContentItem::Session(_)));
        assert!(matches!(doc.root.content[2], ContentItem::Paragraph(_)));
        assert!(matches!(doc.root.content[3], ContentItem::Session(_)));
    }

    #[test]
    fn test_tree_walking_preserves_structure() {
        // Test that the tree walking correctly preserves the indentation structure
        let tree = vec![
            LineTokenTree::Token(make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Para1".to_string())],
            )),
            LineTokenTree::Token(make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Para2".to_string())],
            )),
            LineTokenTree::Block(vec![
                LineTokenTree::Token(make_line_token(
                    LineTokenType::ParagraphLine,
                    vec![Token::Text("Nested1".to_string())],
                )),
                LineTokenTree::Block(vec![LineTokenTree::Token(make_line_token(
                    LineTokenType::ParagraphLine,
                    vec![Token::Text("DeepNested".to_string())],
                ))]),
                LineTokenTree::Token(make_line_token(
                    LineTokenType::ParagraphLine,
                    vec![Token::Text("Nested2".to_string())],
                )),
            ]),
            LineTokenTree::Token(make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Para3".to_string())],
            )),
        ];

        let result = parse_experimental(tree, "");
        assert!(result.is_ok());

        let doc = result.unwrap();
        // Root level: Para1 + Para2 + Container + Para3
        assert_eq!(doc.root.content.len(), 4);

        // Check nested structure
        if let ContentItem::Session(container) = &doc.root.content[2] {
            // Inside container: Nested1 + Container + Nested2
            assert_eq!(container.content.len(), 3);

            // Check deep nesting
            if let ContentItem::Session(deep_container) = &container.content[1] {
                assert_eq!(deep_container.content.len(), 1);
            }
        }
    }

    #[test]
    fn test_engine_handles_blank_lines() {
        let tree = vec![
            LineTokenTree::Token(make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("Text".to_string())],
            )),
            LineTokenTree::Token(make_line_token(LineTokenType::BlankLine, vec![])),
            LineTokenTree::Token(make_line_token(
                LineTokenType::ParagraphLine,
                vec![Token::Text("More text".to_string())],
            )),
        ];

        let result = parse_experimental(tree, "Text\n\nMore text\n");
        assert!(result.is_ok());

        let doc = result.unwrap();
        // Should parse as 3 items (including blank line)
        assert_eq!(doc.root.content.len(), 3);
    }

    #[test]
    fn test_engine_handles_complex_tree_structure() {
        // Complex real-world-like structure
        let tree = vec![
            LineTokenTree::Token(make_line_token(
                LineTokenType::SubjectLine,
                vec![Token::Text("Outer Session".to_string()), Token::Colon],
            )),
            LineTokenTree::Block(vec![
                LineTokenTree::Token(make_line_token(
                    LineTokenType::ParagraphLine,
                    vec![Token::Text("Some text.".to_string())],
                )),
                LineTokenTree::Token(make_line_token(LineTokenType::BlankLine, vec![])),
                LineTokenTree::Token(make_line_token(
                    LineTokenType::SubjectLine,
                    vec![Token::Text("Inner Session".to_string()), Token::Colon],
                )),
                LineTokenTree::Block(vec![
                    LineTokenTree::Token(make_line_token(
                        LineTokenType::ParagraphLine,
                        vec![Token::Text("More text.".to_string())],
                    )),
                    LineTokenTree::Token(make_line_token(
                        LineTokenType::ListLine,
                        vec![
                            Token::Dash,
                            Token::Whitespace,
                            Token::Text("Item".to_string()),
                        ],
                    )),
                ]),
            ]),
        ];

        let result = parse_experimental(tree, "");
        assert!(result.is_ok());

        let doc = result.unwrap();
        // Root: para(Outer) + container
        assert_eq!(doc.root.content.len(), 2);

        if let ContentItem::Session(outer) = &doc.root.content[1] {
            // Inside outer: para + blank + para(Inner) + container
            assert_eq!(outer.content.len(), 4);

            if let ContentItem::Session(inner) = &outer.content[3] {
                // Inside inner: para + listitem
                assert_eq!(inner.content.len(), 2);
            }
        }
    }
}
