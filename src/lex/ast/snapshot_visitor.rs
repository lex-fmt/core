//! Snapshot building for AST nodes
//!
//! This module provides the canonical AST traversal that creates a normalized snapshot
//! representation of the entire tree. All serializers should consume the output
//! of snapshot_node() rather than reimplementing traversal logic.

use super::snapshot::AstSnapshot;
use super::traits::{AstNode, Container};
use super::{Annotation, ContentItem, Definition, Document, List, ListItem, Paragraph, Session};

/// Create a snapshot of a single AST node and all its children
///
/// This function recursively builds a complete snapshot tree for a node and all its descendants.
pub fn snapshot_node<T: AstNode>(node: &T) -> AstSnapshot {
    // We match on concrete types here - since this is called with concrete types from ContentItem,
    // we don't need to do any casting
    let node_type = node.node_type();
    let label = node.display_label();

    // For container types, we need to visit children
    // But without unsafe casting, we can only do this if we have the concrete type
    // This is a limitation of the generic approach
    //
    // The solution: use ContentItem enum variants directly in callers
    // See snapshot_from_content_concrete below

    AstSnapshot::new(node_type.to_string(), label)
}

/// Build snapshot from a concrete ContentItem enum
///
/// This is the preferred way to call the snapshot builder since it avoids unsafe casting.
pub fn snapshot_from_content(item: &ContentItem) -> AstSnapshot {
    match item {
        ContentItem::Session(session) => build_session_snapshot(session),
        ContentItem::Paragraph(para) => build_paragraph_snapshot(para),
        ContentItem::List(list) => build_list_snapshot(list),
        ContentItem::ListItem(li) => build_list_item_snapshot(li),
        ContentItem::Definition(def) => build_definition_snapshot(def),
        ContentItem::VerbatimBlock(fb) => build_verbatim_block_snapshot(fb),
        ContentItem::VerbatimLine(fl) => {
            AstSnapshot::new("VerbatimLine".to_string(), fl.display_label())
        }
        ContentItem::Annotation(ann) => build_annotation_snapshot(ann),
        ContentItem::TextLine(tl) => AstSnapshot::new("TextLine".to_string(), tl.display_label()),
        ContentItem::BlankLineGroup(blg) => AstSnapshot::new(
            "BlankLineGroup".to_string(),
            format!("{} line(s)", blg.count),
        ),
    }
}

/// Build a snapshot for the document root, flattening the root session
///
/// Note: Document metadata (annotations) are not included in this snapshot.
/// This reflects the document structure where metadata is separate from content.
/// The root session is flattened so its children appear as direct children of the Document.
pub fn snapshot_from_document(doc: &Document) -> AstSnapshot {
    let mut snapshot = AstSnapshot::new(
        "Document".to_string(),
        format!(
            "Document ({} metadata, {} items)",
            doc.metadata.len(),
            doc.root.children.len()
        ),
    );

    // Flatten the root session - its children become direct children of the Document
    for child in &doc.root.children {
        snapshot.children.push(snapshot_from_content(child));
    }

    snapshot
}

fn build_session_snapshot(session: &Session) -> AstSnapshot {
    let mut snapshot = AstSnapshot::new("Session".to_string(), session.display_label());
    for child in session.children() {
        snapshot.children.push(snapshot_from_content(child));
    }
    snapshot
}

fn build_paragraph_snapshot(para: &Paragraph) -> AstSnapshot {
    let mut snapshot = AstSnapshot::new("Paragraph".to_string(), para.display_label());
    for line in &para.lines {
        snapshot.children.push(snapshot_from_content(line));
    }
    snapshot
}

fn build_list_snapshot(list: &List) -> AstSnapshot {
    let mut snapshot = AstSnapshot::new("List".to_string(), list.display_label());
    for item in &list.items {
        snapshot.children.push(snapshot_from_content(item));
    }
    snapshot
}

fn build_list_item_snapshot(item: &ListItem) -> AstSnapshot {
    let mut snapshot = AstSnapshot::new("ListItem".to_string(), item.display_label());
    for child in item.children() {
        snapshot.children.push(snapshot_from_content(child));
    }
    snapshot
}

fn build_definition_snapshot(def: &Definition) -> AstSnapshot {
    let mut snapshot = AstSnapshot::new("Definition".to_string(), def.display_label());
    for child in def.children() {
        snapshot.children.push(snapshot_from_content(child));
    }
    snapshot
}

fn build_annotation_snapshot(ann: &Annotation) -> AstSnapshot {
    let mut snapshot = AstSnapshot::new("Annotation".to_string(), ann.display_label());
    for child in ann.children() {
        snapshot.children.push(snapshot_from_content(child));
    }
    snapshot
}

fn build_verbatim_block_snapshot(fb: &super::Verbatim) -> AstSnapshot {
    let label = format!("{} ({} groups)", fb.display_label(), fb.group_len());
    let mut snapshot = AstSnapshot::new("VerbatimBlock".to_string(), label);

    for (idx, group) in fb.group().enumerate() {
        let mut group_snapshot = AstSnapshot::new(
            "VerbatimGroup".to_string(),
            format!("{}#{}", group.subject.as_string(), idx),
        );
        for child in group.children.iter() {
            group_snapshot.children.push(snapshot_from_content(child));
        }
        snapshot.children.push(group_snapshot);
    }

    snapshot
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::ast::elements::annotation::Annotation;
    use crate::lex::ast::elements::paragraph::Paragraph;
    use crate::lex::ast::elements::session::Session;

    #[test]
    fn test_snapshot_from_document_empty() {
        let doc = Document::new();
        let snapshot = snapshot_from_document(&doc);

        assert_eq!(snapshot.node_type, "Document");
        assert_eq!(snapshot.label, "Document (0 metadata, 0 items)");
        assert!(snapshot.children.is_empty());
    }

    #[test]
    fn test_snapshot_from_document_with_content() {
        let mut doc = Document::new();
        doc.root
            .children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Test".to_string(),
            )));
        doc.root
            .children
            .push(ContentItem::Session(Session::with_title(
                "Section".to_string(),
            )));

        let snapshot = snapshot_from_document(&doc);

        assert_eq!(snapshot.node_type, "Document");
        assert_eq!(snapshot.label, "Document (0 metadata, 2 items)");
        assert_eq!(snapshot.children.len(), 2);
        assert_eq!(snapshot.children[0].node_type, "Paragraph");
        assert_eq!(snapshot.children[1].node_type, "Session");
    }

    #[test]
    fn test_snapshot_excludes_metadata() {
        use crate::lex::ast::elements::label::Label;

        let annotation = Annotation::new(Label::new("test-label".to_string()), vec![], vec![]);
        let doc = Document::with_metadata_and_content(
            vec![annotation],
            vec![ContentItem::Paragraph(Paragraph::from_line(
                "Test".to_string(),
            ))],
        );

        let snapshot = snapshot_from_document(&doc);

        assert_eq!(snapshot.label, "Document (1 metadata, 1 items)");
        // Metadata should not appear as children - they are kept separate
        assert_eq!(snapshot.children.len(), 1);
        assert_eq!(snapshot.children[0].node_type, "Paragraph");
        // Verify no Annotation nodes in children
        assert!(snapshot
            .children
            .iter()
            .all(|child| child.node_type != "Annotation"));
    }

    #[test]
    fn test_snapshot_from_document_preserves_structure() {
        let mut session = Session::with_title("Main".to_string());
        session
            .children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Para 1".to_string(),
            )));

        let mut doc = Document::new();
        doc.root.children.push(ContentItem::Session(session));

        let snapshot = snapshot_from_document(&doc);

        assert_eq!(snapshot.node_type, "Document");
        assert_eq!(snapshot.children.len(), 1);

        let session_snapshot = &snapshot.children[0];
        assert_eq!(session_snapshot.node_type, "Session");
        assert_eq!(session_snapshot.children.len(), 1);
        assert_eq!(session_snapshot.children[0].node_type, "Paragraph");
    }
}
