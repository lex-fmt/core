//! Basic AST conversion functions
//!
//! Converts intermediate AST structures (with spans) to final AST structures
//! with extracted text content (no position information).

use super::super::intermediate_ast::{
    AnnotationWithSpans, ContentItemWithSpans, DefinitionWithSpans, DocumentWithSpans,
    ForeignBlockWithSpans, ListItemWithSpans, ListWithSpans, ParagraphWithSpans, SessionWithSpans,
};
use super::super::parameters::convert_parameter;
use super::text::{extract_line_text, extract_text, reconstruct_raw_content};
use txxt_ast::{
    Annotation, ContentItem, Definition, Document, ForeignBlock, Label, List, ListItem, Paragraph,
    Session,
};

/// Convert document with spans to final document
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

/// Convert content item enum by dispatching to appropriate converter
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

/// Convert paragraph with extracted text content
pub(crate) fn convert_paragraph(source: &str, para: ParagraphWithSpans) -> Paragraph {
    Paragraph {
        lines: para
            .line_spans
            .iter()
            .map(|spans| extract_line_text(source, spans))
            .collect(),
        span: None,
    }
}

/// Convert session with extracted text content
pub(crate) fn convert_session(source: &str, sess: SessionWithSpans) -> Session {
    Session {
        title: extract_line_text(source, &sess.title_spans),
        content: sess
            .content
            .into_iter()
            .map(|item| convert_content_item(source, item))
            .collect(),
        span: None,
    }
}

/// Convert definition with extracted text content
pub(crate) fn convert_definition(source: &str, def: DefinitionWithSpans) -> Definition {
    let subject = extract_line_text(source, &def.subject_spans);

    Definition {
        subject,
        content: def
            .content
            .into_iter()
            .map(|item| convert_content_item(source, item))
            .collect(),
        span: None,
    }
}

/// Convert annotation with extracted text content
pub(crate) fn convert_annotation(source: &str, ann: AnnotationWithSpans) -> Annotation {
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

/// Convert list with item conversions
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

/// Convert list item with content
pub(crate) fn convert_list_item(source: &str, item: ListItemWithSpans) -> ListItem {
    ListItem::with_content(
        extract_line_text(source, &item.text_spans),
        item.content
            .into_iter()
            .map(|content_item| convert_content_item(source, content_item))
            .collect(),
    )
}

/// Convert foreign block with content
pub(crate) fn convert_foreign_block(source: &str, fb: ForeignBlockWithSpans) -> ForeignBlock {
    let subject = extract_line_text(source, &fb.subject_spans);
    let content = fb
        .content_spans
        .map(|spans| reconstruct_raw_content(source, &spans))
        .unwrap_or_default();
    let closing_annotation = convert_annotation(source, fb.closing_annotation);

    ForeignBlock::new(subject, content, closing_annotation)
}
