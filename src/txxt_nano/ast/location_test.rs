#[cfg(test)]
mod tests {
    use crate::txxt_nano::ast::{
        elements::Session,
        location::{Location, Position},
        traits::{AstNode, Container},
    };

    #[test]
    fn test_get_location() {
        let location = Location::new(Position::new(1, 0), Position::new(1, 10));
        let session = Session::with_title("Title".to_string()).with_span(Some(location));
        assert_eq!(session.get_location(), Some(Position::new(1, 0)));
    }

    #[test]
    fn test_find_nodes_at_position() {
        use crate::txxt_nano::ast::elements::ContentItem;
        use crate::txxt_nano::ast::elements::Document;
        use crate::txxt_nano::ast::find_nodes_at_position;

        let span1 = Location::new(Position::new(1, 0), Position::new(1, 10));
        let span2 = Location::new(Position::new(2, 0), Position::new(2, 10));
        let session1 = Session::with_title("Title1".to_string()).with_span(Some(span1));
        let session2 = Session::with_title("Title2".to_string()).with_span(Some(span2));
        let document = Document::with_content(vec![
            ContentItem::Session(session1),
            ContentItem::Session(session2),
        ]);
        let nodes = find_nodes_at_position(&document, Position::new(1, 5));
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].node_type(), "Session");
        assert_eq!(nodes[0].display_label(), "Title1");
    }

    #[test]
    fn test_find_nested_nodes_at_position() {
        use crate::txxt_nano::ast::elements::{ContentItem, Document, Paragraph};
        use crate::txxt_nano::ast::find_nodes_at_position;

        let para_span = Location::new(Position::new(2, 0), Position::new(2, 10));
        let paragraph = Paragraph::from_line("Nested".to_string()).with_span(Some(para_span));
        let session_span = Location::new(Position::new(1, 0), Position::new(3, 0));
        let mut session = Session::with_title("Title".to_string()).with_span(Some(session_span));
        session
            .children_mut()
            .push(ContentItem::Paragraph(paragraph));
        let document = Document::with_content(vec![ContentItem::Session(session)]);
        let nodes = find_nodes_at_position(&document, Position::new(2, 5));
        assert_eq!(nodes.len(), 2);
        // Results are returned deepest to shallowest, so paragraph (nested) comes first
        assert_eq!(nodes[0].node_type(), "Paragraph");
        assert_eq!(nodes[1].node_type(), "Session");
    }
}
