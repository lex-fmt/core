//! Parser implementation for the txxt format using chumsky
//!
//! This module implements a parser combinator-based parser for txxt documents.
//! It builds on the token stream from the lexer and produces an AST.
//!
//! ## Testing
//!
//! All parser tests must follow strict guidelines. See the [testing module](crate::txxt_nano::testing)
//! for comprehensive documentation on using verified txxt sources and AST assertions.

use chumsky::prelude::*;
use std::ops::Range;

use super::ast::{
    Annotation, ContentItem, Definition, Document, ForeignBlock, Label, List, ListItem, Paragraph,
    Session,
};
use super::labels::parse_label_from_tokens;
use super::parameters::{convert_parameter, parse_parameters_from_tokens, ParameterWithSpans};
use crate::txxt_nano::lexer::Token;

/// Type alias for token with span
type TokenSpan = (Token, Range<usize>);

/// Type alias for parser error
type ParserError = Simple<TokenSpan>;

/// Intermediate AST structures that hold spans instead of extracted text
/// These are converted to final AST structures after parsing completes

#[derive(Debug, Clone)]
#[allow(dead_code)] // Used internally in parser, may not be directly constructed elsewhere
pub(crate) struct ParagraphWithSpans {
    line_spans: Vec<Vec<Range<usize>>>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct SessionWithSpans {
    title_spans: Vec<Range<usize>>,
    content: Vec<ContentItemWithSpans>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct DefinitionWithSpans {
    subject_spans: Vec<Range<usize>>,
    content: Vec<ContentItemWithSpans>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct ForeignBlockWithSpans {
    subject_spans: Vec<Range<usize>>,
    content_spans: Option<Vec<Range<usize>>>,
    closing_annotation: AnnotationWithSpans,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct AnnotationWithSpans {
    label_span: Option<Range<usize>>, // Optional: can have label, params, or both
    parameters: Vec<ParameterWithSpans>,
    content: Vec<ContentItemWithSpans>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct ListItemWithSpans {
    text_spans: Vec<Range<usize>>,
    content: Vec<ContentItemWithSpans>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct ListWithSpans {
    items: Vec<ListItemWithSpans>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) enum ContentItemWithSpans {
    Paragraph(ParagraphWithSpans),
    Session(SessionWithSpans),
    List(ListWithSpans),
    Definition(DefinitionWithSpans),
    Annotation(AnnotationWithSpans),
    ForeignBlock(ForeignBlockWithSpans),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct DocumentWithSpans {
    metadata: Vec<AnnotationWithSpans>,
    content: Vec<ContentItemWithSpans>,
}

/// Helper to extract text from source using a span
#[allow(dead_code)] // Reserved for future use
fn extract_text(source: &str, span: &Range<usize>) -> String {
    if span.start >= span.end || span.end > source.len() {
        // Empty or synthetic span (like for IndentLevel/DedentLevel)
        return String::new();
    }
    source[span.start..span.end].to_string()
}

/// Helper to extract and concatenate text from multiple spans
fn extract_line_text(source: &str, spans: &[Range<usize>]) -> String {
    if spans.is_empty() {
        return String::new();
    }

    // Find the overall span from first to last
    let start = spans.first().map(|s| s.start).unwrap_or(0);
    let end = spans.last().map(|s| s.end).unwrap_or(0);

    if start >= end || end > source.len() {
        return String::new();
    }

    source[start..end].trim().to_string()
}

/// Convert intermediate AST with spans to final AST with extracted text
fn convert_document(source: &str, doc_with_spans: DocumentWithSpans) -> Document {
    Document {
        metadata: doc_with_spans
            .metadata
            .into_iter()
            .map(|ann| convert_annotation(source, ann))
            .collect(),
        content: doc_with_spans
            .content
            .into_iter()
            .map(|item| convert_content_item(source, item))
            .collect(),
    }
}

fn convert_content_item(source: &str, item: ContentItemWithSpans) -> ContentItem {
    match item {
        ContentItemWithSpans::Paragraph(p) => ContentItem::Paragraph(convert_paragraph(source, p)),
        ContentItemWithSpans::Session(s) => ContentItem::Session(convert_session(source, s)),
        ContentItemWithSpans::List(l) => ContentItem::List(convert_list(source, l)),
        ContentItemWithSpans::Definition(d) => {
            ContentItem::Definition(convert_definition(source, d))
        }
        ContentItemWithSpans::Annotation(a) => {
            ContentItem::Annotation(convert_annotation(source, a))
        }
        ContentItemWithSpans::ForeignBlock(fb) => {
            ContentItem::ForeignBlock(convert_foreign_block(source, fb))
        }
    }
}

fn convert_paragraph(source: &str, para: ParagraphWithSpans) -> Paragraph {
    Paragraph {
        lines: para
            .line_spans
            .iter()
            .map(|spans| extract_line_text(source, spans))
            .collect(),
    }
}

fn convert_session(source: &str, sess: SessionWithSpans) -> Session {
    Session {
        title: extract_line_text(source, &sess.title_spans),
        content: sess
            .content
            .into_iter()
            .map(|item| convert_content_item(source, item))
            .collect(),
    }
}

fn convert_definition(source: &str, def: DefinitionWithSpans) -> Definition {
    // Extract subject (colon already excluded from spans by definition_subject parser)
    let subject = extract_line_text(source, &def.subject_spans);

    Definition {
        subject,
        content: def
            .content
            .into_iter()
            .map(|item| convert_content_item(source, item))
            .collect(),
    }
}

fn convert_annotation(source: &str, ann: AnnotationWithSpans) -> Annotation {
    // Extract label if present, otherwise use empty string
    let label_text = ann
        .label_span
        .as_ref()
        .map(|span| extract_text(source, span).trim().to_string())
        .unwrap_or_default();
    let label = Label::new(label_text);

    let parameters = ann
        .parameters
        .into_iter()
        .map(|param| convert_parameter(source, param))
        .collect();

    let content = ann
        .content
        .into_iter()
        .map(|item| convert_content_item(source, item))
        .collect();

    Annotation {
        label,
        parameters,
        content,
    }
}

fn convert_list(source: &str, list: ListWithSpans) -> List {
    List {
        items: list
            .items
            .into_iter()
            .map(|item| convert_list_item(source, item))
            .collect(),
    }
}

fn convert_list_item(source: &str, item: ListItemWithSpans) -> ListItem {
    ListItem::with_content(
        extract_line_text(source, &item.text_spans),
        item.content
            .into_iter()
            .map(|content_item| convert_content_item(source, content_item))
            .collect(),
    )
}

fn convert_foreign_block(source: &str, fb: ForeignBlockWithSpans) -> ForeignBlock {
    let subject = extract_line_text(source, &fb.subject_spans);
    let content = fb
        .content_spans
        .map(|spans| reconstruct_raw_content(source, &spans))
        .unwrap_or_default();
    let closing_annotation = convert_annotation(source, fb.closing_annotation);

    ForeignBlock::new(subject, content, closing_annotation)
}

/// Parse a text line (sequence of text and whitespace tokens)
/// Returns the collected spans for this line
fn text_line() -> impl Parser<TokenSpan, Vec<Range<usize>>, Error = ParserError> + Clone {
    filter(|(t, _span): &TokenSpan| {
        matches!(
            t,
            Token::Text
                | Token::Whitespace
                | Token::Number
                | Token::Dash
                | Token::Period
                | Token::OpenParen
                | Token::CloseParen
                | Token::Colon
                | Token::Comma
                | Token::Quote
                | Token::Equals
        )
    })
    .repeated()
    .at_least(1)
    .map(|tokens_with_spans: Vec<TokenSpan>| {
        // Collect all spans for this line
        tokens_with_spans.into_iter().map(|(_, s)| s).collect()
    })
}

/// Helper: match a specific token type, ignoring the span
fn token(t: Token) -> impl Parser<TokenSpan, (), Error = ParserError> + Clone {
    filter(move |(tok, _)| tok == &t).ignored()
}

/// Parse a list item line - a line that starts with a list marker
/// Grammar: <list-item-line> = <plain-marker> <text>+ | <ordered-marker> <text>+
/// Where: <plain-marker> = "-" " "
///        <ordered-marker> = (<number> | <letter> | <roman>) ("|"|")") " "
fn list_item_line() -> impl Parser<TokenSpan, Vec<Range<usize>>, Error = ParserError> + Clone {
    // Just check that the line starts with a valid list marker, then collect all tokens
    // We validate the marker and collect the full line content
    let rest_of_line = filter(|(t, _span): &TokenSpan| {
        matches!(
            t,
            Token::Text
                | Token::Whitespace
                | Token::Number
                | Token::Dash
                | Token::Period
                | Token::OpenParen
                | Token::CloseParen
                | Token::Colon
                | Token::Comma
                | Token::Quote
                | Token::Equals
        )
    })
    .repeated();

    // Pattern 1: Dash + whitespace + rest
    let dash_pattern = filter(|(t, _): &TokenSpan| matches!(t, Token::Dash))
        .then(filter(|(t, _): &TokenSpan| matches!(t, Token::Whitespace)))
        .chain(rest_of_line);

    // Pattern 2: Number/Text + Period/CloseParen + whitespace + rest
    let ordered_pattern = filter(|(t, _): &TokenSpan| matches!(t, Token::Number | Token::Text))
        .then(filter(|(t, _): &TokenSpan| {
            matches!(t, Token::Period | Token::CloseParen)
        }))
        .then(filter(|(t, _): &TokenSpan| matches!(t, Token::Whitespace)))
        .chain(rest_of_line);

    // Pattern 3: OpenParen + Number + CloseParen + whitespace + rest
    let paren_pattern = filter(|(t, _): &TokenSpan| matches!(t, Token::OpenParen))
        .then(filter(|(t, _): &TokenSpan| matches!(t, Token::Number)))
        .then(filter(|(t, _): &TokenSpan| matches!(t, Token::CloseParen)))
        .then(filter(|(t, _): &TokenSpan| matches!(t, Token::Whitespace)))
        .chain(rest_of_line);

    // Try each pattern and collect all spans
    dash_pattern
        .or(ordered_pattern)
        .or(paren_pattern)
        .map(|tokens_with_spans: Vec<TokenSpan>| {
            tokens_with_spans.into_iter().map(|(_, s)| s).collect()
        })
}

/// Container types that define what content elements are allowed
///
/// NOTE: Container restrictions are NOT currently enforced - all containers
/// allow all element types in the unified recursive parser. This enum and
/// associated logic in build_content_parser() are kept for future enhancement
/// but are not active in the current implementation.
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)] // Not currently used - kept for future container restriction implementation
enum ContainerType {
    /// Session containers - can contain everything (sessions, definitions, lists, paragraphs, annotations, foreign blocks)
    /// (Currently: no restrictions enforced)
    Session,
    /// Content containers - used by definitions and list items (cannot contain sessions)
    /// (Currently: no restrictions enforced, sessions are allowed)
    Content,
    /// Annotation containers - used by annotation block content (cannot contain sessions or nested annotations)
    /// (Currently: no restrictions enforced, all elements allowed)
    Annotation,
}

/// Build the unified content parser that implements the canonical parse order.
///
/// Parse a single list item with optional indented content - ACCEPTS content_parser parameter
/// This is the new parameterized version that doesn't have its own recursive() block.
/// The content_parser is provided from above (from build_content_parser) and defines what
/// elements are allowed inside this list item.
///
/// Grammar: <list-item> = <list-item-line> (<indent> <list-item-content>+ <dedent>)?
#[allow(dead_code)] // Will be used in Phase 4.5 when we update build_content_parser
fn list_item_with_content(
    content_parser: impl Parser<TokenSpan, ContentItemWithSpans, Error = ParserError> + Clone + 'static,
) -> impl Parser<TokenSpan, ListItemWithSpans, Error = ParserError> + Clone {
    // Parse the list item line, then optionally parse indented content
    list_item_line()
        .then_ignore(token(Token::Newline))
        .then(
            // Optional indented block - uses the provided content_parser
            token(Token::IndentLevel)
                .ignore_then(content_parser.repeated().at_least(1))
                .then_ignore(token(Token::DedentLevel))
                .or_not(),
        )
        .map(|(text_spans, maybe_content)| ListItemWithSpans {
            text_spans,
            content: maybe_content.unwrap_or_default(),
        })
}

/// Parse a single list item with optional indented content - OLD VERSION (temporary wrapper)
/// This is kept temporarily for compatibility. It uses the old recursive approach.
/// Will be removed once build_content_parser is updated to use list_item_with_content.
///
/// Grammar: <list-item> = <list-item-line> (<indent> <list-item-content>+ <dedent>)?
#[allow(dead_code)] // To be removed in next step
fn list_item() -> impl Parser<TokenSpan, ListItemWithSpans, Error = ParserError> + Clone {
    recursive(|list_item_parser| {
        // Define nested list parser that uses the recursive list_item_parser
        let nested_list = list_item_parser
            .clone()
            .repeated()
            .at_least(2) // Lists require at least 2 items
            .then_ignore(token(Token::Newline).or_not()) // Optional blank line at end
            .map(|items| ListWithSpans { items })
            .map(ContentItemWithSpans::List);

        // Content inside list items: paragraphs and nested lists (NO sessions)
        let list_item_content = nested_list.or(paragraph().map(ContentItemWithSpans::Paragraph));

        // Parse the list item line, then optionally parse indented content
        list_item_line()
            .then_ignore(token(Token::Newline))
            .then(
                // Optional indented block
                token(Token::IndentLevel)
                    .ignore_then(
                        // Use recursive pattern similar to main document parser
                        recursive(|content_items| {
                            choice((
                                // Skip any leading blank lines, then check what comes next
                                token(Token::Newline)
                                    .repeated()
                                    .at_least(1)
                                    .ignore_then(choice((
                                        // After blank lines, check for DedentLevel (end of content)
                                        filter(|(t, _)| matches!(t, Token::DedentLevel))
                                            .rewind()
                                            .to(vec![]),
                                        // Or continue with more content
                                        content_items.clone(),
                                    ))),
                                // Parse content item and continue
                                list_item_content.clone().then(content_items.or_not()).map(
                                    |(first, rest)| {
                                        let mut result = vec![first];
                                        if let Some(mut rest_items) = rest {
                                            result.append(&mut rest_items);
                                        }
                                        result
                                    },
                                ),
                                // Base case: at DedentLevel
                                filter(|(t, _)| matches!(t, Token::DedentLevel))
                                    .rewind()
                                    .to(vec![]),
                            ))
                        }),
                    )
                    .then_ignore(token(Token::DedentLevel))
                    .or_not(),
            )
            .map(|(text_spans, maybe_content)| ListItemWithSpans {
                text_spans,
                content: maybe_content.unwrap_or_default(),
            })
    })
}

/// Parse a paragraph - one or more lines of text separated by newlines, ending with a blank line
/// A paragraph is a catch-all that matches when nothing else does.
///
/// Simplified rule: Paragraphs can contain ANYTHING (including single list-item-lines).
/// Lists require a blank line before them, so disambiguation is handled by parse order:
/// 1. Try list first (needs blank line + 2+ items)
/// 2. Try session (needs title + blank + indent)
/// 3. Try paragraph (catches everything else)
fn paragraph() -> impl Parser<TokenSpan, ParagraphWithSpans, Error = ParserError> + Clone {
    // Parse a paragraph - consecutive non-blank text lines
    // Simplified to work in both document and recursive contexts

    // A paragraph is one or more text lines followed by newlines
    // We stop at blank lines (double newlines) which separate paragraphs
    text_line()
        .then_ignore(token(Token::Newline))
        .repeated()
        .at_least(1)
        // Don't consume trailing blank lines - they're element boundaries!
        .map(|line_spans| ParagraphWithSpans { line_spans })
}

/// Parse a paragraph for use in recursive contexts (simplified)
#[allow(dead_code)] // Kept for future improvements
fn paragraph_recursive() -> impl Parser<TokenSpan, ParagraphWithSpans, Error = ParserError> + Clone
{
    // In recursive contexts, collect consecutive non-blank text lines
    // A blank line (double newline) ends the paragraph

    // Parse lines that are NOT followed by blank lines
    let paragraph_line = text_line().then_ignore(token(Token::Newline));

    // Collect consecutive lines, stopping at a blank line
    paragraph_line
        .then_ignore(
            // Continue only if NOT followed by another newline (which would be a blank line)
            token(Token::Newline).not().rewind(),
        )
        .repeated()
        .then(
            // Last line might not have continuation check
            text_line().then_ignore(token(Token::Newline)).or_not(),
        )
        .map(|(mut lines, last)| {
            if let Some(last_line) = last {
                lines.push(last_line);
            }
            ParagraphWithSpans { line_spans: lines }
        })
        .then_ignore(token(Token::Newline).or_not()) // Consume trailing blank line if present
}

/// Parse a definition subject - a line of text ending with colon, followed immediately by newline (no blank line)
/// The key difference from session_title is the absence of a blank line before indented content
fn definition_subject() -> impl Parser<TokenSpan, Vec<Range<usize>>, Error = ParserError> + Clone {
    // Parse text tokens before the colon (explicitly excluding colon from subject spans)
    filter(|(t, _span): &TokenSpan| !matches!(t, Token::Colon | Token::Newline))
        .repeated()
        .at_least(1)
        .map(|tokens_with_spans: Vec<TokenSpan>| {
            // Collect spans for the subject text (without colon)
            tokens_with_spans.into_iter().map(|(_, s)| s).collect()
        })
        // Explicitly consume the colon and newline
        .then_ignore(token(Token::Colon))
        .then_ignore(token(Token::Newline))
}

/// Parse a session title - a line of text followed by a newline and blank line
fn session_title() -> impl Parser<TokenSpan, Vec<Range<usize>>, Error = ParserError> + Clone {
    text_line()
        .then_ignore(token(Token::Newline))
        .then_ignore(token(Token::Newline))
}

/// Parse the bounded region between :: markers
/// Format: :: <label>? <params>? ::
///
/// Strategy: Collect all tokens between :: markers, then parse them to determine:
/// - If first word has no '=' after it → it's a label
/// - Everything else → parameters (comma-separated key=value pairs)
fn annotation_header(
) -> impl Parser<TokenSpan, (Option<Range<usize>>, Vec<ParameterWithSpans>), Error = ParserError> + Clone
{
    // Collect all tokens between opening :: and closing ::
    let bounded_region =
        filter(|(t, _): &TokenSpan| !matches!(t, Token::TxxtMarker | Token::Newline))
            .repeated()
            .at_least(1);

    bounded_region.validate(|tokens, span, emit| {
        if tokens.is_empty() {
            emit(ParserError::expected_input_found(span, None, None));
            return (None, Vec::new());
        }

        // Parse label from tokens
        let (label_span, mut i) = parse_label_from_tokens(&tokens);

        // If no label was found and i is 0, we need to restart parsing for parameters
        if label_span.is_none() && i == 0 {
            // Reset to start for parameter parsing
            while i < tokens.len() && matches!(tokens[i].0, Token::Whitespace) {
                i += 1;
            }
        }

        // Parse remaining tokens as parameters
        let params = parse_parameters_from_tokens(&tokens[i..]);

        (label_span, params)
    })
}

/// Parse annotation - ACCEPTS content_parser parameter
/// This is the new parameterized version that doesn't have its own recursive() block.
/// The content_parser is provided from above and defines what elements are allowed
/// inside block-form annotations.
///
/// Forms:
/// 1. Marker: `:: label ::\n` - No content
/// 2. Single-line: `:: label :: text\n` - Text after :: captured as paragraph
/// 3. Block: `:: label \n<indent>content<dedent>::` - Uses provided content_parser
#[allow(dead_code)] // Will be used in Phase 4.5 when we update build_content_parser
fn annotation_with_content(
    content_parser: impl Parser<TokenSpan, ContentItemWithSpans, Error = ParserError> + Clone + 'static,
) -> impl Parser<TokenSpan, AnnotationWithSpans, Error = ParserError> + Clone {
    // Parse the header: :: <bounded region> ::
    let header = token(Token::TxxtMarker)
        .ignore_then(annotation_header())
        .then_ignore(token(Token::TxxtMarker));

    // Block form: :: label params :: \n <indent>content<dedent> ::
    // Uses the provided content_parser for block content
    let block_form = header
        .clone()
        .then_ignore(token(Token::Newline))
        .then(
            token(Token::IndentLevel)
                .ignore_then(content_parser.repeated().at_least(1))
                .then_ignore(token(Token::DedentLevel)),
        )
        .then_ignore(token(Token::TxxtMarker)) // Closing :: after content
        .map(|((label_span, parameters), content)| AnnotationWithSpans {
            label_span,
            parameters,
            content,
        })
        .then_ignore(token(Token::Newline).repeated());

    // Single-line and marker forms (unchanged - don't use content_parser)
    let single_line_or_marker = header
        .then(token(Token::Whitespace).ignore_then(text_line()).or_not())
        .map(|((label_span, parameters), content_span)| {
            let content = content_span
                .map(|span| {
                    vec![ContentItemWithSpans::Paragraph(ParagraphWithSpans {
                        line_spans: vec![span],
                    })]
                })
                .unwrap_or_default();

            AnnotationWithSpans {
                label_span,
                parameters,
                content,
            }
        })
        .then_ignore(token(Token::Newline).repeated());

    block_form.or(single_line_or_marker)
}

/// Parse a definition - ACCEPTS content_parser parameter
/// This is the new parameterized version that doesn't have its own recursive() block.
/// The content_parser is provided from above (from build_content_parser) and defines what
/// elements are allowed inside this definition.
///
/// IMPORTANT: NO blank line between subject and indented content (unlike sessions)
/// Grammar: <definition> = <definition-subject> <newline> <indent> <content>+ <dedent>
#[allow(dead_code)] // Will be used in Phase 4.5 when we update build_content_parser
fn definition_with_content(
    content_parser: impl Parser<TokenSpan, ContentItemWithSpans, Error = ParserError> + Clone + 'static,
) -> impl Parser<TokenSpan, DefinitionWithSpans, Error = ParserError> + Clone {
    definition_subject()
        .then(
            // Must immediately see IndentLevel (no blank line)
            token(Token::IndentLevel)
                .ignore_then(content_parser.repeated().at_least(1))
                .then_ignore(token(Token::DedentLevel)),
        )
        .map(|(subject_spans, content)| DefinitionWithSpans {
            subject_spans,
            content,
        })
}

/// Parse a definition - OLD VERSION (temporary wrapper)
/// This is kept temporarily for compatibility. It uses the old recursive approach.
/// Will be removed once build_content_parser is updated to use definition_with_content.
///
/// IMPORTANT: NO blank line between subject and indented content (unlike sessions)
/// Content can include paragraphs and lists, but NOT sessions
#[allow(dead_code)] // To be removed in next step
fn definition() -> impl Parser<TokenSpan, DefinitionWithSpans, Error = ParserError> + Clone {
    // Content parser for definitions - excludes sessions, only paragraphs and lists
    let definition_content = recursive(|_definition_content_parser| {
        // Nested list parser
        let nested_list = list_item()
            .repeated()
            .at_least(2)
            .then_ignore(token(Token::Newline).or_not())
            .map(|items| ListWithSpans { items })
            .map(ContentItemWithSpans::List);

        // Definition content can contain lists and paragraphs (NO sessions)
        nested_list.or(paragraph().map(ContentItemWithSpans::Paragraph))
    });

    definition_subject()
        .then(
            // Must immediately see IndentLevel (no blank line)
            token(Token::IndentLevel)
                .ignore_then(definition_content.repeated().at_least(1))
                .then_ignore(token(Token::DedentLevel)),
        )
        .map(|(subject_spans, content)| DefinitionWithSpans {
            subject_spans,
            content,
        })
}

/// Helper to reconstruct raw content from token spans
fn reconstruct_raw_content(source: &str, spans: &[Range<usize>]) -> String {
    if spans.is_empty() {
        return String::new();
    }
    // Find the overall span from first to last
    let start = spans.first().map(|s| s.start).unwrap_or(0);
    let end = spans.last().map(|s| s.end).unwrap_or(0);

    if start >= end || end > source.len() {
        return String::new();
    }
    source[start..end].to_string()
}

/// Parse a foreign block - subject line, optional content, closing annotation
/// Uses "Indentation Wall" rule: content must be indented deeper than subject,
/// closing annotation must be at same level as subject
///
/// Token sequence with content:
/// definition_subject (which includes Colon and Newline) IndentLevel content... DedentLevel+ :: label :: \n
/// Token sequence without content:
/// definition_subject (which includes Colon and Newline) Newline :: label :: \n
fn foreign_block() -> impl Parser<TokenSpan, ForeignBlockWithSpans, Error = ParserError> + Clone {
    let subject_parser = definition_subject(); // This consumes: text... Colon Newline

    // Content: everything except TxxtMarker (stops naturally when hitting ::)
    let content_token = filter(|(t, _span): &TokenSpan| !matches!(t, Token::TxxtMarker));

    // Content block: IndentLevel + content, returns Vec<Range>
    let with_content = token(Token::IndentLevel)
        .ignore_then(content_token.repeated().at_least(1))
        .map(|tokens: Vec<TokenSpan>| {
            // Remove trailing DedentLevel tokens
            let mut content_tokens = tokens;
            while content_tokens
                .last()
                .map(|(t, _)| matches!(t, Token::DedentLevel))
                .unwrap_or(false)
            {
                content_tokens.pop();
            }
            content_tokens
                .into_iter()
                .map(|(_, s)| s)
                .collect::<Vec<_>>()
        });

    // Parse closing annotation marker (:: label params :: with optional text after for marker form)
    let closing_annotation_parser = token(Token::TxxtMarker)
        .ignore_then(annotation_header())
        .then_ignore(token(Token::TxxtMarker))
        .then(
            // Optional single-line text content after closing :: (for marker form)
            token(Token::Whitespace).ignore_then(text_line()).or_not(),
        )
        .map(|((label_span, parameters), content_span)| {
            // Text after :: becomes paragraph content (annotation single-line form)
            let content = content_span
                .map(|span| {
                    vec![ContentItemWithSpans::Paragraph(ParagraphWithSpans {
                        line_spans: vec![span],
                    })]
                })
                .unwrap_or_default();

            AnnotationWithSpans {
                label_span,
                parameters,
                content,
            }
        });

    subject_parser
        .then_ignore(token(Token::Newline).or_not()) // Consume optional blank line after subject (marker form)
        .then(with_content.or_not()) // Content is optional
        // Don't consume DedentLevel before annotation - content parser handles them
        .then(closing_annotation_parser)
        // Don't consume newlines after annotation - they belong to document-level parsing
        .map(
            |((subject_spans, content_spans), closing_annotation)| ForeignBlockWithSpans {
                subject_spans,
                content_spans,
                closing_annotation,
            },
        )
}

/// Build the Multi-Parser Bundle for document-level content parsing.
///
/// Uses manual recursion to build repetition without .repeated()
/// This avoids the recursive() + .repeated() interaction issue in Chumsky
fn build_document_content_parser(
) -> impl Parser<TokenSpan, Vec<ContentItemWithSpans>, Error = ParserError> + Clone {
    // HACK: Build repetition through recursion itself, not .repeated()
    // This avoids the problematic recursive() + .repeated() pattern

    recursive(|items| {
        // First, define the single item parser (not recursive yet)
        let single_item = {
            // For sessions, we need nested recursion
            let session_parser = session_title()
                .then(
                    token(Token::IndentLevel)
                        .ignore_then(items.clone()) // Sessions contain multiple items recursively
                        .then_ignore(token(Token::DedentLevel)),
                )
                .map(|(title_spans, content)| {
                    ContentItemWithSpans::Session(SessionWithSpans {
                        title_spans,
                        content,
                    })
                });

            // Definitions also need recursion for their content
            let definition_parser = definition_subject()
                .then(
                    token(Token::IndentLevel)
                        .ignore_then(items.clone()) // Definitions contain items recursively
                        .then_ignore(token(Token::DedentLevel)),
                )
                .map(|(subject_spans, content)| {
                    ContentItemWithSpans::Definition(DefinitionWithSpans {
                        subject_spans,
                        content,
                    })
                });

            // Lists - now using unified recursion for full content support
            let list_parser = {
                // Parse a single list item with optional recursive content
                let single_list_item = list_item_line()
                    .then_ignore(token(Token::Newline))
                    .then(
                        // Optional indented block with full recursive content
                        token(Token::IndentLevel)
                            .ignore_then(items.clone()) // List items can contain any element recursively
                            .then_ignore(token(Token::DedentLevel))
                            .or_not(),
                    )
                    .map(|(text_spans, maybe_content)| ListItemWithSpans {
                        text_spans,
                        content: maybe_content.unwrap_or_default(),
                    });

                // Lists require at least 2 items
                single_list_item
                    .repeated()
                    .at_least(2)
                    .then_ignore(token(Token::Newline).or_not())
                    .map(|items| ContentItemWithSpans::List(ListWithSpans { items }))
            };

            // Annotations - now using unified recursion for full content support
            let annotation_parser = {
                // Parse the header: :: <bounded region> ::
                let header = token(Token::TxxtMarker)
                    .ignore_then(annotation_header())
                    .then_ignore(token(Token::TxxtMarker));

                // Block form: :: label params :: \n <indent>content<dedent> ::
                let block_form = header
                    .clone()
                    .then_ignore(token(Token::Newline))
                    .then(
                        token(Token::IndentLevel)
                            .ignore_then(items.clone()) // Annotations can contain any element recursively
                            .then_ignore(token(Token::DedentLevel)),
                    )
                    .then_ignore(token(Token::TxxtMarker)) // Second closing :: after content
                    .map(|((label_span, parameters), content)| AnnotationWithSpans {
                        label_span,
                        parameters,
                        content,
                    })
                    .then_ignore(token(Token::Newline).repeated());

                // Single-line and marker forms
                let single_line_or_marker = header
                    .then(
                        // Optional single-line text content after closing ::
                        token(Token::Whitespace).ignore_then(text_line()).or_not(),
                    )
                    .map(|((label_span, parameters), content_span)| {
                        // Text after :: becomes paragraph content (annotation single-line form)
                        let content = content_span
                            .map(|span| {
                                vec![ContentItemWithSpans::Paragraph(ParagraphWithSpans {
                                    line_spans: vec![span],
                                })]
                            })
                            .unwrap_or_default();

                        AnnotationWithSpans {
                            label_span,
                            parameters,
                            content,
                        }
                    })
                    .then_ignore(token(Token::Newline).repeated());

                block_form
                    .or(single_line_or_marker)
                    .map(ContentItemWithSpans::Annotation)
            };

            // Parse order from docs/parsing.txxt
            choice((
                foreign_block().map(ContentItemWithSpans::ForeignBlock),
                annotation_parser,
                list_parser,
                definition_parser,
                session_parser,
                paragraph().map(ContentItemWithSpans::Paragraph),
            ))
        };

        // Parse content, with optional leading/trailing blank lines
        choice((
            // Skip any leading blank lines, then try to parse item
            token(Token::Newline)
                .repeated()
                .at_least(1)
                .ignore_then(choice((
                    // After blank lines, check for boundary
                    filter(|(t, _)| matches!(t, Token::DocEnd | Token::DedentLevel))
                        .rewind()
                        .to(vec![]),
                    // Or continue with more items
                    items.clone(),
                ))),
            // Parse item without leading blank lines
            single_item
                .then(items.clone().or_not())
                .map(|(first, rest)| {
                    let mut result = vec![first];
                    if let Some(mut rest_items) = rest {
                        result.append(&mut rest_items);
                    }
                    result
                }),
            // Base case: At a boundary (DocEnd or DedentLevel)
            filter(|(t, _)| matches!(t, Token::DocEnd | Token::DedentLevel))
                .rewind()
                .to(vec![]),
        ))
    })
}

/// Parse a document - a sequence of annotations, paragraphs, lists, sessions, and definitions
/// Returns intermediate AST with spans
///
/// Documents are conceptually SessionContainers - they parse content the same way sessions do.
/// The only difference is that documents don't have a title and aren't indented.
#[allow(private_interfaces)] // DocumentWithSpans is internal implementation detail
pub fn document() -> impl Parser<TokenSpan, DocumentWithSpans, Error = ParserError> {
    // Use the Multi-Parser Bundle for document-level content parsing
    // This ensures definitions use content_content parser (which excludes sessions)
    let content_item = build_document_content_parser();

    // Parse document content using the manual recursive approach
    // This returns a Vec<ContentItemWithSpans> directly
    token(Token::DocStart)
        .ignore_then(content_item)
        .then_ignore(token(Token::DocEnd))
        .map(|content| DocumentWithSpans {
            metadata: Vec::new(), // TODO: Parse document-level metadata
            content,
        })
}

/// Parse with source text - extracts actual content from spans
pub fn parse_with_source(
    tokens_with_spans: Vec<TokenSpan>,
    source: &str,
) -> Result<Document, Vec<ParserError>> {
    let doc_with_spans = document().parse(tokens_with_spans)?;
    Ok(convert_document(source, doc_with_spans))
}

/// Parse a txxt document from a token stream (legacy - doesn't preserve source text)
pub fn parse(tokens: Vec<Token>) -> Result<Document, Vec<Simple<Token>>> {
    // Convert tokens to token-span tuples with empty spans
    let tokens_with_spans: Vec<TokenSpan> = tokens.into_iter().map(|t| (t, 0..0)).collect();

    // Parse with empty source
    parse_with_source(tokens_with_spans, "")
        .map_err(|errs| errs.into_iter().map(|e| e.map(|(t, _)| t)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::txxt_nano::lexer::{lex, lex_with_spans};
    use crate::txxt_nano::processor::txxt_sources::TxxtSources;

    #[test]
    fn test_simple_paragraph() {
        let input = "Hello world\n\n";
        let mut tokens_with_spans = lex_with_spans(input);

        // Skip DocStart and DocEnd tokens for direct paragraph test
        tokens_with_spans.retain(|(t, _)| !matches!(t, Token::DocStart | Token::DocEnd));

        let result = paragraph().parse(tokens_with_spans);
        assert!(result.is_ok(), "Failed to parse paragraph: {:?}", result);

        let para_with_spans = result.unwrap();
        assert_eq!(para_with_spans.line_spans.len(), 1);

        // Verify actual content is preserved
        let para = convert_paragraph(input, para_with_spans);
        assert_eq!(para.lines.len(), 1);
        assert_eq!(para.lines[0], "Hello world");
    }

    #[test]
    fn test_real_content_extraction() {
        use crate::txxt_nano::testing::assert_ast;

        // Test that we extract real content, not placeholder strings
        let input = "First paragraph with numbers 123 and symbols (like this).\n\nSecond paragraph.\n\n1. Session Title\n\n    Session content here.\n\n";

        let doc = crate::txxt_nano::parser::parse_document(input).expect("Failed to parse");

        assert_ast(&doc)
            .item_count(3)
            .item(0, |item| {
                item.assert_paragraph()
                    .text("First paragraph with numbers 123 and symbols (like this).")
                    .line_count(1);
            })
            .item(1, |item| {
                item.assert_paragraph()
                    .text("Second paragraph.")
                    .line_count(1);
            })
            .item(2, |item| {
                item.assert_session()
                    .label("1. Session Title")
                    .child_count(1)
                    .child(0, |child| {
                        child
                            .assert_paragraph()
                            .text("Session content here.")
                            .line_count(1);
                    });
            });
    }

    #[test]
    fn test_malformed_session_title_with_indent_but_no_content() {
        // Test the exact scenario from the code review:
        // A text line followed by blank line and IndentLevel, but no actual parseable content
        // Session parser should fail (expects content after IndentLevel)
        // Then paragraph parser tries and consumes the text line
        // This leaves IndentLevel token unconsumed, causing confusing error

        // We need actual indented content to get an IndentLevel token
        // So let's use a session title followed by just a newline at the indent level
        let input = "This looks like a session title\n\n    \n"; // Title + blank + indented newline
        let tokens = lex(input);

        println!("\n=== Test: Session title pattern with IndentLevel but no parseable content ===");
        println!("Input: {:?}", input);
        println!("Tokens: {:?}", tokens);

        let result = parse(tokens.clone());

        match &result {
            Ok(doc) => {
                println!("\n✓ Parsed successfully");
                println!("Document has {} items:", doc.content.len());
                for (i, item) in doc.content.iter().enumerate() {
                    println!("  {}: {}", i, item);
                }
                // This might actually be fine - the blank indented line might be ignored
            }
            Err(errors) => {
                println!("\n✗ Parse failed with errors:");
                for error in errors {
                    println!("  Error at span {:?}: {:?}", error.span(), error.reason());
                    println!("  Found: {:?}", error.found());
                }

                // This is expected to fail, but the question is:
                // Does it fail with a GOOD error message or a CONFUSING one?

                // If paragraph parser consumed the title line, the error will be about
                // finding IndentLevel when it expected something else (paragraph content or end)
            }
        }
    }

    #[test]
    fn test_session_title_followed_by_bare_indent_level() {
        // Test case 1: Session with empty content (IndentLevel immediately followed by DedentLevel)
        // This actually SHOULD be allowed or give a clear error
        let tokens = vec![
            Token::Text,
            Token::Newline,
            Token::Newline,
            Token::IndentLevel,
            Token::DedentLevel,
            Token::DedentLevel,
        ];

        println!("\n=== Test: Session with empty content ===");
        println!("Tokens: {:?}", tokens);

        let result = parse(tokens.clone());

        match &result {
            Ok(doc) => {
                println!("\n✓ Parsed as session with 0 children");
                println!("Document has {} items:", doc.content.len());
                for (i, item) in doc.content.iter().enumerate() {
                    match item {
                        ContentItem::Paragraph(p) => {
                            println!("  {}: Paragraph with {} lines", i, p.lines.len());
                        }
                        ContentItem::Session(s) => {
                            println!(
                                "  {}: Session '{}' with {} children",
                                i,
                                s.title,
                                s.content.len()
                            );
                        }
                        ContentItem::List(l) => {
                            println!("  {}: List with {} items", i, l.items.len());
                        }
                        ContentItem::Definition(d) => {
                            println!(
                                "  {}: Definition '{}' with {} children",
                                i,
                                d.subject,
                                d.content.len()
                            );
                        }
                        ContentItem::Annotation(a) => {
                            println!(
                                "  {}: Annotation '{}' with {} children",
                                i,
                                a.label.value,
                                a.content.len()
                            );
                        }
                        ContentItem::ForeignBlock(fb) => {
                            println!(
                                "  {}: ForeignBlock '{}' with {} chars, closing: {}",
                                i,
                                fb.subject,
                                fb.content.len(),
                                fb.closing_annotation.label.value
                            );
                        }
                    }
                }
                // This is actually fine - empty session
            }
            Err(errors) => {
                println!("\n✗ Parse failed:");
                for error in errors {
                    println!("  Error at span {:?}: {:?}", error.span(), error.reason());
                }
            }
        }
    }

    #[test]
    fn test_greedy_paragraph_parser_bug() {
        // THIS is the greedy paragraph bug from the code review:
        // Text Newline Newline IndentLevel [something that's not a valid content item]
        //
        // When session parser fails to parse content after IndentLevel,
        // it backtracks and paragraph parser gets a chance
        // Paragraph parser matches "Text Newline" leaving "Newline IndentLevel ..."
        // This causes a confusing error
        //
        // With the fix: Paragraph parser uses `.not()` to reject patterns followed by IndentLevel
        // So it won't consume the session title, and the error will be about the malformed session

        let tokens = vec![
            Token::Text, // "title"
            Token::Newline,
            Token::Newline,
            Token::IndentLevel,
            Token::Colon, // This is not valid content (can't start a paragraph or session)
            Token::DedentLevel,
            Token::DedentLevel,
        ];

        println!(
            "\n=== Test: Greedy paragraph bug - session title + IndentLevel + unparseable content ==="
        );
        println!("Tokens: {:?}", tokens);

        let result = parse(tokens.clone());

        match &result {
            Ok(doc) => {
                println!("\n✓ Parsed successfully (shouldn't happen!):");
                for (i, item) in doc.content.iter().enumerate() {
                    match item {
                        ContentItem::Paragraph(p) => {
                            println!("  {}: Paragraph with {} lines", i, p.lines.len());
                        }
                        ContentItem::Session(s) => {
                            println!(
                                "  {}: Session '{}' with {} children",
                                i,
                                s.title,
                                s.content.len()
                            );
                        }
                        ContentItem::List(l) => {
                            println!("  {}: List with {} items", i, l.items.len());
                        }
                        ContentItem::Definition(d) => {
                            println!(
                                "  {}: Definition '{}' with {} children",
                                i,
                                d.subject,
                                d.content.len()
                            );
                        }
                        ContentItem::Annotation(a) => {
                            println!(
                                "  {}: Annotation '{}' with {} children",
                                i,
                                a.label.value,
                                a.content.len()
                            );
                        }
                        ContentItem::ForeignBlock(fb) => {
                            println!(
                                "  {}: ForeignBlock '{}' with {} chars, closing: {}",
                                i,
                                fb.subject,
                                fb.content.len(),
                                fb.closing_annotation.label.value
                            );
                        }
                    }
                }
                panic!("Should have failed to parse!");
            }
            Err(errors) => {
                println!("\n✗ Parse failed with {} error(s):", errors.len());
                for (i, error) in errors.iter().enumerate() {
                    println!("  Error {}: at span {:?}", i, error.span());
                    println!("    Reason: {:?}", error.reason());
                    println!("    Found: {:?}", error.found());
                }

                // With the bug: error says "unexpected IndentLevel at position 3"
                //   because paragraph consumed "Text Newline", left "Newline IndentLevel Colon..."
                //
                // With the fix: error is NOT at position 3 (IndentLevel)
                //   It could be at position 4 (Colon - can't start content item)
                //   Or at position 5+ (trailing tokens after session attempts to match)

                assert_eq!(errors.len(), 1, "Should have exactly one error");
                let error = &errors[0];

                // The critical check: error should NOT be at position 3 (IndentLevel)
                // If it is, that means paragraph parser consumed the title
                assert_ne!(
                    error.span().start,
                    3,
                    "BUG STILL PRESENT: Paragraph parser consumed session title, error is at IndentLevel (position 3)"
                );

                println!(
                    "\n✓ Fix verified: Error is at position {}, not at IndentLevel (position 3)",
                    error.span().start
                );
                println!(
                    "  This means the paragraph parser correctly rejected the session title pattern"
                );
            }
        }
    }

    #[test]
    fn test_session_title_pattern_without_indent() {
        // This is just a paragraph - text + blank line
        // Should parse fine as a paragraph
        let input = "Normal paragraph\n\nAnother paragraph\n\n";
        let tokens = lex(input);

        println!("\n=== Test: Normal paragraphs (no IndentLevel) ===");
        let result = parse(tokens);

        match &result {
            Ok(doc) => {
                println!("✓ Parsed successfully");
                println!("Document has {} items:", doc.content.len());
                assert_eq!(doc.content.len(), 2, "Should have 2 paragraphs");
            }
            Err(e) => {
                panic!("Should have parsed successfully: {:?}", e);
            }
        }
    }

    #[test]
    fn test_verified_paragraphs_sample() {
        use crate::txxt_nano::testing::assert_ast;

        let source =
            TxxtSources::get_string("000-paragraphs.txxt").expect("Failed to load sample file");
        let tokens = lex(&source);

        let result = parse(tokens);
        assert!(
            result.is_ok(),
            "Failed to parse 000-paragraphs.txxt: {:?}",
            result
        );

        let doc = result.unwrap();

        // Expected structure based on 000-paragraphs.txxt:
        // 7 paragraphs total, with specific line counts
        assert_ast(&doc)
            .item_count(7)
            .item(0, |item| {
                item.assert_paragraph().line_count(1); // "Simple Paragraphs Test"
            })
            .item(1, |item| {
                item.assert_paragraph().line_count(1); // "This is a simple paragraph with just one line."
            })
            .item(2, |item| {
                item.assert_paragraph().line_count(3); // Multi-line paragraph
            })
            .item(3, |item| {
                item.assert_paragraph().line_count(1); // "Another paragraph follows..."
            })
            .item(4, |item| {
                item.assert_paragraph().line_count(1); // Paragraph with special chars
            })
            .item(5, |item| {
                item.assert_paragraph().line_count(1); // Paragraph with numbers
            })
            .item(6, |item| {
                item.assert_paragraph().line_count(1); // Paragraph with mixed content
            });
    }

    #[test]
    fn test_verified_single_session_sample() {
        use crate::txxt_nano::testing::assert_ast;

        let source = TxxtSources::get_string("010-paragraphs-sessions-flat-single.txxt")
            .expect("Failed to load sample file");
        let tokens = lex(&source);

        let result = parse(tokens.clone());
        assert!(
            result.is_ok(),
            "Failed to parse 010-paragraphs-sessions-flat-single.txxt: {:?}",
            result
        );

        let doc = result.unwrap();

        // Expected structure based on 010-paragraphs-sessions-flat-single.txxt:
        // Line 1: "Paragraphs and Single Session Test" - paragraph (1 line)
        // Line 3: "This document tests..." - paragraph (1 line)
        // Line 5: "1. Introduction" - session with 2 paragraphs
        //   Line 7: "This is the content..." - paragraph (1 line)
        //   Line 9: "The session can contain..." - paragraph (1 line)
        // Line 11: "This paragraph comes after..." - paragraph (1 line)
        // Line 13: "Another Session" - session with 1 paragraph
        //   Line 15: "This session demonstrates..." - paragraph (1 line)
        // Line 17: "Final paragraph..." - paragraph (1 line)

        assert_ast(&doc)
            .item_count(6)
            .item(0, |item| {
                item.assert_paragraph().line_count(1);
            })
            .item(1, |item| {
                item.assert_paragraph().line_count(1);
            })
            .item(2, |item| {
                item.assert_session()
                    .child_count(2)
                    .child(0, |child| {
                        child.assert_paragraph().line_count(1);
                    })
                    .child(1, |child| {
                        child.assert_paragraph().line_count(1);
                    });
            })
            .item(3, |item| {
                item.assert_paragraph().line_count(1);
            })
            .item(4, |item| {
                item.assert_session().child_count(1).child(0, |child| {
                    child.assert_paragraph().line_count(1);
                });
            })
            .item(5, |item| {
                item.assert_paragraph().line_count(1);
            });
    }

    #[test]
    fn test_verified_multiple_sessions_sample() {
        use crate::txxt_nano::testing::assert_ast;

        let source = TxxtSources::get_string("020-paragraphs-sessions-flat-multiple.txxt")
            .expect("Failed to load sample file");
        let tokens = lex(&source);

        let result = parse(tokens.clone());
        assert!(
            result.is_ok(),
            "Failed to parse 020-paragraphs-sessions-flat-multiple.txxt: {:?}",
            result
        );

        let doc = result.unwrap();

        // Expected structure based on 020-paragraphs-sessions-flat-multiple.txxt:
        // Line 1: "Multiple Sessions Flat Test" - paragraph (1 line)
        // Line 3: "This document tests..." - paragraph (1 line)
        // Line 5: "1. First Session" - session with 2 paragraphs
        //   Line 7: "This is the content..." - paragraph (1 line)
        //   Line 9: "It can have multiple..." - paragraph (1 line)
        // Line 11: "2. Second Session" - session with 1 paragraph
        //   Line 13: "The second session..." - paragraph (1 line)
        // Line 15: "A paragraph between sessions" - paragraph (1 line)
        // Line 17: "3. Third Session" - session with 1 paragraph
        //   Line 19: "Sessions can have..." - paragraph (1 line)
        // Line 21: "Another paragraph" - paragraph (1 line)
        // Line 23: "4. Session Without Numbering" - session with 1 NESTED session
        //   Line 25: "Session titles don't require..." - nested session title with 1 paragraph
        //     Line 27: "They just need..." - paragraph (1 line)
        // Line 29: "Final paragraph..." - paragraph (1 line)

        assert_ast(&doc)
            .item_count(9)
            // Item 0: Paragraph (1 line)
            .item(0, |item| {
                item.assert_paragraph().line_count(1);
            })
            // Item 1: Paragraph (1 line)
            .item(1, |item| {
                item.assert_paragraph().line_count(1);
            })
            // Item 2: Session with 2 paragraphs
            .item(2, |item| {
                item.assert_session().child_count(2).children(|children| {
                    children
                        .all_paragraphs()
                        .item(0, |p| {
                            p.assert_paragraph().line_count(1);
                        })
                        .item(1, |p| {
                            p.assert_paragraph().line_count(1);
                        });
                });
            })
            // Item 3: Session with 1 paragraph
            .item(3, |item| {
                item.assert_session().child_count(1).child(0, |child| {
                    child.assert_paragraph().line_count(1);
                });
            })
            // Item 4: Paragraph (1 line)
            .item(4, |item| {
                item.assert_paragraph().line_count(1);
            })
            // Item 5: Session with 1 paragraph
            .item(5, |item| {
                item.assert_session().child_count(1).child(0, |child| {
                    child.assert_paragraph().line_count(1);
                });
            })
            // Item 6: Paragraph (1 line)
            .item(6, |item| {
                item.assert_paragraph().line_count(1);
            })
            // Item 7: Session with 1 nested session
            .item(7, |item| {
                item.assert_session().child_count(1).child(0, |child| {
                    // This should be a nested session, not a paragraph
                    child
                        .assert_session()
                        .child_count(1)
                        .child(0, |nested_child| {
                            nested_child.assert_paragraph().line_count(1);
                        });
                });
            })
            // Item 8: Paragraph (1 line)
            .item(8, |item| {
                item.assert_paragraph().line_count(1);
            });
    }

    #[test]
    fn test_verified_nested_sessions_sample() {
        use crate::txxt_nano::testing::assert_ast;

        let source = TxxtSources::get_string("030-paragraphs-sessions-nested-multiple.txxt")
            .expect("Failed to load sample file");
        let tokens = lex(&source);

        let result = parse(tokens.clone());
        assert!(
            result.is_ok(),
            "Failed to parse 030-paragraphs-sessions-nested-multiple.txxt: {:?}",
            result
        );

        let doc = result.unwrap();

        // Expected structure based on 030-paragraphs-sessions-nested-multiple.txxt:
        // Line 1: "Nested Sessions Test" - paragraph (1 line)
        // Line 3: "This document tests..." - paragraph (1 line)
        // Line 5: "1. Root Session" - session with complex nested structure
        //   Line 7: "This is content..." - paragraph (1 line)
        //   Line 9: "1.1. First Sub-session" - session with 2 paragraphs
        //     Line 11: "This is content..." - paragraph (1 line)
        //     Line 13: "It can have..." - paragraph (1 line)
        //   Line 15: "1.2. Second Sub-session" - session with nested session + paragraph
        //     Line 17: "Another sub-session..." - paragraph (1 line)
        //     Line 19: "1.2.1. Deeply Nested Session" - session with 2 paragraphs
        //       Line 21: "This is content..." - paragraph (1 line)
        //       Line 23: "Sessions can be..." - paragraph (1 line)
        //   Line 25: "Back to the first..." - paragraph (1 line)
        // Line 27: "2. Another Root Session" - session with nested session
        //   Line 29: "This session is..." - paragraph (1 line)
        //   Line 31: "2.1. Its Sub-session" - session with 1 paragraph
        //     Line 33: "Sub-sessions can..." - paragraph (1 line)
        // Line 35: "Final paragraph..." - paragraph (1 line)

        assert_ast(&doc)
            .item_count(5)
            // Item 0: Paragraph (1 line)
            .item(0, |item| {
                item.assert_paragraph().line_count(1);
            })
            // Item 1: Paragraph (1 line)
            .item(1, |item| {
                item.assert_paragraph().line_count(1);
            })
            // Item 2: "1. Root Session" with 4 children (paragraph, session, session, paragraph)
            .item(2, |item| {
                item.assert_session()
                    .child_count(4)
                    // Child 0: Paragraph
                    .child(0, |child| {
                        child.assert_paragraph().line_count(1);
                    })
                    // Child 1: "1.1. First Sub-session" with 2 paragraphs
                    .child(1, |child| {
                        child
                            .assert_session()
                            .child_count(2)
                            .child(0, |para| {
                                para.assert_paragraph().line_count(1);
                            })
                            .child(1, |para| {
                                para.assert_paragraph().line_count(1);
                            });
                    })
                    // Child 2: "1.2. Second Sub-session" with 2 children (paragraph + nested session)
                    .child(2, |child| {
                        child
                            .assert_session()
                            .child_count(2)
                            .child(0, |para| {
                                para.assert_paragraph().line_count(1);
                            })
                            // "1.2.1. Deeply Nested Session" with 2 paragraphs
                            .child(1, |deeply_nested| {
                                deeply_nested
                                    .assert_session()
                                    .child_count(2)
                                    .child(0, |para| {
                                        para.assert_paragraph().line_count(1);
                                    })
                                    .child(1, |para| {
                                        para.assert_paragraph().line_count(1);
                                    });
                            });
                    })
                    // Child 3: Paragraph ("Back to the first...")
                    .child(3, |child| {
                        child.assert_paragraph().line_count(1);
                    });
            })
            // Item 3: "2. Another Root Session" with 2 children (paragraph + session)
            .item(3, |item| {
                item.assert_session()
                    .child_count(2)
                    .child(0, |child| {
                        child.assert_paragraph().line_count(1);
                    })
                    // "2.1. Its Sub-session" with 1 paragraph
                    .child(1, |child| {
                        child.assert_session().child_count(1).child(0, |para| {
                            para.assert_paragraph().line_count(1);
                        });
                    });
            })
            // Item 4: Final paragraph
            .item(4, |item| {
                item.assert_paragraph().line_count(1);
            });
    }

    // ==================== LIST TESTS ====================
    // Following the complexity ladder: simplest → variations → documents

    #[test]
    fn test_simplest_dash_list() {
        // Simplest possible list: 2 dashed items
        use crate::txxt_nano::testing::assert_ast;

        let source = TxxtSources::get_string("040-lists.txxt").unwrap();
        let tokens = lex_with_spans(&source);
        let doc = parse_with_source(tokens, &source).unwrap();

        // Find the first list (after "Plain dash lists:" paragraph)
        // Document structure: Para Para Para List Para List...
        assert_ast(&doc).item(3, |item| {
            item.assert_list()
                .item_count(3)
                .item(0, |list_item| {
                    list_item
                        .text("- First item {{list-item}}")
                        .text_contains("First item");
                })
                .item(1, |list_item| {
                    list_item
                        .text("- Second item {{list-item}}")
                        .text_contains("Second item");
                })
                .item(2, |list_item| {
                    list_item
                        .text("- Third item {{list-item}}")
                        .text_contains("Third item");
                });
        });
    }

    #[test]
    fn test_numbered_list() {
        // Test numbered list: "1. ", "2. ", "3. "
        use crate::txxt_nano::testing::assert_ast;

        let source = TxxtSources::get_string("040-lists.txxt").unwrap();
        let tokens = lex_with_spans(&source);
        let doc = parse_with_source(tokens, &source).unwrap();

        // Numerical lists (item 5)
        assert_ast(&doc).item(5, |item| {
            item.assert_list()
                .item_count(3)
                .item(0, |list_item| {
                    list_item.text_starts_with("1.");
                })
                .item(1, |list_item| {
                    list_item.text_starts_with("2.");
                })
                .item(2, |list_item| {
                    list_item.text_starts_with("3.");
                });
        });
    }

    #[test]
    fn test_alphabetical_list() {
        // Test alphabetical list: "a. ", "b. ", "c. "
        use crate::txxt_nano::testing::assert_ast;

        let source = TxxtSources::get_string("040-lists.txxt").unwrap();
        let tokens = lex_with_spans(&source);
        let doc = parse_with_source(tokens, &source).unwrap();

        // Alphabetical lists (item 7)
        assert_ast(&doc).item(7, |item| {
            item.assert_list()
                .item_count(3)
                .item(0, |list_item| {
                    list_item.text_starts_with("a.");
                })
                .item(1, |list_item| {
                    list_item.text_starts_with("b.");
                })
                .item(2, |list_item| {
                    list_item.text_starts_with("c.");
                });
        });
    }

    #[test]
    fn test_mixed_decoration_list() {
        // Test mixed decorations: different markers in same list
        use crate::txxt_nano::testing::assert_ast;

        let source = TxxtSources::get_string("040-lists.txxt").unwrap();
        let tokens = lex_with_spans(&source);
        let doc = parse_with_source(tokens, &source).unwrap();

        // Mixed decoration lists (item 9)
        assert_ast(&doc).item(9, |item| {
            item.assert_list()
                .item_count(3)
                .item(0, |list_item| {
                    list_item.text_starts_with("1.");
                })
                .item(1, |list_item| {
                    list_item.text_starts_with("-");
                })
                .item(2, |list_item| {
                    list_item.text_starts_with("a.");
                });
        });
    }

    #[test]
    fn test_parenthetical_list() {
        // Test parenthetical numbering: "(1) ", "(2) ", "(3) "
        use crate::txxt_nano::testing::assert_ast;

        let source = TxxtSources::get_string("040-lists.txxt").unwrap();
        let tokens = lex_with_spans(&source);
        let doc = parse_with_source(tokens, &source).unwrap();

        // Parenthetical numbering (item 11)
        assert_ast(&doc).item(11, |item| {
            item.assert_list()
                .item_count(3)
                .item(0, |list_item| {
                    list_item.text_starts_with("(1)");
                })
                .item(1, |list_item| {
                    list_item.text_starts_with("(2)");
                })
                .item(2, |list_item| {
                    list_item.text_starts_with("(3)");
                });
        });
    }

    #[test]
    fn test_paragraph_list_disambiguation() {
        // Critical test: single list-like line becomes paragraph, 2+ with blank line become list
        use crate::txxt_nano::testing::assert_ast;

        let source = TxxtSources::get_string("050-paragraph-lists.txxt").unwrap();
        let tokens = lex_with_spans(&source);
        let doc = parse_with_source(tokens, &source).unwrap();

        // Items 2-4: Single list-item-lines merged into paragraphs
        assert_ast(&doc).item(2, |item| {
            item.assert_paragraph()
                .text_contains("- This is not a list");
        });

        assert_ast(&doc).item(3, |item| {
            item.assert_paragraph()
                .text_contains("1. This is also not a list");
        });

        // Item 6: First actual list (after blank line) - 0-indexed!
        assert_ast(&doc).item(6, |item| {
            item.assert_list()
                .item_count(2)
                .item(0, |list_item| {
                    list_item.text_contains("This is a list");
                })
                .item(1, |list_item| {
                    list_item.text_contains("Blank line required");
                });
        });
    }

    #[test]
    fn test_verified_lists_document() {
        // Full document test with lists from TxxtSources
        use crate::txxt_nano::testing::assert_ast;

        let source = TxxtSources::get_string("040-lists.txxt").unwrap();
        let tokens = lex_with_spans(&source);
        let doc = parse_with_source(tokens, &source).unwrap();

        // Verify document structure: paragraphs + lists alternating
        assert_ast(&doc)
            .item(0, |item| {
                item.assert_paragraph().text_contains("Lists Only Test");
            })
            .item(1, |item| {
                item.assert_paragraph()
                    .text_contains("various list formats");
            })
            .item(2, |item| {
                item.assert_paragraph().text_contains("Plain dash lists");
            })
            .item(3, |item| {
                item.assert_list().item_count(3); // Dash list
            })
            .item(4, |item| {
                item.assert_paragraph().text_contains("Numerical lists");
            })
            .item(5, |item| {
                item.assert_list().item_count(3); // Numbered list
            })
            .item(6, |item| {
                item.assert_paragraph().text_contains("Alphabetical lists");
            })
            .item(7, |item| {
                item.assert_list().item_count(3); // Alphabetical list
            });
    }

    #[test]
    fn test_list_requires_preceding_blank_line() {
        // Critical test: Lists MUST have a preceding blank line for disambiguation
        // Without the blank line, consecutive list-item-lines should be parsed as paragraphs
        use crate::txxt_nano::testing::assert_ast;

        let source = "First paragraph\n- Item one\n- Item two\n";
        let tokens = lex_with_spans(source);
        let doc = parse_with_source(tokens, source).unwrap();

        // Should be parsed as a single paragraph, NOT a paragraph + list
        // because there's no blank line before the list-item-lines
        assert_eq!(
            doc.content.len(),
            1,
            "Should be 1 paragraph, not paragraph + list"
        );
        assert_ast(&doc).item(0, |item| {
            item.assert_paragraph()
                .text_contains("First paragraph")
                .text_contains("- Item one")
                .text_contains("- Item two");
        });

        // Now test the positive case: with blank line, it becomes separate items
        let source_with_blank = "First paragraph\n\n- Item one\n- Item two\n";
        let tokens2 = lex_with_spans(source_with_blank);
        let doc2 = parse_with_source(tokens2, source_with_blank).unwrap();

        // Should be parsed as paragraph + list
        assert_eq!(
            doc2.content.len(),
            2,
            "Should be paragraph + list with blank line"
        );
        assert_ast(&doc2)
            .item(0, |item| {
                item.assert_paragraph().text_contains("First paragraph");
            })
            .item(1, |item| {
                item.assert_list()
                    .item_count(2)
                    .item(0, |list_item| {
                        list_item.text_contains("Item one");
                    })
                    .item(1, |list_item| {
                        list_item.text_contains("Item two");
                    });
            });
    }

    // ==================== TRIFECTA TESTS ====================
    // Testing paragraphs + sessions + lists together

    #[test]
    fn test_trifecta_flat_simple() {
        // Test flat structure with all three elements
        use crate::txxt_nano::testing::assert_ast;

        let source = TxxtSources::get_string("050-trifecta-flat-simple.txxt").unwrap();
        let tokens = lex_with_spans(&source);
        let doc = parse_with_source(tokens, &source).unwrap();

        // Item 0-1: Opening paragraphs
        assert_ast(&doc)
            .item(0, |item| {
                item.assert_paragraph()
                    .text_contains("Trifecta Flat Structure Test");
            })
            .item(1, |item| {
                item.assert_paragraph()
                    .text_contains("all three core elements");
            });

        // Item 2: Session with only paragraphs
        assert_ast(&doc).item(2, |item| {
            item.assert_session()
                .label_contains("Session with Paragraph Content")
                .child_count(2)
                .child(0, |child| {
                    child
                        .assert_paragraph() // "Session with Paragraph Content"
                        .text_contains("starts with a paragraph");
                })
                .child(1, |child| {
                    child
                        .assert_paragraph() // "multiple paragraphs"
                        .text_contains("multiple paragraphs");
                });
        });

        // Item 3: Session with only a list
        assert_ast(&doc).item(3, |item| {
            item.assert_session()
                .label_contains("Session with List Content")
                .child_count(1)
                .child(0, |child| {
                    child.assert_list().item_count(3);
                });
        });

        // Item 4: Session with mixed content (para + list + para)
        assert_ast(&doc).item(4, |item| {
            item.assert_session()
                .label_contains("Session with Mixed Content")
                .child_count(3)
                .child(0, |child| {
                    child
                        .assert_paragraph()
                        .text_contains("starts with a paragraph");
                })
                .child(1, |child| {
                    child.assert_list().item_count(2);
                })
                .child(2, |child| {
                    child
                        .assert_paragraph()
                        .text_contains("ends with another paragraph");
                });
        });

        // Item 5: Root level paragraph
        assert_ast(&doc).item(5, |item| {
            item.assert_paragraph().text_contains("root level");
        });

        // Item 6: Root level list
        assert_ast(&doc).item(6, |item| {
            item.assert_list().item_count(2);
        });

        // Item 7: Session with list + para + list
        assert_ast(&doc).item(7, |item| {
            item.assert_session()
                .label_contains("Another Session")
                .child_count(3)
                .child(0, |child| {
                    child.assert_list().item_count(2);
                })
                .child(1, |child| {
                    child.assert_paragraph().text_contains("has a paragraph");
                })
                .child(2, |child| {
                    child.assert_list().item_count(2);
                });
        });
    }

    #[test]
    fn test_trifecta_nesting() {
        // Test nested structure with all three elements
        use crate::txxt_nano::testing::assert_ast;

        let source = TxxtSources::get_string("060-trifecta-nesting.txxt").unwrap();
        let tokens = lex_with_spans(&source);
        let doc = parse_with_source(tokens, &source).unwrap();

        // Item 0-1: Opening paragraphs
        assert_ast(&doc)
            .item(0, |item| {
                item.assert_paragraph() // "Trifecta Nesting Test"
                    .text_contains("Trifecta Nesting Test");
            })
            .item(1, |item| {
                item.assert_paragraph() // "various levels of nesting"
                    .text_contains("various levels of nesting");
            });

        // Item 2: Root session with nested sessions and mixed content
        // The structure has been updated to include nested lists, which may affect the child count
        assert_ast(&doc).item(2, |item| {
            item.assert_session()
                .label_contains("1. Root Session")
                .child_count(5); // para, subsession, subsession, para, list
        });

        // Verify first child of root session is paragraph
        assert_ast(&doc).item(2, |item| {
            item.assert_session().child(0, |child| {
                child.assert_paragraph().text_contains("nested elements");
            });
        });

        // Verify first nested session (1.1)
        assert_ast(&doc).item(2, |item| {
            item.assert_session().child(1, |child| {
                child
                    .assert_session()
                    .label_contains("1.1. Sub-session")
                    .child_count(2) // para + list
                    .child(0, |para| {
                        para.assert_paragraph();
                    })
                    .child(1, |list| {
                        list.assert_list().item_count(2);
                    });
            });
        });

        // Verify deeply nested session (1.2 containing 1.2.1)
        assert_ast(&doc).item(2, |item| {
            item.assert_session().child(2, |child| {
                child
                    .assert_session()
                    .label_contains("1.2. Sub-session with List")
                    .child_count(3) // list, para, nested session
                    .child(2, |nested| {
                        nested
                            .assert_session()
                            .label_contains("1.2.1. Deeply Nested")
                            .child_count(3); // para + list + list
                    });
            });
        });

        // Verify the deeply nested session has 2 lists
        assert_ast(&doc).item(2, |item| {
            item.assert_session().child(2, |subsession| {
                subsession.assert_session().child(2, |deeply_nested| {
                    deeply_nested
                        .assert_session()
                        .child(1, |first_list| {
                            first_list.assert_list().item_count(2);
                        })
                        .child(2, |second_list| {
                            second_list.assert_list().item_count(2);
                        });
                });
            });
        });

        // Item 3: Another root session with different nesting
        assert_ast(&doc).item(3, |item| {
            item.assert_session()
                .label_contains("2. Another Root Session")
                .child_count(2); // para + subsession
        });

        // Verify even deeper nesting (2.1.1)
        assert_ast(&doc).item(3, |item| {
            item.assert_session().child(1, |subsession| {
                subsession
                    .assert_session()
                    .label_contains("2.1. Mixed Content")
                    .child_count(4) // list, para, list, nested session
                    .child(3, |deeply_nested| {
                        deeply_nested
                            .assert_session()
                            .label_contains("2.1.1. Even Deeper")
                            .child_count(4); // para, list, para, list
                    });
            });
        });

        // Final root paragraph
        assert_ast(&doc).item(4, |item| {
            item.assert_paragraph()
                .text_contains("Final root level paragraph");
        });
    }

    // ==================== NESTED LISTS TESTS ====================
    // Testing nested list structures

    #[test]
    fn test_verified_nested_lists_simple() {
        use crate::txxt_nano::testing::assert_ast;

        let source = TxxtSources::get_string("070-nested-lists-simple.txxt")
            .expect("Failed to load sample file");
        let tokens = lex_with_spans(&source);
        let doc = parse_with_source(tokens, &source).unwrap();

        // Item 0-1: Opening paragraphs
        assert_ast(&doc)
            .item(0, |item| {
                item.assert_paragraph() // "Simple Nested Lists Test"
                    .text_contains("Simple Nested Lists Test");
            })
            .item(1, |item| {
                item.assert_paragraph() // "simple list-in-list nesting"
                    .text_contains("simple list-in-list nesting");
            });

        // Item 2: Paragraph before first list
        assert_ast(&doc).item(2, |item| {
            item.assert_paragraph().text_contains("Basic nested list");
        });

        // Item 3: First nested list structure
        assert_ast(&doc).item(3, |item| {
            item.assert_list()
                .item_count(2)
                // First item with nested list
                .item(0, |list_item| {
                    list_item
                        .text_contains("First outer item")
                        .child_count(1)
                        .child(0, |child| {
                            child.assert_list().item_count(2);
                        });
                })
                // Second item with nested list
                .item(1, |list_item| {
                    list_item
                        .text_contains("Second outer item")
                        .child_count(1)
                        .child(0, |child| {
                            child.assert_list().item_count(2);
                        });
                });
        });

        // Item 4: Paragraph before second list
        assert_ast(&doc).item(4, |item| {
            item.assert_paragraph()
                .text_contains("Numbered list with nested dashed list");
        });

        // Item 5: Numbered list with nested dashed lists
        assert_ast(&doc).item(5, |item| {
            item.assert_list()
                .item_count(2)
                .item(0, |list_item| {
                    list_item
                        .text_starts_with("1.")
                        .text_contains("First numbered item")
                        .child_count(1)
                        .child(0, |child| {
                            child.assert_list().item_count(2);
                        });
                })
                .item(1, |list_item| {
                    list_item
                        .text_starts_with("2.")
                        .text_contains("Second numbered item")
                        .child_count(1)
                        .child(0, |child| {
                            child.assert_list().item_count(2);
                        });
                });
        });

        // Item 6: Final paragraph
        assert_ast(&doc).item(6, |item| {
            item.assert_paragraph()
                .text_contains("Final paragraph after lists");
        });
    }

    #[test]
    fn test_verified_nested_lists_mixed_content() {
        use crate::txxt_nano::testing::assert_ast;

        let source = TxxtSources::get_string("080-nested-lists-mixed-content.txxt")
            .expect("Failed to load sample file");
        let tokens = lex_with_spans(&source);
        let doc = parse_with_source(tokens, &source).unwrap();

        // Item 0-1: Opening paragraphs
        assert_ast(&doc)
            .item(0, |item| {
                item.assert_paragraph() // "Nested Lists with Mixed Content Test"
                    .text_contains("Nested Lists with Mixed Content Test");
            })
            .item(1, |item| {
                item.assert_paragraph() // "mix of paragraphs and other lists"
                    .text_contains("mix of paragraphs and other lists");
            });

        // Item 2: Paragraph before first list
        assert_ast(&doc).item(2, |item| {
            item.assert_paragraph()
                .text_contains("List with paragraph content");
        });

        // Item 3: List with paragraph content in items
        assert_ast(&doc).item(3, |item| {
            item.assert_list()
                .item_count(2)
                // First item with one paragraph
                .item(0, |list_item| {
                    list_item
                        .text_contains("First item with nested paragraph")
                        .child_count(1)
                        .child(0, |child| {
                            child
                                .assert_paragraph()
                                .text_contains("paragraph nested inside the first list item");
                        });
                })
                // Second item with two paragraphs
                .item(1, |list_item| {
                    list_item
                        .text_contains("Second item with multiple paragraphs")
                        .child_count(2)
                        .child(0, |child| {
                            child
                                .assert_paragraph()
                                .text_contains("first paragraph in the second item");
                        })
                        .child(1, |child| {
                            child.assert_paragraph().text_contains("second paragraph");
                        });
                });
        });

        // Item 4: Paragraph before mixed content list
        assert_ast(&doc).item(4, |item| {
            item.assert_paragraph()
                .text_contains("mixed paragraphs and nested lists");
        });

        // Item 5: List with mixed content (paragraphs and nested lists)
        assert_ast(&doc).item(5, |item| {
            item.assert_list()
                .item_count(2)
                // First complex item: para + list + para
                .item(0, |list_item| {
                    list_item
                        .text_starts_with("1.")
                        .text_contains("First complex item")
                        .child_count(3)
                        .child(0, |child| {
                            child
                                .assert_paragraph()
                                .text_contains("paragraph explaining the first item");
                        })
                        .child(1, |child| {
                            child.assert_list().item_count(2);
                        })
                        .child(2, |child| {
                            child
                                .assert_paragraph()
                                .text_contains("Another paragraph after the nested list");
                        });
                })
                // Second complex item: para + list + para
                .item(1, |list_item| {
                    list_item
                        .text_starts_with("2.")
                        .text_contains("Second complex item")
                        .child_count(3)
                        .child(0, |child| {
                            child
                                .assert_paragraph()
                                .text_contains("Opening paragraph for item two");
                        })
                        .child(1, |child| {
                            child.assert_list().item_count(2);
                        })
                        .child(2, |child| {
                            child
                                .assert_paragraph()
                                .text_contains("Closing paragraph for item two");
                        });
                });
        });

        // Item 6: Paragraph before deeply nested structure
        assert_ast(&doc).item(6, |item| {
            item.assert_paragraph()
                .text_contains("Deeply nested structure");
        });

        // Item 7: Deeply nested list structure
        assert_ast(&doc).item(7, |item| {
            item.assert_list()
                .item_count(2)
                // First outer item with deep nesting
                .item(0, |outer_item| {
                    outer_item
                        .text_contains("Outer item one")
                        .child_count(2) // para + nested list
                        .child(0, |child| {
                            child
                                .assert_paragraph()
                                .text_contains("Paragraph in outer item");
                        })
                        .child(1, |middle_list| {
                            middle_list
                                .assert_list()
                                .item_count(2)
                                // Middle item one with inner list
                                .item(0, |middle_item| {
                                    middle_item
                                        .text_contains("Middle item one")
                                        .child_count(1)
                                        .child(0, |inner_list| {
                                            inner_list.assert_list().item_count(2);
                                        });
                                })
                                // Middle item two with paragraph
                                .item(1, |middle_item| {
                                    middle_item
                                        .text_contains("Middle item two")
                                        .child_count(1)
                                        .child(0, |para| {
                                            para.assert_paragraph()
                                                .text_contains("Paragraph in middle item");
                                        });
                                });
                        });
                })
                // Second outer item with paragraph
                .item(1, |outer_item| {
                    outer_item
                        .text_contains("Outer item two")
                        .child_count(1)
                        .child(0, |child| {
                            child.assert_paragraph().text_contains("Final paragraph");
                        });
                });
        });

        // Item 8: Final paragraph
        assert_ast(&doc).item(8, |item| {
            item.assert_paragraph().text_contains("End of document");
        });
    }

    // ==================== DEFINITION TESTS ====================
    // Testing definition structures

    #[test]
    fn test_unified_recursive_parser_simple() {
        // Minimal test for the unified recursive parser
        let source = "First paragraph\n\nDefinition:\n    Content of definition\n";
        let tokens = lex_with_spans(source);
        println!("Testing simple definition with unified parser:");
        println!("Source: {:?}", source);

        let result = parse_with_source(tokens, source);
        if let Err(ref e) = result {
            println!("Parse error: {:?}", e);
        }
        assert!(result.is_ok(), "Failed to parse simple definition");
        let doc = result.unwrap();
        assert_eq!(doc.content.len(), 2, "Should have paragraph and definition");
    }

    #[test]
    fn test_unified_recursive_nested_definitions() {
        // Test nested definitions with the unified parser
        let source = "Outer:\n    Inner:\n        Nested content\n";
        let tokens = lex_with_spans(source);
        println!("Testing nested definitions with unified parser:");
        println!("Source: {:?}", source);

        let result = parse_with_source(tokens, source);
        if let Err(ref e) = result {
            println!("Parse error: {:?}", e);
        }
        assert!(result.is_ok(), "Failed to parse nested definitions");

        let doc = result.unwrap();
        assert_eq!(doc.content.len(), 1, "Should have one outer definition");

        // Check outer definition
        let outer_def = doc.content[0]
            .as_definition()
            .expect("Should be a definition");
        assert_eq!(outer_def.subject, "Outer");
        assert_eq!(
            outer_def.content.len(),
            1,
            "Outer should have one inner item"
        );

        // Check inner definition
        let inner_def = outer_def.content[0]
            .as_definition()
            .expect("Inner should be a definition");
        assert_eq!(inner_def.subject, "Inner");
        assert_eq!(inner_def.content.len(), 1, "Inner should have content");

        // Check nested content
        let nested_para = inner_def.content[0]
            .as_paragraph()
            .expect("Should be a paragraph");
        assert_eq!(nested_para.lines[0], "Nested content");
    }

    #[test]
    // Previously ignored for issue #35 - now testing if fixed
    fn test_unified_parser_paragraph_then_definition() {
        // Test paragraph followed by definition - similar to failing test
        let source = "Simple paragraph\n\nAnother paragraph\n\nFirst Definition:\n    Definition content\n\nSecond Definition:\n    More content\n";
        let tokens = lex_with_spans(source);
        println!("Testing paragraph then definition:");
        println!("Source: {:?}", source);

        let result = parse_with_source(tokens, source);
        if let Err(ref e) = result {
            println!("Parse error: {:?}", e);
            println!("Error at span: {:?}", &source[e[0].span().clone()]);
        }
        assert!(result.is_ok(), "Failed to parse paragraph then definition");

        let doc = result.unwrap();
        println!("Parsed {} items", doc.content.len());
        for (i, item) in doc.content.iter().enumerate() {
            match item {
                ContentItem::Paragraph(p) => {
                    println!("  Item {}: Paragraph with {} lines", i, p.lines.len())
                }
                ContentItem::Definition(d) => println!("  Item {}: Definition '{}'", i, d.subject),
                _ => println!("  Item {}: Other", i),
            }
        }
        assert_eq!(
            doc.content.len(),
            4,
            "Should have 2 paragraphs and 2 definitions"
        );
    }

    #[test]
    // Previously ignored for issue #35 - now testing if fixed
    fn test_verified_definitions_simple() {
        use crate::txxt_nano::testing::assert_ast;

        let source = TxxtSources::get_string("090-definitions-simple.txxt")
            .expect("Failed to load sample file");
        let tokens = lex_with_spans(&source);

        // Debug: print first few tokens
        println!("First 10 tokens:");
        for (i, token) in tokens.iter().take(10).enumerate() {
            println!("  {}: {:?}", i, token);
        }

        let result = parse_with_source(tokens, &source);
        if let Err(ref e) = result {
            println!("Parse error: {:?}", e);
        }
        let doc = result.unwrap();

        // Item 0-1: Opening paragraphs
        assert_ast(&doc)
            .item(0, |item| {
                item.assert_paragraph() // "Simple Definitions Test"
                    .text_contains("Simple Definitions Test");
            })
            .item(1, |item| {
                item.assert_paragraph() // "basic Definition element"
                    .text_contains("basic Definition element");
            });

        // Item 2: First Definition
        assert_ast(&doc).item(2, |item| {
            item.assert_definition()
                .subject("First Definition")
                .child_count(1)
                .child(0, |child| {
                    child
                        .assert_paragraph()
                        .text_contains("content of the first definition");
                });
        });

        // Item 3: Second Definition
        assert_ast(&doc).item(3, |item| {
            item.assert_definition()
                .subject("Second Definition")
                .child_count(1)
                .child(0, |child| {
                    child
                        .assert_paragraph()
                        .text_contains("content that explains the second term");
                });
        });

        // Item 4: Glossary Term (with multiple paragraphs)
        assert_ast(&doc).item(4, |item| {
            item.assert_definition()
                .subject("Glossary Term")
                .child_count(2)
                .child(0, |child| {
                    child
                        .assert_paragraph()
                        .text_contains("word or phrase that needs explanation");
                })
                .child(1, |child| {
                    child
                        .assert_paragraph()
                        .text_contains("definitions can have complex content");
                });
        });

        // Item 5: API Endpoint
        assert_ast(&doc).item(5, |item| {
            item.assert_definition()
                .subject("API Endpoint")
                .child_count(1)
                .child(0, |child| {
                    child.assert_paragraph().text_contains("specific URL path");
                });
        });

        // Item 6: Regular paragraph
        assert_ast(&doc).item(6, |item| {
            item.assert_paragraph()
                .text_contains("Regular paragraph after definitions");
        });

        // Item 7: Another Term
        assert_ast(&doc).item(7, |item| {
            item.assert_definition()
                .subject("Another Term")
                .child_count(1)
                .child(0, |child| {
                    child
                        .assert_paragraph()
                        .text_contains("appear anywhere in the document");
                });
        });

        // Item 8: Final paragraph
        assert_ast(&doc).item(8, |item| {
            item.assert_paragraph().text_contains("Final paragraph");
        });
    }

    #[test]
    #[ignore = "Still failing - needs investigation"]
    fn test_verified_definitions_mixed_content() {
        use crate::txxt_nano::testing::assert_ast;

        let source = TxxtSources::get_string("100-definitions-mixed-content.txxt")
            .expect("Failed to load sample file");
        let tokens = lex_with_spans(&source);
        let doc = parse_with_source(tokens, &source).unwrap();

        // Item 0-1: Opening paragraphs
        assert_ast(&doc)
            .item(0, |item| {
                item.assert_paragraph() // "Definitions with Mixed Content Test"
                    .text_contains("Definitions with Mixed Content Test");
            })
            .item(1, |item| {
                item.assert_paragraph() // "both paragraphs and lists"
                    .text_contains("both paragraphs and lists");
            });

        // Item 2: Programming Language (paragraph + list)
        assert_ast(&doc).item(2, |item| {
            item.assert_definition()
                .subject("Programming Language")
                .child_count(2)
                .child(0, |child| {
                    child
                        .assert_paragraph()
                        .text_contains("formal language comprising");
                })
                .child(1, |child| {
                    child.assert_list().item_count(3);
                });
        });

        // Item 3: HTTP Methods (list only)
        assert_ast(&doc).item(3, |item| {
            item.assert_definition()
                .subject("HTTP Methods")
                .child_count(1)
                .child(0, |child| {
                    child.assert_list().item_count(4);
                });
        });

        // Item 4: Data Structure (paragraph + 2 lists)
        assert_ast(&doc).item(4, |item| {
            item.assert_definition()
                .subject("Data Structure")
                .child_count(3)
                .child(0, |child| {
                    child
                        .assert_paragraph()
                        .text_contains("organizing and storing data");
                })
                .child(1, |child| {
                    child.assert_list().item_count(4);
                })
                .child(2, |child| {
                    child.assert_list().item_count(3);
                });
        });

        // Item 5: Regular paragraph
        assert_ast(&doc).item(5, |item| {
            item.assert_paragraph()
                .text_contains("Regular paragraph between definitions");
        });

        // Item 6: Design Pattern (paragraph + 3 lists)
        assert_ast(&doc).item(6, |item| {
            item.assert_definition()
                .subject("Design Pattern")
                .child_count(4)
                .child(0, |child| {
                    child.assert_paragraph().text_contains("reusable solution");
                })
                .child(1, |child| {
                    child.assert_list().item_count(3);
                })
                .child(2, |child| {
                    child.assert_list().item_count(3);
                })
                .child(3, |child| {
                    child.assert_list().item_count(3);
                });
        });

        // Item 7: End paragraph
        assert_ast(&doc).item(7, |item| {
            item.assert_paragraph().text_contains("End of document");
        });
    }

    #[test]
    #[ignore = "Still failing - needs investigation"]
    fn test_verified_ensemble_with_definitions() {
        // Comprehensive ensemble test with all core elements including definitions
        use crate::txxt_nano::testing::assert_ast;

        let source = TxxtSources::get_string("110-ensemble-with-definitions.txxt").unwrap();
        let tokens = lex_with_spans(&source);
        let doc = parse_with_source(tokens, &source).unwrap();

        // Item 0-1: Opening paragraphs
        assert_ast(&doc)
            .item(0, |item| {
                item.assert_paragraph() // "Ensemble Test with Definitions"
                    .text_contains("Ensemble Test with Definitions");
            })
            .item(1, |item| {
                item.assert_paragraph() // "all core elements"
                    .text_contains("all core elements");
            });

        // Item 2: Introduction definition (with para + list)
        assert_ast(&doc).item(2, |item| {
            item.assert_definition()
                .subject("Introduction")
                .child_count(2)
                .child(0, |child| {
                    child.assert_paragraph().text_contains("ensemble test");
                })
                .child(1, |child| {
                    child.assert_list().item_count(4);
                });
        });

        // Item 3: Simple Elements Section session
        assert_ast(&doc).item(3, |item| {
            item.assert_session()
                .label("1. Simple Elements Section {{session}}")
                .child_count(5); // para + 2 definitions + para + list
        });

        // Item 4: Nested Elements Section session
        assert_ast(&doc).item(4, |item| {
            item.assert_session()
                .label("2. Nested Elements Section {{session}}")
                .child_count(3); // para + 2 subsections (2.1 and 2.2)
        });
    }

    // ==================== ANNOTATION TESTS ====================
    // Testing the Annotation element
    //
    #[test]
    fn test_annotation_marker_minimal() {
        let source = "Para one. {{paragraph}}\n\n:: note ::\n\nPara two. {{paragraph}}\n";
        let tokens = lex_with_spans(source);
        let doc = parse_with_source(tokens, source).unwrap();

        assert_eq!(doc.content.len(), 3); // paragraph, annotation, paragraph
        assert!(doc.content[1].is_annotation());
    }

    #[test]
    fn test_annotation_single_line() {
        let source = "Para one. {{paragraph}}\n\n:: note :: This is inline text\n\nPara two. {{paragraph}}\n";
        let tokens = lex_with_spans(source);
        let doc = parse_with_source(tokens, source).unwrap();

        assert_eq!(doc.content.len(), 3); // paragraph, annotation, paragraph
        let annotation = doc.content[1].as_annotation().unwrap();
        assert_eq!(annotation.label.value, "note");
        assert_eq!(annotation.content.len(), 1); // One paragraph with inline text
        assert!(annotation.content[0].is_paragraph());
    }

    #[test]
    fn test_verified_annotations_simple() {
        let source = TxxtSources::get_string("120-annotations-simple.txxt")
            .expect("Failed to load sample file");
        let tokens = lex_with_spans(&source);
        let doc = parse_with_source(tokens, &source).unwrap();

        // Verify document parses successfully and contains expected structure

        // Find and verify :: note :: annotation
        let note_annotation = doc
            .content
            .iter()
            .find(|item| {
                item.as_annotation() //
                    .map(|a| a.label.value == "note")
                    .unwrap_or(false)
            })
            .expect("Should contain :: note :: annotation");
        assert!(note_annotation
            .as_annotation()
            .unwrap()
            .parameters
            .is_empty());
        assert!(note_annotation.as_annotation().unwrap().content.is_empty());

        // Find and verify :: warning severity=high :: annotation
        let warning_annotation = doc
            .content
            .iter()
            .find(|item| {
                item.as_annotation()
                    .map(|a| a.label.value == "warning")
                    .unwrap_or(false)
            })
            .expect("Should contain :: warning :: annotation");
        let warning = warning_annotation.as_annotation().unwrap();
        assert_eq!(warning.parameters.len(), 1);
        assert_eq!(warning.parameters[0].key, "severity");
        assert_eq!(warning.parameters[0].value, Some("high".to_string()));

        // Find and verify :: python.typing :: annotation (namespaced label)
        let python_annotation = doc
            .content
            .iter()
            .find(|item| {
                item.as_annotation()
                    .map(|a| a.label.value.contains("python"))
                    .unwrap_or(false)
            })
            .expect("Should contain :: python.typing :: annotation");
        assert_eq!(
            python_annotation.as_annotation().unwrap().label.value,
            "python.typing"
        );
    }

    #[test]
    fn test_verified_annotations_block_content() {
        let source = TxxtSources::get_string("130-annotations-block-content.txxt")
            .expect("Failed to load sample file");
        let tokens = lex_with_spans(&source);
        let doc = parse_with_source(tokens, &source).unwrap();

        // Find and verify :: note author="Jane Doe" :: annotation with block content
        let note_annotation = doc
            .content
            .iter()
            .find(|item| {
                item.as_annotation()
                    .map(|a| a.label.value == "note")
                    .unwrap_or(false)
            })
            .expect("Should contain :: note :: annotation");
        let note = note_annotation.as_annotation().unwrap();
        assert_eq!(note.parameters.len(), 2);
        assert_eq!(note.parameters[0].key, "author");
        assert_eq!(note.parameters[0].value, Some("Jane Doe".to_string()));
        assert_eq!(note.parameters[1].key, "date");
        assert_eq!(note.parameters[1].value, Some("2025-01-15".to_string()));
        assert_eq!(note.content.len(), 2); // Two paragraphs
        assert!(note.content[0].is_paragraph());
        assert!(note.content[1].is_paragraph());

        // Find and verify :: warning severity=critical :: annotation with list
        let warning_annotation = doc
            .content
            .iter()
            .find(|item| {
                item.as_annotation()
                    .map(|a| a.label.value == "warning")
                    .unwrap_or(false)
            })
            .expect("Should contain :: warning :: annotation");
        let warning = warning_annotation.as_annotation().unwrap();
        assert_eq!(warning.parameters.len(), 3);
        assert_eq!(warning.parameters[0].key, "severity");
        assert_eq!(warning.parameters[0].value, Some("critical".to_string()));
        assert_eq!(warning.parameters[1].key, "priority");
        assert_eq!(warning.parameters[1].value, Some("high".to_string()));
        assert_eq!(warning.parameters[2].key, "reviewer");
        assert_eq!(warning.parameters[2].value, Some("Alice Smith".to_string()));
        assert_eq!(warning.content.len(), 2); // Paragraph + List
        assert!(warning.content[0].is_paragraph());
        assert!(warning.content[1].is_list());

        // Verify the list has 3 items
        let list = warning.content[1].as_list().unwrap();
        assert_eq!(list.items.len(), 3);
    }

    // ==================== FOREIGN BLOCK TESTS ====================
    // Testing the Foreign Block element

    #[test]
    fn test_foreign_block_simple_with_content() {
        let source = "Code Example:\n    function hello() {\n        return \"world\";\n    }\n:: javascript caption=\"Hello World\" ::\n\n";
        let tokens = lex_with_spans(source);
        println!("Tokens: {:?}", tokens);
        let doc = parse_with_source(tokens, source).unwrap();

        assert_eq!(doc.content.len(), 1);
        let foreign_block = doc.content[0].as_foreign_block().unwrap();
        assert_eq!(foreign_block.subject, "Code Example");
        assert!(foreign_block.content.contains("function hello()"));
        assert!(foreign_block.content.contains("return \"world\""));
        assert_eq!(foreign_block.closing_annotation.label.value, "javascript");
        assert_eq!(foreign_block.closing_annotation.parameters.len(), 1);
        assert_eq!(
            foreign_block.closing_annotation.parameters[0].key,
            "caption"
        );
        assert_eq!(
            foreign_block.closing_annotation.parameters[0].value,
            Some("Hello World".to_string())
        );
    }

    #[test]
    fn test_foreign_block_marker_form() {
        let source = "Image Reference:\n\n:: image type=jpg, src=sunset.jpg :: As the sun sets, we see a colored sea bed.\n\n";
        let tokens = lex_with_spans(source);
        let doc = parse_with_source(tokens, source).unwrap();

        assert_eq!(doc.content.len(), 1);
        let foreign_block = doc.content[0].as_foreign_block().unwrap();
        assert_eq!(foreign_block.subject, "Image Reference");
        assert_eq!(foreign_block.content, ""); // No content in marker form
        assert_eq!(foreign_block.closing_annotation.label.value, "image");
        assert_eq!(foreign_block.closing_annotation.parameters.len(), 2);
        assert_eq!(foreign_block.closing_annotation.parameters[0].key, "type");
        assert_eq!(
            foreign_block.closing_annotation.parameters[0].value,
            Some("jpg".to_string())
        );
        assert_eq!(foreign_block.closing_annotation.parameters[1].key, "src");
        assert_eq!(
            foreign_block.closing_annotation.parameters[1].value,
            Some("sunset.jpg".to_string())
        );
    }

    #[test]
    fn test_foreign_block_preserves_whitespace() {
        let source = "Indented Code:\n\n    // This has    multiple    spaces\n    const regex = /[a-z]+/g;\n    \n    console.log(\"Hello, World!\");\n\n:: javascript ::\n\n";
        let tokens = lex_with_spans(source);
        let doc = parse_with_source(tokens, source).unwrap();

        let foreign_block = doc.content[0].as_foreign_block().unwrap();
        assert!(foreign_block.content.contains("    multiple    spaces")); // Preserves multiple spaces
        assert!(foreign_block.content.contains("    \n")); // Preserves blank lines
    }

    #[test]
    fn test_foreign_block_multiple_blocks() {
        // Fixed by reordering parsers: foreign_block before session
        // Since foreign blocks have stricter requirements (must have closing annotation),
        // trying them first resolves the ambiguity

        let source = "First Block:\n\n    code1\n\n:: lang1 ::\n\nSecond Block:\n\n    code2\n\n:: lang2 ::\n\n";
        let tokens = lex_with_spans(source);
        let doc = parse_with_source(tokens, source).unwrap();

        assert_eq!(doc.content.len(), 2);

        let first_block = doc.content[0].as_foreign_block().unwrap();
        assert_eq!(first_block.subject, "First Block");
        assert!(first_block.content.contains("code1"));
        assert_eq!(first_block.closing_annotation.label.value, "lang1");

        let second_block = doc.content[1].as_foreign_block().unwrap();
        assert_eq!(second_block.subject, "Second Block");
        assert!(second_block.content.contains("code2"));
        assert_eq!(second_block.closing_annotation.label.value, "lang2");
    }

    #[test]
    fn test_foreign_block_with_paragraphs() {
        let source = "Intro paragraph.\n\nCode Block:\n\n    function test() {\n        return true;\n    }\n\n:: javascript ::\n\nOutro paragraph.\n\n";
        let tokens = lex_with_spans(source);
        let doc = parse_with_source(tokens, source).unwrap();

        assert_eq!(doc.content.len(), 3);
        assert!(doc.content[0].is_paragraph());
        assert!(doc.content[1].is_foreign_block());
        assert!(doc.content[2].is_paragraph());
    }

    #[test]
    fn test_verified_foreign_blocks_simple() {
        let source = TxxtSources::get_string("140-foreign-blocks-simple.txxt")
            .expect("Failed to load sample file");
        let tokens = lex_with_spans(&source);
        let doc = parse_with_source(tokens, &source).unwrap();

        // Find JavaScript code block
        let js_block = doc
            .content
            .iter()
            .find(|item| {
                item.as_foreign_block()
                    .map(|fb| fb.closing_annotation.label.value == "javascript")
                    .unwrap_or(false)
            })
            .expect("Should contain JavaScript foreign block");
        let js = js_block.as_foreign_block().unwrap();
        assert_eq!(js.subject, "Code Example");
        assert!(js.content.contains("function hello()"));
        assert!(js.content.contains("console.log"));
        assert_eq!(js.closing_annotation.parameters.len(), 1);
        assert_eq!(js.closing_annotation.parameters[0].key, "caption");

        // Find Python code block
        let py_block = doc
            .content
            .iter()
            .find(|item| {
                item.as_foreign_block()
                    .map(|fb| fb.closing_annotation.label.value == "python")
                    .unwrap_or(false)
            })
            .expect("Should contain Python foreign block");
        let py = py_block.as_foreign_block().unwrap();
        assert_eq!(py.subject, "Another Code Block");
        assert!(py.content.contains("def fibonacci"));
        assert!(py.content.contains("for i in range"));

        // Find SQL block
        let sql_block = doc
            .content
            .iter()
            .find(|item| {
                item.as_foreign_block()
                    .map(|fb| fb.closing_annotation.label.value == "sql")
                    .unwrap_or(false)
            })
            .expect("Should contain SQL foreign block");
        let sql = sql_block.as_foreign_block().unwrap();
        assert_eq!(sql.subject, "SQL Example");
        assert!(sql.content.contains("SELECT"));
        assert!(sql.content.contains("FROM users"));
    }

    #[test]
    fn test_verified_foreign_blocks_no_content() {
        let source = TxxtSources::get_string("150-foreign-blocks-no-content.txxt")
            .expect("Failed to load sample file");
        let tokens = lex_with_spans(&source);
        let doc = parse_with_source(tokens, &source).unwrap();

        // Find image reference
        let image_block = doc
            .content
            .iter()
            .find(|item| {
                item.as_foreign_block()
                    .map(|fb| fb.closing_annotation.label.value == "image")
                    .unwrap_or(false)
            })
            .expect("Should contain image foreign block");
        let image = image_block.as_foreign_block().unwrap();
        assert_eq!(image.subject, "Image Reference");
        assert_eq!(image.content, ""); // No content in marker form
        assert_eq!(image.closing_annotation.parameters.len(), 2);
        assert_eq!(image.closing_annotation.parameters[0].key, "type");
        assert_eq!(
            image.closing_annotation.parameters[0].value,
            Some("jpg".to_string())
        );

        // Find binary file reference
        let binary_block = doc
            .content
            .iter()
            .find(|item| {
                item.as_foreign_block()
                    .map(|fb| fb.closing_annotation.label.value == "binary")
                    .unwrap_or(false)
            })
            .expect("Should contain binary foreign block");
        let binary = binary_block.as_foreign_block().unwrap();
        assert_eq!(binary.subject, "Binary File Reference");
        assert_eq!(binary.content, "");
        assert_eq!(binary.closing_annotation.parameters.len(), 2);
        assert_eq!(binary.closing_annotation.parameters[0].key, "type");
        assert_eq!(
            binary.closing_annotation.parameters[0].value,
            Some("pdf".to_string())
        );
    }
}
