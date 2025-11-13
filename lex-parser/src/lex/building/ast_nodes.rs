//! AST Node Creation from Extracted Data
use super::extraction::{
    AnnotationData, DefinitionData, ListItemData, ParagraphData, SessionData, VerbatimBlockkData,
    VerbatimGroupData,
};
use super::location::{
    aggregate_locations, byte_range_to_ast_range, compute_location_from_locations,
};
use crate::lex::ast::elements::typed_content::{
    ContentElement, ListContent, SessionContent, VerbatimContent,
};
use crate::lex::ast::elements::verbatim::VerbatimGroupItem;
use crate::lex::ast::{
    Annotation, Definition, Label, List, ListItem, Paragraph, Range, Session, TextContent,
    TextLine, Verbatim,
};
use crate::lex::parsing::ContentItem;
// ============================================================================
// PARAGRAPH CREATION
// ============================================================================
/// Create a Paragraph AST node from extracted paragraph data.
pub(super) fn paragraph_node(data: ParagraphData, source: &str) -> ContentItem {
    // Convert byte ranges to AST ranges and build TextLines
    let lines: Vec<ContentItem> = data
        .text_lines
        .into_iter()
        .map(|(text, byte_range)| {
            let location = byte_range_to_ast_range(byte_range, source);
            let text_content = TextContent::from_string(text, Some(location.clone()));
            let text_line = TextLine::new(text_content).at(location);
            ContentItem::TextLine(text_line)
        })
        .collect();
    // Convert overall byte range to AST range
    let overall_location = byte_range_to_ast_range(data.overall_byte_range, source);
    ContentItem::Paragraph(Paragraph {
        lines,
        location: overall_location,
    })
}
// ============================================================================
// SESSION CREATION
// ============================================================================
/// Create a Session AST node from extracted session data.
pub(super) fn session_node(
    data: SessionData,
    content: Vec<SessionContent>,
    source: &str,
) -> ContentItem {
    let title_location = byte_range_to_ast_range(data.title_byte_range, source);
    let title = TextContent::from_string(data.title_text, Some(title_location.clone()));
    let child_items: Vec<ContentItem> = content.iter().cloned().map(ContentItem::from).collect();
    let location = aggregate_locations(title_location, &child_items);
    let session = Session::new(title, content).at(location);
    ContentItem::Session(session)
}
// ============================================================================
// DEFINITION CREATION
// ============================================================================
/// Create a Definition AST node from extracted definition data.
pub(super) fn definition_node(
    data: DefinitionData,
    content: Vec<ContentElement>,
    source: &str,
) -> ContentItem {
    let subject_location = byte_range_to_ast_range(data.subject_byte_range, source);
    let subject = TextContent::from_string(data.subject_text, Some(subject_location.clone()));
    let child_items: Vec<ContentItem> = content.iter().cloned().map(ContentItem::from).collect();
    let location = aggregate_locations(subject_location, &child_items);
    let definition = Definition::new(subject, content).at(location);
    ContentItem::Definition(definition)
}
// ============================================================================
// LIST CREATION
// ============================================================================
/// Create a List AST node from list items.
pub(super) fn list_node(items: Vec<ListItem>) -> ContentItem {
    let item_locations: Vec<Range> = items.iter().map(|item| item.location.clone()).collect();
    let typed_items: Vec<ListContent> = items.into_iter().map(ListContent::ListItem).collect();
    let location = if item_locations.is_empty() {
        Range::default()
    } else {
        compute_location_from_locations(&item_locations)
    };
    ContentItem::List(List {
        items: crate::lex::ast::elements::container::ListContainer::from_typed(typed_items),
        location,
    })
}
// ============================================================================
// LIST ITEM CREATION
// ============================================================================
/// Create a ListItem AST node from extracted list item data.
pub(super) fn list_item_node(
    data: ListItemData,
    content: Vec<ContentElement>,
    source: &str,
) -> ListItem {
    let marker_location = byte_range_to_ast_range(data.marker_byte_range, source);
    let marker = TextContent::from_string(data.marker_text, Some(marker_location.clone()));
    let child_items: Vec<ContentItem> = content.iter().cloned().map(ContentItem::from).collect();
    let location = aggregate_locations(marker_location, &child_items);
    ListItem::with_text_content(marker, content).at(location)
}
// ============================================================================
// ANNOTATION CREATION
// ============================================================================
/// Create an Annotation AST node from extracted annotation data.
pub(super) fn annotation_node(
    data: AnnotationData,
    content: Vec<ContentElement>,
    source: &str,
) -> ContentItem {
    use crate::lex::ast::Parameter;
    let label_location = byte_range_to_ast_range(data.label_byte_range, source);
    let label = Label::new(data.label_text).at(label_location.clone());
    // Convert ParameterData to Parameter AST nodes
    let parameters: Vec<Parameter> = data
        .parameters
        .into_iter()
        .map(|param_data| {
            let location = byte_range_to_ast_range(param_data.overall_byte_range, source);
            Parameter {
                key: param_data.key_text,
                value: param_data.value_text.unwrap_or_default(),
                location,
            }
        })
        .collect();
    // Aggregate location from label and content
    let child_items: Vec<ContentItem> = content.iter().cloned().map(ContentItem::from).collect();
    let location = aggregate_locations(label_location, &child_items);
    let annotation = Annotation::new(label, parameters, content).at(location);
    ContentItem::Annotation(annotation)
}
// ============================================================================
// VERBATIM BLOCK CREATION
// ============================================================================
/// Create a VerbatimBlock AST node from extracted verbatim block data.
pub(super) fn verbatim_block_node(
    data: VerbatimBlockkData,
    closing_annotation: Annotation,
    source: &str,
) -> ContentItem {
    if data.groups.is_empty() {
        panic!("Verbatim blocks must contain at least one subject/content pair");
    }
    let mode = data.mode;
    let mut data_groups = data.groups.into_iter();
    let (first_subject, first_children, mut location_sources) =
        build_verbatim_group(data_groups.next().unwrap(), source);
    let mut additional_groups: Vec<VerbatimGroupItem> = Vec::new();
    for group_data in data_groups {
        let (subject, children, mut group_locations) = build_verbatim_group(group_data, source);
        location_sources.append(&mut group_locations);
        additional_groups.push(VerbatimGroupItem::new(subject, children));
    }
    location_sources.push(closing_annotation.location.clone());
    let location = compute_location_from_locations(&location_sources);
    let verbatim_block = Verbatim::new(first_subject, first_children, closing_annotation, mode)
        .with_additional_groups(additional_groups)
        .at(location);
    ContentItem::VerbatimBlock(Box::new(verbatim_block))
}
fn build_verbatim_group(
    group_data: VerbatimGroupData,
    source: &str,
) -> (TextContent, Vec<VerbatimContent>, Vec<Range>) {
    use crate::lex::ast::elements::VerbatimLine;
    let subject_location = byte_range_to_ast_range(group_data.subject_byte_range, source);
    let subject = TextContent::from_string(group_data.subject_text, Some(subject_location.clone()));
    let mut children: Vec<VerbatimContent> = Vec::new();
    let mut locations: Vec<Range> = vec![subject_location];
    for (line_text, line_byte_range) in group_data.content_lines {
        let line_location = byte_range_to_ast_range(line_byte_range, source);
        locations.push(line_location.clone());
        let line_content = TextContent::from_string(line_text, Some(line_location.clone()));
        let verbatim_line = VerbatimLine::from_text_content(line_content).at(line_location);
        children.push(VerbatimContent::VerbatimLine(verbatim_line));
    }
    // Children are all VerbatimLines by construction - no validation needed
    (subject, children, locations)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::ast::elements::typed_content::{ContentElement, SessionContent};
    use crate::lex::ast::Position;
    #[test]
    fn test_paragraph_node() {
        let source = "hello";
        let data = ParagraphData {
            text_lines: vec![("hello".to_string(), 0..5)],
            overall_byte_range: 0..5,
        };
        let result = paragraph_node(data, source);
        match result {
            ContentItem::Paragraph(para) => {
                assert_eq!(para.lines.len(), 1);
                assert_eq!(para.location.start, Position::new(0, 0));
                assert_eq!(para.location.end, Position::new(0, 5));
            }
            _ => panic!("Expected Paragraph"),
        }
    }
    #[test]
    fn test_session_node() {
        let source = "Session";
        let data = SessionData {
            title_text: "Session".to_string(),
            title_byte_range: 0..7,
        };
        let result = session_node(data, Vec::<SessionContent>::new(), source);
        match result {
            ContentItem::Session(session) => {
                assert_eq!(session.title.as_string(), "Session");
                assert_eq!(session.location.start, Position::new(0, 0));
                assert_eq!(session.location.end, Position::new(0, 7));
            }
            _ => panic!("Expected Session"),
        }
    }
    // ============================================================================
    // VALIDATION TESTS
    // ============================================================================
    #[test]
    fn test_session_allows_session_child() {
        use crate::lex::ast::elements::Session;
        let source = "Parent Session\n    Nested Session\n";
        let nested_session = Session::with_title("Nested Session".to_string());
        let content = vec![SessionContent::Session(nested_session)];
        let data = SessionData {
            title_text: "Parent Session".to_string(),
            title_byte_range: 0..14,
        };
        // This should succeed - Sessions can contain Sessions
        let result = session_node(data, content, source);
        match result {
            ContentItem::Session(session) => {
                assert_eq!(session.children.len(), 1);
                assert_eq!(session.title.as_string(), "Parent Session");
            }
            _ => panic!("Expected Session"),
        }
    }
    #[test]
    fn test_definition_allows_non_session_children() {
        use crate::lex::ast::elements::Paragraph;
        let source = "Test Subject:\n    Some content\n";
        let para = Paragraph::from_line("Some content".to_string());
        let content = vec![ContentElement::Paragraph(para)];
        let data = DefinitionData {
            subject_text: "Test Subject".to_string(),
            subject_byte_range: 0..12,
        };
        // This should succeed - Definitions can contain Paragraphs
        let result = definition_node(data, content, source);
        match result {
            ContentItem::Definition(def) => {
                assert_eq!(def.children.len(), 1);
                assert_eq!(def.subject.as_string(), "Test Subject");
            }
            _ => panic!("Expected Definition"),
        }
    }
    #[test]
    fn test_annotation_allows_non_session_children() {
        use crate::lex::ast::elements::Paragraph;
        let source = ":: note ::\n    Some content\n";
        let para = Paragraph::from_line("Some content".to_string());
        let content = vec![ContentElement::Paragraph(para)];
        let data = AnnotationData {
            label_text: "note".to_string(),
            label_byte_range: 0..4,
            parameters: vec![],
        };
        // This should succeed - Annotations can contain Paragraphs
        let result = annotation_node(data, content, source);
        match result {
            ContentItem::Annotation(ann) => {
                assert_eq!(ann.children.len(), 1);
                assert_eq!(ann.label.value, "note");
            }
            _ => panic!("Expected Annotation"),
        }
    }
    #[test]
    fn test_list_item_allows_non_session_children() {
        use crate::lex::ast::elements::Paragraph;
        let source = "- Item\n    Some content\n";
        let para = Paragraph::from_line("Item content".to_string());
        let content = vec![ContentElement::Paragraph(para)];
        let data = ListItemData {
            marker_text: "- ".to_string(),
            marker_byte_range: 0..2,
        };
        // This should succeed - ListItems can contain Paragraphs
        let result = list_item_node(data, content, source);
        assert_eq!(result.children.len(), 1);
        assert_eq!(result.text(), "- ");
    }
}
