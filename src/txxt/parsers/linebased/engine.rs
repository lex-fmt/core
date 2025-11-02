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

use super::txxt_grammar::TxxtGrammarRules;
use crate::txxt::ast::elements::{BlankLineGroup, Paragraph};
use crate::txxt::ast::TextContent;
use crate::txxt::lexers::{LineTokenTree, LineTokenType};
use crate::txxt::parsers::{ContentItem, Document, Location, Position, Session};

/// Parse a semantic line token tree into an AST Document.
///
/// This is the main entry point for the linebased parser.
/// It orchestrates the tree walking and pattern matching process.
///
/// # Arguments
/// * `tree` - The token tree from the linebased lexer
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

        // Convert blank line paragraphs to BlankLineGroup nodes
        // These are detected as single-line paragraphs from BlankLine tokens
        let is_blank_line_paragraph = if let LineTokenTree::Token(token) = &remaining_tree[0] {
            token.line_type == LineTokenType::BlankLine
                && matches!(item, ContentItem::Paragraph(_))
                && consumed == 1
        } else {
            false
        };

        if is_blank_line_paragraph {
            // Collect consecutive blank line tokens
            let mut blank_count = 0;
            let mut blank_tokens = Vec::new();
            let mut idx = 0;

            while idx < remaining_tree.len() {
                if let LineTokenTree::Token(token) = &remaining_tree[idx] {
                    if token.line_type == LineTokenType::BlankLine {
                        blank_count += 1;
                        blank_tokens.extend(token.source_tokens.clone());
                        idx += 1;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            // Create BlankLineGroup with the first token's location
            if let LineTokenTree::Token(first_token) = &remaining_tree[0] {
                let location = if let Some(span) = &first_token.source_span {
                    // Calculate location from source span
                    let start_line = source[..span.start].matches('\n').count();
                    let end_line = source[..span.end].matches('\n').count();
                    Location {
                        start: Position {
                            line: start_line,
                            column: 0,
                        },
                        end: Position {
                            line: end_line + 1,
                            column: 0,
                        },
                    }
                } else {
                    Location::new(Position::new(0, 0), Position::new(0, 0))
                };

                let blank_group = BlankLineGroup::new(blank_count, blank_tokens).at(location);
                content_items.push(ContentItem::BlankLineGroup(blank_group));
                i += blank_count;
            } else {
                // Fallback if we can't process blank lines properly
                content_items.push(item);
                i += consumed;
            }
        } else {
            content_items.push(item);
            i += consumed;
        }
    }

    Ok(content_items)
}

/// Try to match a block annotation pattern
/// Pattern: ANNOTATION_LINE + [BLANK_LINE?] + BLOCK + [ANNOTATION_LINE?]
fn try_match_block_annotation(
    tree: &[LineTokenTree],
    grammar: &TxxtGrammarRules,
    source: &str,
) -> Result<Option<(ContentItem, usize)>, String> {
    if let Some(consumed) = grammar.try_annotation_from_tree(tree) {
        if let LineTokenTree::Token(opening_token) = &tree[0] {
            let mut block_content = vec![];
            let mut block_idx = 1;

            // Check for optional blank line after opening annotation
            if let Some(LineTokenTree::Token(blank_token)) = tree.get(1) {
                if blank_token.line_type == LineTokenType::BlankLine {
                    // Create BlankLineGroup for the blank line
                    let location = if let Some(span) = &blank_token.source_span {
                        let start_line = source[..span.start].matches('\n').count();
                        let end_line = source[..span.end].matches('\n').count();
                        Location {
                            start: Position {
                                line: start_line,
                                column: 0,
                            },
                            end: Position {
                                line: end_line + 1,
                                column: 0,
                            },
                        }
                    } else {
                        Location::new(Position::new(0, 0), Position::new(0, 0))
                    };

                    let blank_group =
                        BlankLineGroup::new(1, blank_token.source_tokens.clone()).at(location);
                    block_content.push(ContentItem::BlankLineGroup(blank_group));
                    block_idx = 2;
                }
            }

            if let LineTokenTree::Block(block_children) = &tree[block_idx] {
                let parsed_block = walk_and_parse(block_children, source)?;
                block_content.extend(parsed_block);
                let item = super::unwrapper::unwrap_annotation_with_content(
                    opening_token,
                    block_content,
                    source,
                )?;
                return Ok(Some((item, consumed)));
            }
        }
    }
    Ok(None)
}

/// Try to match a single-line annotation pattern
/// Pattern: ANNOTATION_LINE (:: label ::)
fn try_match_single_annotation(
    tree: &[LineTokenTree],
    token_types: &[LineTokenType],
    source: &str,
) -> Result<Option<(ContentItem, usize)>, String> {
    if let Some(_consumed) = token_types.first().and_then(|t| {
        if matches!(t, LineTokenType::AnnotationLine) {
            Some(())
        } else {
            None
        }
    }) {
        if let LineTokenTree::Token(line_token) = &tree[0] {
            let item = super::unwrapper::unwrap_annotation(line_token, source)?;
            return Ok(Some((item, 1)));
        }
    }
    Ok(None)
}

/// Try to match a foreign block pattern
/// Patterns:
/// - Block form: SUBJECT_LINE + [BLANK_LINE?] + BLOCK + ANNOTATION_LINE
/// - Marker form: SUBJECT_LINE + [BLANK_LINE?] + ANNOTATION_LINE
fn try_match_foreign_block(
    tree: &[LineTokenTree],
    grammar: &TxxtGrammarRules,
    source: &str,
) -> Result<Option<(ContentItem, usize)>, String> {
    if let Some(consumed) = grammar.try_foreign_block_from_tree(tree) {
        if let LineTokenTree::Token(subject_token) = &tree[0] {
            let mut check_idx = 1;
            let has_blank = matches!(tree.get(check_idx), Some(LineTokenTree::Token(t)) if t.line_type == LineTokenType::BlankLine);
            if has_blank {
                check_idx += 1;
            }

            match tree.get(check_idx) {
                Some(LineTokenTree::Block(block_children)) => {
                    if let LineTokenTree::Token(annotation_token) = &tree[consumed - 1] {
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
                            source,
                        )?;
                        return Ok(Some((item, consumed)));
                    }
                }
                Some(LineTokenTree::Token(annotation_token))
                    if annotation_token.line_type == LineTokenType::AnnotationLine =>
                {
                    let item = super::unwrapper::unwrap_foreign_block(
                        subject_token,
                        vec![],
                        annotation_token,
                        source,
                    )?;
                    return Ok(Some((item, consumed)));
                }
                _ => {}
            }
        }
    }
    Ok(None)
}

/// Try to match a session pattern
/// Pattern: [BLANK_LINE?] <ANY_LINE> BLANK_LINE BLOCK
///
/// This function uses try_session_from_tree which validates and fully consumes the entire pattern.
/// The grammar rule is self-contained and returns the total consumed count.
/// We extract the content and return the full consumed count (including leading blank, lead, blank after, and block).
fn try_match_session(
    tree: &[LineTokenTree],
    grammar: &TxxtGrammarRules,
    source: &str,
) -> Result<Option<(ContentItem, usize)>, String> {
    if let Some(total_consumed) = grammar.try_session_from_tree(tree) {
        // Grammar has validated the entire pattern: [blank?] lead blank block
        // Now extract content from the identified positions

        let mut lead_tree_idx = 0;

        // Check if there's a leading blank line at position 0
        if matches!(tree.first(), Some(LineTokenTree::Token(t)) if t.line_type == LineTokenType::BlankLine)
        {
            lead_tree_idx = 1;
        }

        // lead is at lead_tree_idx
        // blank is at lead_tree_idx + 1
        // block is at lead_tree_idx + 2

        if let Some(LineTokenTree::Token(lead_token)) = tree.get(lead_tree_idx) {
            if let Some(LineTokenTree::Block(block_children)) = tree.get(lead_tree_idx + 2) {
                let block_content = walk_and_parse(block_children, source)?;
                let item = super::unwrapper::unwrap_session(lead_token, block_content, source)?;

                // Return the total consumed count from grammar (includes everything)
                return Ok(Some((item, total_consumed)));
            }
        }
    }
    Ok(None)
}

/// Try to match a definition pattern
/// Pattern: <ANY_LINE> NO-BLANK BLOCK
fn try_match_definition(
    tree: &[LineTokenTree],
    token_types: &[LineTokenType],
    grammar: &TxxtGrammarRules,
    source: &str,
    has_following_block: bool,
) -> Result<Option<(ContentItem, usize)>, String> {
    if has_following_block && grammar.try_definition(token_types).is_some() {
        if let LineTokenTree::Token(subject_token) = &tree[0] {
            let block_idx = token_types.len();
            if let LineTokenTree::Block(block_children) = &tree[block_idx] {
                let block_content = walk_and_parse(block_children, source)?;
                let item =
                    super::unwrapper::unwrap_definition(subject_token, block_content, source)?;
                return Ok(Some((item, block_idx + 1)));
            }
        }
    }
    Ok(None)
}

/// Try to match a list pattern
/// Pattern: LIST_LINE (BLANK_LINE? BLOCK)? LIST_LINE (BLANK_LINE? BLOCK)? ... (2+ items)
fn try_match_list(
    tree: &[LineTokenTree],
    grammar: &TxxtGrammarRules,
    source: &str,
) -> Result<Option<(ContentItem, usize)>, String> {
    if let Some(consumed) = grammar.try_list_from_tree(tree) {
        let mut list_items = Vec::new();
        let mut tree_idx = 0;

        while tree_idx < consumed {
            if let LineTokenTree::Token(item_token) = &tree[tree_idx] {
                if item_token.line_type == LineTokenType::ListLine {
                    tree_idx += 1;

                    // Check if there's a blank line before the content block
                    let mut nested_content = vec![];

                    if let Some(LineTokenTree::Token(blank_token)) = tree.get(tree_idx) {
                        if blank_token.line_type == LineTokenType::BlankLine {
                            // Create BlankLineGroup for the blank line before content
                            let location = if let Some(span) = &blank_token.source_span {
                                let start_line = source[..span.start].matches('\n').count();
                                let end_line = source[..span.end].matches('\n').count();
                                Location {
                                    start: Position {
                                        line: start_line,
                                        column: 0,
                                    },
                                    end: Position {
                                        line: end_line + 1,
                                        column: 0,
                                    },
                                }
                            } else {
                                Location::new(Position::new(0, 0), Position::new(0, 0))
                            };

                            let blank_group =
                                BlankLineGroup::new(1, blank_token.source_tokens.clone())
                                    .at(location);
                            nested_content.push(ContentItem::BlankLineGroup(blank_group));
                            tree_idx += 1;
                        }
                    }

                    if let Some(LineTokenTree::Block(block_children)) = tree.get(tree_idx) {
                        tree_idx += 1;
                        let block_content = walk_and_parse(block_children, source)?;
                        nested_content.extend(block_content);
                    }

                    let item =
                        super::unwrapper::unwrap_list_item(item_token, nested_content, source)?;
                    list_items.push(item);
                    continue;
                }
            }
            tree_idx += 1;
        }

        if list_items.len() >= 2 {
            let list = super::unwrapper::unwrap_list(list_items, source)?;
            return Ok(Some((list, consumed)));
        }
    }
    Ok(None)
}

/// Try to match a paragraph pattern
/// Pattern: any-line+ (all non-blank lines not matching above patterns)
fn try_match_paragraph(
    tree: &[LineTokenTree],
    token_types: &[LineTokenType],
    grammar: &TxxtGrammarRules,
    source: &str,
) -> Result<Option<(ContentItem, usize)>, String> {
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
            return Ok(Some((item, consumed)));
        }
    }
    Ok(None)
}

/// Parse a single node or pattern starting at the current position in the tree.
///
/// Tries patterns in order of specificity using a matcher loop for better maintainability.
fn parse_node_at_level(
    tree: &[LineTokenTree],
    token_types: &[LineTokenType],
    grammar: &TxxtGrammarRules,
    source: &str,
) -> Result<(ContentItem, usize), String> {
    if tree.is_empty() {
        return Err("Empty tree at node level".to_string());
    }

    // Special case: BlankLine tokens are handled separately by walk_and_parse
    // They'll be converted to BlankLineGroup nodes there, so we return a dummy Paragraph
    // that walk_and_parse will recognize and convert.
    if let LineTokenTree::Token(token) = &tree[0] {
        if token.line_type == LineTokenType::BlankLine {
            // Create a minimal paragraph that walk_and_parse will recognize and convert
            let paragraph = Paragraph {
                lines: vec![],
                location: Location::new(Position::new(0, 0), Position::new(0, 0)),
            };
            return Ok((ContentItem::Paragraph(paragraph), 1));
        }
    }

    // Check if a Block follows the current tokens (implicit INDENT)
    let has_following_block = token_types.len() < tree.len()
        && matches!(tree.get(token_types.len()), Some(LineTokenTree::Block(_)));

    // PATTERN MATCHING ORDER (based on blank line context and specificity)
    // Annotation → Foreign Block → Session → Definition → List → Paragraph
    //
    // Key reasons for this order:
    // - Annotations are standalone (::), detect first
    // - Foreign blocks have unambiguous pattern (subject→block→annotation)
    // - Sessions: BLANK-before + any-lead + BLANK-after + block (requires blanks around lead!)
    // - Definitions: any-lead + NO-blank + block (no breathing room)
    // - Lists: seq-marker + (blank?+block)* + seq-marker (requires 2+ items)
    // - Paragraphs are the fallback

    // Try matchers in order of specificity (data-driven pattern matching)
    // Each matcher returns Some((item, consumed)) if it matches, None if it doesn't

    // 1. Try block annotation (most specific annotation form)
    if let Some((item, consumed)) = try_match_block_annotation(tree, grammar, source)? {
        return Ok((item, consumed));
    }

    // 2. Try single-line annotation
    if let Some((item, consumed)) = try_match_single_annotation(tree, token_types, source)? {
        return Ok((item, consumed));
    }

    // 3. Try foreign block (both block and marker forms)
    if let Some((item, consumed)) = try_match_foreign_block(tree, grammar, source)? {
        return Ok((item, consumed));
    }

    // 4. Try session (requires specific blank-line signature)
    if let Some((item, consumed)) = try_match_session(tree, grammar, source)? {
        return Ok((item, consumed));
    }

    // 5. Try definition (before list, different blank-line semantics)
    if let Some((item, consumed)) =
        try_match_definition(tree, token_types, grammar, source, has_following_block)?
    {
        return Ok((item, consumed));
    }

    // 6. Try list (requires 2+ items)
    if let Some((item, consumed)) = try_match_list(tree, grammar, source)? {
        return Ok((item, consumed));
    }

    // 7. Try paragraph (fallback for any non-matching lines)
    if let Some((item, consumed)) = try_match_paragraph(tree, token_types, grammar, source)? {
        return Ok((item, consumed));
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
    use crate::txxt::lexers::transformations::_lex;

    #[test]
    fn test_parse_simple_paragraphs() {
        // Use tokens from the linebased lexer pipeline (returns token tree directly)
        let source = "Simple paragraph\n";
        let tree = _lex(source).expect("Failed to tokenize");

        let result = parse_experimental(tree, source);
        assert!(result.is_ok(), "Parser should succeed");

        let doc = result.unwrap();
        // Should have 1 paragraph with 1 line
        assert!(!doc.root.content.is_empty(), "Should have content");
        assert!(matches!(doc.root.content[0], ContentItem::Paragraph(_)));
    }

    #[test]
    fn test_parse_definition() {
        // Use tokens from the linebased lexer pipeline
        let source = "Definition:\n    This is the definition content\n";
        let tree = _lex(source).expect("Failed to tokenize");

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
        // Use tokens from the linebased lexer pipeline
        let source = "Session:\n\n    Session content here\n";
        let tree = _lex(source).expect("Failed to tokenize");

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
        // Use tokens from the linebased lexer pipeline
        let source = ":: note ::\n";
        let tree = _lex(source).expect("Failed to tokenize");

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

    #[test]
    fn test_annotations_120_simple() {
        let source = std::fs::read_to_string("docs/specs/v1/samples/120-annotations-simple.txxt")
            .expect("Could not read 120 sample");
        let tree = _lex(&source).expect("Failed to tokenize");
        let doc = parse_experimental(tree, &source).expect("Parser failed");

        eprintln!("\n=== 120 ANNOTATIONS SIMPLE ===");
        eprintln!("Root items count: {}", doc.root.content.len());
        for (i, item) in doc.root.content.iter().enumerate() {
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
                    eprintln!("  [{}] Session: {} items", i, s.content.len())
                }
                ContentItem::List(l) => eprintln!("  [{}] List: {} items", i, l.content.len()),
                _ => eprintln!("  [{}] Other", i),
            }
        }

        // Verify we have paragraphs and annotations
        let has_annotations = doc
            .root
            .content
            .iter()
            .any(|item| matches!(item, ContentItem::Annotation(_)));
        let has_paragraphs = doc
            .root
            .content
            .iter()
            .any(|item| matches!(item, ContentItem::Paragraph(_)));

        assert!(has_annotations, "Should contain Annotation nodes");
        assert!(has_paragraphs, "Should contain Paragraph nodes");
    }

    #[test]
    fn test_annotations_130_block_content() {
        let source =
            std::fs::read_to_string("docs/specs/v1/samples/130-annotations-block-content.txxt")
                .expect("Could not read 130 sample");
        let tree = _lex(&source).expect("Failed to tokenize");
        let doc = parse_experimental(tree, &source).expect("Parser failed");

        eprintln!("\n=== 130 ANNOTATIONS BLOCK CONTENT ===");
        eprintln!("Root items count: {}", doc.root.content.len());
        for (i, item) in doc.root.content.iter().enumerate() {
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
                        a.content.len()
                    )
                }
                ContentItem::Session(s) => {
                    eprintln!("  [{}] Session: {} items", i, s.content.len())
                }
                ContentItem::List(l) => eprintln!("  [{}] List: {} items", i, l.content.len()),
                _ => eprintln!("  [{}] Other", i),
            }
        }

        // Verify we have annotations with block content
        let annotations_with_content = doc
            .root
            .content
            .iter()
            .filter_map(|item| match item {
                ContentItem::Annotation(a) => Some(a),
                _ => None,
            })
            .filter(|a| !a.content.is_empty())
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

        let tree = _lex(source).expect("Failed to tokenize");
        let doc = parse_experimental(tree, source).expect("Parser failed");

        eprintln!("\n=== ANNOTATIONS + TRIFECTA COMBINED ===");
        eprintln!("Root items count: {}", doc.root.content.len());
        for (i, item) in doc.root.content.iter().enumerate() {
            match item {
                ContentItem::Paragraph(p) => {
                    eprintln!("  [{}] Paragraph: {} lines", i, p.lines.len())
                }
                ContentItem::Annotation(a) => {
                    eprintln!(
                        "  [{}] Annotation: label='{}' content={} items",
                        i,
                        a.label.value,
                        a.content.len()
                    )
                }
                ContentItem::Session(s) => {
                    eprintln!("  [{}] Session: {} items", i, s.content.len())
                }
                ContentItem::List(l) => eprintln!("  [{}] List: {} items", i, l.content.len()),
                _ => eprintln!("  [{}] Other", i),
            }
        }

        // Verify mixed content
        let has_annotations = doc
            .root
            .content
            .iter()
            .any(|item| matches!(item, ContentItem::Annotation(_)));
        let has_paragraphs = doc
            .root
            .content
            .iter()
            .any(|item| matches!(item, ContentItem::Paragraph(_)));
        let has_sessions = doc
            .root
            .content
            .iter()
            .any(|item| matches!(item, ContentItem::Session(_)));

        assert!(has_annotations, "Should contain annotations");
        assert!(has_paragraphs, "Should contain paragraphs");
        assert!(has_sessions, "Should contain sessions");
    }
}
