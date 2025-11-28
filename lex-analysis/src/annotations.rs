use lex_parser::lex::ast::{Annotation, AstNode, ContentItem, Document, Position, Range, Session};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnotationDirection {
    Forward,
    Backward,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnnotationNavigationResult {
    pub label: String,
    pub parameters: Vec<(String, String)>,
    pub header: Range,
    pub body: Option<Range>,
}

pub fn next_annotation(
    document: &Document,
    position: Position,
) -> Option<AnnotationNavigationResult> {
    navigate(document, position, AnnotationDirection::Forward)
}

pub fn previous_annotation(
    document: &Document,
    position: Position,
) -> Option<AnnotationNavigationResult> {
    navigate(document, position, AnnotationDirection::Backward)
}

pub fn navigate(
    document: &Document,
    position: Position,
    direction: AnnotationDirection,
) -> Option<AnnotationNavigationResult> {
    let mut annotations = collect_annotations(document);
    if annotations.is_empty() {
        return None;
    }
    annotations.sort_by_key(|annotation| annotation.header_location().start);

    let idx = match direction {
        AnnotationDirection::Forward => next_index(&annotations, position),
        AnnotationDirection::Backward => previous_index(&annotations, position),
    };
    annotations
        .get(idx)
        .map(|annotation| annotation_to_result(annotation))
}

fn annotation_to_result(annotation: &Annotation) -> AnnotationNavigationResult {
    AnnotationNavigationResult {
        label: annotation.data.label.value.clone(),
        parameters: annotation
            .data
            .parameters
            .iter()
            .map(|param| (param.key.clone(), param.value.clone()))
            .collect(),
        header: annotation.header_location().clone(),
        body: annotation.body_location(),
    }
}

fn next_index(entries: &[&Annotation], position: Position) -> usize {
    if let Some(current) = containing_index(entries, position) {
        if current + 1 >= entries.len() {
            0
        } else {
            current + 1
        }
    } else {
        entries
            .iter()
            .enumerate()
            .find(|(_, annotation)| annotation.header_location().start > position)
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    }
}

fn previous_index(entries: &[&Annotation], position: Position) -> usize {
    if let Some(current) = containing_index(entries, position) {
        if current == 0 {
            entries.len() - 1
        } else {
            current - 1
        }
    } else {
        entries
            .iter()
            .enumerate()
            .filter(|(_, annotation)| annotation.header_location().start < position)
            .map(|(idx, _)| idx)
            .last()
            .unwrap_or(entries.len() - 1)
    }
}

fn containing_index(entries: &[&Annotation], position: Position) -> Option<usize> {
    entries
        .iter()
        .position(|annotation| annotation.range().contains(position))
}

pub(crate) fn collect_annotations(document: &Document) -> Vec<&Annotation> {
    let mut entries = Vec::new();
    for annotation in document.annotations() {
        entries.push(annotation);
    }
    collect_from_session(&document.root, &mut entries);
    entries
}

fn collect_from_session<'a>(session: &'a Session, entries: &mut Vec<&'a Annotation>) {
    for annotation in session.annotations() {
        entries.push(annotation);
    }
    for item in session.iter_items() {
        collect_from_item(item, entries);
    }
}

fn collect_from_item<'a>(item: &'a ContentItem, entries: &mut Vec<&'a Annotation>) {
    match item {
        ContentItem::Annotation(annotation) => {
            entries.push(annotation);
            for child in annotation.children.iter() {
                collect_from_item(child, entries);
            }
        }
        ContentItem::Paragraph(paragraph) => {
            for annotation in paragraph.annotations() {
                entries.push(annotation);
            }
            for line in &paragraph.lines {
                collect_from_item(line, entries);
            }
        }
        ContentItem::List(list) => {
            for annotation in list.annotations() {
                entries.push(annotation);
            }
            for item in list.items.iter() {
                collect_from_item(item, entries);
            }
        }
        ContentItem::ListItem(list_item) => {
            for annotation in list_item.annotations() {
                entries.push(annotation);
            }
            for child in list_item.children.iter() {
                collect_from_item(child, entries);
            }
        }
        ContentItem::Definition(definition) => {
            for annotation in definition.annotations() {
                entries.push(annotation);
            }
            for child in definition.children.iter() {
                collect_from_item(child, entries);
            }
        }
        ContentItem::Session(session) => collect_from_session(session, entries),
        ContentItem::VerbatimBlock(verbatim) => {
            for annotation in verbatim.annotations() {
                entries.push(annotation);
            }
        }
        ContentItem::TextLine(_)
        | ContentItem::VerbatimLine(_)
        | ContentItem::BlankLineGroup(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lex_parser::lex::ast::SourceLocation;
    use lex_parser::lex::parsing;

    const SAMPLE: &str = r#":: note ::
    Doc note.
::

Intro:

    :: todo ::
        Body
    ::

Paragraph text.

:: info ::
    Extra details.
::
"#;

    fn parse() -> Document {
        parsing::parse_document(SAMPLE).expect("fixture parses")
    }

    fn position_of(needle: &str) -> Position {
        let offset = SAMPLE.find(needle).expect("needle present");
        SourceLocation::new(SAMPLE).byte_to_position(offset)
    }

    #[test]
    fn navigates_forward_including_wrap() {
        let document = parse();
        let start = position_of("Intro:");
        let first = next_annotation(&document, start).expect("annotation");
        assert_eq!(first.label, "todo");

        let within_second = position_of("Paragraph");
        let second = next_annotation(&document, within_second).expect("next");
        assert_eq!(second.label, "info");

        let after_last = position_of("Extra details");
        let wrap = next_annotation(&document, after_last).expect("wrap");
        assert_eq!(wrap.label, "note");
    }

    #[test]
    fn navigates_backward_including_wrap() {
        let document = parse();
        let start = position_of("Paragraph text");
        let prev = previous_annotation(&document, start).expect("previous");
        assert_eq!(prev.label, "todo");

        let wrap = previous_annotation(&document, position_of(":: note")).expect("wrap");
        assert_eq!(wrap.label, "info");
    }
}
