//! Annotation attachment stage
//!
//! Converts annotations from being content items to metadata attached to AST nodes.
//! Implements the attachment rules specified in docs/dev/proposals/annottions-attachment.lex.
//!
//! # Attachment Rules
//!
//! 1. **Closest Element**: An annotation attaches to the closest content element,
//!    measured by the number of blank lines separating them.
//! 2. **Tie-breaker**: If equidistant, the next element wins.
//! 3. **Document-level**: Annotations at document start followed by a blank line attach to Document.
//! 4. **Container-end**: When an annotation is the last element in a container, the container
//!    itself becomes the "next" element for distance comparisons.

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use crate::lex::ast::elements::annotation::Annotation;
use crate::lex::ast::elements::content_item::ContentItem;
use crate::lex::ast::range::Range;
use crate::lex::ast::traits::AstNode;
use crate::lex::ast::Document;
use crate::lex::transforms::{Runnable, TransformError};

/// Annotation attachment stage
pub struct AttachAnnotations;

impl AttachAnnotations {
    pub fn new() -> Self {
        AttachAnnotations
    }
}

impl Default for AttachAnnotations {
    fn default() -> Self {
        Self::new()
    }
}

impl Runnable<Document, Document> for AttachAnnotations {
    fn run(&self, mut input: Document) -> Result<Document, TransformError> {
        attach_annotations_in_container(
            &mut input.root.children,
            AnnotationSink::Enabled(&mut input.annotations),
            ContainerKind::DocumentRoot,
            ContainerSpan::from_range(&input.root.location),
        );
        process_children(&mut input.root.children);
        Ok(input)
    }
}

// Allow &mut Vec instead of &mut [ContentItem] because we need to recursively call
// attach_annotations_in_container which requires Vec::retain() to remove annotation
// items after attachment. Using a slice would require converting back to Vec.
#[allow(clippy::ptr_arg)]
fn process_children(children: &mut Vec<ContentItem>) {
    for item in children.iter_mut() {
        match item {
            ContentItem::Session(session) => {
                attach_annotations_in_container(
                    &mut session.children,
                    AnnotationSink::Enabled(&mut session.annotations),
                    ContainerKind::Regular,
                    ContainerSpan::from_range(&session.location),
                );
                process_children(&mut session.children);
            }
            ContentItem::Definition(definition) => {
                attach_annotations_in_container(
                    &mut definition.children,
                    AnnotationSink::Enabled(&mut definition.annotations),
                    ContainerKind::Regular,
                    ContainerSpan::from_range(&definition.location),
                );
                process_children(&mut definition.children);
            }
            ContentItem::ListItem(list_item) => {
                attach_annotations_in_container(
                    &mut list_item.children,
                    AnnotationSink::Enabled(&mut list_item.annotations),
                    ContainerKind::Regular,
                    ContainerSpan::from_range(&list_item.location),
                );
                process_children(&mut list_item.children);
            }
            ContentItem::List(list) => {
                for item in list.items.iter_mut() {
                    if let ContentItem::ListItem(list_item) = item {
                        attach_annotations_in_container(
                            &mut list_item.children,
                            AnnotationSink::Enabled(&mut list_item.annotations),
                            ContainerKind::Regular,
                            ContainerSpan::from_range(&list_item.location),
                        );
                        process_children(&mut list_item.children);
                    }
                }
            }
            ContentItem::Annotation(annotation) => {
                attach_annotations_in_container(
                    &mut annotation.children,
                    AnnotationSink::Disabled,
                    ContainerKind::Detached,
                    ContainerSpan::from_range(&annotation.location),
                );
                process_children(&mut annotation.children);
            }
            _ => {}
        }
    }
}

// Allow &mut Vec instead of &mut [ContentItem] because we need Vec::retain()
// at line 184 to remove annotations from the content tree after attachment.
// The retain() method is only available on Vec, not slices.
#[allow(clippy::ptr_arg)]
fn attach_annotations_in_container(
    children: &mut Vec<ContentItem>,
    mut annotation_sink: AnnotationSink<'_>,
    kind: ContainerKind,
    container_span: ContainerSpan,
) {
    let entries = build_entries(children);
    if entries.is_empty() {
        return;
    }

    let mut extracted = HashMap::new();
    for (idx, item) in children.iter().enumerate() {
        if let ContentItem::Annotation(annotation) = item {
            extracted.insert(idx, annotation.clone());
        }
    }

    if extracted.is_empty() {
        return;
    }

    let mut attachments = Vec::new();
    for (entry_idx, entry) in entries.iter().enumerate() {
        let EntryKind::Annotation(child_index) = entry.kind else {
            continue;
        };

        let previous = find_previous_content(&entries, entry_idx);
        let next = find_next_content(&entries, entry_idx, &container_span);
        let blank_after = blank_gap_after(&entries, entry_idx, &container_span);

        if let Some(target) = decide_attachment(
            previous,
            next.next,
            next.distance_to_end,
            blank_after,
            &kind,
            annotation_sink.allows_container(),
        ) {
            attachments.push(PendingAttachment {
                annotation_index: child_index,
                target,
            });
        }
    }

    if attachments.is_empty() {
        return;
    }

    let mut removed_indices = HashSet::new();

    for pending in attachments {
        if let Some(annotation) = extracted.remove(&pending.annotation_index) {
            match pending.target {
                AttachmentTarget::Content(content_index) => {
                    attach_to_item_at_index(children, content_index, annotation);
                }
                AttachmentTarget::Container => {
                    annotation_sink.push(annotation);
                }
            }
            removed_indices.insert(pending.annotation_index);
        }
    }

    if removed_indices.is_empty() {
        return;
    }

    let mut current_index = 0;
    children.retain(|item| {
        let keep = match item {
            ContentItem::Annotation(_) => !removed_indices.contains(&current_index),
            _ => true,
        };
        current_index += 1;
        keep
    });
}

#[derive(Clone, Copy)]
struct Entry {
    kind: EntryKind,
    start_line: usize,
    end_line: usize,
}

#[derive(Clone, Copy)]
enum EntryKind {
    Content(usize),
    Annotation(usize),
}

struct PendingAttachment {
    annotation_index: usize,
    target: AttachmentTarget,
}

enum AttachmentTarget {
    Content(usize),
    Container,
}

struct NextSearchResult {
    next: Option<(usize, usize)>,
    distance_to_end: usize,
}

enum ContainerKind {
    DocumentRoot,
    Regular,
    Detached,
}

impl ContainerKind {
    fn is_document(&self) -> bool {
        matches!(self, ContainerKind::DocumentRoot)
    }
}

enum AnnotationSink<'a> {
    Enabled(&'a mut Vec<Annotation>),
    Disabled,
}

impl<'a> AnnotationSink<'a> {
    fn allows_container(&self) -> bool {
        matches!(self, AnnotationSink::Enabled(_))
    }

    fn push(&mut self, annotation: Annotation) {
        if let AnnotationSink::Enabled(target) = self {
            target.push(annotation);
        }
    }
}

fn build_entries(children: &[ContentItem]) -> Vec<Entry> {
    let mut entries = Vec::new();

    for (idx, item) in children.iter().enumerate() {
        match item {
            ContentItem::BlankLineGroup(_) => {}
            _ => {
                let range = item.range();
                entries.push(Entry {
                    kind: if matches!(item, ContentItem::Annotation(_)) {
                        EntryKind::Annotation(idx)
                    } else {
                        EntryKind::Content(idx)
                    },
                    start_line: range.start.line,
                    end_line: range.end.line,
                });
            }
        }
    }

    entries
}

fn find_previous_content(entries: &[Entry], entry_index: usize) -> Option<(usize, usize)> {
    if entry_index == 0 {
        return None;
    }

    let mut distance = 0;
    let mut cursor = entry_index;

    while cursor > 0 {
        let prev_idx = cursor - 1;
        distance += blank_lines_between(&entries[prev_idx], &entries[cursor]);
        cursor = prev_idx;

        match entries[cursor].kind {
            EntryKind::Content(idx) => return Some((distance, idx)),
            EntryKind::Annotation(_) => continue,
        }
    }

    None
}

fn find_next_content(
    entries: &[Entry],
    entry_index: usize,
    container_span: &ContainerSpan,
) -> NextSearchResult {
    let mut distance = 0;
    let mut cursor = entry_index;

    while cursor + 1 < entries.len() {
        let next_idx = cursor + 1;
        distance += blank_lines_between(&entries[cursor], &entries[next_idx]);
        cursor = next_idx;

        match entries[cursor].kind {
            EntryKind::Content(idx) => {
                return NextSearchResult {
                    next: Some((distance, idx)),
                    distance_to_end: distance,
                }
            }
            EntryKind::Annotation(_) => continue,
        }
    }

    NextSearchResult {
        next: None,
        distance_to_end: blank_lines_to_end(&entries[cursor], container_span),
    }
}

fn blank_gap_after(entries: &[Entry], entry_index: usize, container_span: &ContainerSpan) -> usize {
    if entry_index + 1 < entries.len() {
        blank_lines_between(&entries[entry_index], &entries[entry_index + 1])
    } else {
        blank_lines_to_end(&entries[entry_index], container_span)
    }
}

fn decide_attachment(
    previous: Option<(usize, usize)>,
    next: Option<(usize, usize)>,
    distance_to_end: usize,
    blank_after: usize,
    kind: &ContainerKind,
    container_allowed: bool,
) -> Option<AttachmentTarget> {
    if kind.is_document() && previous.is_none() && blank_after > 0 {
        return Some(AttachmentTarget::Container);
    }

    let prev_candidate = previous.map(|(distance, idx)| Candidate {
        distance,
        target: AttachmentTarget::Content(idx),
    });

    let next_candidate = match next {
        Some((distance, idx)) => Some(Candidate {
            distance,
            target: AttachmentTarget::Content(idx),
        }),
        None if container_allowed => Some(Candidate {
            distance: distance_to_end,
            target: AttachmentTarget::Container,
        }),
        None => None,
    };

    match (prev_candidate, next_candidate) {
        (Some(prev), Some(next)) => match prev.distance.cmp(&next.distance) {
            Ordering::Less => Some(prev.target),
            Ordering::Greater | Ordering::Equal => Some(next.target),
        },
        (Some(prev), None) => Some(prev.target),
        (None, Some(next)) => Some(next.target),
        (None, None) => None,
    }
}

struct Candidate {
    distance: usize,
    target: AttachmentTarget,
}

#[derive(Clone, Copy)]
struct ContainerSpan {
    end_line: usize,
}

impl ContainerSpan {
    fn from_range(range: &Range) -> Self {
        ContainerSpan {
            end_line: range.end.line,
        }
    }
}

/// Calculate the number of blank lines between two AST entries.
///
/// For multi-line elements (paragraphs, annotations spanning multiple lines),
/// we need to handle the case where elements might be adjacent or overlapping
/// in line numbers. The calculation uses the end line of the left element and
/// determines the effective start of the right element.
///
/// # Edge Cases
/// - If right starts before left ends (overlapping/adjacent multi-line elements):
///   Use right's end line as the effective start to ensure correct distance.
/// - If elements are on consecutive lines with no blank lines between: returns 0.
/// - Otherwise: counts the gap between left.end_line and right's effective start.
fn blank_lines_between(left: &Entry, right: &Entry) -> usize {
    // For multi-line elements, determine the effective starting line of the right element.
    // If right starts at or before left ends (overlapping ranges), use right's end line
    // as the effective start. This handles cases like multi-line annotations adjacent
    // to multi-line paragraphs.
    let effective_start = if right.start_line <= left.end_line {
        right.end_line // Overlapping/adjacent elements
    } else {
        right.start_line
    };

    // Calculate blank lines: if effective_start is more than one line after left.end_line,
    // there are blank lines in between. The formula is: gap - 1 to exclude the line
    // boundaries themselves.
    if effective_start > left.end_line + 1 {
        effective_start - left.end_line - 1
    } else {
        0
    }
}

/// Calculate the number of blank lines from an entry to the end of its container.
///
/// This is used for container-end attachment rules: when an annotation is the last
/// element in a container, we measure its distance to the container's closing boundary.
///
/// # Arguments
/// - `entry`: The AST entry (typically an annotation at container end)
/// - `span`: The container's span information (contains end line)
///
/// # Returns
/// The number of blank lines between the entry and container end, or 0 if adjacent.
fn blank_lines_to_end(entry: &Entry, span: &ContainerSpan) -> usize {
    if span.end_line > entry.end_line + 1 {
        span.end_line - entry.end_line - 1
    } else {
        0
    }
}

fn attach_to_item_at_index(children: &mut [ContentItem], idx: usize, annotation: Annotation) {
    if idx >= children.len() {
        return;
    }

    match &mut children[idx] {
        ContentItem::Paragraph(paragraph) => paragraph.annotations.push(annotation),
        ContentItem::Session(session) => session.annotations.push(annotation),
        ContentItem::List(list) => list.annotations.push(annotation),
        ContentItem::ListItem(list_item) => list_item.annotations.push(annotation),
        ContentItem::Definition(definition) => definition.annotations.push(annotation),
        ContentItem::VerbatimBlock(verbatim) => verbatim.annotations.push(annotation),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::ast::traits::Container;
    use crate::lex::testing::lexplore::{ElementType, Lexplore};
    use crate::lex::testing::parse_without_annotation_attachment;

    #[test]
    fn test_attach_annotations_passthrough() {
        let stage = AttachAnnotations::new();
        let doc = Document::new();
        let result = stage.run(doc).unwrap();
        assert_eq!(result.root.children.len(), 0);
        assert!(result.annotations.is_empty());
    }

    #[test]
    fn test_simple_annotation_manual() {
        use crate::lex::ast::elements::blank_line_group::BlankLineGroup;
        use crate::lex::ast::elements::label::Label;
        use crate::lex::ast::elements::paragraph::Paragraph;

        let mut doc = Document::new();
        doc.root
            .children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "First".to_string(),
            )));
        doc.root
            .children
            .push(ContentItem::BlankLineGroup(BlankLineGroup::new(1, vec![])));

        let annotation = Annotation::new(Label::from_string("test"), vec![], vec![]);
        doc.root.children.push(ContentItem::Annotation(annotation));

        doc.root
            .children
            .push(ContentItem::BlankLineGroup(BlankLineGroup::new(1, vec![])));
        doc.root
            .children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Second".to_string(),
            )));

        let stage = AttachAnnotations::new();
        let result = stage.run(doc).unwrap();

        let paragraphs: Vec<_> = result
            .root
            .children
            .iter()
            .filter_map(|item| match item {
                ContentItem::Paragraph(p) => Some(p),
                _ => None,
            })
            .collect();

        assert_eq!(paragraphs.len(), 2);
        assert!(paragraphs[0].annotations.is_empty());
        assert_eq!(paragraphs[1].annotations.len(), 1);
        assert_eq!(paragraphs[1].annotations[0].data.label.value, "test");
    }

    #[test]
    fn test_example_a_closest_wins() {
        let source = Lexplore::load(ElementType::Annotation, 20).source();
        let doc = parse_without_annotation_attachment(&source).unwrap();

        let stage = AttachAnnotations::new();
        let result = stage.run(doc).unwrap();

        let paragraphs: Vec<_> = result
            .root
            .children
            .iter()
            .filter_map(|item| match item {
                ContentItem::Paragraph(p) => Some(p),
                _ => None,
            })
            .collect();

        assert_eq!(paragraphs.len(), 2);
        assert!(paragraphs[0].annotations.is_empty());
        assert_eq!(paragraphs[1].annotations.len(), 1);
        assert_eq!(paragraphs[1].annotations[0].data.label.value, "foo");
    }

    #[test]
    fn test_example_b_tie_next_wins() {
        let source = Lexplore::load(ElementType::Annotation, 21).source();
        let doc = parse_without_annotation_attachment(&source).unwrap();

        let stage = AttachAnnotations::new();
        let result = stage.run(doc).unwrap();

        let paragraphs: Vec<_> = result
            .root
            .children
            .iter()
            .filter_map(|item| match item {
                ContentItem::Paragraph(p) => Some(p),
                _ => None,
            })
            .collect();

        assert_eq!(paragraphs.len(), 2);
        assert!(paragraphs[0].annotations.is_empty());
        assert_eq!(paragraphs[1].annotations.len(), 1);
    }

    #[test]
    fn test_example_d_closer_to_previous() {
        let source = Lexplore::load(ElementType::Annotation, 22).source();
        let doc = parse_without_annotation_attachment(&source).unwrap();

        let stage = AttachAnnotations::new();
        let result = stage.run(doc).unwrap();

        let paragraphs: Vec<_> = result
            .root
            .children
            .iter()
            .filter_map(|item| match item {
                ContentItem::Paragraph(p) => Some(p),
                _ => None,
            })
            .collect();

        assert_eq!(paragraphs.len(), 2);
        assert_eq!(paragraphs[0].annotations.len(), 1);
        assert!(paragraphs[1].annotations.is_empty());
    }

    #[test]
    fn test_example_e_document_start() {
        let source = Lexplore::load(ElementType::Annotation, 23).source();
        let doc = parse_without_annotation_attachment(&source).unwrap();

        let stage = AttachAnnotations::new();
        let result = stage.run(doc).unwrap();

        assert_eq!(result.annotations.len(), 1);
        assert!(result
            .root
            .children
            .iter()
            .all(|item| !matches!(item, ContentItem::Annotation(_))));
    }

    #[test]
    fn test_example_f_document_start_no_blank() {
        let source = Lexplore::load(ElementType::Annotation, 24).source();
        let doc = parse_without_annotation_attachment(&source).unwrap();

        let stage = AttachAnnotations::new();
        let result = stage.run(doc).unwrap();

        assert!(result.annotations.is_empty());
        let paragraph = result.root.children[0]
            .as_paragraph()
            .expect("expected paragraph");
        assert_eq!(paragraph.annotations.len(), 1);
    }

    #[test]
    fn test_example_h_document_end() {
        let source = Lexplore::load(ElementType::Annotation, 25).source();
        let doc = parse_without_annotation_attachment(&source).unwrap();

        let stage = AttachAnnotations::new();
        let result = stage.run(doc).unwrap();

        assert_eq!(result.annotations.len(), 1);
        assert_eq!(result.annotations[0].data.label.value, "foo");
    }

    #[test]
    fn test_example_i_document_end_equidistant() {
        let source = Lexplore::load(ElementType::Annotation, 26).source();
        let doc = parse_without_annotation_attachment(&source).unwrap();

        let stage = AttachAnnotations::new();
        let result = stage.run(doc).unwrap();

        assert_eq!(result.annotations.len(), 1);
    }

    #[test]
    fn test_example_j_session_inner_attachment() {
        let source = Lexplore::load(ElementType::Annotation, 28).source();
        let doc = parse_without_annotation_attachment(&source).unwrap();

        let stage = AttachAnnotations::new();
        let result = stage.run(doc).unwrap();

        let outer_session = result
            .root
            .children
            .iter()
            .find_map(|item| item.as_session())
            .expect("expected outer session");
        let inner_session = outer_session
            .children()
            .iter()
            .find_map(|item| item.as_session())
            .expect("expected inner session");
        let inner_paragraph = inner_session
            .children()
            .iter()
            .find_map(|item| item.as_paragraph())
            .expect("expected inner paragraph");

        assert_eq!(inner_paragraph.annotations.len(), 1);
    }

    #[test]
    fn test_example_k_session_attaches_to_container() {
        let source = Lexplore::load(ElementType::Annotation, 29).source();
        let doc = parse_without_annotation_attachment(&source).unwrap();

        let stage = AttachAnnotations::new();
        let result = stage.run(doc).unwrap();

        let outer_session = result
            .root
            .children
            .iter()
            .find_map(|item| item.as_session())
            .expect("expected outer session");
        let inner_session = outer_session
            .children()
            .iter()
            .find_map(|item| item.as_session())
            .expect("expected inner session");

        assert_eq!(inner_session.annotations.len(), 1);
    }

    #[test]
    fn test_list_item_container_attachment() {
        use crate::lex::ast::elements::label::Label;
        use crate::lex::ast::elements::paragraph::Paragraph;
        use crate::lex::ast::{List, ListItem};

        let mut list_item = ListItem::new("- Task".to_string());
        list_item
            .children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Detail line.".to_string(),
            )));
        let annotation = Annotation::new(Label::from_string("note"), vec![], vec![]);
        list_item.children.push(ContentItem::Annotation(annotation));

        let list = List::new(vec![list_item]);
        let mut doc = Document::new();
        doc.root.children.push(ContentItem::List(list));

        let stage = AttachAnnotations::new();
        let result = stage.run(doc).unwrap();

        let list = result
            .root
            .children
            .iter()
            .find_map(|item| item.as_list())
            .expect("expected list");
        let first_item = list
            .items
            .iter()
            .find_map(|item| item.as_list_item())
            .expect("expected list item");

        assert_eq!(first_item.annotations.len(), 1);
    }

    #[test]
    fn test_annotation_attaches_to_list() {
        let source = "Intro paragraph.\n\n:: note ::\n- Bread\n- Milk\n";
        let doc = parse_without_annotation_attachment(source).unwrap();

        let stage = AttachAnnotations::new();
        let result = stage.run(doc).unwrap();

        let list = result
            .root
            .children
            .iter()
            .find_map(|item| item.as_list())
            .expect("expected list");

        assert_eq!(list.annotations.len(), 1);
    }

    #[test]
    fn test_example_l_multiple_document_level() {
        let source = Lexplore::load(ElementType::Annotation, 27).source();
        let doc = parse_without_annotation_attachment(&source).unwrap();

        let stage = AttachAnnotations::new();
        let result = stage.run(doc).unwrap();

        assert!(result.annotations.len() >= 4);
        assert!(result
            .root
            .children
            .iter()
            .all(|item| !matches!(item, ContentItem::Annotation(_))));

        let trailing_paragraph = result
            .root
            .children
            .last()
            .and_then(|item| item.as_paragraph())
            .expect("expected trailing paragraph");
        assert_eq!(trailing_paragraph.annotations.len(), 1);
    }
}
