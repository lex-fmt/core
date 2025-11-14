//! Annotation attachment stage
//!
//! Converts annotations from being content items to metadata attached to AST nodes.
//! Implements the attachment rules specified in docs/dev/proposals/annottions-attachment.lex.
//!
//! # Attachment Rules
//!
//! 1. **Closest Element**: An annotation attaches to the closest content element,
//!    measured by distance (blank lines) to both previous and next elements.
//! 2. **Tie-breaker**: If equidistant, the next element wins.
//! 3. **Document-level**: Annotations at document start followed by blank line attach to Document.
//! 4. **Container-end**: Annotations at the end of a container attach to the container itself.

use crate::lex::ast::elements::annotation::Annotation;
use crate::lex::ast::elements::content_item::ContentItem;
use crate::lex::ast::traits::AstNode;
use crate::lex::ast::Document;
use crate::lex::transforms::{Runnable, TransformError};

/// Annotation attachment stage
///
/// Transforms annotations from content items into metadata attached to AST nodes
/// based on their proximity to other elements.
///
/// # Input
/// - `Document` - AST with annotations as content items
///
/// # Output
/// - `Document` - AST with annotations attached as metadata
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
        // Process document-level annotations in root session
        attach_annotations_in_container(&mut input.root.children, &mut input.metadata, true);

        // Recursively process all containers in the document
        process_tree_recursive(&mut input.root.children);

        Ok(input)
    }
}

/// Attach annotations in a container based on distance rules
fn attach_annotations_in_container(
    children: &mut Vec<ContentItem>,
    document_metadata: &mut Vec<Annotation>,
    is_document_root: bool,
) {
    // Build a list of positions: content elements and annotations
    let mut positions: Vec<Position> = Vec::new();

    for (idx, item) in children.iter().enumerate() {
        match item {
            ContentItem::Annotation(_) => {
                positions.push(Position::Annotation(idx));
            }
            ContentItem::BlankLineGroup(_) => {
                // Skip blank lines in position tracking
            }
            _ => {
                positions.push(Position::Content(idx));
            }
        }
    }

    // For each annotation, calculate distances and decide attachment
    let mut attachments: Vec<(usize, AttachTarget)> = Vec::new();

    for (pos_idx, pos) in positions.iter().enumerate() {
        if let Position::Annotation(ann_idx) = pos {
            // Calculate distance to previous content
            let dist_before = if pos_idx == 0 {
                0
            } else if let Position::Content(prev_content_idx) = positions[pos_idx - 1] {
                count_blank_lines_between(children, prev_content_idx, *ann_idx)
            } else {
                0
            };

            // Calculate distance to next content
            let dist_after = if pos_idx + 1 >= positions.len() {
                usize::MAX
            } else if let Position::Content(next_content_idx) = positions[pos_idx + 1] {
                count_blank_lines_between(children, *ann_idx, next_content_idx)
            } else {
                usize::MAX
            };

            // Decide attachment target
            let target = if is_document_root && pos_idx == 0 && dist_before == 0 && dist_after > 0 {
                // Document-level annotation
                AttachTarget::Document
            } else if pos_idx == 0
                || (pos_idx > 0 && !matches!(positions[pos_idx - 1], Position::Content(_)))
            {
                // No previous content - attach to next
                if pos_idx + 1 < positions.len() {
                    if let Position::Content(idx) = positions[pos_idx + 1] {
                        AttachTarget::ContentAtPosition(idx)
                    } else {
                        AttachTarget::Document
                    }
                } else {
                    AttachTarget::Document
                }
            } else if dist_after == usize::MAX {
                // No next content
                if is_document_root {
                    AttachTarget::Document
                } else if let Position::Content(idx) = positions[pos_idx - 1] {
                    AttachTarget::ContentAtPosition(idx)
                } else {
                    AttachTarget::Document
                }
            } else if dist_before < dist_after {
                // Closer to previous
                if let Position::Content(idx) = positions[pos_idx - 1] {
                    AttachTarget::ContentAtPosition(idx)
                } else {
                    AttachTarget::Document
                }
            } else {
                // Closer to or tied with next (tie-breaker: next wins)
                if pos_idx + 1 < positions.len() {
                    if let Position::Content(idx) = positions[pos_idx + 1] {
                        AttachTarget::ContentAtPosition(idx)
                    } else {
                        AttachTarget::Document
                    }
                } else {
                    AttachTarget::Document
                }
            };

            attachments.push((*ann_idx, target));
        }
    }

    // Extract annotations first (before we modify children)
    let mut extracted_annotations: Vec<(usize, Annotation)> = Vec::new();
    for (idx, item) in children.iter().enumerate() {
        if let ContentItem::Annotation(ann) = item {
            extracted_annotations.push((idx, ann.clone()));
        }
    }

    // Attach annotations to their targets (do this BEFORE removing annotations from children)
    for (ann_idx, target) in attachments {
        if let Some((_, ann)) = extracted_annotations
            .iter()
            .find(|(idx, _)| *idx == ann_idx)
        {
            match target {
                AttachTarget::Document => {
                    document_metadata.push(ann.clone());
                }
                AttachTarget::ContentAtPosition(content_idx) => {
                    // Attach to the content item at this index
                    // NOTE: content_idx is still valid because we haven't removed annotations yet
                    attach_to_item_at_index(children, content_idx, ann.clone());
                }
            }
        }
    }

    // Remove annotations from children AFTER attaching them
    children.retain(|item| !matches!(item, ContentItem::Annotation(_)));
}

#[derive(Debug, Clone, Copy)]
enum Position {
    Content(usize),
    Annotation(usize),
}

#[derive(Debug, Clone, Copy)]
enum AttachTarget {
    Document,
    ContentAtPosition(usize),
}

/// Count blank lines between two indices using Range information
fn count_blank_lines_between(children: &[ContentItem], start_idx: usize, end_idx: usize) -> usize {
    if start_idx >= children.len() || end_idx >= children.len() {
        return 0;
    }

    let start_range = children[start_idx].range();
    let end_range = children[end_idx].range();

    // Calculate distance based on line numbers
    // Use the END line of the start element and the START line of the end element
    let start_line = start_range.end.line;
    let end_line = end_range.start.line;

    if end_line > start_line + 1 {
        // The number of blank lines is the gap minus 1
        // Example: if start ends at line 1 and end starts at line 3, there's 1 blank line (line 2)
        // Formula: end_line - start_line - 1
        end_line - start_line - 1
    } else {
        // Adjacent or overlapping lines - no blank lines between
        0
    }
}

/// Attach annotation to content item at specific index
fn attach_to_item_at_index(children: &mut [ContentItem], idx: usize, annotation: Annotation) {
    if idx < children.len() {
        match &mut children[idx] {
            ContentItem::Paragraph(p) => p.annotations.push(annotation),
            ContentItem::Session(s) => s.annotations.push(annotation),
            ContentItem::List(l) => l.annotations.push(annotation),
            ContentItem::ListItem(li) => li.annotations.push(annotation),
            ContentItem::Definition(d) => d.annotations.push(annotation),
            ContentItem::VerbatimBlock(v) => v.annotations.push(annotation),
            _ => {}
        }
    }
}

/// Recursively process all containers in the tree
fn process_tree_recursive(children: &mut [ContentItem]) {
    for item in children.iter_mut() {
        match item {
            ContentItem::Session(s) => {
                let mut dummy_metadata = Vec::new();
                attach_annotations_in_container(&mut s.children, &mut dummy_metadata, false);
                process_tree_recursive(&mut s.children);
            }
            ContentItem::Definition(d) => {
                let mut dummy_metadata = Vec::new();
                attach_annotations_in_container(&mut d.children, &mut dummy_metadata, false);
                process_tree_recursive(&mut d.children);
            }
            ContentItem::ListItem(li) => {
                let mut dummy_metadata = Vec::new();
                attach_annotations_in_container(&mut li.children, &mut dummy_metadata, false);
                process_tree_recursive(&mut li.children);
            }
            ContentItem::List(l) => {
                let mut dummy_metadata = Vec::new();
                attach_annotations_in_container(&mut l.items, &mut dummy_metadata, false);
                process_tree_recursive(&mut l.items);
            }
            _ => {
                // Other items don't have children
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::ast::traits::AstNode;
    use crate::lex::parsing::engine::parse_from_flat_tokens;
    use crate::lex::testing::lexplore::{ElementType, Lexplore};
    use crate::lex::transforms::standard::LEXING;
    use crate::lex::transforms::Runnable;

    /// Helper to parse without annotation attachment (for testing the attachment stage itself)
    fn parse_without_attachment(source: &str) -> Result<Document, String> {
        let source = if !source.is_empty() && !source.ends_with('\n') {
            format!("{}\n", source)
        } else {
            source.to_string()
        };
        let tokens = LEXING.run(source.clone()).map_err(|e| e.to_string())?;
        parse_from_flat_tokens(tokens, &source)
    }

    #[test]
    fn test_attach_annotations_passthrough() {
        let stage = AttachAnnotations::new();
        let doc = Document::new();
        let result = stage.run(doc).unwrap();
        assert_eq!(result.root.children.len(), 0);
    }

    #[test]
    fn test_simple_annotation_manual() {
        use crate::lex::ast::elements::annotation::Annotation;
        use crate::lex::ast::elements::blank_line_group::BlankLineGroup;
        use crate::lex::ast::elements::label::Label;
        use crate::lex::ast::elements::paragraph::Paragraph;

        // Build a simple document manually: Para1, Blank, Anno, Blank, Para2
        let mut doc = Document::new();

        doc.root
            .children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "First".to_string(),
            )));
        doc.root
            .children
            .push(ContentItem::BlankLineGroup(BlankLineGroup::new(1, vec![])));

        let anno = Annotation::new(Label::from_string("test"), vec![], vec![]);
        doc.root.children.push(ContentItem::Annotation(anno));

        doc.root
            .children
            .push(ContentItem::BlankLineGroup(BlankLineGroup::new(1, vec![])));
        doc.root
            .children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Second".to_string(),
            )));

        // Before attachment
        assert_eq!(doc.root.children.len(), 5);

        let stage = AttachAnnotations::new();
        let result = stage.run(doc).unwrap();

        // After attachment - should have 2 paragraphs (annotations and blank lines removed)
        let paragraphs: Vec<_> = result
            .root
            .children
            .iter()
            .filter_map(|item| {
                if let ContentItem::Paragraph(p) = item {
                    Some(p)
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(paragraphs.len(), 2, "Should have 2 paragraphs");

        // dist_before = 1, dist_after = 1, tie goes to next (Second)
        assert_eq!(
            paragraphs[0].annotations.len(),
            0,
            "First para should have no annotations"
        );
        assert_eq!(
            paragraphs[1].annotations.len(),
            1,
            "Second para should have 1 annotation"
        );
    }

    #[test]
    fn test_example_a_closest_wins() {
        // Load test document: annotation closer to following element
        let source = Lexplore::load(ElementType::Annotation, 20).source();
        let doc = parse_without_attachment(&source).unwrap();

        // Before attachment: should have annotation as content item
        let has_annotation_before = doc
            .root
            .children
            .iter()
            .any(|item| matches!(item, ContentItem::Annotation(_)));
        assert!(
            has_annotation_before,
            "Document should contain annotation as content item before attachment"
        );

        // Run attachment stage
        let stage = AttachAnnotations::new();
        let result = stage.run(doc).unwrap();

        // After attachment: annotation should be removed from content
        let has_annotation_after = result
            .root
            .children
            .iter()
            .any(|item| matches!(item, ContentItem::Annotation(_)));
        assert!(
            !has_annotation_after,
            "Annotation should be removed from content after attachment"
        );

        // Annotation should be attached to the second paragraph (closer)
        let paragraphs: Vec<_> = result
            .root
            .children
            .iter()
            .filter_map(|item| {
                if let ContentItem::Paragraph(p) = item {
                    Some(p)
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(paragraphs.len(), 2, "Should have 2 paragraphs");
        assert_eq!(
            paragraphs[0].annotations.len(),
            0,
            "First paragraph should have no annotations"
        );
        assert_eq!(
            paragraphs[1].annotations.len(),
            1,
            "Second paragraph should have 1 annotation"
        );
        assert_eq!(paragraphs[1].annotations[0].data.label.value, "foo");
    }

    #[test]
    fn test_example_b_tie_next_wins() {
        // Load test document: equidistant, next element wins
        let source = Lexplore::load(ElementType::Annotation, 21).source();
        let doc = parse_without_attachment(&source).unwrap();

        eprintln!("\n=== Test B: Tie case ===");
        for (i, item) in doc.root.children.iter().enumerate() {
            let range = item.range();
            eprintln!(
                "{}: {} at line {}-{}",
                i,
                item.node_type(),
                range.start.line,
                range.end.line
            );
        }

        let stage = AttachAnnotations::new();
        let result = stage.run(doc).unwrap();

        // Annotation should be attached to the second paragraph (next wins on tie)
        let paragraphs: Vec<_> = result
            .root
            .children
            .iter()
            .filter_map(|item| {
                if let ContentItem::Paragraph(p) = item {
                    Some(p)
                } else {
                    None
                }
            })
            .collect();

        eprintln!("Para[0] annotations: {}", paragraphs[0].annotations.len());
        eprintln!("Para[1] annotations: {}", paragraphs[1].annotations.len());

        assert_eq!(paragraphs.len(), 2);
        assert_eq!(paragraphs[0].annotations.len(), 0);
        assert_eq!(paragraphs[1].annotations.len(), 1);
        assert_eq!(paragraphs[1].annotations[0].data.label.value, "foo");
    }

    #[test]
    fn test_example_d_closer_to_previous() {
        // Load test document: annotation closer to previous element
        let source = Lexplore::load(ElementType::Annotation, 22).source();
        let doc = parse_without_attachment(&source).unwrap();

        // Debug: print what we got
        eprintln!("\n=== Before attachment ===");
        eprintln!("Children count: {}", doc.root.children.len());
        for (i, item) in doc.root.children.iter().enumerate() {
            eprintln!("{}: {:?}", i, item.node_type());
        }

        let stage = AttachAnnotations::new();
        let result = stage.run(doc).unwrap();

        eprintln!("\n=== After attachment ===");
        eprintln!("Children count: {}", result.root.children.len());
        for (i, item) in result.root.children.iter().enumerate() {
            eprintln!("{}: {:?}", i, item.node_type());
        }

        // Annotation should be attached to the first paragraph (closer)
        let paragraphs: Vec<_> = result
            .root
            .children
            .iter()
            .filter_map(|item| {
                if let ContentItem::Paragraph(p) = item {
                    Some(p)
                } else {
                    None
                }
            })
            .collect();

        eprintln!("Paragraphs found: {}", paragraphs.len());
        if !paragraphs.is_empty() {
            eprintln!("Para[0] annotations: {}", paragraphs[0].annotations.len());
        }
        if paragraphs.len() > 1 {
            eprintln!("Para[1] annotations: {}", paragraphs[1].annotations.len());
        }

        assert_eq!(paragraphs.len(), 2);
        assert_eq!(paragraphs[0].annotations.len(), 1);
        assert_eq!(paragraphs[0].annotations[0].data.label.value, "foo");
        assert_eq!(paragraphs[1].annotations.len(), 0);
    }

    #[test]
    fn test_example_e_document_start() {
        // Load test document: document-level annotation
        let source = Lexplore::load(ElementType::Annotation, 23).source();
        let doc = parse_without_attachment(&source).unwrap();

        let stage = AttachAnnotations::new();
        let result = stage.run(doc).unwrap();

        // Annotation should be attached to document metadata
        assert_eq!(
            result.metadata.len(),
            1,
            "Document should have 1 metadata annotation"
        );
        assert_eq!(result.metadata[0].data.label.value, "foo");

        // Paragraph should have no annotations
        let paragraphs: Vec<_> = result
            .root
            .children
            .iter()
            .filter_map(|item| {
                if let ContentItem::Paragraph(p) = item {
                    Some(p)
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(paragraphs.len(), 1);
        assert_eq!(paragraphs[0].annotations.len(), 0);
    }

    #[test]
    fn test_example_f_document_start_no_blank() {
        // Load test document: document start without blank line
        let source = Lexplore::load(ElementType::Annotation, 24).source();
        let doc = parse_without_attachment(&source).unwrap();

        let stage = AttachAnnotations::new();
        let result = stage.run(doc).unwrap();

        // Annotation should be attached to paragraph (no blank line after annotation)
        assert_eq!(
            result.metadata.len(),
            0,
            "Document should have no metadata annotations"
        );

        let paragraphs: Vec<_> = result
            .root
            .children
            .iter()
            .filter_map(|item| {
                if let ContentItem::Paragraph(p) = item {
                    Some(p)
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(paragraphs.len(), 1);
        assert_eq!(paragraphs[0].annotations.len(), 1);
        assert_eq!(paragraphs[0].annotations[0].data.label.value, "foo");
    }

    #[test]
    fn test_example_h_document_end() {
        // Load test document: annotation at document end
        let source = Lexplore::load(ElementType::Annotation, 25).source();
        let doc = parse_without_attachment(&source).unwrap();

        let stage = AttachAnnotations::new();
        let result = stage.run(doc).unwrap();

        // Annotation should be attached to document
        assert_eq!(result.metadata.len(), 1);
        assert_eq!(result.metadata[0].data.label.value, "foo");
    }

    #[test]
    fn test_example_i_document_end_equidistant() {
        // Load test document: annotation at document end, equidistant
        let source = Lexplore::load(ElementType::Annotation, 26).source();
        let doc = parse_without_attachment(&source).unwrap();

        let stage = AttachAnnotations::new();
        let result = stage.run(doc).unwrap();

        // Annotation should be attached to document (next = container wins on tie)
        assert_eq!(result.metadata.len(), 1);
        assert_eq!(result.metadata[0].data.label.value, "foo");
    }

    #[test]
    fn test_example_l_multiple_document_level() {
        // Load test document: multiple annotations with complex cases
        let source = Lexplore::load(ElementType::Annotation, 27).source();
        let doc = parse_without_attachment(&source).unwrap();

        let stage = AttachAnnotations::new();
        let result = stage.run(doc).unwrap();

        // First 4 annotations should be document-level
        assert!(
            result.metadata.len() >= 4,
            "Document should have at least 4 metadata annotations"
        );

        // Verify no annotations remain in content
        let has_annotation = result
            .root
            .children
            .iter()
            .any(|item| matches!(item, ContentItem::Annotation(_)));
        assert!(
            !has_annotation,
            "No annotations should remain in content after attachment"
        );
    }
}
