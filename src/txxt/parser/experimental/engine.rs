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
        // Extract token types at current level (including blank lines - needed for pattern matching!)
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

        // Skip structural blank lines (paragraphs created from standalone blank lines)
        // These are detected as single-line paragraphs from BlankLine tokens
        let is_blank_line_paragraph = if let LineTokenTree::Token(token) = &remaining_tree[0] {
            token.line_type == LineTokenType::BlankLine
                && matches!(item, ContentItem::Paragraph(_))
                && consumed == 1
        } else {
            false
        };

        if !is_blank_line_paragraph {
            content_items.push(item);
        }

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
    if tree.is_empty() {
        return Err("Empty tree at node level".to_string());
    }

    // Check if a Block follows the current tokens (implicit INDENT)
    let has_following_block = token_types.len() < tree.len()
        && matches!(tree.get(token_types.len()), Some(LineTokenTree::Block(_)));

    // PATTERN MATCHING ORDER (from parsing.txxt and user feedback)
    // Annotation → Foreign Block → Definition → List → Session → Paragraph
    //
    // Key reasons for this order:
    // - Annotations can appear in foreign blocks (as closing markers), detect first
    // - Foreign blocks have unambiguous indentation wall boundaries
    // - Definitions/Sessions both use colon-ending, disambiguated by blank line presence
    // - Lists require blank line before + 2+ items
    // - Paragraphs are the catch-all fallback

    // 1. ANNOTATION: Lines with :: markers (take precedence everywhere)
    if let Some(_consumed) = grammar.try_annotation(token_types) {
        if let LineTokenTree::Token(line_token) = &tree[0] {
            let item = super::unwrapper::unwrap_annotation(line_token, source)?;
            return Ok((item, 1));
        }
    }

    // 2. FOREIGN BLOCK: SUBJECT_LINE + Block + ANNOTATION_LINE
    if let Some((end_idx, _indent_idx)) = grammar.try_foreign_block(token_types) {
        if end_idx <= tree.len() && end_idx >= 3 {
            if let LineTokenTree::Token(subject_token) = &tree[0] {
                if let LineTokenTree::Block(block_children) = &tree[1] {
                    if let LineTokenTree::Token(annotation_token) = &tree[2] {
                        let content_lines = block_children
                            .iter()
                            .filter_map(|child| {
                                if let LineTokenTree::Token(t) = child {
                                    Some(t)
                                } else {
                                    None
                                }
                            })
                            .collect();

                        let item = super::unwrapper::unwrap_foreign_block(
                            subject_token,
                            content_lines,
                            annotation_token,
                        )?;
                        return Ok((item, 3));
                    }
                }
            }
        }
    }

    // 3. DEFINITION: SUBJECT_LINE + Block (no blank line between)
    if has_following_block {
        if let Some(_consumed) = grammar.try_definition(token_types) {
            if let LineTokenTree::Token(subject_token) = &tree[0] {
                let block_idx = token_types.len();
                if let LineTokenTree::Block(block_children) = &tree[block_idx] {
                    let block_content = walk_and_parse(block_children, source)?;
                    let item = super::unwrapper::unwrap_definition(subject_token, block_content)?;
                    return Ok((item, block_idx + 1));
                }
            }
        }
    }

    // 4. LIST: BLANK_LINE + 2+ list items
    if let Some(consumed) = grammar.try_list(token_types) {
        let mut list_items = Vec::new();
        let mut tree_idx = 1; // Skip blank line

        while tree_idx < tree.len() && list_items.len() < consumed - 1 {
            if let LineTokenTree::Token(item_token) = &tree[tree_idx] {
                let item = super::unwrapper::unwrap_list_item(item_token, vec![])?;
                list_items.push(item);
                tree_idx += 1;
            } else {
                break;
            }
        }

        if list_items.len() >= 2 {
            let list = super::unwrapper::unwrap_list(list_items)?;
            return Ok((list, consumed));
        }
    }

    // 5. SESSION: SUBJECT_LINE + BLANK_LINE + Block
    if has_following_block {
        if let Some(_consumed) = grammar.try_session(token_types) {
            if let LineTokenTree::Token(subject_token) = &tree[0] {
                let block_idx = token_types.len();
                if let LineTokenTree::Block(block_children) = &tree[block_idx] {
                    let block_content = walk_and_parse(block_children, source)?;
                    let item = super::unwrapper::unwrap_session(subject_token, block_content)?;
                    return Ok((item, block_idx + 1));
                }
            }
        }
    }

    // 6. PARAGRAPH (fallback): any-line+ (all non-blank lines not matching above patterns)
    if let Some(consumed) = grammar.try_paragraph(token_types) {
        let mut paragraph_tokens = Vec::new();
        for i in 0..consumed {
            if i < tree.len() {
                if let LineTokenTree::Token(line_token) = &tree[i] {
                    paragraph_tokens.push(line_token.clone());
                }
            }
        }

        if !paragraph_tokens.is_empty() {
            let item = super::unwrapper::unwrap_tokens_to_paragraph(paragraph_tokens, source)?;
            return Ok((item, consumed));
        }
    }

    // If block is next with no pattern match, wrap it in a default session (shouldn't happen)
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

    Err("No pattern matched".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::txxt::lexer::transformations::experimental_pipeline::experimental_lex;

    #[test]
    fn test_parse_simple_paragraphs() {
        // Use tokens from the experimental lexer pipeline (returns token tree directly)
        let source = "Simple paragraph\n";
        let tree = experimental_lex(source).expect("Failed to tokenize");

        let result = parse_experimental(tree, source);
        assert!(result.is_ok(), "Parser should succeed");

        let doc = result.unwrap();
        // Should have 1 paragraph with 1 line
        assert!(!doc.root.content.is_empty(), "Should have content");
        assert!(matches!(doc.root.content[0], ContentItem::Paragraph(_)));
    }

    #[test]
    fn test_parse_definition() {
        // Use tokens from the experimental lexer pipeline
        let source = "Definition:\n    This is the definition content\n";
        let tree = experimental_lex(source).expect("Failed to tokenize");

        let result = parse_experimental(tree, source);
        assert!(result.is_ok(), "Parser should succeed");

        let doc = result.unwrap();
        // Should have Definition at root level
        let has_definition = doc
            .root
            .content
            .iter()
            .any(|item| matches!(item, ContentItem::Definition(_)));
        assert!(has_definition, "Should contain Definition node");
    }

    #[test]
    fn test_parse_session() {
        // Use tokens from the experimental lexer pipeline
        let source = "Session:\n\n    Session content here\n";
        let tree = experimental_lex(source).expect("Failed to tokenize");

        let result = parse_experimental(tree, source);
        assert!(result.is_ok(), "Parser should succeed");

        let doc = result.unwrap();
        // Should have Session at root level (with blank line before content)
        let has_session = doc
            .root
            .content
            .iter()
            .any(|item| matches!(item, ContentItem::Session(_)));
        assert!(has_session, "Should contain a Session node");
    }

    #[test]
    fn test_parse_annotation() {
        // Use tokens from the experimental lexer pipeline
        let source = ":: note ::\n";
        let tree = experimental_lex(source).expect("Failed to tokenize");

        let result = parse_experimental(tree, source);
        assert!(result.is_ok(), "Parser should succeed");

        let doc = result.unwrap();
        // Should have Annotation at root level
        let has_annotation = doc
            .root
            .content
            .iter()
            .any(|item| matches!(item, ContentItem::Annotation(_)));
        assert!(has_annotation, "Should contain an Annotation node");
    }
}
