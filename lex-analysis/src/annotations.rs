use crate::utils::find_annotation_at_position;
use lex_parser::lex::ast::{
    Annotation, AstNode, ContentItem, Document, Parameter, Position, Range, Session,
};

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

#[derive(Debug, Clone, PartialEq)]
pub struct AnnotationEdit {
    pub range: Range,
    pub new_text: String,
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

pub fn toggle_annotation_resolution(
    document: &Document,
    position: Position,
    resolved: bool,
) -> Option<AnnotationEdit> {
    let annotation = find_annotation_at_position(document, position)?;
    resolution_edit(annotation, resolved)
}

pub fn resolution_edit(annotation: &Annotation, resolved: bool) -> Option<AnnotationEdit> {
    let mut params = annotation.data.parameters.clone();
    let status_index = params
        .iter()
        .position(|param| param.key.eq_ignore_ascii_case("status"));

    if resolved {
        match status_index {
            Some(idx) if params[idx].value.eq_ignore_ascii_case("resolved") => return None,
            Some(idx) => params[idx].value = "resolved".to_string(),
            None => params.push(Parameter::new("status".to_string(), "resolved".to_string())),
        }
    } else if let Some(idx) = status_index {
        params.remove(idx);
    } else {
        return None;
    }

    Some(AnnotationEdit {
        range: annotation.header_location().clone(),
        new_text: format_header(&annotation.data.label.value, &params),
    })
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
            .next_back()
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

fn format_header(label: &str, params: &[Parameter]) -> String {
    let mut header = format!(":: {}", label);
    for param in params {
        header.push(' ');
        header.push_str(&param.key);
        header.push('=');
        header.push_str(&param.value);
    }
    header.push_str(" ::");
    header
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

    #[test]
    fn adds_status_parameter_when_resolving() {
        let source = ":: note ::\n";
        let document = parsing::parse_document(source).unwrap();
        let position = SourceLocation::new(source).byte_to_position(source.find("note").unwrap());
        let edit = toggle_annotation_resolution(&document, position, true).expect("edit");
        assert_eq!(edit.new_text, ":: note status=resolved ::");
    }

    #[test]
    fn removes_status_parameter_when_unresolving() {
        use lex_parser::lex::ast::{Data, Label};
        let data = Data::new(
            Label::new("note".to_string()),
            vec![
                Parameter::new("priority".to_string(), "high".to_string()),
                Parameter::new("status".to_string(), "resolved".to_string()),
            ],
        );
        let annotation = Annotation::from_data(data, Vec::new()).at(Range::new(
            0..0,
            Position::new(0, 0),
            Position::new(0, 0),
        ));
        let edit = resolution_edit(&annotation, false).expect("edit");
        assert_eq!(edit.new_text, ":: note priority=high ::");
    }
}
