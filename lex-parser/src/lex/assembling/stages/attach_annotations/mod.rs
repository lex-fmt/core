//! Annotation attachment stage
//!
//! Converts annotations from being content items to metadata attached to AST nodes.
//! Implements the attachment rules specified in docs/dev/proposals/annottions-attachment.lex.
//!
//! # Attachment Rules
//!
//! 1. Closest Element: An annotation attaches to the closest content element,
//!    measured by the number of blank lines separating them.
//! 2. Tie-breaker: If equidistant, the next element wins.
//! 3. Document-level: Annotations at document start followed by a blank line attach to Document.
//! 4. Container-end: When an annotation is the last element in a container, the container
//!    itself becomes the "next" element for distance comparisons.
//!
//! # Module Organization
//!
//! - `types`: Shared data structures
//! - `distance`: Distance calculation and attachment decision logic
//! - Main module: Orchestration and tree traversal

mod distance;
mod types;

use std::collections::{HashMap, HashSet};

use crate::lex::ast::elements::annotation::Annotation;
use crate::lex::ast::elements::content_item::ContentItem;
use crate::lex::ast::traits::AstNode;
use crate::lex::ast::Document;
use crate::lex::transforms::{Runnable, TransformError};

pub use types::{AttachmentTarget, ContainerKind, ContainerSpan, Entry, EntryKind, PendingAttachment};

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

        let previous = distance::find_previous_content(&entries, entry_idx);
        let next = distance::find_next_content(&entries, entry_idx, &container_span);
        let blank_after = distance::blank_gap_after(&entries, entry_idx, &container_span);

        if let Some(target) = distance::decide_attachment(
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

        // Document-level annotations (marker form) should remain detached at the root.
        assert!(result.annotations.len() >= 3);
        assert!(result
            .root
            .children
            .iter()
            .all(|item| !matches!(item, ContentItem::Annotation(_))));

        let trailing_paragraph = result
            .root
            .children
            .iter()
            .rev()
            .find_map(|item| item.as_paragraph())
            .expect("expected trailing paragraph");
        assert_eq!(trailing_paragraph.annotations.len(), 1);
    }
}
