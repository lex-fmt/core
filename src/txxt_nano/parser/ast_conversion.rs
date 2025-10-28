//! AST conversion functions that transform intermediate, span-based AST nodes
//! into the final AST with extracted text and source positions.

use super::intermediate_ast::{
    AnnotationWithSpans, ContentItemWithSpans, DefinitionWithSpans, DocumentWithSpans,
    ForeignBlockWithSpans, ListItemWithSpans, ListWithSpans, ParagraphWithSpans, SessionWithSpans,
};
use super::parameters::convert_parameter;
use crate::txxt_nano::ast::{
    Annotation, ContentItem, Definition, Document, ForeignBlock, Label, List, ListItem, Paragraph,
    Session, SourceLocation, Span, TextContent,
};

/// Helper to extract text from source using a span
#[allow(dead_code)] // Reserved for future use
pub(crate) fn extract_text(source: &str, span: &std::ops::Range<usize>) -> String {
    if span.start >= span.end || span.end > source.len() {
        // Empty or synthetic span (like for IndentLevel/DedentLevel)
        return String::new();
    }
    source[span.start..span.end].to_string()
}

/// Helper to extract and concatenate text from multiple spans
pub(crate) fn extract_line_text(source: &str, spans: &[std::ops::Range<usize>]) -> String {
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

/// Helper to reconstruct raw content from token spans
/// For foreign blocks, this includes the leading indentation that comes before the first token
pub(crate) fn reconstruct_raw_content(source: &str, spans: &[std::ops::Range<usize>]) -> String {
    if spans.is_empty() {
        return String::new();
    }
    // Find the overall span from first to last
    let first_start = spans.first().map(|s| s.start).unwrap_or(0);
    let last_end = spans.last().map(|s| s.end).unwrap_or(0);

    if first_start >= last_end || last_end > source.len() {
        return String::new();
    }

    // For foreign blocks, we need to include the leading indentation.
    // Look backwards from first_start to find the previous newline.
    // Everything from after the newline to last_end is the content.
    let mut start = first_start;

    // Scan backwards to find the beginning of this line (after previous newline)
    if first_start > 0 {
        let bytes = source.as_bytes();
        // Look for the previous newline
        for i in (0..first_start).rev() {
            if bytes[i] == b'\n' {
                // Found the newline, content starts after it
                start = i + 1;
                break;
            }
        }
        // If no newline found, start from the beginning of the source
        // (This handles the first line case)
    }

    source[start..last_end].to_string()
}

/// Convert intermediate AST with spans to final AST with extracted text
pub(crate) fn convert_document(source: &str, doc_with_spans: DocumentWithSpans) -> Document {
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
        span: None,
    }
}

/// Convert intermediate AST with spans to final AST, preserving position information
pub(crate) fn convert_document_with_positions(
    source: &str,
    doc_with_spans: DocumentWithSpans,
) -> Document {
    let source_loc = SourceLocation::new(source);

    Document {
        metadata: doc_with_spans
            .metadata
            .into_iter()
            .map(|ann| convert_annotation_with_positions(source, &source_loc, ann))
            .collect(),
        content: doc_with_spans
            .content
            .into_iter()
            .map(|item| convert_content_item_with_positions(source, &source_loc, item))
            .collect(),
        span: None,
    }
}

pub(crate) fn convert_content_item(source: &str, item: ContentItemWithSpans) -> ContentItem {
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

pub(crate) fn convert_paragraph(source: &str, para: ParagraphWithSpans) -> Paragraph {
    Paragraph {
        lines: para
            .line_spans
            .iter()
            .map(|spans| {
                let text = extract_line_text(source, spans);
                TextContent::from_string(text, None)
            })
            .collect(),
        span: None,
    }
}

pub(crate) fn convert_session(source: &str, sess: SessionWithSpans) -> Session {
    let title_text = extract_line_text(source, &sess.title_spans);
    Session {
        title: TextContent::from_string(title_text, None),
        content: sess
            .content
            .into_iter()
            .map(|item| convert_content_item(source, item))
            .collect(),
        span: None,
    }
}

pub(crate) fn convert_definition(source: &str, def: DefinitionWithSpans) -> Definition {
    // Extract subject (colon already excluded from spans by definition_subject parser)
    let subject_text = extract_line_text(source, &def.subject_spans);

    Definition {
        subject: TextContent::from_string(subject_text, None),
        content: def
            .content
            .into_iter()
            .map(|item| convert_content_item(source, item))
            .collect(),
        span: None,
    }
}

pub(crate) fn convert_annotation(source: &str, ann: AnnotationWithSpans) -> Annotation {
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
        span: None,
    }
}

pub(crate) fn convert_list(source: &str, list: ListWithSpans) -> List {
    List {
        items: list
            .items
            .into_iter()
            .map(|item| convert_list_item(source, item))
            .collect(),
        span: None,
    }
}

pub(crate) fn convert_list_item(source: &str, item: ListItemWithSpans) -> ListItem {
    ListItem::with_content(
        extract_line_text(source, &item.text_spans),
        item.content
            .into_iter()
            .map(|content_item| convert_content_item(source, content_item))
            .collect(),
    )
}

pub(crate) fn convert_foreign_block(source: &str, fb: ForeignBlockWithSpans) -> ForeignBlock {
    let subject = extract_line_text(source, &fb.subject_spans);
    let content = fb
        .content_spans
        .map(|spans| reconstruct_raw_content(source, &spans))
        .unwrap_or_default();
    let closing_annotation = convert_annotation(source, fb.closing_annotation);

    ForeignBlock::new(subject, content, closing_annotation)
}

// ============================================================================
// Position-Preserving Conversion Functions
// ============================================================================

pub(crate) fn convert_content_item_with_positions(
    source: &str,
    source_loc: &SourceLocation,
    item: ContentItemWithSpans,
) -> ContentItem {
    match item {
        ContentItemWithSpans::Paragraph(p) => {
            ContentItem::Paragraph(convert_paragraph_with_positions(source, source_loc, p))
        }
        ContentItemWithSpans::Session(s) => {
            ContentItem::Session(convert_session_with_positions(source, source_loc, s))
        }
        ContentItemWithSpans::List(l) => {
            ContentItem::List(convert_list_with_positions(source, source_loc, l))
        }
        ContentItemWithSpans::Definition(d) => {
            ContentItem::Definition(convert_definition_with_positions(source, source_loc, d))
        }
        ContentItemWithSpans::Annotation(a) => {
            ContentItem::Annotation(convert_annotation_with_positions(source, source_loc, a))
        }
        ContentItemWithSpans::ForeignBlock(fb) => {
            ContentItem::ForeignBlock(convert_foreign_block_with_positions(source, source_loc, fb))
        }
    }
}

pub(crate) fn convert_paragraph_with_positions(
    source: &str,
    source_loc: &SourceLocation,
    para: ParagraphWithSpans,
) -> Paragraph {
    let span = if !para.line_spans.is_empty() {
        let start_range = para.line_spans.first().and_then(|spans| spans.first());
        let end_range = para.line_spans.last().and_then(|spans| spans.last());
        match (start_range, end_range) {
            (Some(start), Some(end)) => Some(Span::new(
                source_loc.byte_to_position(start.start),
                source_loc.byte_to_position(end.end),
            )),
            _ => None,
        }
    } else {
        None
    };

    Paragraph {
        lines: para
            .line_spans
            .iter()
            .map(|spans| {
                let text = extract_line_text(source, spans);
                TextContent::from_string(text, None)
            })
            .collect(),
        span,
    }
}

pub(crate) fn convert_session_with_positions(
    source: &str,
    source_loc: &SourceLocation,
    sess: SessionWithSpans,
) -> Session {
    let span = if !sess.title_spans.is_empty() {
        let start_range = sess.title_spans.first();
        let end_range = sess.title_spans.last();
        match (start_range, end_range) {
            (Some(start), Some(end)) => Some(Span::new(
                source_loc.byte_to_position(start.start),
                source_loc.byte_to_position(end.end),
            )),
            _ => None,
        }
    } else {
        None
    };

    let title_text = extract_line_text(source, &sess.title_spans);
    Session {
        title: TextContent::from_string(title_text, None),
        content: sess
            .content
            .into_iter()
            .map(|item| convert_content_item_with_positions(source, source_loc, item))
            .collect(),
        span,
    }
}

pub(crate) fn convert_definition_with_positions(
    source: &str,
    source_loc: &SourceLocation,
    def: DefinitionWithSpans,
) -> Definition {
    let span = if !def.subject_spans.is_empty() {
        let start_range = def.subject_spans.first();
        let end_range = def.subject_spans.last();
        match (start_range, end_range) {
            (Some(start), Some(end)) => Some(Span::new(
                source_loc.byte_to_position(start.start),
                source_loc.byte_to_position(end.end),
            )),
            _ => None,
        }
    } else {
        None
    };

    let subject_text = extract_line_text(source, &def.subject_spans);
    Definition {
        subject: TextContent::from_string(subject_text, None),
        content: def
            .content
            .into_iter()
            .map(|item| convert_content_item_with_positions(source, source_loc, item))
            .collect(),
        span,
    }
}

pub(crate) fn convert_annotation_with_positions(
    source: &str,
    source_loc: &SourceLocation,
    ann: AnnotationWithSpans,
) -> Annotation {
    let label_text = ann
        .label_span
        .as_ref()
        .map(|span| extract_text(source, span).trim().to_string())
        .unwrap_or_default();
    let label_span = ann.label_span.as_ref().map(|range| {
        Span::new(
            source_loc.byte_to_position(range.start),
            source_loc.byte_to_position(range.end),
        )
    });
    let label = Label::new(label_text).with_span(label_span);

    let parameters = ann
        .parameters
        .into_iter()
        .map(|param| convert_parameter(source, param))
        .collect();

    let content = ann
        .content
        .into_iter()
        .map(|item| convert_content_item_with_positions(source, source_loc, item))
        .collect();

    Annotation {
        label,
        parameters,
        content,
        span: None,
    }
}

pub(crate) fn convert_list_with_positions(
    source: &str,
    source_loc: &SourceLocation,
    list: ListWithSpans,
) -> List {
    List {
        items: list
            .items
            .into_iter()
            .map(|item| convert_list_item_with_positions(source, source_loc, item))
            .collect(),
        span: None,
    }
}

pub(crate) fn convert_list_item_with_positions(
    source: &str,
    source_loc: &SourceLocation,
    item: ListItemWithSpans,
) -> ListItem {
    let span = if !item.text_spans.is_empty() {
        let start_range = item.text_spans.first();
        let end_range = item.text_spans.last();
        match (start_range, end_range) {
            (Some(start), Some(end)) => Some(Span::new(
                source_loc.byte_to_position(start.start),
                source_loc.byte_to_position(end.end),
            )),
            _ => None,
        }
    } else {
        None
    };

    ListItem::with_content(
        extract_line_text(source, &item.text_spans),
        item.content
            .into_iter()
            .map(|content_item| {
                convert_content_item_with_positions(source, source_loc, content_item)
            })
            .collect(),
    )
    .with_span(span)
}

pub(crate) fn convert_foreign_block_with_positions(
    source: &str,
    source_loc: &SourceLocation,
    fb: ForeignBlockWithSpans,
) -> ForeignBlock {
    let span = if !fb.subject_spans.is_empty() {
        let start_range = fb.subject_spans.first();
        let end_range = fb.subject_spans.last();
        match (start_range, end_range) {
            (Some(start), Some(end)) => Some(Span::new(
                source_loc.byte_to_position(start.start),
                source_loc.byte_to_position(end.end),
            )),
            _ => None,
        }
    } else {
        None
    };

    let subject = extract_line_text(source, &fb.subject_spans);
    let content = fb
        .content_spans
        .map(|spans| reconstruct_raw_content(source, &spans))
        .unwrap_or_default();
    let closing_annotation =
        convert_annotation_with_positions(source, source_loc, fb.closing_annotation);

    ForeignBlock::new(subject, content, closing_annotation).with_span(span)
}
