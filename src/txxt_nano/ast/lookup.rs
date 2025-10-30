
use super::elements::Document;
use super::span::Position;
use super::traits::AstNode;

pub fn find_nodes_at_position<'a>(
    document: &'a Document,
    position: Position,
) -> Vec<&'a dyn AstNode> {
    document
        .elements_at(position)
        .into_iter()
        .map(|item| item as &dyn AstNode)
        .collect()
}

pub fn format_at_position<'a>(document: &'a Document, position: Position) -> String {
    let nodes = find_nodes_at_position(document, position);
    if nodes.is_empty() {
        "No AST nodes at this position".to_string()
    } else {
        nodes
            .iter()
            .map(|node| format!("- {}: {}", node.node_type(), node.display_label()))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
